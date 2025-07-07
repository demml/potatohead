use crate::agents::error::AgentError;
use crate::agents::provider::openai::OpenAIChatResponse;
use crate::agents::provider::openai::ToolCall;
use crate::agents::provider::openai::Usage;
use potato_prompt::{
    prompt::{PromptContent, Role},
    Message,
};
use potato_util::{json_to_pyobject, PyHelperFuncs};
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;
use tracing::instrument;
use tracing::warn;

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatResponse {
    OpenAI(OpenAIChatResponse),
}

#[pymethods]
impl ChatResponse {
    pub fn to_py<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        // try unwrapping the prompt, if it exists
        match self {
            ChatResponse::OpenAI(resp) => Ok(resp.clone().into_bound_py_any(py)?),
        }
    }
    pub fn __str__(&self) -> String {
        match self {
            ChatResponse::OpenAI(resp) => PyHelperFuncs::__str__(resp),
        }
    }
}

impl ChatResponse {
    pub fn is_empty(&self) -> bool {
        match self {
            ChatResponse::OpenAI(resp) => resp.choices.is_empty(),
        }
    }

    #[instrument(skip_all)]
    pub fn to_message(&self, role: Role) -> Result<Vec<Message>, AgentError> {
        debug!("Converting chat response to message with role");
        match self {
            ChatResponse::OpenAI(resp) => {
                let first_choice = resp
                    .choices
                    .first()
                    .ok_or_else(|| AgentError::ClientNoResponseError)?;

                let content = PromptContent::Str(first_choice.message.content.to_string());

                Ok(vec![Message::from(content, role)])
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
    pub fn content(&self) -> Option<&Value> {
        match self {
            ChatResponse::OpenAI(resp) => resp.choices.first().map(|c| &c.message.content),
        }
    }

    /// Check for tool calls in the chat response
    pub fn tool_calls(&self) -> Option<&Vec<ToolCall>> {
        match self {
            ChatResponse::OpenAI(resp) => {
                let tool_calls: Option<&Vec<ToolCall>> =
                    resp.choices.first().map(|c| c.message.tool_calls.as_ref());
                tool_calls
            }
        }
    }

    /// Extracts structured data from a chat response
    pub fn extract_structured_data(&self) -> Option<Value> {
        if let Some(content) = self.content() {
            match content {
                Value::String(s) => {
                    let trimmed = s.trim();
                    // Check if the string is a JSON object or array
                    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
                        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
                    {
                        serde_json::from_str(trimmed).ok()
                    } else {
                        None
                    }
                }
                // If array or object, return as is
                Value::Array(_) | Value::Object(_) => Some(content.clone()),
                _ => None,
            }
        } else if let Some(tool_calls) = self.tool_calls() {
            if !tool_calls.is_empty() {
                serde_json::to_value(tool_calls).ok()
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentResponse {
    #[pyo3(get)]
    pub id: String,
    pub response: ChatResponse,
}

#[pymethods]
impl AgentResponse {
    #[getter]
    pub fn token_usage(&self) -> Usage {
        match &self.response {
            ChatResponse::OpenAI(resp) => resp.usage.clone(),
        }
    }

    pub fn result<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        let pyobj = json_to_pyobject(py, &self.content())?;

        // Convert plain string output to Python string
        Ok(pyobj.into_bound_py_any(py)?)
    }
}

impl AgentResponse {
    pub fn new(id: String, response: ChatResponse) -> Self {
        Self { id, response }
    }

    pub fn content(&self) -> Value {
        match &self.response {
            ChatResponse::OpenAI(resp) => resp
                .choices
                .first()
                .map_or(Value::Null, |c| c.message.content.clone()),
        }
    }
}

#[pyclass(name = "AgentResponse")]
#[derive(Debug, Serialize)]
pub struct PyAgentResponse {
    pub response: AgentResponse,

    #[serde(skip_serializing)]
    pub output_type: Option<PyObject>,

    #[pyo3(get)]
    pub failed_conversion: bool,
}

#[pymethods]
impl PyAgentResponse {
    #[getter]
    pub fn id(&self) -> &str {
        &self.response.id
    }

    #[getter]
    pub fn token_usage(&self) -> Usage {
        self.response.token_usage()
    }

    /// This will map a the content of the response to a python object.
    /// A python object in this case will be either a passed pydantic model or support potatohead types.
    /// If neither is porvided, an attempt is made to parse the serde Value into an appropriate Python type.
    /// Types:
    /// - Serde Null -> Python None
    /// - Serde Bool -> Python bool
    /// - Serde String -> Python str
    /// - Serde Number -> Python int or float
    /// - Serde Array -> Python list (with each item converted to Python type)
    /// - Serde Object -> Python dict (with each key-value pair converted to Python type)
    #[getter]
    #[instrument(skip_all)]
    pub fn result<'py>(&mut self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        let content_value = self.response.content();
        // convert content_value to string

        match &self.output_type {
            Some(output_type) => {
                // Match the value. For loading into pydantic models, it's expected that the api response is a JSON string.
                match content_value {
                    Value::String(s) => {
                        // If the content is a string, we can directly convert it to a Python string
                        let bound = output_type
                            .bind(py)
                            .call_method1("model_validate_json", (&s,))?;
                        Ok(bound)
                    }
                    Value::Object(_) => {
                        // Attempt to convert content_value to a JSON string
                        let content_string = serde_json::to_string(&content_value)?;
                        let bound = output_type
                            .bind(py)
                            .call_method1("model_validate_json", (&content_string,))?;
                        Ok(bound)
                    }

                    _ => {
                        warn!(
                            "Expected a string for model validation, but got: {:?}. Defaulting to JSON conversion.",
                            content_value
                        );
                        self.failed_conversion = true;
                        Ok(json_to_pyobject(py, &content_value)?.into_bound_py_any(py)?)
                    }
                }
                // Convert structured output using model_validate_json
            }
            None => {
                // Convert plain string output to Python string
                Ok(json_to_pyobject(py, &content_value)?.into_bound_py_any(py)?)
            }
        }
    }
}

impl PyAgentResponse {
    pub fn new(response: AgentResponse, output_type: Option<PyObject>) -> Self {
        Self {
            response,
            output_type,
            failed_conversion: false,
        }
    }
}
