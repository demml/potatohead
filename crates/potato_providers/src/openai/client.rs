use crate::openai::OpenAIConfig;
use potato_client::{BaseHTTPClient, HTTPConfig, LLMClient, RequestType};
use potato_error::{HttpError, PotatoError};
use reqwest::blocking::Response;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use tracing::error;

#[derive(Debug, Clone)]
pub struct OpenAIClient(BaseHTTPClient);

impl OpenAIClient {
    pub fn create_headers(config: &HTTPConfig) -> Result<HeaderMap, PotatoError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", config.token.clone())).map_err(|e| {
                error!("Failed to create header: {}", e);
                PotatoError::Error(e.to_string())
            })?,
        );

        if let Some(org) = config.organization.clone() {
            headers.insert(
                "OpenAI-Organization",
                HeaderValue::from_str(&org).map_err(|e| {
                    error!("Failed to create header: {}", e);
                    PotatoError::Error(e.to_string())
                })?,
            );
        }

        if let Some(project) = config.project.clone() {
            headers.insert(
                "OpenAI-Project",
                HeaderValue::from_str(&project).map_err(|e| {
                    error!("Failed to create header: {}", e);
                    PotatoError::Error(e.to_string())
                })?,
            );
        }

        Ok(headers)
    }
    pub fn new(config: OpenAIConfig) -> Result<Self, PotatoError> {
        let http_config = config.into_http_config();
        let headers = Self::create_headers(&http_config)?;
        let client = BaseHTTPClient::new(http_config, headers)?;

        Ok(Self(client))
    }
}

impl LLMClient for OpenAIClient {
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
