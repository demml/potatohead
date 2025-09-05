use potato_type::google::EmbeddingConfigTrait;
use potato_type::Provider;

use crate::agents::client::GenAiClient;
use crate::agents::provider::gemini::GeminiClient;
use crate::agents::provider::openai::OpenAIClient;
use crate::AgentError;
use potato_type::google::GeminiEmbeddingConfig;
use potato_type::google::GeminiEmbeddingResponse;
use potato_type::openai::embedding::{OpenAIEmbeddingConfig, OpenAIEmbeddingResponse};
use pyo3::prelude::*;
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum EmbeddingConfig {
    OpenAI(OpenAIEmbeddingConfig),
    Gemini(GeminiEmbeddingConfig),
}

impl EmbeddingConfig {
    pub fn extract_config(
        config: Option<&Bound<'_, PyAny>>,
        provider: &Provider,
    ) -> Result<Self, AgentError> {
        match provider {
            Provider::OpenAI => {
                let config = if config.is_none() {
                    OpenAIEmbeddingConfig::default()
                } else {
                    config
                        .unwrap()
                        .extract::<OpenAIEmbeddingConfig>()
                        .map_err(|e| {
                            AgentError::EmbeddingConfigExtractionError(format!(
                                "Failed to extract OpenAIEmbeddingConfig: {}",
                                e
                            ))
                        })?
                };

                Ok(EmbeddingConfig::OpenAI(config))
            }
            Provider::Gemini => {
                let config = if config.is_none() {
                    GeminiEmbeddingConfig::default()
                } else {
                    config
                        .unwrap()
                        .extract::<GeminiEmbeddingConfig>()
                        .map_err(|e| {
                            AgentError::EmbeddingConfigExtractionError(format!(
                                "Failed to extract GeminiEmbeddingConfig: {}",
                                e
                            ))
                        })?
                };

                Ok(EmbeddingConfig::Gemini(config))
            }
            _ => Err(AgentError::ProviderNotSupportedError(provider.to_string())),
        }
    }
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

    pub fn into_py_bound_any<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        match self {
            EmbeddingResponse::OpenAI(response) => Ok(response.into_py_bound_any(py)?),
            EmbeddingResponse::Gemini(response) => Ok(response.into_py_bound_any(py)?),
        }
    }
}

#[pyclass(name = "Embedder")]
#[derive(Debug, Clone)]
pub struct PyEmbedder {
    pub embedder: Arc<Embedder>,
    pub runtime: Arc<tokio::runtime::Runtime>,
}

#[pymethods]
impl PyEmbedder {
    #[new]
    fn new(provider: &Bound<'_, PyAny>) -> Result<Self, AgentError> {
        let provider = Provider::extract_provider(provider)?;
        let embedder = Arc::new(Embedder::new(provider).unwrap());
        Ok(Self {
            embedder,
            runtime: Arc::new(
                tokio::runtime::Runtime::new()
                    .map_err(|e| AgentError::RuntimeError(e.to_string()))?,
            ),
        })
    }

    /// Create a new embedding from a single input string
    /// # Arguments
    /// * `inputs`: The input string to embed.
    /// * `config`: The configuration for the embedding.
    pub fn create<'py>(
        &self,
        py: Python<'py>,
        input: String,
        config: Option<&Bound<'py, PyAny>>,
    ) -> Result<Bound<'py, PyAny>, AgentError> {
        let config = EmbeddingConfig::extract_config(config, &self.embedder.client.provider())?;
        let embedder = self.embedder.clone();
        let embeddings = self
            .runtime
            .block_on(async { embedder.create(vec![input], config).await })?;
        Ok(embeddings.into_py_bound_any(py)?)
    }
}
