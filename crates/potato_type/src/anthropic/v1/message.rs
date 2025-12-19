use crate::TypeError;
use potato_util::json_to_pydict;
use potato_util::{pyobject_to_json, PyHelperFuncs, UtilError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Keep the original enum for internal Rust usage (no pyclass)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
        #[serde(skip_serializing_if = "Option::is_none")]
        citations: Option<Value>,
    },
    Image {
        source: ImageSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    Document {
        source: DocumentSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        context: Option<String>,
    },
    ToolUse {
        id: String,
        name: String,
        input: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    ToolResult {
        tool_use_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<Vec<ContentBlock>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    Thinking {
        thinking: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    RedactedThinking {
        data: String,
    },
    SearchResult {
        title: String,
        content: Vec<ContentBlock>,
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
        #[serde(skip_serializing_if = "Option::is_none")]
        citations: Option<Value>,
    },
    WebSearchToolResult {
        tool_use_id: String,
        content: Vec<ContentBlock>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    ServerToolUse {
        id: String,
        name: String,
        input: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
}

// Python wrapper struct that holds the enum
#[pyclass(name = "ContentBlock")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PyContentBlock {
    #[serde(flatten)]
    pub inner: ContentBlock,
}

#[pymethods]
impl PyContentBlock {
    #[staticmethod]
    #[pyo3(signature = (text, cache_control=None, citations=None))]
    pub fn text(
        text: String,
        cache_control: Option<CacheControl>,
        citations: Option<&Bound<'_, PyAny>>,
    ) -> Result<Self, UtilError> {
        let citations_value = match citations {
            Some(cit) => Some(pyobject_to_json(cit)?),
            None => None,
        };

        Ok(Self {
            inner: ContentBlock::Text {
                text,
                cache_control,
                citations: citations_value,
            },
        })
    }

    #[staticmethod]
    #[pyo3(signature = (media_type, data, cache_control=None))]
    pub fn image_base64(
        media_type: String,
        data: String,
        cache_control: Option<CacheControl>,
    ) -> Self {
        Self {
            inner: ContentBlock::Image {
                source: ImageSource::Base64 { media_type, data },
                cache_control,
            },
        }
    }

    #[staticmethod]
    #[pyo3(signature = (url, cache_control=None))]
    pub fn image_url(url: String, cache_control: Option<CacheControl>) -> Self {
        Self {
            inner: ContentBlock::Image {
                source: ImageSource::Url { url },
                cache_control,
            },
        }
    }

    #[staticmethod]
    #[pyo3(signature = (media_type, data, cache_control=None, title=None, context=None))]
    pub fn document_base64(
        media_type: String,
        data: String,
        cache_control: Option<CacheControl>,
        title: Option<String>,
        context: Option<String>,
    ) -> Self {
        Self {
            inner: ContentBlock::Document {
                source: DocumentSource::Base64 { media_type, data },
                cache_control,
                title,
                context,
            },
        }
    }

    #[staticmethod]
    #[pyo3(signature = (url, cache_control=None, title=None, context=None))]
    pub fn document_url(
        url: String,
        cache_control: Option<CacheControl>,
        title: Option<String>,
        context: Option<String>,
    ) -> Self {
        Self {
            inner: ContentBlock::Document {
                source: DocumentSource::Url { url },
                cache_control,
                title,
                context,
            },
        }
    }

    #[staticmethod]
    #[pyo3(signature = (id, name, input, cache_control=None))]
    pub fn tool_use(
        id: String,
        name: String,
        input: &Bound<'_, PyAny>,
        cache_control: Option<CacheControl>,
    ) -> Result<Self, UtilError> {
        let input_value = pyobject_to_json(input)?;

        Ok(Self {
            inner: ContentBlock::ToolUse {
                id,
                name,
                input: input_value,
                cache_control,
            },
        })
    }

    #[staticmethod]
    #[pyo3(signature = (tool_use_id, content=None, is_error=None, cache_control=None))]
    pub fn tool_result(
        tool_use_id: String,
        content: Option<Vec<PyContentBlock>>,
        is_error: Option<bool>,
        cache_control: Option<CacheControl>,
    ) -> Self {
        Self {
            inner: ContentBlock::ToolResult {
                tool_use_id,
                content: content.map(|blocks| blocks.into_iter().map(|b| b.inner).collect()),
                is_error,
                cache_control,
            },
        }
    }

    #[staticmethod]
    #[pyo3(signature = (thinking, signature=None))]
    pub fn thinking(thinking: String, signature: Option<String>) -> Self {
        Self {
            inner: ContentBlock::Thinking {
                thinking,
                signature,
            },
        }
    }

    #[staticmethod]
    pub fn redacted_thinking(data: String) -> Self {
        Self {
            inner: ContentBlock::RedactedThinking { data },
        }
    }

    #[staticmethod]
    #[pyo3(signature = (title, content, source, cache_control=None, citations=None))]
    pub fn search_result(
        title: String,
        content: Vec<PyContentBlock>,
        source: String,
        cache_control: Option<CacheControl>,
        citations: Option<&Bound<'_, PyAny>>,
    ) -> Result<Self, UtilError> {
        let citations_value = match citations {
            Some(cit) => Some(pyobject_to_json(cit)?),
            None => None,
        };

        Ok(Self {
            inner: ContentBlock::SearchResult {
                title,
                content: content.into_iter().map(|b| b.inner).collect(),
                source,
                cache_control,
                citations: citations_value,
            },
        })
    }

    #[getter]
    pub fn block_type(&self) -> &str {
        match &self.inner {
            ContentBlock::Text { .. } => "text",
            ContentBlock::Image { .. } => "image",
            ContentBlock::Document { .. } => "document",
            ContentBlock::ToolUse { .. } => "tool_use",
            ContentBlock::ToolResult { .. } => "tool_result",
            ContentBlock::Thinking { .. } => "thinking",
            ContentBlock::RedactedThinking { .. } => "redacted_thinking",
            ContentBlock::SearchResult { .. } => "search_result",
            ContentBlock::WebSearchToolResult { .. } => "web_search_tool_result",
            ContentBlock::ServerToolUse { .. } => "server_tool_use",
        }
    }

    /// Check if this is a text block
    pub fn is_text(&self) -> bool {
        matches!(&self.inner, ContentBlock::Text { .. })
    }

    /// Check if this is an image block
    pub fn is_image(&self) -> bool {
        matches!(&self.inner, ContentBlock::Image { .. })
    }

    /// Check if this is a tool use block
    pub fn is_tool_use(&self) -> bool {
        matches!(&self.inner, ContentBlock::ToolUse { .. })
    }

    pub fn model_dump_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

// Conversion traits for seamless Rust usage
impl From<ContentBlock> for PyContentBlock {
    fn from(inner: ContentBlock) -> Self {
        Self { inner }
    }
}

impl From<PyContentBlock> for ContentBlock {
    fn from(py_block: PyContentBlock) -> Self {
        py_block.inner
    }
}

// Keep existing types unchanged
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    Base64 { media_type: String, data: String },
    Url { url: String },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DocumentSource {
    Base64 { media_type: String, data: String },
    Url { url: String },
    Text { media_type: String, data: String },
    Content { content: Vec<ContentBlock> },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

#[pymethods]
impl Metadata {
    #[new]
    pub fn new(user_id: Option<String>) -> Self {
        Self { user_id }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub cache_type: String, // "ephemeral"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>, // "5m" or "1h"
}

#[pymethods]
impl CacheControl {
    #[new]
    pub fn new(cache_type: String, ttl: Option<String>) -> Self {
        Self { cache_type, ttl }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input_schema: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[pymethods]
impl Tool {
    #[new]
    pub fn new(
        name: String,
        description: Option<String>,
        input_schema: &Bound<'_, PyAny>,
        cache_control: Option<CacheControl>,
    ) -> Result<Self, UtilError> {
        Ok(Self {
            name,
            description,
            input_schema: pyobject_to_json(input_schema)?,
            cache_control,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct ThinkingConfig {
    #[pyo3(get)]
    pub r#type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub budget_tokens: Option<i32>,
}

#[pymethods]
impl ThinkingConfig {
    #[new]
    pub fn new(r#type: String, budget_tokens: Option<i32>) -> Self {
        Self {
            r#type,
            budget_tokens,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct ToolChoice {
    #[pyo3(get)]
    pub r#type: String, // "auto", "any", "tool", "none"

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    disable_parallel_tool_use: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub name: Option<String>,
}

#[pymethods]
impl ToolChoice {
    #[new]
    pub fn new(
        r#type: String,
        disable_parallel_tool_use: Option<bool>,
        name: Option<String>,
    ) -> Result<Self, UtilError> {
        match name {
            Some(_) if r#type != "tool" => {
                return Err(UtilError::PyError(
                    "ToolChoice name can only be set if type is 'tool'".to_string(),
                ))
            }
            None if r#type == "tool" => {
                return Err(UtilError::PyError(
                    "ToolChoice of type 'tool' requires a name".to_string(),
                ))
            }
            _ => {}
        }

        Ok(Self {
            r#type,
            disable_parallel_tool_use,
            name,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
#[serde(default)]
pub struct AnthropicSettings {
    #[pyo3(get)]
    pub max_tokens: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub metadata: Option<Metadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub service_tier: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub stop_sequences: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub stream: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub system: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub thinking: Option<ThinkingConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub tool_choice: Option<ToolChoice>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub tools: Option<Vec<Tool>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub top_k: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub top_p: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_body: Option<Value>,
}

impl Default for AnthropicSettings {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            metadata: None,
            service_tier: None,
            stop_sequences: None,
            stream: Some(false),
            system: None,
            temperature: None,
            thinking: None,
            top_k: None,
            top_p: None,
            tools: None,
            tool_choice: None,
            extra_body: None,
        }
    }
}

#[pymethods]
impl AnthropicSettings {
    #[new]
    #[pyo3(signature = (
        max_tokens=4096,
        metadata=None,
        service_tier=None,
        stop_sequences=None,
        stream=None,
        system =None,
        temperature=None,
        thinking=None,
        top_k=None,
        top_p=None,
        tools=None,
        tool_choice=None,
        extra_body=None
    ))]
    pub fn new(
        max_tokens: i32,
        metadata: Option<Metadata>,
        service_tier: Option<String>,
        stop_sequences: Option<Vec<String>>,
        stream: Option<bool>,
        system: Option<String>,
        temperature: Option<f32>,
        thinking: Option<ThinkingConfig>,
        top_k: Option<i32>,
        top_p: Option<f32>,
        tools: Option<Vec<Tool>>,
        tool_choice: Option<ToolChoice>,
        extra_body: Option<&Bound<'_, PyAny>>,
    ) -> Result<Self, UtilError> {
        let extra = match extra_body {
            Some(obj) => Some(pyobject_to_json(obj)?),
            None => None,
        };

        Ok(Self {
            max_tokens,
            metadata,
            service_tier,
            stop_sequences,
            stream,
            system,
            temperature,
            thinking,
            top_k,
            top_p,
            tools,
            tool_choice,
            extra_body: extra,
        })
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }

    pub fn model_dump<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyDict>, TypeError> {
        // iterate over each field in model_settings and add to the dict if it is not None
        let json = serde_json::to_value(self)?;
        let pydict = PyDict::new(py);
        json_to_pydict(py, &json, &pydict)?;
        Ok(pydict)
    }
}
