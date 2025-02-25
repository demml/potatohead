use crate::client::ClientURL;
use crate::client::RequestType;
use crate::client::{AuthStrategy, BaseHTTPClient, HTTPConfig, LLMClient};
use crate::error::HttpError;
use crate::error::TongueError;
use crate::tongues::common::PromptType;
use pyo3::prelude::*;
use reqwest::blocking::Response;
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::env;

#[pyclass(eq)]
#[derive(Debug, PartialEq, Clone)]
pub enum OpenAIEndpoints {
    Chat,
    Batch,
}

impl OpenAIEndpoints {
    pub fn as_str(&self) -> &'static str {
        match self {
            OpenAIEndpoints::Chat => "v1/chat/completions",
            OpenAIEndpoints::Batch => "v1/batches",
        }
    }
}

pub fn resolve_url(url: Option<&str>) -> Result<String, TongueError> {
    let url = url
        .map(|s| s.to_string())
        .or_else(|| env::var("WORMTONGUE_URL").ok())
        .unwrap_or_else(|| ClientURL::OpenAI.as_str().to_string());

    Ok(url)
}

pub fn resolve_route(url: &str, prompt_type: &PromptType) -> Result<String, TongueError> {
    match prompt_type {
        PromptType::Chat => Ok(format!("{}/v1/chat/completions", url)),
        PromptType::Image => Ok(format!("{}/v1/images/generations", url)),
        PromptType::Voice => Ok(format!("{}/v1/audio/speech", url)),
        PromptType::Batch => Ok(format!("{}/v1/batches", url)),
        PromptType::Embedding => Ok(format!("{}/v1/embeddings", url)),
        _ => Err(TongueError::UnsupportedInteractionType),
    }
}

pub fn resolve_api_key(url: &str, api_key: Option<&str>) -> Result<String, TongueError> {
    let api_key = api_key
        .map(|s| s.to_string())
        .or_else(|| env::var("WORMTONGUE_API_KEY").ok());

    // if url contains ClientURL::OpenAI.as_str() and api_key is None, return error
    if url.contains(ClientURL::OpenAI.as_str()) && api_key.is_none() {
        return Err(TongueError::MissingAPIKey);
    }

    Ok(api_key.unwrap())
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub url: String,
    pub organization: Option<String>,
    pub project: Option<String>,
}

#[pymethods]
impl OpenAIConfig {
    #[new]
    #[pyo3(signature = (api_key=None, url=None, organization=None, project=None))]
    pub fn new(
        api_key: Option<&str>,
        url: Option<&str>,
        organization: Option<&str>,
        project: Option<&str>,
    ) -> PyResult<Self> {
        let url = resolve_url(url)?;
        let api_key = resolve_api_key(&url, api_key)?;

        Ok(Self {
            api_key: api_key.to_string(),
            url: url.to_string(),
            organization: organization.map(|s| s.to_string()),
            project: project.map(|s| s.to_string()),
        })
    }
}

impl OpenAIConfig {
    pub fn into_http_config(&self) -> HTTPConfig {
        HTTPConfig::new(
            self.url.clone(),
            self.api_key.clone(),
            self.organization.clone(),
            self.project.clone(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct OpenAIClient(BaseHTTPClient);

impl OpenAIClient {
    pub fn new(config: OpenAIConfig) -> Result<Self, TongueError> {
        let http_config = config.into_http_config();
        let auth = AuthStrategy::Bearer(http_config.token.clone());
        let client = BaseHTTPClient::new(http_config, auth)?;
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
