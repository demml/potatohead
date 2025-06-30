use pyo3::exceptions::PyRuntimeError;
use pyo3::PyErr;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum MockError {
    #[error("Error: {0}")]
    Error(String),
}

impl From<MockError> for PyErr {
    fn from(err: MockError) -> PyErr {
        let msg = err.to_string();
        error!("{}", msg);
        PyRuntimeError::new_err(msg)
    }
}

impl From<PyErr> for MockError {
    fn from(err: PyErr) -> Self {
        MockError::Error(err.to_string())
    }
}
