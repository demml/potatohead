use potato_util::UtilError;
use pyo3::exceptions::PyRuntimeError;
use pyo3::PyErr;
use pythonize::PythonizeError;
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

    #[error(transparent)]
    UtilError(#[from] UtilError),

    #[error("Failed to create runtime: {0}")]
    RuntimeError(String),

    #[error("Invalid output type provided for task: {0}")]
    InvalidOutputType(String),

    #[error(transparent)]
    SerializationError(#[from] serde_json::Error),

    #[error("Failed to acquire lock on workflow")]
    LockAcquireError,

    #[error("Failed to acquire read lock on workflow")]
    ReadLockAcquireError,

    #[error("Failed to acquire write lock on workflow")]
    WriteLockAcquireError,

    #[error(transparent)]
    AgentError(#[from] potato_agent::AgentError),

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Task lock error")]
    TaskLockError,

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Response validation against JSON schema failed for task: {task_id}. Expected schema: {expected_schema}, Received response: {received_response}")]
    ResponseValidationFailed {
        task_id: String,
        expected_schema: String,
        received_response: String,
    },
}
impl From<PythonizeError> for WorkflowError {
    fn from(err: PythonizeError) -> Self {
        WorkflowError::Error(err.to_string())
    }
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
