use crate::agents::error::AgentError;
use potato_provider::ChatResponse;
use potato_util::utils::ResponseLogProbs;
use potato_util::utils::TokenLogProbs;
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentResponse {
    pub id: String,
    pub response: ChatResponse,
}

impl AgentResponse {
    pub fn token_usage<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        Ok(self.response.token_usage(py)?)
    }

    /// Returns the response as a Python object
    pub fn response<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        Ok(self.response.to_bound_py_object(py)?)
    }
}

impl AgentResponse {
    pub fn new(id: String, response: ChatResponse) -> Self {
        Self { id, response }
    }

    pub fn log_probs(&self) -> Vec<TokenLogProbs> {
        self.response.get_log_probs()
    }

    pub fn structured_output<'py>(
        &self,
        py: Python<'py>,
        output_type: Option<&Bound<'py, PyAny>>,
    ) -> Result<Bound<'py, PyAny>, AgentError> {
        Ok(self.response.structured_output(py, output_type)?)
    }

    pub fn response_text(&self) -> String {
        self.response.response_text()
    }

    pub fn response_value(&self) -> Option<Value> {
        self.response.extract_structured_data()
    }
}

#[pyclass(name = "AgentResponse")]
#[derive(Debug, Serialize)]
pub struct PyAgentResponse {
    pub inner: AgentResponse,

    #[serde(skip_serializing)]
    pub output_type: Option<Py<PyAny>>,

    #[pyo3(get)]
    pub failed_conversion: bool,
}

#[pymethods]
impl PyAgentResponse {
    #[getter]
    pub fn id(&self) -> &str {
        &self.inner.id
    }

    /// Return the token usage of the response
    #[getter]
    pub fn token_usage<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        self.inner.token_usage(py)
    }

    /// Returns the actual response object from the provider
    #[getter]
    pub fn response<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        self.inner.response(py)
    }

    #[getter]
    pub fn log_probs(&self) -> ResponseLogProbs {
        ResponseLogProbs {
            tokens: self.inner.log_probs(),
        }
    }

    #[getter]
    #[instrument(skip_all)]
    pub fn structured_output<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        let bound = self
            .output_type
            .as_ref()
            .map(|output_type| output_type.bind(py));
        self.inner.structured_output(py, bound)
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }

    pub fn response_text(&self) -> String {
        self.inner.response_text()
    }
}

impl PyAgentResponse {
    pub fn new(response: AgentResponse, output_type: Option<Py<PyAny>>) -> Self {
        Self {
            inner: response,
            output_type,
            failed_conversion: false,
        }
    }
}
