use crate::anthropic::v1::request::{ContentBlockParam, MessageParam};
use crate::prompt::Role;
use crate::prompt::{MessageNum, ResponseContent};
use crate::traits::{MessageResponseExt, ResponseAdapter};
use crate::TypeError;
use potato_util::utils::construct_structured_response;
use potato_util::utils::ResponseLogProbs;
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
#[pyclass]
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
    pub content: Vec<ResponseContentBlock>,
}

#[pymethods]
impl AnthropicChatResponse {
    #[getter]
    pub fn content<'py>(&self, py: Python<'py>) -> Result<Vec<Bound<'py, PyAny>>, TypeError> {
        self.content
            .iter()
            .map(|block| block.to_pyobject(py))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TypeError::Error(e.to_string()))
    }
}

impl ResponseAdapter for AnthropicChatResponse {
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
        ResponseContent::Anthropic(self.content.first().cloned().unwrap())
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

        let inner = self.content.first().cloned().unwrap().inner;

        match inner {
            ResponseContentBlockInner::Text(block) => {
                return Ok(construct_structured_response(py, block.text, output_model)?)
            }
            _ => return Ok(py.None().into_bound_py_any(py)?),
        };
    }

    fn structured_output_value(&self) -> Option<Value> {
        if self.content.is_empty() {
            return None;
        }

        let inner = self.content.first().cloned().unwrap().inner;
        match inner {
            ResponseContentBlockInner::Text(block) => serde_json::from_str(&block.text).ok(),
            _ => None,
        }
    }

    fn usage<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        Ok(PyHelperFuncs::to_bound_py_object(py, &self.usage)?)
    }

    fn get_log_probs(&self) -> Vec<ResponseLogProbs> {
        // Anthropic responses do not include log probabilities
        Vec::new()
    }

    fn response_text(&self) -> Option<String> {
        if self.content.is_empty() {
            return None;
        }

        let inner = self.content.first().cloned().unwrap().inner;

        match inner {
            ResponseContentBlockInner::Text(block) => Some(block.text),
            _ => None,
        }
    }
}
