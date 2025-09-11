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
