use pyo3::exceptions::PyRuntimeError;
use pyo3::PyErr;
use serde::Deserialize;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug, Deserialize)]
pub enum PromptError {
    #[error("Error: {0}")]
    Error(String),

    #[error("Missing API Key")]
    MissingAPIKey,

    #[error("Failed to serialize string")]
    SerializeError,

    #[error("Failed to deserialize string")]
    DeSerializeError,

    #[error("Failed to create path")]
    CreatePathError,

    #[error("Failed to get parent path")]
    GetParentPathError,

    #[error("Failed to create directory")]
    CreateDirectoryError,

    #[error("Failed to write to file")]
    WriteError,

    #[error("Unsupported interaction type")]
    UnsupportedInteractionType,

    #[error("Sanitization error: {0}")]
    SanitizationError(String),

    #[error("Failed to serialize Python object: {0}")]
    PySerializationError(String),
}

impl From<PromptError> for PyErr {
    fn from(err: PromptError) -> PyErr {
        PyRuntimeError::new_err(err.to_string())
    }
}

impl From<PyErr> for PromptError {
    fn from(err: PyErr) -> Self {
        PromptError::Error(err.to_string())
    }
}
