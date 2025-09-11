use crate::error::ProviderError;
use crate::providers::embed::EmbeddingConfig;
use crate::providers::embed::EmbeddingResponse;
use crate::providers::google::GeminiClient;
use crate::providers::openai::OpenAIClient;
use crate::providers::types::ChatResponse;
use potato_prompt::Prompt;
use potato_type::Provider;
use reqwest::header::HeaderName;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{error, instrument};
const TIMEOUT_SECS: u64 = 30;

/// Create the blocking HTTP client with optional headers.
pub fn build_http_client(
    client_headers: Option<HashMap<String, String>>,
) -> Result<Client, ProviderError> {
    let mut headers = HeaderMap::new();

    if let Some(headers_map) = client_headers {
        for (key, value) in headers_map {
            headers.insert(
                HeaderName::from_str(&key).map_err(ProviderError::CreateHeaderNameError)?,
                HeaderValue::from_str(&value).map_err(ProviderError::CreateHeaderValueError)?,
            );
        }
    }

    let client_builder = Client::builder().timeout(std::time::Duration::from_secs(TIMEOUT_SECS));

    let client = client_builder
        .default_headers(headers)
        .build()
        .map_err(ProviderError::CreateClientError)?;

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
    pub async fn execute(&self, task: &Prompt) -> Result<ChatResponse, ProviderError> {
        match self {
            GenAiClient::OpenAI(client) => {
                let response = client.async_chat_completion(task).await.inspect_err(|e| {
                    error!(error = %e, "Failed to complete chat");
                })?;
                Ok(ChatResponse::OpenAI(response))
            }
            GenAiClient::Gemini(client) => {
                let response = client.generate_content(task).await.inspect_err(|e| {
                    error!(error = %e, "Failed to generate content");
                })?;
                Ok(ChatResponse::Gemini(response))
            }
            GenAiClient::Undefined => Err(ProviderError::NoProviderError),
        }
    }

    #[instrument(skip_all)]
    pub async fn create_embedding(
        &self,
        inputs: Vec<String>,
        config: &EmbeddingConfig,
    ) -> Result<EmbeddingResponse, ProviderError> {
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
                    .create_embedding(inputs, config)
                    .await
                    .inspect_err(|e| {
                        error!(error = %e, "Failed to create embedding");
                    })?;

                Ok(response)
            }
            GenAiClient::Undefined => Err(ProviderError::NoProviderError),
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
