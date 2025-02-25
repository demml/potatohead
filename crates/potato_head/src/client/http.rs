use crate::client::types::RequestType;
use crate::error::HttpError;

use reqwest::blocking::Response;
use reqwest::blocking::{Client as BlockingClient, RequestBuilder};
use reqwest::header::{self, HeaderMap};
use serde_json::Value;

const TIMEOUT_SECS: u64 = 60;

#[derive(Debug, Clone)]
pub struct HTTPConfig {
    pub url: String,
    pub token: String,
    pub organization: Option<String>,
    pub project: Option<String>,
}

impl HTTPConfig {
    pub fn new(
        url: String,
        token: String,
        organization: Option<String>,
        project: Option<String>,
    ) -> Self {
        HTTPConfig {
            url,
            token,
            organization,
            project,
        }
    }
}

/// Create a new HTTP client that can be shared across different clients
pub fn build_http_client() -> Result<BlockingClient, HttpError> {
    let client_builder =
        BlockingClient::builder().timeout(std::time::Duration::from_secs(TIMEOUT_SECS));
    let client = client_builder
        .build()
        .map_err(|e| HttpError::Error(format!("Failed to create client with error: {}", e)))?;
    Ok(client)
}

pub trait LLMClient {
    fn request_with_retry(
        &self,
        route: String,
        request_type: RequestType,
        body_params: Option<Value>,
        query_params: Option<String>,
        headers: Option<HeaderMap>,
    ) -> Result<Response, HttpError>;

    fn url(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct BaseHTTPClient {
    client: BlockingClient,
    pub config: HTTPConfig,
    headers: HeaderMap,
}

impl BaseHTTPClient {
    pub fn new(config: HTTPConfig, headers: HeaderMap) -> Result<Self, HttpError> {
        let client = build_http_client()?;
        Ok(Self {
            config,
            client,
            headers,
        })
    }

    pub fn request(
        &self,
        route: String,
        request_type: RequestType,
        body: Option<Value>,
        query_string: Option<String>,
        headers: Option<HeaderMap>,
    ) -> Result<Response, HttpError> {
        // if headers is provided, merge it with the client headers

        let mut client_headers = self.headers.clone();
        if let Some(runtime_headers) = headers {
            for (key, value) in runtime_headers.iter() {
                client_headers.insert(key, value.clone());
            }
        }

        let response = match request_type {
            RequestType::Get => {
                let url = if let Some(query_string) = query_string {
                    format!("{route}?{query_string}")
                } else {
                    self.config.url.to_string()
                };

                let builder = self.client.get(url).headers(client_headers);
                builder
                    .send()
                    .map_err(|e| HttpError::Error(format!("Failed to send request: {}", e)))?
            }
            RequestType::Post => {
                let builder = self.client.post(&route).headers(client_headers);

                if let Some(body) = body {
                    builder.json(&body)
                } else {
                    builder
                }
                .send()
                .map_err(|e| HttpError::Error(format!("Failed to send request: {}", e)))?
            }
        };

        Ok(response)
    }

    pub fn request_with_retry(
        &self,
        route: String,
        request_type: RequestType,
        body_params: Option<Value>,
        query_params: Option<String>,
        headers: Option<HeaderMap>,
    ) -> Result<Response, HttpError> {
        // this will attempt to send a request. If the request fails, it will refresh the token and try again. If it fails all 3 times it will return an error
        let mut attempts = 0;
        let max_attempts = 3;
        let mut response: Result<Response, HttpError>;

        loop {
            attempts += 1;

            let client = self.clone();
            response = client
                .request(
                    route.clone(),
                    request_type.clone(),
                    body_params.clone(),
                    query_params.clone(),
                    headers.clone(),
                )
                .map_err(|e| HttpError::Error(format!("Failed to send request with error: {}", e)));

            if response.is_ok() || attempts >= max_attempts {
                break;
            }
        }

        let response = response
            .map_err(|e| HttpError::Error(format!("Failed to send request with error: {}", e)))?;

        Ok(response)
    }
}

#[derive(Debug, Clone)]
pub struct ClaudeClient(BaseHTTPClient);

impl ClaudeClient {
    pub async fn new(config: HTTPConfig) -> Result<Self, HttpError> {
        let headers = {
            let mut headers = HeaderMap::new();
            headers.insert(
                "x-api-key",
                header::HeaderValue::from_str(&config.token)
                    .map_err(|e| HttpError::Error(format!("Failed to create header: {}", e)))?,
            );
            headers
        };

        let client = BaseHTTPClient::new(config, headers)?;
        Ok(Self(client))
    }
}

impl LLMClient for ClaudeClient {
    fn request_with_retry(
        &self,
        route: String,
        request_type: RequestType,
        body_params: Option<Value>,
        query_params: Option<String>,
        headers: Option<HeaderMap>,
    ) -> Result<Response, HttpError> {
        self.0
            .request_with_retry(route, request_type, body_params, query_params, headers)
    }

    fn url(&self) -> &str {
        self.0.config.url.as_str()
    }
}
