use crate::anthropic::v1::message::{AnthropicMessageRequestV1, AnthropicSettings};
use crate::google::v1::generate::request::GeminiGenerateContentRequestV1;
use crate::google::v1::generate::GeminiSettings;
use crate::openai::v1::chat::request::OpenAIChatCompletionRequestV1;
use crate::openai::v1::create_structured_output_schema;
use crate::prompt::types::MessageNum;
use crate::prompt::ModelSettings;
use crate::TypeError;
use pyo3::types::PyList;
use pyo3::types::PyListMethods;
use pyo3::IntoPyObjectExt;
use pyo3::Python;
use pyo3::{Bound, PyAny};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Trait for converting a list of messages into a provider-specific request
pub trait RequestBuilder {
    type Request: Serialize;

    /// Build a request from messages and settings
    fn build_request(
        messages: Vec<MessageNum>,
        system_instructions: Vec<MessageNum>,
        model: String,
        settings: ModelSettings,
        response_format: Option<Value>,
    ) -> Result<Self::Request, TypeError>;
}

/// Type marker for request routing
#[derive(Debug, Clone, Copy)]
pub enum RequestType {
    OpenAIChatV1,
    AnthropicMessageV1,
    GeminiContentV1,
}

impl MessageNum {
    /// Determine the request type from this message variant
    pub fn request_type(&self) -> RequestType {
        match self {
            MessageNum::OpenAIMessageV1(_) => RequestType::OpenAIChatV1,
            MessageNum::AnthropicMessageV1(_) => RequestType::AnthropicMessageV1,
            MessageNum::GeminiContentV1(_) => RequestType::GeminiContentV1,
        }
    }
}

/// Unified enum for provider-specific requests
/// This serves as a central access point for accessing request attributes from within a Prompt
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ProviderRequest {
    OpenAIV1(OpenAIChatCompletionRequestV1),
    AnthropicV1(AnthropicMessageRequestV1),
    GeminiV1(GeminiGenerateContentRequestV1),
}

impl ProviderRequest {
    pub fn messages(&self) -> &[MessageNum] {
        match self {
            ProviderRequest::OpenAIV1(req) => &req.messages,
            ProviderRequest::AnthropicV1(req) => &req.messages,
            ProviderRequest::GeminiV1(req) => &req.contents,
        }
    }

    pub(crate) fn messages_mut(&mut self) -> &mut Vec<MessageNum> {
        match self {
            ProviderRequest::OpenAIV1(req) => &mut req.messages,
            ProviderRequest::AnthropicV1(req) => &mut req.messages,
            ProviderRequest::GeminiV1(req) => &mut req.contents,
        }
    }

    pub(crate) fn get_py_messages<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyList>, TypeError> {
        let py_messages = PyList::empty(py);

        for msg in self.messages() {
            py_messages.append(msg.to_bound_py_object(py)?)?;
        }

        Ok(py_messages)
    }

    pub(crate) fn get_py_system_instructions<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyList>, TypeError> {
        let py_system_instructions = PyList::empty(py);

        match self {
            ProviderRequest::OpenAIV1(req) => {
                for msg in &req.messages {
                    // OpenAI does not have separate system instructions
                    // but we can filter them out if needed
                    if msg.is_system_message() {
                        py_system_instructions.append(msg.to_bound_py_object(py)?)?;
                    }
                }
            }
            ProviderRequest::AnthropicV1(req) => {
                for msg in &req.system {
                    py_system_instructions.append(msg.to_bound_py_object(py)?)?;
                }
            }
            ProviderRequest::GeminiV1(req) => {
                if let Some(system_msg) = &req.system_instruction {
                    py_system_instructions.append(system_msg.to_bound_py_object(py)?)?;
                }
            }
        }

        Ok(py_system_instructions)
    }

    pub(crate) fn model_settings<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyAny>, TypeError> {
        match &self {
            ProviderRequest::OpenAIV1(req) => {
                let settings = req.settings.as_ref().cloned().unwrap_or_default();
                Ok(settings.into_bound_py_any(py)?)
            }
            ProviderRequest::AnthropicV1(req) => {
                let settings = req.settings.clone();
                Ok(settings.into_bound_py_any(py)?)
            }
            ProviderRequest::GeminiV1(req) => {
                let settings = req.settings.clone();
                Ok(settings.into_bound_py_any(py)?)
            }
        }
    }

