use crate::agents::error::AgentError;
use crate::agents::provider::traits::{LogProbExt, ResponseExt, TokenUsage};
use potato_prompt::{prompt::types::PromptContent, Message};
use potato_type::openai::chat::OpenAIChatSettings;
use potato_util::utils::ResponseLogProbs;
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct Function {
    pub arguments: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct ToolCall {
    pub id: String,
    pub type_: String,
    pub function: Function,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct UrlCitation {
    pub end_index: u64,
    pub start_index: u64,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct Annotations {
    pub r#type: String,
    pub url_citations: Vec<UrlCitation>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct Audio {
    pub data: String,
    pub expires_at: u64, // Unix timestamp
    pub id: String,
    pub transcript: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct ChatCompletionMessage {
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    pub role: String,
    pub annotations: Vec<Annotations>,
    pub tool_calls: Vec<ToolCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<Audio>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct TopLogProbs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
    pub logprob: f64,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct LogContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
    pub logprob: f64,
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<Vec<TopLogProbs>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct LogProbs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<LogContent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<Vec<LogContent>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct Choice {
    pub message: ChatCompletionMessage,
    pub finish_reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<LogProbs>,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct CompletionTokenDetails {
    pub accepted_prediction_tokens: u64,
    pub audio_tokens: u64,
    pub reasoning_tokens: u64,
    pub rejected_prediction_tokens: u64,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct PromptTokenDetails {
    pub audio_tokens: u64,
    pub cached_tokens: u64,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct Usage {
    pub completion_tokens: u64,
    pub prompt_tokens: u64,
    pub total_tokens: u64,
    pub completion_tokens_details: CompletionTokenDetails,
    pub prompt_tokens_details: PromptTokenDetails,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

impl TokenUsage for Usage {
    fn total_tokens(&self) -> u64 {
        self.total_tokens
    }

    fn prompt_tokens(&self) -> u64 {
        self.prompt_tokens
    }

    fn completion_tokens(&self) -> u64 {
        self.completion_tokens
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct OpenAIChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

impl ResponseExt for OpenAIChatResponse {
    fn get_content(&self) -> Option<String> {
        self.choices.first().and_then(|c| c.message.content.clone())
    }
}

impl LogProbExt for OpenAIChatResponse {
    fn get_log_probs(&self) -> Vec<ResponseLogProbs> {
        let mut probabilities = Vec::new();
        if let Some(choice) = self.choices.first() {
            if let Some(logprobs) = &choice.logprobs {
                if let Some(content) = &logprobs.content {
                    for log_content in content {
                        // Look for single digit tokens (1, 2, 3, 4, 5)
                        if log_content.token.len() == 1
                            && log_content.token.chars().next().unwrap().is_ascii_digit()
                        {
                            probabilities.push(ResponseLogProbs {
                                token: log_content.token.clone(),
                                logprob: log_content.logprob,
                            });
                        }
                    }
                }
            }
        }

        probabilities
    }
}

#[pymethods]
impl OpenAIChatResponse {
    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenAIChatRequest {
    pub model: String,
    pub messages: Vec<OpenAIChatMessage>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub settings: Option<OpenAIChatSettings>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAITextMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAIImageUrl {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAIInputAudio {
    pub data: String,   // base64-encoded audio
    pub format: String, // e.g., "wav"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAIContentPart {
    Text { text: String },
    ImageUrl { image_url: OpenAIImageUrl },
    InputAudio { input_audio: OpenAIInputAudio },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAIChatMessage {
    pub role: String,
    pub content: Vec<OpenAIContentPart>,
}

impl OpenAIChatMessage {
    /// Convert the Prompt Message to an OpenAI multimodal chat message
    pub fn from_message(message: &Message) -> Result<Self, AgentError> {
        let content = match &message.content {
            PromptContent::Str(text) => vec![OpenAIContentPart::Text { text: text.clone() }],
            PromptContent::Image(image) => vec![OpenAIContentPart::ImageUrl {
                image_url: OpenAIImageUrl {
                    url: image.url.clone(),
                },
            }],
            // need to implement audio and file for chat
            _ => {
                // Handle other content types as needed
                return Err(AgentError::UnsupportedContentTypeError);
            }
        };

        Ok(OpenAIChatMessage {
            role: message.role.to_string(),
            content,
        })
    }
}
