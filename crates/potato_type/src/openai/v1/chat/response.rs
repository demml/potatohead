use crate::prompt::ResponseContent;
use crate::traits::PromptMessageExt;
use crate::{
    openai::v1::ChatMessage,
    prompt::MessageNum,
    traits::{MessageResponseExt, ResponseAdapter},
    TypeError,
};

use potato_util::utils::{construct_structured_response, ResponseLogProbs};
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
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
    #[serde(rename = "type")]
    pub r#type: String,
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

impl MessageResponseExt for Choice {
    fn to_message_num(&self) -> Result<MessageNum, TypeError> {
        // if content is None, return None
        if let Some(content) = &self.message.content {
            let chat_msg = ChatMessage::from_text(content.clone(), &self.message.role)?;
            let message_num = MessageNum::OpenAIMessageV1(chat_msg);
            Ok(message_num)
        } else {
            Err(TypeError::EmptyOpenAIResponseContent)
        }
    }
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

#[pymethods]
impl OpenAIChatResponse {
    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

impl ResponseAdapter for OpenAIChatResponse {
    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }

    fn is_empty(&self) -> bool {
        self.choices.is_empty()
    }

    fn to_bound_py_object<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        Ok(PyHelperFuncs::to_bound_py_object(py, self)?)
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn to_message_num(&self) -> Result<Vec<MessageNum>, TypeError> {
        let mut results = Vec::new();
        for choice in &self.choices {
            match choice.to_message_num() {
                Ok(message_num) => results.push(message_num),
                Err(_) => continue,
            }
        }
        Ok(results)
    }

    fn get_content(&self) -> ResponseContent {
        ResponseContent::OpenAI(self.choices.first().cloned().unwrap_or_default())
    }

    fn tool_call_output(&self) -> Option<Value> {
        if self.choices.is_empty() {
            return None;
        }
        let content = self.choices.first().cloned().unwrap_or_default();

        if !content.message.tool_calls.is_empty() {
            serde_json::to_value(&content.message.tool_calls).ok()
        } else {
            None
        }
    }

    fn structured_output_value(&self) -> Option<Value> {
        if self.choices.is_empty() {
            return None;
        }
        let content = self.choices.first().cloned().unwrap_or_default();

        if let Some(text) = content.message.content {
            serde_json::from_str(&text).ok()
        } else {
            None
        }
    }

    fn structured_output<'py>(
        &self,
        py: Python<'py>,
        output_model: Option<&Bound<'py, PyAny>>,
    ) -> Result<Bound<'py, PyAny>, TypeError> {
        if self.choices.is_empty() {
            // return Py None if no content
            return Ok(py.None().into_bound_py_any(py)?);
        }

        let content = self.choices.first().cloned().unwrap_or_default();

        if let Some(text) = content.message.content {
            return Ok(construct_structured_response(py, text, output_model)?);
        } else {
            // return Py None if no content
            return Ok(py.None().into_bound_py_any(py)?);
        }
    }

    /// Returns the total token count across all modalities.
    fn usage<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        Ok(PyHelperFuncs::to_bound_py_object(py, &self.usage)?)
    }

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
