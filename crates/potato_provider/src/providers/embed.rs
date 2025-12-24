use potato_type::google::EmbeddingConfigTrait;
use potato_type::Provider;
use tracing::debug;
use tracing::instrument;

use crate::error::ProviderError;
use crate::providers::client::GenAiClient;
use crate::providers::google::GeminiClient;
use crate::providers::google::VertexClient;
use crate::providers::openai::OpenAIClient;
use crate::providers::types::ServiceType;
use potato_type::google::v1::embedding::{PredictRequest, PredictResponse};
use potato_type::google::GeminiEmbeddingConfig;
use potato_type::google::GeminiEmbeddingResponse;
use potato_type::openai::v1::embedding::{OpenAIEmbeddingConfig, OpenAIEmbeddingResponse};
use pyo3::prelude::*;
use serde::Serialize;
use std::sync::Arc;
/// Input types for embedding creation
#[derive(Debug, Clone)]
pub enum EmbeddingInput {
    Texts(Vec<String>),
    PredictRequest(PredictRequest),
}

impl From<Vec<String>> for EmbeddingInput {
    fn from(texts: Vec<String>) -> Self {
        EmbeddingInput::Texts(texts)
    }
}

impl From<PredictRequest> for EmbeddingInput {
    fn from(request: PredictRequest) -> Self {
        EmbeddingInput::PredictRequest(request)
    }
}

#[instrument(skip_all)]
pub fn modify_predict_request(
    mut request: PredictRequest,
    config: &EmbeddingConfig,
) -> PredictRequest {
    match config {
        EmbeddingConfig::OpenAI(_) => request, // OpenAI config does not apply to PredictRequest
        EmbeddingConfig::Gemini(gemini_config) => {
            // If configured, we need to modify the request
            // If not configured, return early

            if !gemini_config.is_configured {
                return request;
            }

            // Handle parameters modification
            let mut params = match &request.parameters {
                serde_json::Value::Object(map) => map.clone(),
                _ => serde_json::Map::new(),
            };

            // :predict endpoints expect dimensionality in parameters
            if let Some(dim) = gemini_config.output_dimensionality {
                params.insert(
                    "outputDimensionality".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(dim)),
                );
            }

            if let serde_json::Value::Array(ref mut instances) = request.instances {
                for instance in instances.iter_mut() {
                    if let Some(task_type) = &gemini_config.task_type {
                        if let serde_json::Value::Object(ref mut map) = instance {
                            map.entry("task_type".to_string())
                                .or_insert_with(|| serde_json::json!(task_type));
                        }
                    }
                }
            }

            request.parameters = serde_json::Value::Object(params);

            debug!("Modified PredictRequest: {:?}", request);
            request
        }
    }
}

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
                let config = match config {
                    None => OpenAIEmbeddingConfig::default(),
                    Some(cfg) => cfg.extract::<OpenAIEmbeddingConfig>().map_err(|e| {
                        ProviderError::EmbeddingConfigExtractionError(format!(
                            "Failed to extract OpenAIEmbeddingConfig: {}",
                            e
                        ))
                    })?,
                };

                Ok(EmbeddingConfig::OpenAI(config))
            }
            Provider::Gemini | Provider::Vertex => {
                let config = match config {
                    None => GeminiEmbeddingConfig::default(),
                    Some(cfg) => cfg.extract::<GeminiEmbeddingConfig>().map_err(|e| {
                        ProviderError::EmbeddingConfigExtractionError(format!(
                            "Failed to extract GeminiEmbeddingConfig: {}",
                            e
                        ))
                    })?,
                };

                Ok(EmbeddingConfig::Gemini(config))
            }
            _ => Err(ProviderError::ProviderNotSupportedError(
                provider.to_string(),
            )),
        }
    }

    pub fn is_configured(&self) -> bool {
        match self {
            // is configured only applies to Gemini and Vertex at the moment
            EmbeddingConfig::OpenAI(_config) => true,
            EmbeddingConfig::Gemini(config) => config.is_configured,
        }
    }

    pub fn get_vertex_config(&self) -> Result<serde_json::Value, ProviderError> {
        match self {
            EmbeddingConfig::Gemini(config) => Ok(config.get_parameters_for_predict()),
            _ => Err(ProviderError::InvalidConfigType(
                "Only Gemini config can be converted to Vertex config".to_string(),
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
    provider: Provider,
}

impl Embedder {
    /// Create a new Embedder instance that can be used to generate embeddings.
    /// # Arguments
    /// * `provider`: The provider to use for generating embeddings.
    /// * `config`: The configuration for the embedding.
    pub async fn new(provider: Provider, config: EmbeddingConfig) -> Result<Self, ProviderError> {
        let client = match provider {
            Provider::OpenAI => GenAiClient::OpenAI(OpenAIClient::new(ServiceType::Embed)?),
            Provider::Gemini => GenAiClient::Gemini(GeminiClient::new(ServiceType::Embed).await?),
            Provider::Vertex => GenAiClient::Vertex(VertexClient::new(ServiceType::Embed).await?),
            _ => {
                let msg = "No provider specified";
                error!("{}", msg);
                return Err(ProviderError::UndefinedError(msg.to_string()));
            } // Add other providers here as needed
        };

        Ok(Self {
            client,
            config,
            provider,
        })
    }

    pub async fn embed(&self, inputs: EmbeddingInput) -> Result<EmbeddingResponse, ProviderError> {
        // Implementation for creating an embedding
        self.client.create_embedding(inputs, &self.config).await
    }
}

pub enum EmbeddingResponse {
    OpenAI(OpenAIEmbeddingResponse),
    Gemini(GeminiEmbeddingResponse),
    Vertex(PredictResponse),
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
            _ => Err(ProviderError::InvalidResponseType(
                "values not available for this response type".to_string(),
            )),
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
        input: Bound<'py, PyAny>,
    ) -> Result<Bound<'py, PyAny>, ProviderError> {
        let embedder = self.embedder.clone();

        let embedding_input = if input.is_instance_of::<pyo3::types::PyList>() {
            let texts: Vec<String> = input.extract()?;
            EmbeddingInput::Texts(texts)
        } else if input.is_instance_of::<pyo3::types::PyString>() {
            let request: String = input.extract()?;
            EmbeddingInput::Texts(vec![request])
        } else if input.is_instance_of::<PredictRequest>() {
            let request: PredictRequest = input.extract()?;
            EmbeddingInput::PredictRequest(request)
        } else {
            return Err(ProviderError::InvalidInputType(
                "Input must be a string, list of strings, or PredictRequest dict".to_string(),
            ));
        };

        let embeddings = self
            .runtime
            .block_on(async { embedder.embed(embedding_input).await })?;
        embeddings.into_py_bound_any(py)
    }
}
