use crate::client::types::RequestType;
use crate::error::HttpError;

use reqwest::Response;
use reqwest::{header::HeaderMap, Client};

use serde_json::Value;

const TIMEOUT_SECS: u64 = 30;
const REDACTED: &str = "REDACTED";

/// Create a new HTTP client that can be shared across different clients
pub fn build_http_client() -> Result<Client, HttpError> {
    let client_builder = Client::builder().timeout(std::time::Duration::from_secs(TIMEOUT_SECS));
    let client = client_builder
        .build()
        .map_err(|e| HttpError::Error(format!("Failed to create client with error: {}", e)))?;
    Ok(client)
}

#[derive(Debug, Clone)]
pub struct HTTPClient {
    client: Client,
}

impl HTTPClient {
    pub async fn new() -> Result<Self, HttpError> {
        let client = build_http_client()?;

        let api_client = HTTPClient { client };

        Ok(api_client)
    }

    async fn request(
        self,
        url: &str,
        bearer_token: &str,
        request_type: RequestType,
        body_params: Option<Value>,
        query_string: Option<String>,
        headers: Option<HeaderMap>,
    ) -> Result<Response, HttpError> {
        let headers = headers.unwrap_or_default();

        let response = match request_type {
            RequestType::Get => {
                let url = if let Some(query_string) = query_string {
                    format!("{}?{}", url, query_string)
                } else {
                    url.to_string()
                };

                self.client
                    .get(url)
                    .headers(headers)
                    .bearer_auth(&bearer_token)
                    .send()
                    .await
                    .map_err(|e| {
                        HttpError::Error(format!("Failed to send request with error: {}", e))
                    })?
            }
            RequestType::Post => self
                .client
                .post(url)
                .headers(headers)
                .json(&body_params)
                .bearer_auth(&bearer_token)
                .send()
                .await
                .map_err(|e| {
                    HttpError::Error(format!("Failed to send request with error: {}", e))
                })?,
        };

        Ok(response)
    }

    pub async fn request_with_retry(
        &mut self,
        url: &str,
        bearer_token: &str,
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
                    url,
                    bearer_token,
                    request_type.clone(),
                    body_params.clone(),
                    query_params.clone(),
                    headers.clone(),
                )
                .await;

            if response.is_ok() || attempts >= max_attempts {
                break;
            }
        }

        let response = response
            .map_err(|e| HttpError::Error(format!("Failed to send request with error: {}", e)))?;

        Ok(response)
    }
}
