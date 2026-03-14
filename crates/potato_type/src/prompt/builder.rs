use crate::anthropic::v1::request::AnthropicMessageRequestV1;
use crate::anthropic::MessageParam;
use crate::google::v1::generate::request::GeminiGenerateContentRequestV1;
use crate::google::GeminiContent;
use crate::openai::v1::chat::request::OpenAIChatCompletionRequestV1;
use crate::openai::ChatMessage;
use crate::prompt::types::MessageNum;
use crate::prompt::ModelSettings;
use crate::tools::{AgentToolDefinition, ToolCall};
use crate::traits::RequestAdapter;
use crate::{Provider, TypeError};
use potatohead_macro::dispatch_trait_method;
use pyo3::types::PyList;
use pyo3::types::PyListMethods;
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
            MessageNum::AnthropicSystemMessageV1(_) => RequestType::AnthropicMessageV1,
            // RawV1 is a fallback — should not appear in initial messages list
            MessageNum::RawV1(_) => RequestType::OpenAIChatV1,
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
    pub fn insert_message(&mut self, message: MessageNum, idx: Option<usize>) {
        self.messages_mut().insert(idx.unwrap_or(0), message);
    }

    pub fn push_message(&mut self, message: MessageNum) {
        self.messages_mut().push(message);
    }

    pub fn messages(&self) -> &[MessageNum] {
        dispatch_trait_method!(self, RequestAdapter, messages())
    }

    pub fn system_instructions(&self) -> Vec<&MessageNum> {
        dispatch_trait_method!(self, RequestAdapter, system_instructions())
    }

    pub fn messages_mut(&mut self) -> &mut Vec<MessageNum> {
        dispatch_trait_method!(mut self, RequestAdapter, messages_mut())
    }

    pub fn add_tools(&mut self, tools: Vec<AgentToolDefinition>) -> Result<(), TypeError> {
        dispatch_trait_method!(mut self, RequestAdapter, add_tools(tools))
    }

    pub fn prepend_system_instructions(
        &mut self,
        instructions: Vec<MessageNum>,
    ) -> Result<(), TypeError> {
        dispatch_trait_method!(mut self, RequestAdapter, preprend_system_instructions(instructions))
    }

    /// Returns the messages as a Python list
    pub(crate) fn get_py_messages<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyList>, TypeError> {
        let py_messages = PyList::empty(py);

        for msg in self.messages() {
            if msg.is_user_message() {
                py_messages.append(msg.to_bound_py_object(py)?)?;
            }
        }

        Ok(py_messages)
    }

    pub(crate) fn get_all_py_messages<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyList>, TypeError> {
        let py_messages = PyList::empty(py);

        for msg in self.messages() {
            py_messages.append(msg.to_bound_py_object(py)?)?;
        }

        Ok(py_messages)
    }

    /// Returns the last message in the request as a Python object
    pub(crate) fn get_py_message<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyAny>, TypeError> {
        let last = self
            .messages()
            .iter()
            .rev()
            .find(|msg| msg.is_user_message())
            .ok_or_else(|| {
                TypeError::Error("No messages in request to convert to Python object".to_string())
            })?;

        last.to_bound_py_object(py)
    }

    pub(crate) fn get_openai_message(&self) -> Result<ChatMessage, TypeError> {
        let last = self
            .messages()
            .iter()
            .rev()
            .find(|msg| msg.is_user_message())
            .ok_or_else(|| {
                TypeError::Error("No messages in request to convert to Python object".to_string())
            })?;

        match last {
            MessageNum::OpenAIMessageV1(msg) => Ok(msg.clone()),
            _ => Err(TypeError::Error(
                "Last message is not an OpenAI ChatMessage".to_string(),
            )),
        }
    }

    /// Returns the messages as Anthropic MessageParam Python objects
    pub(crate) fn get_gemini_message(&self) -> Result<GeminiContent, TypeError> {
        let last = self
            .messages()
            .iter()
            .rev()
            .find(|msg| msg.is_user_message())
            .ok_or_else(|| {
                TypeError::Error("No messages in request to convert to Python object".to_string())
            })?;

        match last {
            MessageNum::GeminiContentV1(msg) => Ok(msg.clone()),
            _ => Err(TypeError::Error(
                "Last message is not a GeminiContent".to_string(),
            )),
        }
    }

    /// Returns the messages as Anthropic MessageParam Python objects
    pub(crate) fn get_anthropic_message(&self) -> Result<MessageParam, TypeError> {
        let last = self
            .messages()
            .iter()
            .rev()
            .find(|msg| msg.is_user_message())
            .ok_or_else(|| {
                TypeError::Error("No messages in request to convert to Python object".to_string())
            })?;

        Ok(match last {
            MessageNum::AnthropicMessageV1(msg) => msg.clone(),
            _ => {
                return Err(TypeError::Error(
                    "Last message is not an Anthropic MessageParam".to_string(),
                ))
            }
        })
    }

    pub(crate) fn get_py_system_instructions<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyList>, TypeError> {
        dispatch_trait_method!(self, RequestAdapter, get_py_system_instructions(py))
    }

    pub(crate) fn model_settings<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyAny>, TypeError> {
        dispatch_trait_method!(self, RequestAdapter, model_settings(py))
    }

    pub fn response_json_schema(&self) -> Option<&Value> {
        dispatch_trait_method!(self, RequestAdapter, response_json_schema())
    }

    pub fn has_structured_output(&self) -> bool {
        self.response_json_schema().is_some()
    }

    /// Retrieve the JSON request body for the specified provider
    /// This method will first attempt to match the provider type,
    /// returning an error if there is a mismatch.
    pub fn to_request(&self, provider: &Provider) -> Result<Value, TypeError> {
        let is_matched = dispatch_trait_method!(self, RequestAdapter, match_provider(provider));

        if !is_matched {
            return Err(TypeError::Error(
                "ProviderRequest does not match the specified provider".to_string(),
            ));
        }
        dispatch_trait_method!(self, RequestAdapter, to_request_body())
    }

    /// Serialize to JSON for API requests
    pub fn to_json(&self) -> Result<Value, TypeError> {
        Ok(serde_json::to_value(self)?)
    }

    pub fn set_response_json_schema(&mut self, response_json_schema: Option<Value>) {
        dispatch_trait_method!(mut self, RequestAdapter, set_response_json_schema(response_json_schema))
    }

    /// Injects a tool result message into the conversation in the appropriate provider format.
    ///
    /// - OpenAI: Pushes a RawV1 message with role="tool" and tool_call_id
    /// - Anthropic: Pushes a user MessageParam with a tool_result content block
    /// - Gemini: Pushes a user GeminiContent with a functionResponse part
    pub fn add_tool_result(
        &mut self,
        call: &ToolCall,
        result: &serde_json::Value,
    ) -> Result<(), TypeError> {
        match self {
            ProviderRequest::OpenAIV1(_) => {
                let call_id = call
                    .call_id
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string());
                let content = result.to_string();
                let raw = serde_json::json!({
                    "role": "tool",
                    "tool_call_id": call_id,
                    "content": content,
                });
                self.messages_mut().push(MessageNum::RawV1(raw));
                Ok(())
            }
            ProviderRequest::AnthropicV1(_) => {
                use crate::anthropic::v1::request::{
                    ContentBlock, ContentBlockParam, TextBlockParam, ToolResultBlockParam,
                    ToolResultContentEnum,
                };
                let tool_use_id = call
                    .call_id
                    .clone()
                    .unwrap_or_else(|| call.tool_name.clone());
                let result_text = result.to_string();
                let text_block = TextBlockParam::new_rs(result_text, None, None);
                let tool_result = ToolResultBlockParam {
                    tool_use_id,
                    is_error: None,
                    cache_control: None,
                    r#type: "tool_result".to_string(),
                    content: Some(ToolResultContentEnum::Text(vec![text_block])),
                };
                let content_block = ContentBlockParam {
                    inner: ContentBlock::ToolResult(tool_result),
                };
                let message = MessageParam {
                    role: "user".to_string(),
                    content: vec![content_block],
                };
                self.messages_mut()
                    .push(MessageNum::AnthropicMessageV1(message));
                Ok(())
            }
            ProviderRequest::GeminiV1(_) => {
                use crate::google::v1::generate::request::{
                    DataNum, FunctionResponse, GeminiContent, Part,
                };
                let name = call.tool_name.clone();
                let mut response_map = std::collections::HashMap::new();
                response_map.insert("output".to_string(), result.clone());
                let func_response = FunctionResponse {
                    name,
                    response: response_map,
                };
                let part = Part {
                    data: DataNum::FunctionResponse(func_response),
                    ..Default::default()
                };
                let content = GeminiContent {
                    role: "user".to_string(),
                    parts: vec![part],
                };
                self.messages_mut()
                    .push(MessageNum::GeminiContentV1(content));
                Ok(())
            }
        }
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
        RequestType::OpenAIChatV1 => OpenAIChatCompletionRequestV1::build_provider_enum(
            messages,
            system_instructions,
            model,
            model_settings,
            response_json_schema,
        ),
        RequestType::AnthropicMessageV1 => AnthropicMessageRequestV1::build_provider_enum(
            messages,
            system_instructions,
            model,
            model_settings,
            response_json_schema,
        ),
        RequestType::GeminiContentV1 => GeminiGenerateContentRequestV1::build_provider_enum(
            messages,
            system_instructions,
            model,
            model_settings,
            response_json_schema,
        ),
    }
}
