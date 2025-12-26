use crate::error::ProviderError;
use potato_macro::dispatch_response_trait_method;
use potato_type::anthropic::v1::response::AnthropicChatResponse;
use potato_type::google::v1::generate::GenerateContentResponse;
use potato_type::google::PredictResponse;
use potato_type::openai::v1::OpenAIChatResponse;
use potato_type::prompt::MessageNum;
use potato_type::traits::ResponseAdapter;
use potato_type::Provider;
use potato_util::utils::ResponseLogProbs;
use pyo3::prelude::*;
use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const GENERATE_CONTENT: &str = "generateContent";
pub const EMBED_CONTENT: &str = "embedContent";
pub const CHAT_COMPLETIONS: &str = "chat/completions";
pub const PREDICT: &str = "predict";
pub const EMBEDDINGS: &str = "embeddings";
pub const MESSAGES: &str = "messages";

#[derive(Debug, PartialEq)]
pub enum ServiceType {
    Generate,
    Embed,
}

impl ServiceType {
    /// Get the service type string
    pub fn gemini_endpoint(&self) -> &'static str {
        match self {
            Self::Generate => GENERATE_CONTENT,
            Self::Embed => EMBED_CONTENT,
        }
    }
    pub fn vertex_endpoint(&self) -> &'static str {
        match self {
            Self::Generate => GENERATE_CONTENT,
            Self::Embed => PREDICT, // vertex uses "predict" for embeddings since it calls models directly
        }
    }

    pub fn openai_endpoint(&self) -> &'static str {
        match self {
            Self::Generate => CHAT_COMPLETIONS,
            Self::Embed => EMBEDDINGS,
        }
    }

    pub fn anthropic_endpoint(&self) -> &'static str {
        match self {
            Self::Generate => MESSAGES,
            Self::Embed => EMBEDDINGS,
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

/// Unified ChatResponse enum to encapsulate different provider responses
/// Follows  our strategy pattern for dispatching methods to the appropriate inner type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatResponse {
    OpenAIV1(OpenAIChatResponse),
    GeminiV1(GenerateContentResponse),
    VertexGenerateV1(GenerateContentResponse),
    VertexPredictV1(PredictResponse),
    AnthropicMessageV1(AnthropicChatResponse),
}

impl ChatResponse {
    /// Returns the token usage as a Python object
    pub fn token_usage<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, ProviderError> {
        Ok(dispatch_response_trait_method!(
            self,
            ResponseAdapter,
            usage(py)
        )?)
    }

    /// Converts the response to a Python object
    pub fn to_bound_py_object<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyAny>, ProviderError> {
        Ok(dispatch_response_trait_method!(
            self,
            ResponseAdapter,
            to_bound_py_object(py)
        )?)
    }

    /// Returns the string representation of the response
    pub fn __str__(&self) -> String {
        dispatch_response_trait_method!(self, ResponseAdapter, __str__())
    }

    /// Checks if the response is empty
    pub fn is_empty(&self) -> bool {
        dispatch_response_trait_method!(self, ResponseAdapter, is_empty())
    }

    /// Converts the response to a vector of MessageNum
    pub fn id(&self) -> String {
        dispatch_response_trait_method!(self, ResponseAdapter, id()).to_string()
    }

    /// Converts the response to a vector of MessageNum
    pub fn structured_output<'py>(
        &self,
        py: Python<'py>,
        output_type: Option<&Bound<'py, PyAny>>,
    ) -> Result<Bound<'py, PyAny>, ProviderError> {
        Ok(dispatch_response_trait_method!(
            self,
            ResponseAdapter,
            structured_output(py, output_type)
        )?)
    }

    pub fn get_log_probs(&self) -> Vec<ResponseLogProbs> {
        dispatch_response_trait_method!(self, ResponseAdapter, get_log_probs())
    }

    /// Converts the response messages to a vector of MessageNum for requests
    /// If necessary, will convert each message to the appropriate provider format
    pub fn to_message_num(&self, provider: &Provider) -> Result<Vec<MessageNum>, ProviderError> {
        // convert response to MessageNum of existing provider type
        let mut messages =
            dispatch_response_trait_method!(self, ResponseAdapter, to_message_num())?;

        // convert each message to the target provider type if needed
        // if the response provider type matches the target provider, no conversion done
        for msg in messages.iter_mut() {
            msg.convert_message(provider)?;
        }
        Ok(messages)
    }

    /// Retrieves the structured output value as a serde_json::Value
    /// output is either a structure response or tool call data
    pub fn extract_structured_data(&self) -> Option<Value> {
        if let Some(output) =
            dispatch_response_trait_method!(self, ResponseAdapter, structured_output_value())
        {
            return Some(output);
        } else {
            return dispatch_response_trait_method!(self, ResponseAdapter, tool_call_output());
        }
    }

    pub fn response_text(&self) -> Option<String> {
        dispatch_response_trait_method!(self, ResponseAdapter, response_text())
    }
}
