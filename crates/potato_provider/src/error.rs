use gcloud_auth::error::Error as GCloudAuthError;
use potato_prompt::PromptError;
use pyo3::exceptions::PyRuntimeError;
use pyo3::PyErr;
use reqwest::StatusCode;
use thiserror::Error;
use tracing::error;
#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Error: {0}")]
    Error(String),

    #[error("Failed to downcast Python object: {0}")]
    DowncastError(String),

    #[error("Client did not provide response")]
    ClientNoResponseError,

    #[error("Failed to create header value for the agent client")]
    CreateHeaderValueError(#[from] reqwest::header::InvalidHeaderValue),

    #[error("Failed to create header name for the agent client")]
    CreateHeaderNameError(#[from] reqwest::header::InvalidHeaderName),

    #[error("Failed to create agent client: {0}")]
    CreateClientError(#[source] reqwest::Error),

    #[error("Request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to serialize chat request: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Failed to extract embedding config. Check provider and config compatibility: {0}")]
    EmbeddingConfigExtractionError(String),

    #[error("Missing authentication information. Failed to find GEMINI_API_KEY or Google credentials in environment variables.")]
    MissingAuthenticationError,

    #[error("Unsupported content type")]
    UnsupportedContentTypeError,

    #[error("Failed to get response: {0} with status code {1}")]
    CompletionError(String, StatusCode),

    #[error("Provider not supported: {0}")]
    ProviderNotSupportedError(String),

    #[error("No provider specified in GenAiClient")]
    NoProviderError,

    #[error("Undefined error: {0}")]
    UndefinedError(String),

    #[error("Invalid response type")]
    InvalidResponseType(String),

    #[error("Failed to create tokio runtime: {0}")]
    RuntimeError(String),

    #[error("No embeddings found in the response")]
    NoEmbeddingsFound,

    #[error(transparent)]
    TypeError(#[from] potato_type::error::TypeError),

    #[error(transparent)]
    PromptError(#[from] PromptError),

    #[error(transparent)]
    DecodeError(#[from] base64::DecodeError),

    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    GCloudAuthError(#[from] GCloudAuthError),

    #[error("No Google credentials found in environment variables")]
    NoCredentialsFound,

    #[error("No project ID found in credentials or environment variables")]
    NoProjectIdFound,

    #[error("Failed to retrieve access token: {0}")]
    TokenError(String),

    #[error("Failed to retrieve OPENAI_API_KEY from the environment")]
    MissingOpenAIApiKeyError,

    #[error("{0}")]
    NotImplementedError(String),

    #[error("Method does not support PredictRequest")]
    DoesNotSupportPredictRequest,

    #[error("Method does not support array inputs")]
    DoesNotSupportArray,

    #[error("{0}")]
    InvalidInputType(String),
}

impl<'a> From<pyo3::DowncastError<'a, 'a>> for ProviderError {
    fn from(err: pyo3::DowncastError) -> Self {
        ProviderError::DowncastError(err.to_string())
    }
}

impl From<ProviderError> for PyErr {
    fn from(err: ProviderError) -> PyErr {
        let msg = err.to_string();
        error!("{}", msg);
        PyRuntimeError::new_err(msg)
    }
}

impl From<PyErr> for ProviderError {
    fn from(err: PyErr) -> Self {
        ProviderError::Error(err.to_string())
    }
}
