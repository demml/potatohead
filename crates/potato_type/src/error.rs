use pyo3::exceptions::PyRuntimeError;
use pyo3::pyclass::PyClassGuardError;
use pyo3::PyErr;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum TypeError {
    #[error("Error: {0}")]
    Error(String),

    #[error("Unknown provider: {0}")]
    UnknownProviderError(String),

    #[error("Unknown model: {0}")]
    UnknownModelError(String),

    #[error("{0}")]
    InvalidInput(String),

    #[error(transparent)]
    UtilError(#[from] potato_util::UtilError),

    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    #[error(transparent)]
    StdError(#[from] std::io::Error),

    #[error("Failed to create GeminiEmbeddingConfig: {0}")]
    GeminiEmbeddingConfigError(String),

    #[error("Invalid media type: {0}")]
    InvalidMediaType(String),

    #[error("Unsupported prompt content type")]
    UnsupportedTypeError,

    #[error("Cannot bind non-string content")]
    CannotBindNonStringContent,

    #[error("Failed to serialize Python object: {0}")]
    PySerializationError(String),

    #[error("Content type is not supported")]
    UnsupportedContentType,

    #[error("Invalid model settings provided. ModelSettings expects either OpenAIChatSettings, GeminiSettings, or AnthropicSettings.")]
    InvalidModelSettings,

    #[error("Expected string, Message, or list of messages")]
    MessageParseError,

    #[error("Invalid message type in list. Received: {0}")]
    InvalidMessageTypeInList(String),

    #[error("Either 'auto_mode' or 'manual_mode' must be provided, but not both.")]
    MissingRoutingConfigMode,

    #[error("Invalid data type for Part. Expected String, Blob, FileData, FunctionCall, FunctionResponse, ExecutableCode, or CodeExecutionResult. Got: {0}")]
    InvalidDataType(String),

    #[error("Invalid list type for Part. Expected a list of Strings or a list of Message objects. Got: {0}")]
    InvalidListType(String),

    #[error("Parts must be a string, Part, DataNum variant, or a list of these types")]
    InvalidPartType,

    #[error("Invalid ranking config provided.")]
    InvalidRankingConfig,

    #[error("Invalid retrieval source provided.")]
    InvalidRetrievalSource,

    #[error("Invalid authentication configuration provided.")]
    InvalidAuthConfig,

    #[error("Invalid OAuth configuration provided.")]
    InvalidOauthConfig,

    #[error("Invalid OIDC configuration provided.")]
    InvalidOidcConfig,

    #[error("More than one system instruction provided where only one is allowed.")]
    MoreThanOneSystemInstruction,
}

impl From<TypeError> for PyErr {
    fn from(err: TypeError) -> PyErr {
        let msg = err.to_string();
        error!("{}", msg);
        PyRuntimeError::new_err(msg)
    }
}

impl From<PyErr> for TypeError {
    fn from(err: PyErr) -> Self {
        TypeError::Error(err.to_string())
    }
}

impl<'a, 'py> From<PyClassGuardError<'a, 'py>> for TypeError {
    fn from(err: PyClassGuardError<'a, 'py>) -> Self {
        TypeError::Error(err.to_string())
    }
}

impl<'a, 'py> From<pyo3::CastError<'a, 'py>> for TypeError {
    fn from(err: pyo3::CastError<'a, 'py>) -> Self {
        TypeError::Error(err.to_string())
    }
}
