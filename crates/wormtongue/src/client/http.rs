use crate::client::types::RequestType;
use crate::error::HttpError;

use reqwest::blocking::Response;
use reqwest::blocking::{Client as BlockingClient, RequestBuilder};
use reqwest::header::HeaderMap;
use serde_json::Value;

const TIMEOUT_SECS: u64 = 30;
const REDACTED: &str = "REDACTED";

#[derive(Debug, Clone)]
pub struct HTTPConfig {
    pub url: String,
    pub token: String,
}

impl HTTPConfig {
    pub fn new(url: String, token: String) -> Self {
        HTTPConfig { url, token }
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

#[derive(Debug, Clone)]
pub enum AuthStrategy {
    Bearer(String),
    Header { name: String, value: String },
}

pub trait LLMClient {
    fn request_with_retry(
        &mut self,
        request_type: RequestType,
        body_params: Option<Value>,
        query_params: Option<String>,
        headers: Option<HeaderMap>,
    ) -> Result<Response, HttpError>;
}

#[derive(Debug, Clone)]
pub struct BaseHTTPClient {
    client: BlockingClient,
    pub config: HTTPConfig,
    auth_strategy: AuthStrategy,
}

impl BaseHTTPClient {
    pub fn new(config: HTTPConfig, auth_strategy: AuthStrategy) -> Result<Self, HttpError> {
        let client = build_http_client()?;
        Ok(Self {
            client,
            config,
            auth_strategy,
        })
    }

    fn apply_auth(&self, builder: RequestBuilder) -> RequestBuilder {
        match &self.auth_strategy {
            AuthStrategy::Bearer(token) => builder.bearer_auth(token),
            AuthStrategy::Header { name, value } => builder.header(name, value),
        }
    }

    pub fn request(
        &self,
        request_type: RequestType,
        body_params: Option<Value>,
        query_string: Option<String>,
        headers: Option<HeaderMap>,
    ) -> Result<Response, HttpError> {
        let headers = headers.unwrap_or_default();

        let response = match request_type {
            RequestType::Get => {
                let url = if let Some(query_string) = query_string {
                    format!("{}?{}", self.config.url, query_string)
                } else {
                    self.config.url.to_string()
                };

                let builder = self.client.get(url).headers(headers);
                let authenticated_builder = self.apply_auth(builder);
                authenticated_builder
                    .send()
                    .map_err(|e| HttpError::Error(format!("Failed to send request: {}", e)))?
            }
            RequestType::Post => {
                let builder = self.client.post(&self.config.url).headers(headers);
                let authenticated_builder = self.apply_auth(builder);
                if let Some(params) = body_params {
                    authenticated_builder.json(&params)
                } else {
                    authenticated_builder
                }
                .send()
                .map_err(|e| HttpError::Error(format!("Failed to send request: {}", e)))?
            }
        };

        Ok(response)
    }

    pub fn request_with_retry(
        &mut self,
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
        let auth = AuthStrategy::Header {
            name: "x-api-key".to_string(),
            value: config.token.clone(),
        };
        let client = BaseHTTPClient::new(config, auth)?;
        Ok(Self(client))
    }
}

impl LLMClient for ClaudeClient {
    fn request_with_retry(
        &mut self,
        request_type: RequestType,
        body_params: Option<Value>,
        query_params: Option<String>,
        headers: Option<HeaderMap>,
    ) -> Result<Response, HttpError> {
        self.0
            .request_with_retry(request_type, body_params, query_params, headers)
    }
}
