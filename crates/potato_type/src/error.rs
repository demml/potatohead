use pyo3::exceptions::PyRuntimeError;
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
