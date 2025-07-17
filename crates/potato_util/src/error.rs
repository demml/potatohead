use pyo3::exceptions::PyRuntimeError;
use pyo3::PyErr;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum UtilError {
    #[error("Error serializing data")]
    SerializationError,

    #[error("Error getting parent path")]
    GetParentPathError,

    #[error("Failed to create directory")]
    CreateDirectoryError,

    #[error("Failed to write to file")]
    WriteError,

    #[error("Invalid number")]
    InvalidNumber,

    #[error("Root must be an object")]
    RootMustBeObjectError,

    #[error("{0}")]
    PyError(String),

    #[error("Failed to downcast Python object: {0}")]
    DowncastError(String),

    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}

impl<'a> From<pyo3::DowncastError<'a, 'a>> for UtilError {
    fn from(err: pyo3::DowncastError) -> Self {
        UtilError::DowncastError(err.to_string())
    }
}

impl From<UtilError> for PyErr {
    fn from(err: UtilError) -> PyErr {
        let msg = err.to_string();
        error!("{}", msg);
        PyRuntimeError::new_err(msg)
    }
}

impl From<PyErr> for UtilError {
    fn from(err: PyErr) -> UtilError {
        UtilError::PyError(err.to_string())
    }
}
