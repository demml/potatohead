use crate::agents::error::AgentError;
use crate::agents::provider::openai::OpenAIChatResponse;
use crate::agents::provider::openai::ToolCall;
use crate::agents::provider::openai::Usage;
use potato_prompt::{
    prompt::{PromptContent, Role},
    Message,
};
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

    /// Get the content of the first choice in the chat response
    pub fn content(&self) -> Option<&String> {
        match self {
            ChatResponse::OpenAI(resp) => resp
                .choices
                .first()
                .and_then(|c| c.message.content.as_ref()),
        }
    }

    /// Check for tool calls in the chat response
    pub fn tool_calls(&self) -> Option<&Vec<ToolCall>> {
        match self {
            ChatResponse::OpenAI(resp) => {
                let tool_calls: Option<&Vec<ToolCall>> = resp
                    .choices
                    .first()
                    .and_then(|c| Some(c.message.tool_calls.as_ref()));
                tool_calls
            }
        }
    }

    /// Extracts structured data from a chat response
    pub fn extract_structured_data(&self) -> Option<Value> {
        // Check for JSON in content field
        if let Some(content) = self.content() {
            let trimmed = content.trim();
            if (trimmed.starts_with('{') && trimmed.ends_with('}'))
                || (trimmed.starts_with('[') && trimmed.ends_with(']'))
            {
                return serde_json::from_str(trimmed).ok();
            }
        }

        // Check for tool calls
        if let Some(tool_calls) = &self.tool_calls() {
            if !tool_calls.is_empty() {
                return serde_json::to_value(tool_calls).ok();
            }
        }

        None
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
