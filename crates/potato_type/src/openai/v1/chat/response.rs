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

    fn response_text(&self) -> String {
        if self.choices.is_empty() {
            return String::new();
        }
        let content = self.choices.first().cloned().unwrap_or_default();

        content.message.content.unwrap_or_default()
    }

    fn model_name(&self) -> Option<&str> {
        Some(&self.model)
    }

    fn finish_reason(&self) -> Option<&str> {
        self.choices.first().map(|c| c.finish_reason.as_str())
    }

    fn input_tokens(&self) -> Option<i64> {
        Some(self.usage.prompt_tokens as i64)
    }

    fn output_tokens(&self) -> Option<i64> {
        Some(self.usage.completion_tokens as i64)
    }

    fn total_tokens(&self) -> Option<i64> {
        Some(self.usage.total_tokens as i64)
    }

    fn get_tool_calls(&self) -> Vec<crate::tools::ToolCallInfo> {
        let mut tool_calls = Vec::new();
        for choice in &self.choices {
            for call in &choice.message.tool_calls {
                tool_calls.push(crate::tools::ToolCallInfo {
                    name: call.function.name.clone(),
                    arguments: serde_json::to_value(&call.function.arguments).unwrap_or_default(),
                    call_id: Some(call.id.clone()),
                    result: None,
                });
            }
        }
        tool_calls
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::ResponseAdapter;

    fn make_text_response(text: &str) -> OpenAIChatResponse {
        OpenAIChatResponse {
            id: "chatcmpl-abc123".to_string(),
            object: "chat.completion".to_string(),
            created: 1700000000,
            model: "gpt-4o".to_string(),
            choices: vec![Choice {
                message: ChatCompletionMessage {
                    content: Some(text.to_string()),
                    refusal: None,
                    role: "assistant".to_string(),
                    annotations: vec![],
                    tool_calls: vec![],
                    audio: None,
                },
                finish_reason: "stop".to_string(),
                logprobs: None,
            }],
            usage: Usage {
                completion_tokens: 10,
                prompt_tokens: 20,
                total_tokens: 30,
                completion_tokens_details: CompletionTokenDetails::default(),
                prompt_tokens_details: PromptTokenDetails::default(),
                finish_reason: None,
            },
            service_tier: None,
            system_fingerprint: Some("fp_abc123".to_string()),
        }
    }

    fn make_tool_call_response() -> OpenAIChatResponse {
        OpenAIChatResponse {
            id: "chatcmpl-tool456".to_string(),
            object: "chat.completion".to_string(),
            created: 1700000000,
            model: "gpt-4o".to_string(),
            choices: vec![Choice {
                message: ChatCompletionMessage {
                    content: None,
                    refusal: None,
                    role: "assistant".to_string(),
                    annotations: vec![],
                    tool_calls: vec![
                        ToolCall {
                            id: "call_1".to_string(),
                            r#type: "function".to_string(),
                            function: Function {
                                name: "get_weather".to_string(),
                                arguments: r#"{"location":"NYC"}"#.to_string(),
                            },
                        },
                        ToolCall {
                            id: "call_2".to_string(),
                            r#type: "function".to_string(),
                            function: Function {
                                name: "get_time".to_string(),
                                arguments: r#"{"timezone":"EST"}"#.to_string(),
                            },
                        },
                    ],
                    audio: None,
                },
                finish_reason: "tool_calls".to_string(),
                logprobs: None,
            }],
            usage: Usage {
                completion_tokens: 5,
                prompt_tokens: 15,
                total_tokens: 20,
                completion_tokens_details: CompletionTokenDetails::default(),
                prompt_tokens_details: PromptTokenDetails::default(),
                finish_reason: None,
            },
            service_tier: None,
            system_fingerprint: None,
        }
    }

    fn make_empty_response() -> OpenAIChatResponse {
        OpenAIChatResponse {
            id: "chatcmpl-empty".to_string(),
            object: "chat.completion".to_string(),
            created: 1700000000,
            model: "gpt-4o".to_string(),
            choices: vec![],
            usage: Usage::default(),
            service_tier: None,
            system_fingerprint: None,
        }
    }

    #[test]
    fn test_id() {
        let resp = make_text_response("hello");
        assert_eq!(resp.id(), "chatcmpl-abc123");
    }

    #[test]
    fn test_is_empty() {
        assert!(!make_text_response("hello").is_empty());
        assert!(make_empty_response().is_empty());
    }

    #[test]
    fn test_response_text() {
        assert_eq!(
            make_text_response("hello world").response_text(),
            "hello world"
        );
        assert_eq!(make_empty_response().response_text(), "");
    }

    #[test]
    fn test_response_text_with_no_content() {
        let resp = make_tool_call_response();
        assert_eq!(resp.response_text(), "");
    }

    #[test]
    fn test_model_name() {
        assert_eq!(make_text_response("x").model_name(), Some("gpt-4o"));
    }

    #[test]
    fn test_finish_reason() {
        assert_eq!(make_text_response("x").finish_reason(), Some("stop"));
        assert_eq!(
            make_tool_call_response().finish_reason(),
            Some("tool_calls")
        );
        assert_eq!(make_empty_response().finish_reason(), None);
    }

    #[test]
    fn test_token_counts() {
        let resp = make_text_response("hello");
        assert_eq!(resp.input_tokens(), Some(20));
        assert_eq!(resp.output_tokens(), Some(10));
        assert_eq!(resp.total_tokens(), Some(30));
    }

    #[test]
    fn test_get_tool_calls() {
        let calls = make_tool_call_response().get_tool_calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "get_weather");
        assert_eq!(calls[0].call_id, Some("call_1".to_string()));
        assert_eq!(calls[1].name, "get_time");
        assert_eq!(calls[1].call_id, Some("call_2".to_string()));
    }

    #[test]
    fn test_get_tool_calls_empty() {
        assert!(make_text_response("hello").get_tool_calls().is_empty());
        assert!(make_empty_response().get_tool_calls().is_empty());
    }

    #[test]
    fn test_tool_call_output() {
        let resp = make_tool_call_response();
        let output = resp.tool_call_output();
        assert!(output.is_some());
        let arr = output.unwrap();
        assert!(arr.is_array());
        assert_eq!(arr.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_tool_call_output_none_for_text() {
        assert!(make_text_response("hello").tool_call_output().is_none());
        assert!(make_empty_response().tool_call_output().is_none());
    }

    #[test]
    fn test_structured_output_value_valid_json() {
        let resp = make_text_response(r#"{"name":"Alice","age":30}"#);
        let val = resp.structured_output_value();
        assert!(val.is_some());
        let obj = val.unwrap();
        assert_eq!(obj["name"], "Alice");
        assert_eq!(obj["age"], 30);
    }

    #[test]
    fn test_structured_output_value_plain_text() {
        let resp = make_text_response("just plain text");
        assert!(resp.structured_output_value().is_none());
    }

    #[test]
    fn test_structured_output_value_empty() {
        assert!(make_empty_response().structured_output_value().is_none());
    }

    #[test]
    fn test_to_message_num() {
        let resp = make_text_response("hello");
        let msgs = resp.to_message_num().unwrap();
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn test_to_message_num_empty() {
        let resp = make_empty_response();
        let msgs = resp.to_message_num().unwrap();
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_get_log_probs_with_digits() {
        let resp = OpenAIChatResponse {
            choices: vec![Choice {
                message: ChatCompletionMessage::default(),
                finish_reason: "stop".to_string(),
                logprobs: Some(LogProbs {
                    content: Some(vec![
                        LogContent {
                            bytes: None,
                            logprob: -0.5,
                            token: "3".to_string(),
                            top_logprobs: None,
                        },
                        LogContent {
                            bytes: None,
                            logprob: -1.0,
                            token: "hello".to_string(),
                            top_logprobs: None,
                        },
                        LogContent {
                            bytes: None,
                            logprob: -0.2,
                            token: "5".to_string(),
                            top_logprobs: None,
                        },
                    ]),
                    refusal: None,
                }),
            }],
            ..Default::default()
        };
        let probs = resp.get_log_probs();
        assert_eq!(probs.len(), 2);
        assert_eq!(probs[0].token, "3");
        assert!((probs[0].logprob - (-0.5)).abs() < f64::EPSILON);
        assert_eq!(probs[1].token, "5");
    }

    #[test]
    fn test_get_log_probs_empty() {
        assert!(make_text_response("hello").get_log_probs().is_empty());
        assert!(make_empty_response().get_log_probs().is_empty());
    }

    #[test]
    fn test_deserialize_from_json() {
        let json = serde_json::json!({
            "id": "chatcmpl-test",
            "object": "chat.completion",
            "created": 1700000000,
            "model": "gpt-4o",
            "choices": [{
                "message": {
                    "content": "Hello!",
                    "role": "assistant"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "completion_tokens": 5,
                "prompt_tokens": 10,
                "total_tokens": 15
            }
        });
        let resp: OpenAIChatResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.response_text(), "Hello!");
        assert_eq!(resp.model_name(), Some("gpt-4o"));
        assert_eq!(resp.finish_reason(), Some("stop"));
        assert_eq!(resp.input_tokens(), Some(10));
        assert_eq!(resp.output_tokens(), Some(5));
        assert_eq!(resp.total_tokens(), Some(15));
    }

    #[test]
    fn test_deserialize_tool_calls_from_json() {
        let raw = r#"{
            "id": "chatcmpl-abc123",
            "object": "chat.completion",
            "created": 1700000000,
            "model": "gpt-4o",
            "choices": [{
                "message": {
                    "content": null,
                    "role": "assistant",
                    "tool_calls": [
                        {
                            "id": "call_abc",
                            "type": "function",
                            "function": {
                                "name": "get_weather",
                                "arguments": "{\"location\":\"San Francisco\",\"unit\":\"celsius\"}"
                            }
                        },
                        {
                            "id": "call_def",
                            "type": "function",
                            "function": {
                                "name": "get_stock_price",
                                "arguments": "{\"ticker\":\"AAPL\"}"
                            }
                        }
                    ]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {
                "completion_tokens": 50,
                "prompt_tokens": 100,
                "total_tokens": 150
            }
        }"#;

        let resp: OpenAIChatResponse = serde_json::from_str(raw).unwrap();
        let tool_calls = resp.get_tool_calls();

        assert_eq!(tool_calls.len(), 2);

        assert_eq!(tool_calls[0].name, "get_weather");
        assert_eq!(tool_calls[0].call_id, Some("call_abc".to_string()));
        assert!(tool_calls[0].result.is_none());

        assert_eq!(tool_calls[1].name, "get_stock_price");
        assert_eq!(tool_calls[1].call_id, Some("call_def".to_string()));
        assert!(tool_calls[1].result.is_none());
    }
}
