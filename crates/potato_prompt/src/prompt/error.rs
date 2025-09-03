use pyo3::exceptions::PyRuntimeError;
use pyo3::PyErr;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
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

    #[error("Failed to parse Python object: {0}")]
    ParseError(String),

    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),

    #[error(transparent)]
    UtilError(#[from] potato_util::UtilError),

    #[error(transparent)]
    TypeError(#[from] potato_type::TypeError),

    #[error("Invalid settings provided. ModelSettings expects either OpenAIChatSettings or GeminiSettings.")]
    InvalidModelSettings,

    #[error("Invalid provider provided. Provider must be either a Provider enum or a string.")]
    InvalidProvider,
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
