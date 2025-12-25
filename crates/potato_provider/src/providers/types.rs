use crate::error::ProviderError;
use potato_macro::dispatch_response_trait_method;
use potato_type::anthropic::v1::response::AnthropicChatResponse;
use potato_type::google::v1::generate::GenerateContentResponse;
use potato_type::google::PredictResponse;
use potato_type::openai::v1::OpenAIChatResponse;
use potato_type::traits::ResponseAdapter;
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

    pub fn anthropic_endpoint(&self) -> &'static str {
        match self {
            Self::Generate => "messages",
            Self::Embed => "embeddings",
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

pub fn build_http_client(default_headers: Option<HeaderMap>) -> Result<Client, ProviderError> {
    let headers = default_headers.unwrap_or_default();

    Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(ProviderError::from)
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatResponse {
    OpenAIV1(OpenAIChatResponse),
    GeminiV1(GenerateContentResponse),
    VertexGenerateV1(GenerateContentResponse),
    VertexPredictV1(PredictResponse),
    AnthropicMessageV1(AnthropicChatResponse),
}

#[pymethods]
impl ChatResponse {
    pub fn token_usage<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, ProviderError> {
        Ok(dispatch_response_trait_method!(
            self,
            ResponseAdapter,
            usage(py)
        )?)
    }

    pub fn to_py<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, ProviderError> {
        // try unwrapping the prompt, if it exists
        match self {
            ChatResponse::OpenAIV1(resp) => Ok(resp.clone().into_bound_py_any(py)?),
            ChatResponse::GeminiV1(resp) => Ok(resp.clone().into_bound_py_any(py)?),
            ChatResponse::VertexGenerateV1(resp) => Ok(resp.clone().into_bound_py_any(py)?),
            ChatResponse::VertexPredictV1(resp) => Ok(resp.clone().into_bound_py_any(py)?),
            ChatResponse::AnthropicMessageV1(resp) => Ok(resp.clone().into_bound_py_any(py)?),
        }
    }
    pub fn __str__(&self) -> String {
        dispatch_response_trait_method!(self, ResponseAdapter, __str__())
    }
}

impl ChatResponse {
    pub fn is_empty(&self) -> bool {
        dispatch_response_trait_method!(self, ResponseAdapter, is_empty())
    }

    pub fn to_python<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, ProviderError> {
        Ok(dispatch_response_trait_method!(
            self,
            ResponseAdapter,
            to_bound_py_object(py)
        )?)
    }

    pub fn id(&self) -> String {
        dispatch_response_trait_method!(self, ResponseAdapter, id()).to_string()
    }
}
