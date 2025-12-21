use crate::error::ProviderError;
use crate::providers::anthropic::client::AnthropicClient;
use crate::providers::embed::modify_predict_request;
use crate::providers::embed::EmbeddingConfig;
use crate::providers::embed::EmbeddingInput;
use crate::providers::embed::EmbeddingResponse;
use crate::providers::google::{GeminiClient, VertexClient};
use crate::providers::openai::OpenAIClient;
use crate::providers::types::ChatResponse;
use potato_prompt::Prompt;
use potato_type::google::predict::PredictRequest;
use potato_type::google::predict::PredictResponse;
use potato_type::google::EmbeddingConfigTrait;
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
    Vertex(VertexClient),
    Anthropic(AnthropicClient),
    Undefined,
}

impl GenAiClient {
    #[instrument(skip_all)]
    pub async fn generate_content(&self, task: &Prompt) -> Result<ChatResponse, ProviderError> {
        match self {
            GenAiClient::OpenAI(client) => {
                let response = client.chat_completion(task).await.inspect_err(|e| {
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
            GenAiClient::Vertex(client) => {
                let response = client.generate_content(task).await.inspect_err(|e| {
                    error!(error = %e, "Failed to generate content");
                })?;
                Ok(ChatResponse::VertexGenerate(response))
            }
            GenAiClient::Anthropic(client) => {
                let response = client.chat_completion(task).await.inspect_err(|e| {
                    error!(error = %e, "Failed to complete chat");
                })?;
                Ok(ChatResponse::AnthropicMessageV1(response))
            }
            GenAiClient::Undefined => Err(ProviderError::NoProviderError),
        }
    }

    #[instrument(skip_all)]
    pub async fn create_embedding(
        &self,
        inputs: EmbeddingInput,
        config: &EmbeddingConfig,
    ) -> Result<EmbeddingResponse, ProviderError> {
        match self {
            // Create embedding using OpenAI client that expects an array of strings
            GenAiClient::OpenAI(client) => {
                let response = match inputs {
                    EmbeddingInput::Texts(texts) => client.create_embedding(texts, config).await,
                    _ => Err(ProviderError::DoesNotSupportPredictRequest),
                }
                .inspect_err(|e| {
                    error!(error = %e, "Failed to create embedding");
                })?;
                Ok(response)
            }
            // Create embedding using Gemini client that expects an array of strings
            GenAiClient::Gemini(client) => {
                let response = match inputs {
                    EmbeddingInput::Texts(texts) => client.create_embedding(texts, config).await,
                    _ => Err(ProviderError::DoesNotSupportPredictRequest),
                }
                .inspect_err(|e| {
                    error!(error = %e, "Failed to create embedding");
                })?;

                Ok(response)
            }
            // Create embedding using Vertex client that expects a PredictRequest
            GenAiClient::Vertex(client) => {
                let model = config.get_model();
                let response = match inputs {
                    EmbeddingInput::PredictRequest(request) => {
                        let request = modify_predict_request(request, config);
                        client.predict(request, model).await
                    }
                    _ => Err(ProviderError::DoesNotSupportArray),
                }
                .inspect_err(|e| {
                    error!(error = %e, "Failed to create embedding");
                })?;

                Ok(EmbeddingResponse::Vertex(response))
            }
            GenAiClient::Undefined => Err(ProviderError::NoProviderError),
            GenAiClient::Anthropic(_) => Err(ProviderError::EmbeddingNotSupported),
        }
    }

    pub async fn predict(
        &self,
        input: PredictRequest,
        model: &str,
    ) -> Result<PredictResponse, ProviderError> {
        match self {
            GenAiClient::Vertex(client) => Ok(client.predict(input, model).await?),
            _ => Err(ProviderError::NotImplementedError(
                "prediction not implemented for this provider".to_string(),
            )),
        }
    }

    pub fn provider(&self) -> &Provider {
        match self {
            GenAiClient::OpenAI(client) => &client.provider,
            GenAiClient::Gemini(client) => &client.provider,
            GenAiClient::Vertex(client) => &client.provider,
            GenAiClient::Anthropic(client) => &client.provider,
            GenAiClient::Undefined => &Provider::Undefined,
        }
    }
}
