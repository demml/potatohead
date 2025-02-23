use crate::client::http::LLMClient;
use crate::client::{HTTPConfig, RequestType};
use crate::error::TongueError;
use crate::tongues::openai::types::{resolve_api_key, resolve_url};
use crate::tongues::openai::{OpenAIClient, OpenAIPrompt};
use pyo3::prelude::*;
use reqwest::blocking::{Client as BlockingClient, Response};

#[derive(Debug, Clone)]
pub struct OpenAIInterface {
    pub client: OpenAIClient,
    pub prompt: OpenAIPrompt,
}

impl OpenAIInterface {
    pub fn new(
        prompt: &OpenAIPrompt,
        url: Option<&str>,
        api_key: Option<&str>,
        organization: Option<&str>,
        project: Option<&str>,
        client: Option<BlockingClient>,
    ) -> Result<Self, TongueError> {
        let url = resolve_url(url, &prompt.prompt_type)?;
        let api_key = resolve_api_key(&url, api_key)?;
        let config = HTTPConfig::new(
            url,
            api_key,
            organization.map(|s| s.to_string()),
            project.map(|s| s.to_string()),
        );
        let client = OpenAIClient::new(config, client)?;

        Ok(Self {
            client,
            prompt: prompt.clone(),
        })
    }

    pub fn send(&self) -> Result<Response, TongueError> {
        let body = self.prompt.request.to_json()?;
        self.client
            .request_with_retry(RequestType::Post, Some(body), None, None)
            .map_err(TongueError::from)
    }
}
