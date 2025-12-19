use crate::ProviderError;
use crate::ResponseExt;
use potato_prompt::{Message, PromptContent};
use potato_type::anthropic::v1::message::AnthropicSettings;
use potato_type::anthropic::v1::message::ContentBlock;
use potato_type::anthropic::v1::message::DocumentSource;
use potato_type::anthropic::v1::message::ImageSource;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

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
