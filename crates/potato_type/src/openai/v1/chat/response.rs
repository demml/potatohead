use crate::prompt::ResponseContent;
use crate::traits::PromptMessageExt;
use crate::{
    openai::v1::ChatMessage,
    prompt::MessageNum,
    traits::{MessageResponseExt, ResponseAdapter},
    TypeError,
};

use potato_util::utils::{construct_structured_response, TokenLogProbs};
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
#[pyclass]
pub struct Function {
    #[pyo3(get)]
    pub arguments: String,
    #[pyo3(get)]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
#[pyclass]
pub struct ToolCall {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
    #[pyo3(get)]
    pub function: Function,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
#[pyclass]
pub struct UrlCitation {
    #[pyo3(get)]
    pub end_index: u64,
    #[pyo3(get)]
    pub start_index: u64,
    #[pyo3(get)]
    pub title: String,
    #[pyo3(get)]
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
#[pyclass]
pub struct Annotations {
    #[pyo3(get)]
    pub r#type: String,
    #[pyo3(get)]
    pub url_citations: Vec<UrlCitation>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
#[pyclass]
pub struct Audio {
    #[pyo3(get)]
    pub data: String,
    #[pyo3(get)]
    pub expires_at: u64, // Unix timestamp
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub transcript: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
#[pyclass]
pub struct ChatCompletionMessage {
    #[pyo3(get)]
    pub content: Option<String>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    #[pyo3(get)]
    pub role: String,
    #[pyo3(get)]
    pub annotations: Vec<Annotations>,
    #[pyo3(get)]
    pub tool_calls: Vec<ToolCall>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<Audio>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
#[pyclass]
pub struct TopLogProbs {
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
    #[pyo3(get)]
    pub logprob: f64,
    #[pyo3(get)]
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
#[pyclass]
pub struct LogContent {
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
    #[pyo3(get)]
    pub logprob: f64,
    #[pyo3(get)]
    pub token: String,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<Vec<TopLogProbs>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
#[pyclass]
pub struct LogProbs {
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<LogContent>>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<Vec<LogContent>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
#[pyclass]
pub struct Choice {
    #[pyo3(get)]
    pub message: ChatCompletionMessage,
    #[pyo3(get)]
    pub finish_reason: String,
    #[pyo3(get)]
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
    #[pyo3(get)]
    pub accepted_prediction_tokens: u64,
    #[pyo3(get)]
    pub audio_tokens: u64,
    #[pyo3(get)]
    pub reasoning_tokens: u64,
    #[pyo3(get)]
    pub rejected_prediction_tokens: u64,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct PromptTokenDetails {
    #[pyo3(get)]
    pub audio_tokens: u64,
    #[pyo3(get)]
    pub cached_tokens: u64,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct Usage {
    #[pyo3(get)]
    pub completion_tokens: u64,
    #[pyo3(get)]
    pub prompt_tokens: u64,
    #[pyo3(get)]
    pub total_tokens: u64,
    #[pyo3(get)]
    pub completion_tokens_details: CompletionTokenDetails,
    #[pyo3(get)]
    pub prompt_tokens_details: PromptTokenDetails,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct OpenAIChatResponse {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub object: String,
    #[pyo3(get)]
    pub created: u64,
    #[pyo3(get)]
    pub model: String,
    #[pyo3(get)]
    pub choices: Vec<Choice>,
    #[pyo3(get)]
    pub usage: Usage,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    #[pyo3(get)]
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

    fn get_log_probs(&self) -> Vec<TokenLogProbs> {
        let mut probabilities = Vec::new();
        if let Some(choice) = self.choices.first() {
            if let Some(logprobs) = &choice.logprobs {
                if let Some(content) = &logprobs.content {
                    for log_content in content {
                        // Look for single digit tokens (1, 2, 3, 4, 5)
                        if log_content.token.len() == 1
                            && log_content.token.chars().next().unwrap().is_ascii_digit()
                        {
                            probabilities.push(TokenLogProbs {
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

    fn response_text(&self) -> Option<String> {
        if self.choices.is_empty() {
            return None;
        }
        let content = self.choices.first().cloned().unwrap_or_default();

        content.message.content
    }
}
