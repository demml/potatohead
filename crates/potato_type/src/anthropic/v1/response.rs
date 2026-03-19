use crate::anthropic::v1::request::{ContentBlockParam, MessageParam};
use crate::prompt::Role;
use crate::prompt::{MessageNum, ResponseContent};
use crate::traits::{MessageResponseExt, ResponseAdapter};
use crate::TypeError;
use potato_util::utils::{construct_structured_response, TokenLogProbs};
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct CitationCharLocation {
    #[pyo3(get, set)]
    pub cited_text: String,
    #[pyo3(get, set)]
    pub document_index: i32,
    #[pyo3(get, set)]
    pub document_title: String,
    #[pyo3(get, set)]
    pub end_char_index: i32,
    #[pyo3(get, set)]
    pub file_id: String,
    #[pyo3(get, set)]
    pub start_char_index: i32,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct CitationPageLocation {
    #[pyo3(get, set)]
    pub cited_text: String,
    #[pyo3(get, set)]
    pub document_index: i32,
    #[pyo3(get, set)]
    pub document_title: String,
    #[pyo3(get, set)]
    pub end_page_number: i32,
    #[pyo3(get, set)]
    pub file_id: String,
    #[pyo3(get, set)]
    pub start_page_number: i32,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct CitationContentBlockLocation {
    #[pyo3(get, set)]
    pub cited_text: String,
    #[pyo3(get, set)]
    pub document_index: i32,
    #[pyo3(get, set)]
    pub document_title: String,
    #[pyo3(get, set)]
    pub end_block_index: i32,
    #[pyo3(get, set)]
    pub file_id: String,
    #[pyo3(get, set)]
    pub start_block_index: i32,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct CitationsWebSearchResultLocation {
    #[pyo3(get, set)]
    pub cited_text: String,
    #[pyo3(get, set)]
    pub encrypted_index: String,
    #[pyo3(get, set)]
    pub title: String,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
    #[pyo3(get, set)]
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct CitationsSearchResultLocation {
    #[pyo3(get, set)]
    pub cited_text: String,
    #[pyo3(get, set)]
    pub end_block_index: i32,
    #[pyo3(get, set)]
    pub search_result_index: i32,
    #[pyo3(get, set)]
    pub source: String,
    #[pyo3(get, set)]
    pub start_block_index: i32,
    #[pyo3(get, set)]
    pub title: String,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Untagged enum for citation types in response content
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum TextCitation {
    CharLocation(CitationCharLocation),
    PageLocation(CitationPageLocation),
    ContentBlockLocation(CitationContentBlockLocation),
    WebSearchResultLocation(CitationsWebSearchResultLocation),
    SearchResultLocation(CitationsSearchResultLocation),
}

/// Text block in response content with citations
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct TextBlock {
    #[pyo3(get, set)]
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<TextCitation>>,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl TextBlock {
    #[getter]
    pub fn citations<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Option<Vec<Bound<'py, PyAny>>>, TypeError> {
        match &self.citations {
            None => Ok(None),
            Some(cits) => {
                let py_citations: Result<Vec<_>, _> = cits
                    .iter()
                    .map(|cit| match cit {
                        TextCitation::CharLocation(c) => c.clone().into_bound_py_any(py),
                        TextCitation::PageLocation(c) => c.clone().into_bound_py_any(py),
                        TextCitation::ContentBlockLocation(c) => c.clone().into_bound_py_any(py),
                        TextCitation::WebSearchResultLocation(c) => c.clone().into_bound_py_any(py),
                        TextCitation::SearchResultLocation(c) => c.clone().into_bound_py_any(py),
                    })
                    .collect();
                Ok(Some(py_citations?))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct ThinkingBlock {
    #[pyo3(get, set)]
    pub thinking: String,
    #[pyo3(get, set)]
    pub signature: Option<String>,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct RedactedThinkingBlock {
    #[pyo3(get, set)]
    pub data: String,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct ToolUseBlock {
    #[pyo3(get, set)]
    pub id: String,
    #[pyo3(get, set)]
    pub name: String,
    pub input: Value,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct ServerToolUseBlock {
    #[pyo3(get, set)]
    pub id: String,
    #[pyo3(get, set)]
    pub name: String,
    pub input: Value,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct WebSearchResultBlock {
    #[pyo3(get, set)]
    pub encrypted_content: String,
    #[pyo3(get, set)]
    pub page_age: Option<String>,
    #[pyo3(get, set)]
    pub title: String,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
    #[pyo3(get, set)]
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct WebSearchToolResultError {
    #[pyo3(get, set)]
    pub error_code: String,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum WebSearchToolResultBlockContent {
    Error(WebSearchToolResultError),
    Results(Vec<WebSearchResultBlock>),
}

/// Web search tool result block
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct WebSearchToolResultBlock {
    pub content: WebSearchToolResultBlockContent,
    #[pyo3(get, set)]
    pub tool_use_id: String,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl WebSearchToolResultBlock {
    #[getter]
    pub fn content<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        match &self.content {
            WebSearchToolResultBlockContent::Error(err) => Ok(err.clone().into_bound_py_any(py)?),
            WebSearchToolResultBlockContent::Results(results) => {
                let py_list: Result<Vec<_>, _> = results
                    .iter()
                    .map(|r| r.clone().into_bound_py_any(py))
                    .collect();
                Ok(py_list?.into_bound_py_any(py)?)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub(crate) enum ResponseContentBlockInner {
    Text(TextBlock),
    Thinking(ThinkingBlock),
    RedactedThinking(RedactedThinkingBlock),
    ToolUse(ToolUseBlock),
    ServerToolUse(ServerToolUseBlock),
    WebSearchToolResult(WebSearchToolResultBlock),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ResponseContentBlock {
    #[serde(flatten)]
    inner: ResponseContentBlockInner,
}

impl ResponseContentBlock {
    /// Convert back to Python object
    pub fn to_pyobject<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        match &self.inner {
            ResponseContentBlockInner::Text(block) => Ok(block.clone().into_bound_py_any(py)?),
            ResponseContentBlockInner::Thinking(block) => {
                Ok(block.clone().into_bound_py_any(py)?)
            }
            ResponseContentBlockInner::RedactedThinking(block) => {
                Ok(block.clone().into_bound_py_any(py)?)
            }
            ResponseContentBlockInner::ToolUse(block) => Ok(block.clone().into_bound_py_any(py)?),
            ResponseContentBlockInner::ServerToolUse(block) => {
                Ok(block.clone().into_bound_py_any(py)?)
            }
            ResponseContentBlockInner::WebSearchToolResult(block) => {
                Ok(block.clone().into_bound_py_any(py)?)
            }
        }
    }
}

impl MessageResponseExt for ResponseContentBlock {
    fn to_message_num(&self) -> Result<MessageNum, TypeError> {
        // Convert the response content block to a request content block parameter
        let content_block_param = ContentBlockParam::from_response_content_block(&self.inner)?;

        // Create a MessageParam with the converted content block
        let message_param = MessageParam {
            content: vec![content_block_param],
            role: Role::Assistant.to_string(),
        };

        // Convert MessageParam to MessageNum
        Ok(MessageNum::AnthropicMessageV1(message_param))
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
#[pyclass(name = "AnthropicUsage")]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct AnthropicMessageResponse {
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
    pub content: Vec<ResponseContentBlock>,
}

#[pymethods]
impl AnthropicMessageResponse {
    #[getter]
    pub fn content<'py>(&self, py: Python<'py>) -> Result<Vec<Bound<'py, PyAny>>, TypeError> {
        self.content
            .iter()
            .map(|block| block.to_pyobject(py))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TypeError::Error(e.to_string()))
    }
}

impl ResponseAdapter for AnthropicMessageResponse {
    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }

    fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    fn to_bound_py_object<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        Ok(PyHelperFuncs::to_bound_py_object(py, self)?)
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn to_message_num(&self) -> Result<Vec<MessageNum>, TypeError> {
        let mut results = Vec::new();
        for content in &self.content {
            match content.to_message_num() {
                Ok(msg) => results.push(msg),
                Err(e) => return Err(e),
            }
        }
        Ok(results)
    }

    fn get_content(&self) -> ResponseContent {
        ResponseContent::Anthropic(self.content.first().cloned().unwrap_or_else(|| {
            ResponseContentBlock {
                inner: ResponseContentBlockInner::Text(TextBlock {
                    text: String::new(),
                    citations: None,
                    r#type: "text".to_string(),
                }),
            }
        }))
    }

    fn tool_call_output(&self) -> Option<Value> {
        for block in &self.content {
            if let ResponseContentBlockInner::ToolUse(tool_use_block) = &block.inner {
                return serde_json::to_value(tool_use_block).ok();
            }
        }
        None
    }

    fn structured_output<'py>(
        &self,
        py: Python<'py>,
        output_model: Option<&Bound<'py, PyAny>>,
    ) -> Result<Bound<'py, PyAny>, TypeError> {
        if self.content.is_empty() {
            return Ok(py.None().into_bound_py_any(py)?);
        }

        let text = self
            .content
            .iter()
            .rev()
            .find_map(|block| {
                if let ResponseContentBlockInner::Text(text_block) = &block.inner {
                    Some(text_block.text.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        if text.is_empty() {
            return Ok(py.None().into_bound_py_any(py)?);
        }
        Ok(construct_structured_response(py, text, output_model)?)
    }

    fn structured_output_value(&self) -> Option<Value> {
        let text = self.content.iter().rev().find_map(|block| {
            if let ResponseContentBlockInner::Text(text_block) = &block.inner {
                Some(text_block.text.clone())
            } else {
                None
            }
        })?;
        serde_json::from_str(&text).ok()
    }

    fn usage<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        Ok(PyHelperFuncs::to_bound_py_object(py, &self.usage)?)
    }

    fn get_log_probs(&self) -> Vec<TokenLogProbs> {
        // Anthropic responses do not include log probabilities
        Vec::new()
    }

    fn response_text(&self) -> String {
        self.content
            .iter()
            .rev()
            .find_map(|block| {
                if let ResponseContentBlockInner::Text(text_block) = &block.inner {
                    Some(text_block.text.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    fn model_name(&self) -> Option<&str> {
        Some(&self.model)
    }

    fn finish_reason(&self) -> Option<&str> {
        self.stop_reason.as_ref().map(|reason| match reason {
            StopReason::EndTurn => "end_turn",
            StopReason::MaxTokens => "max_tokens",
            StopReason::StopSequence => "stop_sequence",
            StopReason::ToolUse => "tool_use",
        })
    }

    fn input_tokens(&self) -> Option<i64> {
        Some(self.usage.input_tokens as i64)
    }

    fn output_tokens(&self) -> Option<i64> {
        Some(self.usage.output_tokens as i64)
    }

    fn total_tokens(&self) -> Option<i64> {
        Some(self.usage.input_tokens as i64 + self.usage.output_tokens as i64)
    }

    fn get_tool_calls(&self) -> Vec<crate::tools::ToolCallInfo> {
        let mut tool_calls = Vec::new();
        for block in &self.content {
            if let ResponseContentBlockInner::ToolUse(tool_use_block) = &block.inner {
                tool_calls.push(crate::tools::ToolCallInfo {
                    name: tool_use_block.name.clone(),
                    arguments: tool_use_block.input.clone(),
                    call_id: Some(tool_use_block.id.clone()),
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

    fn make_text_block(text: &str) -> ResponseContentBlock {
        ResponseContentBlock {
            inner: ResponseContentBlockInner::Text(TextBlock {
                text: text.to_string(),
                citations: None,
                r#type: "text".to_string(),
            }),
        }
    }

    fn make_thinking_block(thinking: &str) -> ResponseContentBlock {
        ResponseContentBlock {
            inner: ResponseContentBlockInner::Thinking(ThinkingBlock {
                thinking: thinking.to_string(),
                signature: None,
                r#type: "thinking".to_string(),
            }),
        }
    }

    fn make_tool_use_block(id: &str, name: &str, input: Value) -> ResponseContentBlock {
        ResponseContentBlock {
            inner: ResponseContentBlockInner::ToolUse(ToolUseBlock {
                id: id.to_string(),
                name: name.to_string(),
                input,
                r#type: "tool_use".to_string(),
            }),
        }
    }

    fn make_usage() -> Usage {
        Usage {
            input_tokens: 100,
            output_tokens: 50,
            cache_creation_input_tokens: Some(10),
            cache_read_input_tokens: Some(5),
            service_tier: None,
        }
    }

    fn make_text_response(text: &str) -> AnthropicMessageResponse {
        AnthropicMessageResponse {
            id: "msg_abc123".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            role: "assistant".to_string(),
            stop_reason: Some(StopReason::EndTurn),
            stop_sequence: None,
            r#type: "message".to_string(),
            usage: make_usage(),
            content: vec![make_text_block(text)],
        }
    }

    fn make_tool_use_response() -> AnthropicMessageResponse {
        AnthropicMessageResponse {
            id: "msg_tool456".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            role: "assistant".to_string(),
            stop_reason: Some(StopReason::ToolUse),
            stop_sequence: None,
            r#type: "message".to_string(),
            usage: make_usage(),
            content: vec![
                make_text_block("I'll look up the weather for you."),
                make_tool_use_block(
                    "toolu_01",
                    "get_weather",
                    serde_json::json!({"location": "NYC"}),
                ),
            ],
        }
    }

    fn make_empty_response() -> AnthropicMessageResponse {
        AnthropicMessageResponse {
            id: "msg_empty".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            role: "assistant".to_string(),
            stop_reason: None,
            stop_sequence: None,
            r#type: "message".to_string(),
            usage: make_usage(),
            content: vec![],
        }
    }

    #[test]
    fn test_id() {
        assert_eq!(make_text_response("hi").id(), "msg_abc123");
    }

    #[test]
    fn test_is_empty() {
        assert!(!make_text_response("hi").is_empty());
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
    fn test_response_text_tool_use_first_block() {
        let resp = make_tool_use_response();
        assert_eq!(resp.response_text(), "I'll look up the weather for you.");
    }

    #[test]
    fn test_response_text_thinking_block_first() {
        let resp = AnthropicMessageResponse {
            id: "msg_think".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            role: "assistant".to_string(),
            stop_reason: Some(StopReason::EndTurn),
            stop_sequence: None,
            r#type: "message".to_string(),
            usage: make_usage(),
            content: vec![
                make_thinking_block("let me think..."),
                make_text_block("final"),
            ],
        };
        assert_eq!(resp.response_text(), "final");
    }

    #[test]
    fn test_response_text_only_thinking_block() {
        let resp = AnthropicMessageResponse {
            id: "msg_think_only".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            role: "assistant".to_string(),
            stop_reason: Some(StopReason::EndTurn),
            stop_sequence: None,
            r#type: "message".to_string(),
            usage: make_usage(),
            content: vec![make_thinking_block("only thinking, no text")],
        };
        assert_eq!(resp.response_text(), "");
    }

    #[test]
    fn test_response_text_tool_use_then_text() {
        let resp = AnthropicMessageResponse {
            id: "msg_reversed".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            role: "assistant".to_string(),
            stop_reason: Some(StopReason::EndTurn),
            stop_sequence: None,
            r#type: "message".to_string(),
            usage: make_usage(),
            content: vec![
                make_tool_use_block("toolu_01", "get_weather", serde_json::json!({})),
                make_text_block("final answer"),
            ],
        };
        assert_eq!(resp.response_text(), "final answer");
    }

    #[test]
    fn test_model_name() {
        assert_eq!(
            make_text_response("x").model_name(),
            Some("claude-sonnet-4-20250514")
        );
    }

    #[test]
    fn test_finish_reason() {
        assert_eq!(make_text_response("x").finish_reason(), Some("end_turn"));
        assert_eq!(make_tool_use_response().finish_reason(), Some("tool_use"));
        assert_eq!(make_empty_response().finish_reason(), None);
    }

    #[test]
    fn test_finish_reason_all_variants() {
        let mut resp = make_text_response("x");
        resp.stop_reason = Some(StopReason::MaxTokens);
        assert_eq!(resp.finish_reason(), Some("max_tokens"));
        resp.stop_reason = Some(StopReason::StopSequence);
        assert_eq!(resp.finish_reason(), Some("stop_sequence"));
    }

    #[test]
    fn test_token_counts() {
        let resp = make_text_response("x");
        assert_eq!(resp.input_tokens(), Some(100));
        assert_eq!(resp.output_tokens(), Some(50));
        assert_eq!(resp.total_tokens(), Some(150));
    }

    #[test]
    fn test_get_tool_calls() {
        let calls = make_tool_use_response().get_tool_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "get_weather");
        assert_eq!(calls[0].call_id, Some("toolu_01".to_string()));
        assert_eq!(calls[0].arguments, serde_json::json!({"location": "NYC"}));
    }

    #[test]
    fn test_get_tool_calls_empty() {
        assert!(make_text_response("hello").get_tool_calls().is_empty());
        assert!(make_empty_response().get_tool_calls().is_empty());
    }

    #[test]
    fn test_tool_call_output() {
        let resp = make_tool_use_response();
        let output = resp.tool_call_output();
        assert!(output.is_some());
    }

    #[test]
    fn test_tool_call_output_none_for_text() {
        assert!(make_text_response("hello").tool_call_output().is_none());
    }

    #[test]
    fn test_structured_output_value_valid_json() {
        let resp = make_text_response(r#"{"name":"Bob","score":95}"#);
        let val = resp.structured_output_value();
        assert!(val.is_some());
        let obj = val.unwrap();
        assert_eq!(obj["name"], "Bob");
        assert_eq!(obj["score"], 95);
    }

    #[test]
    fn test_structured_output_value_plain_text() {
        assert!(make_text_response("not json")
            .structured_output_value()
            .is_none());
    }

    #[test]
    fn test_structured_output_value_empty() {
        assert!(make_empty_response().structured_output_value().is_none());
    }

    #[test]
    fn test_get_content_empty_response_no_panic() {
        let resp = make_empty_response();
        let content = resp.get_content();
        // Should return an empty text block rather than panic
        matches!(content, ResponseContent::Anthropic(_));
    }

    #[test]
    fn test_structured_output_thinking_block_first() {
        let resp = AnthropicMessageResponse {
            id: "msg_think_struct".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            role: "assistant".to_string(),
            stop_reason: Some(StopReason::EndTurn),
            stop_sequence: None,
            r#type: "message".to_string(),
            usage: make_usage(),
            content: vec![
                make_thinking_block("let me think..."),
                make_text_block(r#"{"name":"Alice","score":99}"#),
            ],
        };
        let val = resp.structured_output_value();
        assert!(val.is_some());
        let obj = val.unwrap();
        assert_eq!(obj["name"], "Alice");
        assert_eq!(obj["score"], 99);
    }

    #[test]
    fn test_get_log_probs_always_empty() {
        assert!(make_text_response("x").get_log_probs().is_empty());
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
    fn test_deserialize_from_json() {
        let json = serde_json::json!({
            "id": "msg_test",
            "model": "claude-sonnet-4-20250514",
            "role": "assistant",
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "type": "message",
            "usage": {
                "input_tokens": 25,
                "output_tokens": 10
            },
            "content": [{
                "type": "text",
                "text": "Hello from Anthropic!"
            }]
        });
        let resp: AnthropicMessageResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.response_text(), "Hello from Anthropic!");
        assert_eq!(resp.model_name(), Some("claude-sonnet-4-20250514"));
        assert_eq!(resp.finish_reason(), Some("end_turn"));
        assert_eq!(resp.input_tokens(), Some(25));
        assert_eq!(resp.output_tokens(), Some(10));
        assert_eq!(resp.total_tokens(), Some(35));
    }

    #[test]
    fn test_multiple_tool_use_blocks() {
        let resp = AnthropicMessageResponse {
            id: "msg_multi".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            role: "assistant".to_string(),
            stop_reason: Some(StopReason::ToolUse),
            stop_sequence: None,
            r#type: "message".to_string(),
            usage: make_usage(),
            content: vec![
                make_tool_use_block("toolu_01", "search", serde_json::json!({"q": "rust"})),
                make_tool_use_block("toolu_02", "read_file", serde_json::json!({"path": "/tmp"})),
            ],
        };
        let calls = resp.get_tool_calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "search");
        assert_eq!(calls[1].name, "read_file");
    }

    #[test]
    fn test_deserialize_tool_calls_from_json() {
        let raw = r#"{
            "id": "msg_01XFDUDYJgAACzvnptvVoYEL",
            "type": "message",
            "role": "assistant",
            "model": "claude-sonnet-4-20250514",
            "stop_reason": "tool_use",
            "stop_sequence": null,
            "usage": {
                "input_tokens": 384,
                "output_tokens": 120
            },
            "content": [
                {
                    "type": "text",
                    "text": "I'll check the weather and stock price for you."
                },
                {
                    "type": "tool_use",
                    "id": "toolu_01A09q90qw90lq917835lq9",
                    "name": "get_weather",
                    "input": {"location": "San Francisco", "unit": "celsius"}
                },
                {
                    "type": "tool_use",
                    "id": "toolu_02B19r91rw91mr928946mr0",
                    "name": "get_stock_price",
                    "input": {"ticker": "AAPL"}
                }
            ]
        }"#;

        let resp: AnthropicMessageResponse = serde_json::from_str(raw).unwrap();
        let tool_calls = resp.get_tool_calls();

        assert_eq!(tool_calls.len(), 2);

        assert_eq!(tool_calls[0].name, "get_weather");
        assert_eq!(
            tool_calls[0].call_id,
            Some("toolu_01A09q90qw90lq917835lq9".to_string())
        );
        assert_eq!(
            tool_calls[0].arguments,
            serde_json::json!({"location": "San Francisco", "unit": "celsius"})
        );
        assert!(tool_calls[0].result.is_none());

        assert_eq!(tool_calls[1].name, "get_stock_price");
        assert_eq!(
            tool_calls[1].call_id,
            Some("toolu_02B19r91rw91mr928946mr0".to_string())
        );
        assert_eq!(
            tool_calls[1].arguments,
            serde_json::json!({"ticker": "AAPL"})
        );
        assert!(tool_calls[1].result.is_none());
    }
}
