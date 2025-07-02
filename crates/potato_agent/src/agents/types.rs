use crate::agents::error::AgentError;
use crate::agents::provider::openai::OpenAIChatResponse;
use crate::agents::provider::openai::Usage;
use potato_prompt::{
    prompt::{PromptContent, Role},
    Message,
};
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatResponse {
    OpenAI(OpenAIChatResponse),
}

impl ChatResponse {
    pub fn is_empty(&self) -> bool {
        match self {
            ChatResponse::OpenAI(resp) => resp.choices.is_empty(),
        }
    }

    pub fn to_message(&self, role: Role) -> Result<Vec<Message>, AgentError> {
        match self {
            ChatResponse::OpenAI(resp) => {
                let first_choice = resp
                    .choices
                    .first()
                    .ok_or_else(|| AgentError::ClientNoResponseError)?;

                let message =
                    PromptContent::Str(first_choice.message.content.clone().unwrap_or_default());
                Ok(vec![Message::from(message, role)])
            }
        }
    }

    pub fn to_python<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        match self {
            ChatResponse::OpenAI(resp) => Ok(resp.clone().into_bound_py_any(py)?),
        }
    }

    pub fn id(&self) -> String {
        match self {
            ChatResponse::OpenAI(resp) => resp.id.clone(),
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    #[pyo3(get)]
    pub id: String,
    pub response: ChatResponse,
}

#[pymethods]
impl AgentResponse {
    #[getter]
    pub fn output(&self) -> String {
        match &self.response {
            ChatResponse::OpenAI(resp) => resp.choices.first().map_or("".to_string(), |c| {
                c.message.content.clone().unwrap_or_default()
            }),
        }
    }

    #[getter]
    pub fn token_usage(&self) -> Usage {
        match &self.response {
            ChatResponse::OpenAI(resp) => resp.usage.clone(),
        }
    }
}

impl AgentResponse {
    pub fn new(id: String, response: ChatResponse) -> Self {
        Self { id, response }
    }
}

#[pyclass(name = "AgentResponse")]
#[derive(Debug, Serialize)]
pub struct PyAgentResponse {
    pub response: AgentResponse,

    #[serde(skip_serializing)]
    pub output_type: Option<PyObject>,
}

#[pymethods]
impl PyAgentResponse {
    #[getter]
    pub fn id(&self) -> &str {
        &self.response.id
    }
    #[getter]
    pub fn output(&self) -> String {
        self.response.output()
    }

    #[getter]
    pub fn token_usage(&self) -> Usage {
        self.response.token_usage()
    }

    #[getter]
    pub fn result<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let msg = self.response.output();

        match &self.output_type {
            Some(output_type) => {
                // Convert structured output using model_validate_json
                output_type
                    .bind(py)
                    .call_method1("model_validate_json", (msg,))
            }
            None => {
                // Convert plain string output to Python string
                Ok(msg.into_bound_py_any(py)?)
            }
        }
    }
}

impl PyAgentResponse {
    pub fn new(response: AgentResponse, output_type: Option<PyObject>) -> Self {
        Self {
            response,
            output_type,
        }
    }
}
