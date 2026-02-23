use pyo3::exceptions::PyRuntimeError;
use pyo3::PyErr;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Error: {0}")]
    Error(String),

    #[error("Failed to downcast Python object: {0}")]
    DowncastError(String),

    #[error("Failed to get environment variable: {0}")]
    EnvVarError(#[from] std::env::VarError),

    #[error("Failed to extract client: {0}")]
    ClientExtractionError(String),

    #[error("No ready tasks found but pending tasks remain. Possible circular dependency.")]
    NoTaskFoundError,

    #[error("Failed to create runtime: {0}")]
    CreateRuntimeError(#[source] std::io::Error),

    #[error(transparent)]
    SerializationError(#[from] serde_json::Error),

    #[error(transparent)]
    UtilError(#[from] potato_util::UtilError),

    #[error("Invalid output type: {0}")]
    InvalidOutputType(String),

    #[error("Failed to create tool: {0}")]
    ToolCreationError(String),

    #[error("Invalid tool definition")]
    InvalidToolDefinitionError,

    #[error("{0}")]
    InvalidInput(String),

    #[error("Provider mismatch: prompt provider {0}, agent provider {1}")]
    ProviderMismatch(String, String),

    #[error(transparent)]
    ProviderError(#[from] potato_provider::error::ProviderError),

    #[error("No provider specified for Agent")]
    MissingProviderError,

    #[error(transparent)]
    TypeError(#[from] potato_type::TypeError),

    #[error(transparent)]
    StdIoError(#[from] std::io::Error),

    #[error("Not supported: {0}")]
    NotSupportedError(String),

    #[error("Output validation error: {0}")]
    ValidationError(String),
}

impl<'a, 'py> From<pyo3::CastError<'a, 'py>> for AgentError {
    fn from(err: pyo3::CastError<'a, 'py>) -> Self {
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
