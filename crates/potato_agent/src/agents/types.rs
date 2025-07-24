use crate::agents::error::AgentError;
use crate::agents::provider::gemini::FunctionCall;
use crate::agents::provider::gemini::GenerateContentResponse;
use crate::agents::provider::openai::OpenAIChatResponse;
use crate::agents::provider::openai::ToolCall;
use crate::agents::provider::openai::Usage;
use crate::agents::provider::traits::LogProbExt;
use crate::agents::provider::traits::ResponseExt;
use potato_prompt::{
    prompt::{PromptContent, Role},
    Message,
};
use potato_util::utils::{LogProbs, ResponseLogProbs};
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
    Gemini(GenerateContentResponse),
}

#[pymethods]
impl ChatResponse {
    pub fn to_py<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        // try unwrapping the prompt, if it exists
        match self {
            ChatResponse::OpenAI(resp) => Ok(resp.clone().into_bound_py_any(py)?),
            ChatResponse::Gemini(resp) => Ok(resp.clone().into_bound_py_any(py)?),
        }
    }
    pub fn __str__(&self) -> String {
        match self {
            ChatResponse::OpenAI(resp) => PyHelperFuncs::__str__(resp),
            ChatResponse::Gemini(resp) => PyHelperFuncs::__str__(resp),
        }
    }
}

impl ChatResponse {
    pub fn is_empty(&self) -> bool {
        match self {
            ChatResponse::OpenAI(resp) => resp.choices.is_empty(),
            ChatResponse::Gemini(resp) => resp.candidates.is_empty(),
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

                let content =
                    PromptContent::Str(first_choice.message.content.clone().unwrap_or_default());

                Ok(vec![Message::from(content, role)])
            }

            ChatResponse::Gemini(resp) => {
                let content = resp
                    .candidates
                    .first()
                    .ok_or_else(|| AgentError::ClientNoResponseError)?
                    .content
                    .parts
                    .first()
                    .and_then(|part| part.text.as_ref())
                    .map(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();

                Ok(vec![Message::from(PromptContent::Str(content), role)])
            }
        }
    }

    pub fn to_python<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        match self {
            ChatResponse::OpenAI(resp) => Ok(resp.clone().into_bound_py_any(py)?),
            ChatResponse::Gemini(resp) => Ok(resp.clone().into_bound_py_any(py)?),
        }
    }

    pub fn id(&self) -> String {
        match self {
            ChatResponse::OpenAI(resp) => resp.id.clone(),
            ChatResponse::Gemini(resp) => resp.response_id.clone().unwrap_or("".to_string()),
        }
    }

    /// Get the content of the first choice in the chat response
    pub fn content(&self) -> Option<String> {
        match self {
            ChatResponse::OpenAI(resp) => {
                resp.choices.first().and_then(|c| c.message.content.clone())
            }
            ChatResponse::Gemini(resp) => resp
                .candidates
                .first()
                .and_then(|c| c.content.parts.first())
                .and_then(|part| part.text.as_ref().map(|s| s.to_string())),
        }
    }

    /// Check for tool calls in the chat response
    pub fn tool_calls(&self) -> Option<Value> {
        match self {
            ChatResponse::OpenAI(resp) => {
                let tool_calls: Option<&Vec<ToolCall>> =
                    resp.choices.first().map(|c| c.message.tool_calls.as_ref());
                tool_calls.and_then(|tc| serde_json::to_value(tc).ok())
            }
            ChatResponse::Gemini(resp) => {
                // Collect all function calls from all parts in the first candidate
                let function_calls: Vec<&FunctionCall> = resp
                    .candidates
                    .first()?
                    .content
                    .parts
                    .iter()
                    .filter_map(|part| part.function_call.as_ref())
                    .collect();

                if function_calls.is_empty() {
                    None
                } else {
                    serde_json::to_value(&function_calls).ok()
                }
            }
        }
    }

    /// Extracts structured data from a chat response
    pub fn extract_structured_data(&self) -> Option<Value> {
        if let Some(content) = self.content() {
            serde_json::from_str(&content).ok()
        } else {
            self.tool_calls()
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentResponse {
    pub id: String,
    pub response: ChatResponse,
}

#[pymethods]
impl AgentResponse {
    pub fn token_usage(&self) -> Usage {
        match &self.response {
            ChatResponse::OpenAI(resp) => resp.usage.clone(),
            ChatResponse::Gemini(resp) => resp.get_token_usage(),
        }
    }

    //pub fn result<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
    //    let pyobj = json_to_pyobject(py, &self.content())?;
    //
    //    // Convert plain string output to Python string
    //    Ok(pyobj.into_bound_py_any(py)?)
    //}
}

impl AgentResponse {
    pub fn new(id: String, response: ChatResponse) -> Self {
        Self { id, response }
    }

    pub fn content(&self) -> Option<String> {
        match &self.response {
            ChatResponse::OpenAI(resp) => resp.get_content(),
            ChatResponse::Gemini(resp) => resp.get_content(),
        }
    }

    pub fn log_probs(&self) -> Vec<ResponseLogProbs> {
        match &self.response {
            ChatResponse::OpenAI(resp) => resp.get_log_probs(),
            ChatResponse::Gemini(resp) => resp.get_log_probs(),
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

    #[getter]
    pub fn log_probs(&self) -> LogProbs {
        LogProbs {
            tokens: self.response.log_probs(),
        }
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

        // If the content is None, return None
        if content_value.is_none() {
            return Ok(py.None().into_bound_py_any(py)?);
        }
        // convert content_value to string
        let content_value = content_value.unwrap();

        match &self.output_type {
            Some(output_type) => {
                // Match the value. For loading into pydantic models, it's expected that the api response is a JSON string.

                let bound = output_type
                    .bind(py)
                    .call_method1("model_validate_json", (&content_value,));

                match bound {
                    Ok(obj) => {
                        // Successfully validated the model
                        Ok(obj)
                    }
                    Err(err) => {
                        // Model validation failed
                        // convert string to json and then to python object
                        warn!("Failed to validate model: {}", err);
                        self.failed_conversion = true;
                        let val = serde_json::from_str::<Value>(&content_value)?;
                        Ok(json_to_pyobject(py, &val)?.into_bound_py_any(py)?)
                    }
                }
            }
            None => {
                // If no output type is provided, attempt to parse the content as JSON
                let val = Value::String(content_value);
                Ok(json_to_pyobject(py, &val)?.into_bound_py_any(py)?)
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
