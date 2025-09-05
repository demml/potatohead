use potato_type::google::EmbeddingConfigTrait;
use potato_type::Provider;

use crate::agents::client::GenAiClient;
use crate::agents::provider::gemini::GeminiClient;
use crate::agents::provider::openai::OpenAIClient;
use crate::AgentError;
use potato_type::google::GeminiEmbeddingConfig;
use potato_type::google::GeminiEmbeddingResponse;
use potato_type::openai::embedding::{OpenAIEmbeddingConfig, OpenAIEmbeddingResponse};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum EmbeddingConfig {
    OpenAI(OpenAIEmbeddingConfig),
    Gemini(GeminiEmbeddingConfig),
}

impl EmbeddingConfigTrait for EmbeddingConfig {
    fn get_model(&self) -> &str {
        match self {
            EmbeddingConfig::OpenAI(config) => config.model.as_str(),
            EmbeddingConfig::Gemini(config) => config.get_model(),
        }
    }
}

use tracing::error;
#[derive(Debug, Clone, PartialEq)]
pub struct Embedder {
    client: GenAiClient,
}

impl Embedder {
    pub fn new(provider: Provider) -> Result<Self, AgentError> {
        let client = match provider {
            Provider::OpenAI => GenAiClient::OpenAI(OpenAIClient::new(None, None, None)?),
            Provider::Gemini => GenAiClient::Gemini(GeminiClient::new(None, None, None)?),
            _ => {
                let msg = "No provider specified in ModelSettings";
                error!("{}", msg);
                return Err(AgentError::UndefinedError(msg.to_string()));
            } // Add other providers here as needed
        };

        Ok(Self { client })
    }

    pub async fn create(
        &self,
        inputs: Vec<String>,
        config: EmbeddingConfig,
    ) -> Result<EmbeddingResponse, AgentError> {
        // Implementation for creating an embedding
        self.client.create_embedding(inputs, config).await
    }
}

pub enum EmbeddingResponse {
    OpenAI(OpenAIEmbeddingResponse),
    Gemini(GeminiEmbeddingResponse),
}

impl EmbeddingResponse {
    pub fn to_openai_response(&self) -> Result<&OpenAIEmbeddingResponse, AgentError> {
        match self {
            EmbeddingResponse::OpenAI(response) => Ok(response),
            _ => Err(AgentError::InvalidResponseType("OpenAI".to_string())),
        }
    }

    pub fn to_gemini_response(&self) -> Result<&GeminiEmbeddingResponse, AgentError> {
        match self {
            EmbeddingResponse::Gemini(response) => Ok(response),
            _ => Err(AgentError::InvalidResponseType("Gemini".to_string())),
        }
    }
}
