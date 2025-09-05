use potato_type::Provider;

use crate::agents::client::GenAiClient;
use crate::agents::provider::gemini::GeminiClient;
use crate::agents::provider::openai::OpenAIClient;
use crate::AgentError;
use potato_type::google::GeminiEmbeddingConfig;
use potato_type::openai::embedding::{OpenAIEmbeddingConfig, OpenAIEmbeddingResponse};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum EmbeddingConfig {
    OpenAI(OpenAIEmbeddingConfig),
    Gemini(GeminiEmbeddingConfig),
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
        settings: EmbeddingConfig,
    ) -> Result<OpenAIEmbeddingResponse, AgentError> {
        // Implementation for creating an embedding
        self.client.create_embedding(inputs, settings).await
    }
}
