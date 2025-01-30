use crate::client::RequestType;
use crate::client::{AuthStrategy, BaseHTTPClient, HTTPConfig, LLMClient};
use crate::error::HttpError;
use crate::error::TongueError;
use reqwest::blocking::Response;
use reqwest::header::HeaderMap;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct OpenAIClient(BaseHTTPClient);

impl OpenAIClient {
    pub fn new(config: HTTPConfig) -> Result<Self, TongueError> {
        let auth = AuthStrategy::Bearer(config.token.clone());
        let client = BaseHTTPClient::new(config, auth)?;
        Ok(Self(client))
    }
}

impl LLMClient for OpenAIClient {
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
