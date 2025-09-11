use potato_type::google::predict::PredictResponse;
use potato_type::google::EmbeddingConfigTrait;
use potato_type::Provider;

use crate::error::ProviderError;
use crate::providers::client::GenAiClient;
use crate::providers::google::GeminiClient;
use crate::providers::openai::OpenAIClient;
use crate::providers::types::ServiceType;
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
    ) -> Result<Self, ProviderError> {
        match provider {
            Provider::OpenAI => {
                let config = if config.is_none() {
                    OpenAIEmbeddingConfig::default()
                } else {
                    config
                        .unwrap()
                        .extract::<OpenAIEmbeddingConfig>()
                        .map_err(|e| {
                            ProviderError::EmbeddingConfigExtractionError(format!(
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
                            ProviderError::EmbeddingConfigExtractionError(format!(
                                "Failed to extract GeminiEmbeddingConfig: {}",
                                e
                            ))
                        })?
                };

                Ok(EmbeddingConfig::Gemini(config))
            }
            _ => Err(ProviderError::ProviderNotSupportedError(
                provider.to_string(),
            )),
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
#[derive(Debug, PartialEq)]
pub struct Embedder {
    client: GenAiClient,
    config: EmbeddingConfig,
}

impl Embedder {
    /// Create a new Embedder instance that can be used to generate embeddings.
    /// # Arguments
    /// * `provider`: The provider to use for generating embeddings.
    /// * `config`: The configuration for the embedding.
    pub async fn new(provider: Provider, config: EmbeddingConfig) -> Result<Self, ProviderError> {
        let client = match provider {
            Provider::OpenAI => GenAiClient::OpenAI(OpenAIClient::new(None, None, None)?),
            Provider::Gemini => {
                GenAiClient::Gemini(GeminiClient::new(None, ServiceType::Embed).await?)
            }
            _ => {
                let msg = "No provider specified in ModelSettings";
                error!("{}", msg);
                return Err(ProviderError::UndefinedError(msg.to_string()));
            } // Add other providers here as needed
        };

        Ok(Self { client, config })
    }

    pub async fn embed(&self, inputs: Vec<String>) -> Result<EmbeddingResponse, ProviderError> {
        // Implementation for creating an embedding
        self.client.create_embedding(inputs, &self.config).await
    }
}

pub enum EmbeddingResponse {
    OpenAI(OpenAIEmbeddingResponse),
    Gemini(GeminiEmbeddingResponse),
    Vertex(PredictResponse), // Vertex uses the same response structure as Gemini
}

impl EmbeddingResponse {
    pub fn to_openai_response(&self) -> Result<&OpenAIEmbeddingResponse, ProviderError> {
        match self {
            EmbeddingResponse::OpenAI(response) => Ok(response),
            _ => Err(ProviderError::InvalidResponseType("OpenAI".to_string())),
        }
    }

    pub fn to_gemini_response(&self) -> Result<&GeminiEmbeddingResponse, ProviderError> {
        match self {
            EmbeddingResponse::Gemini(response) => Ok(response),
            _ => Err(ProviderError::InvalidResponseType("Gemini".to_string())),
        }
    }

    pub fn to_vertex_response(&self) -> Result<&PredictResponse, ProviderError> {
        match self {
            EmbeddingResponse::Vertex(response) => Ok(response),
            _ => Err(ProviderError::InvalidResponseType("Vertex".to_string())),
        }
    }

    pub fn into_py_bound_any<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyAny>, ProviderError> {
        match self {
            EmbeddingResponse::OpenAI(response) => Ok(response.into_py_bound_any(py)?),
            EmbeddingResponse::Gemini(response) => Ok(response.into_py_bound_any(py)?),
            EmbeddingResponse::Vertex(response) => Ok(response.into_py_bound_any(py)?),
        }
    }

    pub fn values(&self) -> Result<&Vec<f32>, ProviderError> {
        match self {
            EmbeddingResponse::OpenAI(response) => {
                let first = response
                    .data
                    .first()
                    .ok_or_else(|| ProviderError::NoEmbeddingsFound)?;
                Ok(&first.embedding)
            }

            EmbeddingResponse::Gemini(response) => Ok(&response.embedding.values),
            EmbeddingResponse::Vertex(response) => Ok(&response.predictions),
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
    #[pyo3(signature = (provider, config=None))]
    fn new(
        provider: &Bound<'_, PyAny>,
        config: Option<&Bound<'_, PyAny>>,
    ) -> Result<Self, ProviderError> {
        let provider = Provider::extract_provider(provider)?;
        let config = EmbeddingConfig::extract_config(config, &provider)?;
        let runtime = Arc::new(
            tokio::runtime::Runtime::new()
                .map_err(|e| ProviderError::RuntimeError(e.to_string()))?,
        );
        let embedder = runtime.block_on(async { Embedder::new(provider, config).await })?;

        Ok(Self {
            embedder: Arc::new(embedder),
            runtime,
        })
    }

    /// Create a new embedding from a single input string
    /// # Arguments
    /// * `inputs`: The input string to embed.
    /// * `config`: The configuration for the embedding.
    #[pyo3(signature = (input))]
    pub fn embed<'py>(
        &self,
        py: Python<'py>,
        input: String,
    ) -> Result<Bound<'py, PyAny>, ProviderError> {
        let embedder = self.embedder.clone();
        let embeddings = self
            .runtime
            .block_on(async { embedder.embed(vec![input]).await })?;
        embeddings.into_py_bound_any(py)
    }
}
