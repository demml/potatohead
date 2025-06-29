use pyo3::exceptions::PyRuntimeError;
use pyo3::PyErr;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("Error: {0}")]
    Error(String),

    #[error("Max retries exceeded for task: {0}")]
    MaxRetriesExceeded(String),
}

impl From<WorkflowError> for PyErr {
    fn from(err: WorkflowError) -> PyErr {
        let msg = err.to_string();
        error!("{}", msg);
        PyRuntimeError::new_err(msg)
    }
}

impl From<PyErr> for WorkflowError {
    fn from(err: PyErr) -> Self {
        WorkflowError::Error(err.to_string())
    }
}
