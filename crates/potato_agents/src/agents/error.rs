use pyo3::exceptions::PyRuntimeError;
use pyo3::PyErr;
use reqwest::StatusCode;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Error: {0}")]
    Error(String),

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

    #[error("Failed to get chat completion response: {0} with status code {1}")]
    ChatCompletionError(String, StatusCode),

    #[error("Failed to downcast Python object: {0}")]
    DowncastError(String),

    #[error("Failed to get environment variable: {0}")]
    EnvVarError(#[from] std::env::VarError),

    #[error("Failed to retrieve OPENAI_API_KEY from the environment: {0}")]
    MissingOpenAIApiKeyError(#[source] std::env::VarError),

    #[error("Failed to extract client: {0}")]
    ClientExtractionError(String),

    #[error("Client did not provide response")]
    ClientNoResponseError,

    #[error("No ready tasks found but pending tasks remain. Possible circular dependency.")]
    NoTaskFoundError,

    #[error("Unsupported content type")]
    UnsupportedContentTypeError,

    #[error("Unknown provider: {0}")]
    UnknownProviderError(String),
}

impl<'a> From<pyo3::DowncastError<'a, 'a>> for AgentError {
    fn from(err: pyo3::DowncastError) -> Self {
        AgentError::DowncastError(err.to_string())
    }
}

impl From<AgentError> for PyErr {
    fn from(err: AgentError) -> PyErr {
        let msg = err.to_string();
        error!("{}", msg);
        PyRuntimeError::new_err(msg)
    }
}

impl From<PyErr> for AgentError {
    fn from(err: PyErr) -> Self {
        AgentError::Error(err.to_string())
    }
}
