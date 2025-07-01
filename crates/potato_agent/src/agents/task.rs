use crate::agents::types::ChatResponse;
use potato_prompt::Prompt;
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[pyclass]
#[derive(Debug, Serialize)]
pub struct PyTask {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get, set)]
    pub prompt: Prompt,
    #[pyo3(get, set)]
    pub dependencies: Vec<String>,
    #[pyo3(get)]
    pub status: TaskStatus,
    #[pyo3(get)]
    pub agent_id: String,
    pub result: Option<ChatResponse>,
    #[pyo3(get)]
    pub max_retries: u32,
    pub retry_count: u32,

    #[serde(skip)]
    pub response_type: Option<Arc<PyObject>>,
}

#[pymethods]
impl PyTask {
    #[getter]
    pub fn result<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        match &self.result {
            Some(resp) => Ok(resp.to_python(py).map(Some)?),
            None => Ok(None),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get, set)]
    pub prompt: Prompt,
    #[pyo3(get, set)]
    pub dependencies: Vec<String>,
    #[pyo3(get)]
    pub status: TaskStatus,
    #[pyo3(get)]
    pub agent_id: String,
    pub result: Option<ChatResponse>,
    #[pyo3(get)]
    pub max_retries: u32,
    pub retry_count: u32,
}

#[pymethods]
impl Task {
    #[new]
    #[pyo3(signature = (agent_id, prompt, id, dependencies = None, max_retries=None))]
    pub fn new(
        agent_id: &str,
        prompt: Prompt,
        id: &str,
        dependencies: Option<Vec<String>>,
        max_retries: Option<u32>,
    ) -> Self {
        Self {
            prompt,
            dependencies: dependencies.unwrap_or_default(),
            status: TaskStatus::Pending,
            result: None,
            id: id.to_string(),
            agent_id: agent_id.to_string(),
            max_retries: max_retries.unwrap_or(3),
            retry_count: 0,
        }
    }

    pub fn add_dependency(&mut self, dependency: String) {
        self.dependencies.push(dependency);
    }

    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
    }

    pub fn set_result(&mut self, result: ChatResponse) {
        self.result = Some(result);
    }

    #[getter]
    pub fn result<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        match &self.result {
            Some(resp) => Ok(resp.to_python(py).map(Some)?),
            None => Ok(None),
        }
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

impl Task {
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}
