use crate::AgentError;
use crate::{AgentResponse, PyAgentResponse};
use potato_type::prompt::Prompt;
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, instrument};
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
    #[pyo3(get, set)]
    pub agent_id: String,
    pub result: Option<AgentResponse>,
    #[pyo3(get)]
    pub max_retries: u32,
    pub retry_count: u32,

    #[serde(skip)]
    output_validator: Option<jsonschema::Validator>,
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.prompt == other.prompt
            && self.dependencies == other.dependencies
            && self.status == other.status
            && self.agent_id == other.agent_id
            && self.max_retries == other.max_retries
            && self.retry_count == other.retry_count
    }
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
        let validator = match prompt.response_json_schema() {
            Some(schema) => {
                let compiled_validator = jsonschema::validator_for(&schema).unwrap();
                Some(compiled_validator)
            }
            None => None,
        };

        Self {
            prompt,
            dependencies: dependencies.unwrap_or_default(),
            status: TaskStatus::Pending,
            result: None,
            id: id.to_string(),
            agent_id: agent_id.to_string(),
            max_retries: max_retries.unwrap_or(3),
            retry_count: 0,
            output_validator: validator,
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

    /// Helper to rebuild the validator when workflow is deserialized
    pub fn rebuild_validator(&mut self) {
        if let Some(schema) = self.prompt.response_json_schema() {
            let compiled_validator = jsonschema::validator_for(&schema).unwrap();
            self.output_validator = Some(compiled_validator);
        } else {
            self.output_validator = None;
        }
    }

    /// Validate the output against the task's output schema, if defined.
    /// Make come back to this later and change. Still unsure if this is the right place
    #[instrument(skip_all)]
    pub fn validate_output(&self, output: &Value) -> Result<(), AgentError> {
        if let Some(validator) = &self.output_validator {
            validator.validate(output).map_err(|e| {
                error!(
                    "Failed to validate output: {}, Received output: {:?}",
                    e, output
                );
                AgentError::ValidationError(e.to_string())
            })
        } else {
            Ok(())
        }
    }
}
