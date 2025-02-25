use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::PyErr;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug, Deserialize)]
pub enum HttpError {
    #[error("Error: {0}")]
    Error(String),
}

#[derive(Error, Debug, Deserialize)]
pub enum PotatoError {
    #[error("Error: {0}")]
    Error(String),

    #[error("Missing API Key")]
    MissingAPIKey,

    #[error(transparent)]
    HttpError(#[from] HttpError),

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
}

impl From<PotatoError> for PyErr {
    fn from(err: PotatoError) -> PyErr {
        PyErr::new::<potato_headError, _>(err.to_string())
    }
}

impl From<PyErr> for PotatoError {
    fn from(err: PyErr) -> Self {
        PotatoError::Error(err.to_string())
    }
}

create_exception!(potato_head, potato_headError, PyException);
