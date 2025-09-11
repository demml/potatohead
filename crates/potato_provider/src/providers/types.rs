use crate::error::ProviderError;
use crate::providers::google::FunctionCall;
use crate::providers::google::GenerateContentResponse;
use crate::providers::openai::OpenAIChatResponse;
use crate::providers::openai::ToolCall;
use potato_prompt::{
    prompt::{PromptContent, Role},
    Message,
};
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;
use tracing::instrument;
use tracing::warn;

#[derive(Debug)]
pub enum ServiceType {
    Generate,
    Embed,
}

impl ServiceType {
    /// Get the service type string
    pub fn gemini_endpoint(&self) -> &'static str {
        match self {
            Self::Generate => "generateContent",
            Self::Embed => "embedContent",
        }
    }
    pub fn vertex_endpoint(&self) -> &'static str {
        match self {
            Self::Generate => "generateContent",
            Self::Embed => "predict",
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatResponse {
    OpenAI(OpenAIChatResponse),
    Gemini(GenerateContentResponse),
}

#[pymethods]
impl ChatResponse {
    pub fn to_py<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, ProviderError> {
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
    pub fn to_message(&self, role: Role) -> Result<Vec<Message>, ProviderError> {
        debug!("Converting chat response to message with role");
        match self {
            ChatResponse::OpenAI(resp) => {
                let first_choice = resp
                    .choices
                    .first()
                    .ok_or_else(|| ProviderError::ClientNoResponseError)?;

                let content =
                    PromptContent::Str(first_choice.message.content.clone().unwrap_or_default());

                Ok(vec![Message::from(content, role)])
            }

            ChatResponse::Gemini(resp) => {
                let content = resp
                    .candidates
                    .first()
                    .ok_or_else(|| ProviderError::ClientNoResponseError)?
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

    pub fn to_python<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, ProviderError> {
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
