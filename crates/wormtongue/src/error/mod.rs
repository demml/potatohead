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
pub enum TongueError {
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

impl From<TongueError> for PyErr {
    fn from(err: TongueError) -> PyErr {
        PyErr::new::<WormTongueError, _>(err.to_string())
    }
}

create_exception!(wormtongue, WormTongueError, PyException);
