use potato_prompt::PromptError;
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

    #[error("Failed to retrieve GEMINI_API_KEY from the environment")]
    MissingGeminiApiKeyError,

    #[error("Failed to retrieve OPENAI_API_KEY from the environment")]
    MissingOpenAIApiKeyError,

    #[error("Failed to extract client: {0}")]
    ClientExtractionError(String),

    #[error("No ready tasks found but pending tasks remain. Possible circular dependency.")]
    NoTaskFoundError,

    #[error("Failed to create runtime: {0}")]
    CreateRuntimeError(#[source] std::io::Error),

    #[error(transparent)]
    PromptError(#[from] PromptError),

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
    GoogleError(#[from] crate::agents::provider::google::error::GoogleError),
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
