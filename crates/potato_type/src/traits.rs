use crate::{
    error::TypeError,
    prompt::{MessageNum, ModelSettings, ResponseContent},
    Provider,
};
use potato_util::utils::ResponseLogProbs;
use pyo3::prelude::*;
use pyo3::types::PyList;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;

pub static VAR_REGEX: OnceLock<Regex> = OnceLock::new();
pub fn get_var_regex() -> &'static Regex {
    VAR_REGEX.get_or_init(|| Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap())
}
use crate::prompt::builder::ProviderRequest;
/// Core trait that all message types must implement
pub trait PromptMessageExt:
    Send + Sync + Clone + Serialize + for<'de> Deserialize<'de> + PartialEq
{
    /// Bind a variable in the message content, returning a new instance
    fn bind(&self, name: &str, value: &str) -> Result<Self, TypeError>
    where
        Self: Sized;

    /// Bind a variable in-place
    fn bind_mut(&mut self, name: &str, value: &str) -> Result<(), TypeError>;

    /// Extract variables from the message content
    fn extract_variables(&self) -> Vec<String>;

    fn from_text(content: String, role: &str) -> Result<Self, TypeError>;
}

/// Core trait that must be implemented for all request types
pub trait RequestAdapter {
    /// Returns all messages in the request
    fn messages(&self) -> &[MessageNum];
    /// Returns a mutable reference to the messages in the request
    fn messages_mut(&mut self) -> &mut Vec<MessageNum>;
    /// Returns all system instructions in the request
    fn system_instructions(&self) -> Vec<&MessageNum>;
    /// Returns the response JSON schema if set
    fn response_json_schema(&self) -> Option<&Value>;
    /// Inserts a message at the specified index (or at the start if None)
    fn insert_message(&mut self, message: MessageNum, idx: Option<usize>) -> () {
        self.messages_mut().insert(idx.unwrap_or(0), message);
    }
    /// Prepends system instructions to the messages
    fn preprend_system_instructions(&mut self, messages: Vec<MessageNum>) -> Result<(), TypeError>;

    /// Returns the system instructions as a Python list
    /// # Arguments
    /// * `py` - The Python GIL token
    /// Returns a Python list of system instruction messages
    fn get_py_system_instructions<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyList>, TypeError>;
    /// Returns the model settings for the request
    fn model_settings<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError>;
    /// Converts the request to a JSON value for sending to the provider
    fn to_request_body(&self) -> Result<Value, TypeError>;
    /// Checks if the request matches the given provider
    fn match_provider(&self, provider: &Provider) -> bool;
    /// Builds a provider-specific request enum from the given parameters
    /// The ProviderRequest enum encapsulates all supported provider request types and is an
    /// attribute of the Prompt struct. ProviderRequest is built on instantiation of the Prompt
    fn build_provider_enum(
        messages: Vec<MessageNum>,
        system_instructions: Vec<MessageNum>,
        model: String,
        settings: ModelSettings,
        response_json_schema: Option<Value>,
    ) -> Result<ProviderRequest, TypeError>;

    /// Sets the response JSON schema for the request
    /// Typically used as part of workflows when adding tasks
    fn set_response_json_schema(&mut self, response_json_schema: Option<Value>) -> ();
}

pub trait ResponseAdapter {
    /// Returns a string representation of the response
    fn __str__(&self) -> String;

    /// Checks if the response is empty
    fn is_empty(&self) -> bool;

    /// Converts the response to a Python object
    fn to_bound_py_object<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError>;

    /// Returns the response ID
    fn id(&self) -> &str;

    /// Converts the response to a vector of MessageNum
    fn to_message_num(&self) -> Result<Vec<MessageNum>, TypeError>;

    // Get the token usage as a Python object
    fn usage<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError>;

    /// Retrieves the first content choice from the response
    fn get_content(&self) -> ResponseContent;

    /// Retrieves the log probabilities from the response
    fn get_log_probs(&self) -> Vec<ResponseLogProbs>;

    /// Returns the structured output of the response
    /// For all response types the flow is as follows:
    /// 1. Check if the response has content (string/text)
    /// 2. If no content, return Python None
    /// 3. If content exists, check if an output_type/model is provided
    /// 4. If output_type/model is provided, attempt to convert the content to that type
    /// 5. If conversion fails, attempt to construct a generic Python object from the content
    /// 6. If no output_type/model is provided, return the content as a generic Python object
    /// # Arguments
    /// * `py`: The Python GIL token
    /// * `output_type`: An optional Python type/model to convert the content into. This can be a pydantic model or any object
    /// that implements model_validate_json that can parse from a JSON string.
    /// # Returns
    /// * `Result<Bound<'py, PyAny>, TypeError>`: The structured output as a Python object or an error
    fn structured_output<'py>(
        &self,
        py: Python<'py>,
        output_type: Option<&Bound<'py, PyAny>>,
    ) -> Result<Bound<'py, PyAny>, TypeError>;

    /// Returns the structured output value as a serde_json::Value
    fn structured_output_value(&self) -> Option<Value>;

    /// Returns any tool calls made in the response, if applicable
    fn tool_call_output(&self) -> Option<Value>;

    /// Returns the output text of the response if available
    fn response_text(&self) -> Option<String>;
}

pub trait MessageResponseExt {
    fn to_message_num(&self) -> Result<MessageNum, TypeError>;
}

pub trait MessageFactory: Sized {
    fn from_text(content: String, role: &str) -> Result<Self, TypeError>;
}

/// Trait for converting between different provider message formats
///
/// This trait enables conversion of messages between different LLM provider formats
/// (e.g., Anthropic MessageParam ↔ Google GeminiContent ↔ OpenAI ChatMessage).
///
/// Currently focused on text content conversion, with support for other content
/// types planned for future implementation.
pub trait MessageConversion {
    /// Convert this message to an Anthropic MessageParam
    ///
    /// # Errors
    /// Returns `TypeError::UnsupportedConversion` if the message contains
    /// content types that cannot be represented in Anthropic's format
    fn to_anthropic_message(
        &self,
    ) -> Result<crate::anthropic::v1::request::MessageParam, TypeError>;

    /// Convert this message to a Google GeminiContent
    ///
    /// # Errors
    /// Returns `TypeError::UnsupportedConversion` if the message contains
    /// content types that cannot be represented in Google's format
    fn to_google_message(
        &self,
    ) -> Result<crate::google::v1::generate::request::GeminiContent, TypeError>;

    /// Convert this message to an OpenAI ChatMessage
    ///
    /// # Errors
    /// Returns `TypeError::UnsupportedConversion` if the message contains
    /// content types that cannot be represented in OpenAI's format
    fn to_openai_message(&self)
        -> Result<crate::openai::v1::chat::request::ChatMessage, TypeError>;
}
