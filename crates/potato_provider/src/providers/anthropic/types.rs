use crate::ProviderError;
use crate::ResponseExt;
use potato_prompt::{Message, PromptContent};
use potato_type::anthropic::v1::message::AnthropicSettings;
use potato_type::anthropic::v1::message::ContentBlock;
use potato_type::anthropic::v1::message::DocumentSource;
use potato_type::anthropic::v1::message::ImageSource;
use potato_util::pyobject_to_json;
use potato_util::UtilError;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

/// common content types used in Anthropic messages
pub const BASE64_TYPE: &str = "base64";
pub const EPHEMERAL_TYPE: &str = "ephemeral";
pub const IMAGE_TYPE: &str = "image";
pub const TEXT_TYPE: &str = "text";
pub const DOCUMENT_TYPE: &str = "document";
pub const SEARCH_TYPE: &str = "search_result";
pub const THINKING_TYPE: &str = "thinking";
pub const REDACTED_THINKING_TYPE: &str = "redacted_thinking";
pub const TOOL_USE_TYPE: &str = "tool_use";
pub const TOOL_RESULT_TYPE: &str = "tool_result";
pub const WEB_SEARCH_TOOL_RESULT_TYPE: &str = "web_search_tool_result";
pub const SERVER_TOOL_USE_TYPE: &str = "server_tool_use";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct TextContent {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Value>,
    pub r#type: String,
}

#[pymethods]
impl TextContent {
    #[new]
    pub fn new(
        text: String,
        cache_control: Option<CacheControl>,
        citations: Option<&Bound<'_, PyAny>>,
    ) -> Result<Self, ProviderError> {
        let citations = match citations {
            Some(cit) => Some(pyobject_to_json(cit)?),
            None => None,
        };
        Self {
            text,
            cache_control,
            citations,
            r#type: TEXT_TYPE.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ImageContent {
    pub source: ImageSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DocumentContent {
    pub source: DocumentSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ToolUseContent {
    pub id: String,
    pub name: String,
    pub input: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ToolResultContent {
    pub tool_use_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ContentBlock>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ThinkingContent {
    pub thinking: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RedactedThinkingContent {
    pub data: String,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SearchResultContent {
    pub title: String,
    pub content: Vec<ContentBlock>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Value>,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WebSearchToolResultContent {
    pub tool_use_id: String,
    pub content: Vec<ContentBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ServerToolUseContent {
    pub id: String,
    pub name: String,
    pub input: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: Vec<ContentBlock>,
}

impl AnthropicMessage {
    /// Convert the Prompt Message to an OpenAI multimodal chat message
    pub fn from_message(message: &Message) -> Result<Self, ProviderError> {
        let content = match &message.content {
            PromptContent::Str(text) => vec![ContentBlock::Text {
                text: text.clone(),
                cache_control: None,
                citations: None,
            }],
            PromptContent::Image(image) => vec![ContentBlock::Image {
                source: ImageSource::Url {
                    url: image.url.clone(),
                },
                cache_control: None,
            }],
            PromptContent::Document(doc) => vec![ContentBlock::Document {
                source: DocumentSource::Url {
                    url: doc.url.clone(),
                },
                cache_control: None,
                context: None,
                title: None,
            }],
            _ => {
                // Handle other content types as needed
                return Err(ProviderError::UnsupportedContentTypeError);
            }
        };

        Ok(AnthropicMessage {
            role: message.role.to_string(),
            content,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnthropicMessageRequest {
    pub model: String,
    pub messages: Vec<AnthropicMessage>,
    #[serde(flatten)]
    pub settings: AnthropicSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct AnthropicChatResponse {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub model: String,
    #[pyo3(get)]
    pub role: String,
    #[pyo3(get)]
    pub stop_reason: Option<StopReason>,
    #[pyo3(get)]
    pub stop_sequence: Option<String>,
    #[pyo3(get)]
    pub r#type: String,
    #[pyo3(get)]
    pub usage: Usage,
    pub content: Vec<ContentBlock>,
}

impl ResponseExt for AnthropicChatResponse {
    fn get_content(&self) -> Option<String> {
        self.content.first().and_then(|block| match block {
            ContentBlock::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[pyclass]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct Usage {
    #[pyo3(get)]
    pub input_tokens: i32,
    #[pyo3(get)]
    pub output_tokens: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub cache_creation_input_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub cache_read_input_tokens: Option<i32>,
    #[pyo3(get)]
    pub service_tier: Option<String>,
}