    pub(crate) fn response_json_schema(&self) -> Option<&Value> {
        match &self {
            ProviderRequest::OpenAIV1(req) => req.response_format.as_ref(),
            ProviderRequest::AnthropicV1(req) => req.output_format.as_ref(),
            ProviderRequest::GeminiV1(req) => req
                .settings
                .generation_config
                .as_ref()
                .and_then(|cfg| cfg.response_json_schema.as_ref()),
        }
    }

    fn build_openai_v1(
        messages: Vec<MessageNum>,
        system_instructions: Vec<MessageNum>,
        model: String,
        settings: ModelSettings,
        response_format: Option<Value>,
    ) -> Result<Self, TypeError> {
        let openai_messages: Vec<_> = system_instructions.into_iter().chain(messages).collect();

        let openai_settings = match settings {
            ModelSettings::OpenAIChat(s) => Some(s),
            _ => None,
        };

        let response_json_schema = match &response_format {
            Some(schema) => Some(create_structured_output_schema(schema)),
            None => None,
        };

        Ok(ProviderRequest::OpenAIV1(OpenAIChatCompletionRequestV1 {
            model,
            messages: openai_messages,
            response_format: response_json_schema,
            settings: openai_settings,
        }))
    }

    fn build_anthropic_v1(
        messages: Vec<MessageNum>,
        system_instructions: Vec<MessageNum>,
        model: String,
        settings: ModelSettings,
        output_format: Option<Value>,
    ) -> Result<Self, TypeError> {
        // Extract Anthropic-specific settings
        let anthropic_settings = match settings {
            ModelSettings::AnthropicChat(s) => s,
            _ => AnthropicSettings::default(),
        };

        Ok(ProviderRequest::AnthropicV1(AnthropicMessageRequestV1 {
            model,
            messages,
            system: system_instructions,
            settings: anthropic_settings,
            output_format,
        }))
    }

    fn build_gemini_v1(
        messages: Vec<MessageNum>,
        system_instructions: Vec<MessageNum>,
        settings: ModelSettings,
        response_schema: Option<Value>,
    ) -> Result<Self, TypeError> {
        // check system_instructions only has one element for Gemini
        // Get first element if exists
        let system_instruction = if system_instructions.is_empty() {
            None
        } else if system_instructions.len() > 1 {
            return Err(TypeError::MoreThanOneSystemInstruction);
        } else {
            system_instructions.into_iter().next()
        };

        let mut gemini_settings = match settings {
            ModelSettings::GoogleChat(s) => s,
            _ => GeminiSettings::default(),
        };

        if let Some(schema) = response_schema {
            gemini_settings.configure_for_structured_output(schema);
        }

        Ok(ProviderRequest::GeminiV1(GeminiGenerateContentRequestV1 {
            contents: messages,
            system_instruction,
            settings: gemini_settings,
        }))
    }

    /// Serialize to JSON for API requests
    pub fn to_json(&self) -> Result<Value, TypeError> {
        Ok(serde_json::to_value(self)?)
    }
}

pub fn to_provider_request(
    messages: Vec<MessageNum>,
    system_instructions: Vec<MessageNum>,
    model: String,
    model_settings: ModelSettings,
    response_json_schema: Option<Value>,
) -> Result<ProviderRequest, TypeError> {
    // Determine request type from first message
    let request_type = messages
        .first()
        .ok_or_else(|| TypeError::Error("Prompt has no messages".to_string()))?
        .request_type();

    // Validate all messages are same type
    for msg in &messages {
        if msg.request_type() as u8 != request_type as u8 {
            return Err(TypeError::Error(
                "All messages must be of the same provider type".to_string(),
            ));
        }
    }

    // Build appropriate request based on type
    match request_type {
        RequestType::OpenAIChatV1 => ProviderRequest::build_openai_v1(
            messages,
            system_instructions,
            model,
            model_settings,
            response_json_schema,
        ),
        RequestType::AnthropicMessageV1 => ProviderRequest::build_anthropic_v1(
            messages,
            system_instructions,
            model,
            model_settings,
            response_json_schema,
        ),
        RequestType::GeminiContentV1 => ProviderRequest::build_gemini_v1(
            messages,
            system_instructions,
            model_settings,
            response_json_schema,
        ),
    }
}
