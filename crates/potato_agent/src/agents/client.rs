use crate::agents::embed::EmbeddingConfig;
use crate::agents::embed::EmbeddingResponse;
use crate::agents::error::AgentError;
use crate::agents::provider::gemini::GeminiClient;
use crate::agents::provider::openai::OpenAIClient;
use crate::agents::types::ChatResponse;
use potato_prompt::Prompt;
use potato_type::Provider;
use pyo3::prelude::*;
use reqwest::header::HeaderName;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{error, instrument};
const TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone)]
#[pyclass]
pub enum ClientType {
    OpenAI,
}

pub enum ClientUrl {
    OpenAI,
}

impl ClientUrl {
    pub fn url(&self) -> &str {
        match self {
            ClientUrl::OpenAI => "https://api.openai.com",
        }
    }
}

/// Create the blocking HTTP client with optional headers.
pub fn build_http_client(
    client_headers: Option<HashMap<String, String>>,
) -> Result<Client, AgentError> {
    let mut headers = HeaderMap::new();

    if let Some(headers_map) = client_headers {
        for (key, value) in headers_map {
            headers.insert(
                HeaderName::from_str(&key).map_err(AgentError::CreateHeaderNameError)?,
                HeaderValue::from_str(&value).map_err(AgentError::CreateHeaderValueError)?,
            );
        }
    }

    let client_builder = Client::builder().timeout(std::time::Duration::from_secs(TIMEOUT_SECS));

    let client = client_builder
        .default_headers(headers)
        .build()
        .map_err(AgentError::CreateClientError)?;

    Ok(client)
}

#[derive(Debug, PartialEq)]
pub enum GenAiClient {
    OpenAI(OpenAIClient),
    Gemini(GeminiClient),
    Undefined,
}

impl GenAiClient {
    #[instrument(skip_all)]
    pub async fn execute(&self, task: &Prompt) -> Result<ChatResponse, AgentError> {
        match self {
            GenAiClient::OpenAI(client) => {
                let response = client.async_chat_completion(task).await.inspect_err(|e| {
                    error!(error = %e, "Failed to complete chat");
                })?;
                Ok(ChatResponse::OpenAI(response))
            }
            GenAiClient::Gemini(client) => {
                let response = client.async_generate_content(task).await.inspect_err(|e| {
                    error!(error = %e, "Failed to generate content");
                })?;
                Ok(ChatResponse::Gemini(response))
            }
            GenAiClient::Undefined => Err(AgentError::NoProviderError),
        }
    }

    #[instrument(skip_all)]
    pub async fn create_embedding(
        &self,
        inputs: Vec<String>,
        config: &EmbeddingConfig,
    ) -> Result<EmbeddingResponse, AgentError> {
        match self {
            GenAiClient::OpenAI(client) => {
                let response = client
                    .async_create_embedding(inputs, config)
                    .await
                    .inspect_err(|e| {
                        error!(error = %e, "Failed to create embedding");
                    })?;
                Ok(response)
            }
            GenAiClient::Gemini(client) => {
                let response = client
                    .async_create_embedding(inputs, config)
                    .await
                    .inspect_err(|e| {
                        error!(error = %e, "Failed to create embedding");
                    })?;

                Ok(response)
            }
            GenAiClient::Undefined => Err(AgentError::NoProviderError),
        }
    }

    pub fn provider(&self) -> &Provider {
        match self {
            GenAiClient::OpenAI(client) => &client.provider,
            GenAiClient::Gemini(client) => &client.provider,
            GenAiClient::Undefined => &Provider::Undefined,
        }
    }
}
