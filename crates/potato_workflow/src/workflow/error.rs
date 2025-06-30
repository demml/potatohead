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

    #[error("Task id already exists: {0}")]
    TaskAlreadyExists(String),

    #[error("Task dependency not found in registered tasks: {0}")]
    DependencyNotFound(String),

    #[error("Task not cannot depend on itself: {0}")]
    TaskDependsOnItself(String),
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
