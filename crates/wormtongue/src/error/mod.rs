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
}

impl From<TongueError> for PyErr {
    fn from(err: TongueError) -> PyErr {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(err.to_string())
    }
}

create_exception!(wormtongue, WormTongueError, PyException);
