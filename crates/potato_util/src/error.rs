use pyo3::exceptions::PyRuntimeError;
use pyo3::PyErr;
use pythonize::PythonizeError;
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

    #[error("Failed to check if the context is a Pydantic BaseModel. Error: {0}")]
    FailedToCheckPydanticModel(String),
}

impl From<PythonizeError> for UtilError {
    fn from(err: PythonizeError) -> Self {
        UtilError::PyError(err.to_string())
    }
}

impl<'a, 'py> From<pyo3::CastError<'a, 'py>> for UtilError {
    fn from(err: pyo3::CastError) -> Self {
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
