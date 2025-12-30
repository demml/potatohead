use crate::AgentError;
use crate::{AgentResponse, PyAgentResponse};
use potato_type::prompt::Prompt;
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pyclass(eq)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[pyclass]
#[derive(Debug, Serialize)]
pub struct WorkflowTask {
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
    #[pyo3(get)]
    pub max_retries: u32,
    pub result: Option<PyAgentResponse>,
    pub retry_count: u32,
}

#[pymethods]
impl WorkflowTask {
    #[getter]
    pub fn result<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        if let Some(resp) = &self.result {
            let output = resp.structured_output(py)?;
            Ok(output)
        } else {
            Ok(py.None().bind(py).clone())
        }
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Task {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get, set)]
    pub prompt: Prompt,
    #[pyo3(get, set)]
    pub dependencies: Vec<String>,
    #[pyo3(get)]
    pub status: TaskStatus,
    #[pyo3(get, set)]
    pub agent_id: String,
    pub result: Option<AgentResponse>,
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

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

impl Task {
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
    }

    pub fn set_result(&mut self, result: AgentResponse) {
        self.result = Some(result);
    }
}
