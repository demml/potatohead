use crate::error::ProviderError;
use crate::providers::google::FunctionCall;
use crate::providers::google::GenerateContentResponse;
use crate::providers::openai::OpenAIChatResponse;
use crate::providers::openai::ToolCall;
use potato_prompt::{
    prompt::{PromptContent, Role},
    Message,
};
use potato_type::google::predict::PredictResponse;
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
use reqwest::header::HeaderName;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::debug;
use tracing::instrument;
use tracing::warn;
const TIMEOUT_SECS: u64 = 30;

#[derive(Debug, PartialEq)]
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

    pub fn openai_endpoint(&self) -> &'static str {
        match self {
            Self::Generate => "chat/completions",
            Self::Embed => "embeddings",
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatResponse {
    OpenAI(OpenAIChatResponse),
    Gemini(GenerateContentResponse),
    VertexGenerate(GenerateContentResponse),
    VertexPredict(PredictResponse),
}

#[pymethods]
impl ChatResponse {
    pub fn to_py<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, ProviderError> {
        // try unwrapping the prompt, if it exists
        match self {
            ChatResponse::OpenAI(resp) => Ok(resp.clone().into_bound_py_any(py)?),
            ChatResponse::Gemini(resp) => Ok(resp.clone().into_bound_py_any(py)?),
            ChatResponse::VertexGenerate(resp) => Ok(resp.clone().into_bound_py_any(py)?),
            ChatResponse::VertexPredict(resp) => Ok(resp.clone().into_bound_py_any(py)?),
        }
    }
    pub fn __str__(&self) -> String {
        match self {
            ChatResponse::OpenAI(resp) => PyHelperFuncs::__str__(resp),
            ChatResponse::Gemini(resp) => PyHelperFuncs::__str__(resp),
            ChatResponse::VertexGenerate(resp) => PyHelperFuncs::__str__(resp),
            ChatResponse::VertexPredict(resp) => PyHelperFuncs::__str__(resp),
        }
    }
}

impl ChatResponse {
    pub fn is_empty(&self) -> bool {
        match self {
            ChatResponse::OpenAI(resp) => resp.choices.is_empty(),
            ChatResponse::Gemini(resp) => resp.candidates.is_empty(),
            ChatResponse::VertexGenerate(resp) => resp.candidates.is_empty(),
            _ => true,
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
            _ => {
                warn!("to_message not implemented for this provider");
                Err(ProviderError::NotImplementedError(
                    "to_message not implemented for this provider".to_string(),
                ))
            }
        }
    }

    pub fn to_python<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, ProviderError> {
        match self {
            ChatResponse::OpenAI(resp) => Ok(resp.clone().into_bound_py_any(py)?),
            ChatResponse::Gemini(resp) => Ok(resp.clone().into_bound_py_any(py)?),
            ChatResponse::VertexGenerate(resp) => Ok(resp.clone().into_bound_py_any(py)?),
            _ => Err(ProviderError::NotImplementedError(
                "to_python not implemented for this provider".to_string(),
            )),
        }
    }

    pub fn id(&self) -> String {
        match self {
            ChatResponse::OpenAI(resp) => resp.id.clone(),
            ChatResponse::Gemini(resp) => resp.response_id.clone().unwrap_or("".to_string()),
            ChatResponse::VertexGenerate(resp) => {
                resp.response_id.clone().unwrap_or("".to_string())
            }
            _ => "".to_string(),
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
            ChatResponse::VertexGenerate(resp) => resp
                .candidates
                .first()
                .and_then(|c| c.content.parts.first())
                .and_then(|part| part.text.as_ref().map(|s| s.to_string())),
            _ => {
                warn!("content not implemented for this provider");
                None
            }
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

            ChatResponse::VertexGenerate(resp) => {
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
            _ => {
                warn!("tool_calls not implemented for this provider");
                None
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

/// Merges extra_body fields into the serialized prompt JSON object.
///
/// # Arguments
/// * `serialized_prompt` - Mutable reference to the JSON value to modify
/// * `extra_body` - Reference to the extra body JSON object to merge
///
/// # Example
/// ```rust
/// let mut prompt = serde_json::json!({"model": "gpt-4"});
/// let extra = serde_json::json!({"temperature": 0.7});
/// add_extra_body_to_prompt(&mut prompt, &extra);
/// ```
pub fn add_extra_body_to_prompt(serialized_prompt: &mut Value, extra_body: &Value) {
    if let (Some(prompt_obj), Some(extra_obj)) =
        (serialized_prompt.as_object_mut(), extra_body.as_object())
    {
        // Merge the extra_body fields into prompt
        for (key, value) in extra_obj {
            prompt_obj.insert(key.clone(), value.clone());
        }
    }
}

pub fn build_http_client(
    client_headers: Option<HashMap<String, String>>,
) -> Result<Client, ProviderError> {
    let mut headers = HeaderMap::new();

    if let Some(headers_map) = client_headers {
        for (key, value) in headers_map {
            headers.insert(
                HeaderName::from_str(&key).map_err(ProviderError::CreateHeaderNameError)?,
                HeaderValue::from_str(&value).map_err(ProviderError::CreateHeaderValueError)?,
            );
        }
    }

    let client_builder = Client::builder().timeout(std::time::Duration::from_secs(TIMEOUT_SECS));

    let client = client_builder
        .default_headers(headers)
        .build()
        .map_err(ProviderError::CreateClientError)?;

    Ok(client)
}
