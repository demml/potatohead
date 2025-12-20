use crate::common::get_image_media_types;
use crate::TypeError;
use potato_util::{json_to_pydict, json_to_pyobject};
use potato_util::{pyobject_to_json, PyHelperFuncs, UtilError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::IntoPyObjectExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// common content types used in Anthropic messages
pub const BASE64_TYPE: &str = "base64";
pub const URL_TYPE: &str = "url";
pub const EPHEMERAL_TYPE: &str = "ephemeral";
pub const IMAGE_TYPE: &str = "image";
pub const TEXT_TYPE: &str = "text";
pub const DOCUMENT_TYPE: &str = "document";
pub const DOCUMENT_BASE64_PDF_TYPE: &str = "application/pdf";
pub const DOCUMENT_PLAIN_TEXT_TYPE: &str = "text/plain";
pub const SEARCH_TYPE: &str = "search_result";
pub const THINKING_TYPE: &str = "thinking";
pub const REDACTED_THINKING_TYPE: &str = "redacted_thinking";
pub const TOOL_USE_TYPE: &str = "tool_use";
pub const TOOL_RESULT_TYPE: &str = "tool_result";
pub const WEB_SEARCH_TOOL_RESULT_TYPE: &str = "web_search_tool_result";
pub const SERVER_TOOL_USE_TYPE: &str = "server_tool_use";

// Citation type constants
pub const CHAR_LOCATION_TYPE: &str = "char_location";
pub const PAGE_LOCATION_TYPE: &str = "page_location";
pub const CONTENT_BLOCK_LOCATION_TYPE: &str = "content_block_location";
pub const WEB_SEARCH_RESULT_LOCATION_TYPE: &str = "web_search_result_location";
pub const SEARCH_RESULT_LOCATION_TYPE: &str = "search_result_location";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct CitationCharLocationParam {
    #[pyo3(get, set)]
    pub cited_text: String,
    #[pyo3(get, set)]
    pub document_index: i32,
    #[pyo3(get, set)]
    pub document_title: String,
    #[pyo3(get, set)]
    pub end_char_index: i32,
    #[pyo3(get, set)]
    pub start_char_index: i32,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl CitationCharLocationParam {
    #[new]
    pub fn new(
        cited_text: String,
        document_index: i32,
        document_title: String,
        end_char_index: i32,
        start_char_index: i32,
    ) -> Self {
        Self {
            cited_text,
            document_index,
            document_title,
            end_char_index,
            start_char_index,
            r#type: CHAR_LOCATION_TYPE.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct CitationPageLocationParam {
    #[pyo3(get, set)]
    pub cited_text: String,
    #[pyo3(get, set)]
    pub document_index: i32,
    #[pyo3(get, set)]
    pub document_title: String,
    #[pyo3(get, set)]
    pub end_page_number: i32,
    #[pyo3(get, set)]
    pub start_page_number: i32,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl CitationPageLocationParam {
    #[new]
    pub fn new(
        cited_text: String,
        document_index: i32,
        document_title: String,
        end_page_number: i32,
        start_page_number: i32,
    ) -> Self {
        Self {
            cited_text,
            document_index,
            document_title,
            end_page_number,
            start_page_number,
            r#type: PAGE_LOCATION_TYPE.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct CitationContentBlockLocationParam {
    #[pyo3(get, set)]
    pub cited_text: String,
    #[pyo3(get, set)]
    pub document_index: i32,
    #[pyo3(get, set)]
    pub document_title: String,
    #[pyo3(get, set)]
    pub end_block_index: i32,
    #[pyo3(get, set)]
    pub start_block_index: i32,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl CitationContentBlockLocationParam {
    #[new]
    pub fn new(
        cited_text: String,
        document_index: i32,
        document_title: String,
        end_block_index: i32,
        start_block_index: i32,
    ) -> Self {
        Self {
            cited_text,
            document_index,
            document_title,
            end_block_index,
            start_block_index,
            r#type: CONTENT_BLOCK_LOCATION_TYPE.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct CitationWebSearchResultLocationParam {
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

#[pymethods]
impl CitationWebSearchResultLocationParam {
    #[new]
    pub fn new(cited_text: String, encrypted_index: String, title: String, url: String) -> Self {
        Self {
            cited_text,
            encrypted_index,
            title,
            r#type: WEB_SEARCH_RESULT_LOCATION_TYPE.to_string(),
            url,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct CitationSearchResultLocationParam {
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

#[pymethods]
impl CitationSearchResultLocationParam {
    #[new]
    pub fn new(
        cited_text: String,
        end_block_index: i32,
        search_result_index: i32,
        source: String,
        start_block_index: i32,
        title: String,
    ) -> Self {
        Self {
            cited_text,
            end_block_index,
            search_result_index,
            source,
            start_block_index,
            title,
            r#type: SEARCH_RESULT_LOCATION_TYPE.to_string(),
        }
    }
}

/// Untagged enum for internal Rust usage - serializes without wrapper
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum TextCitationParam {
    CharLocation(CitationCharLocationParam),
    PageLocation(CitationPageLocationParam),
    ContentBlockLocation(CitationContentBlockLocationParam),
    WebSearchResultLocation(CitationWebSearchResultLocationParam),
    SearchResultLocation(CitationSearchResultLocationParam),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct TextBlockParam {
    #[pyo3(get, set)]
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get, set)]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<TextCitationParam>,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

fn parse_text_citation(cit: &Bound<'_, PyAny>) -> Result<TextCitationParam, TypeError> {
    if cit.is_instance_of::<CitationCharLocationParam>() {
        Ok(TextCitationParam::CharLocation(
            cit.extract::<CitationCharLocationParam>()?,
        ))
    } else if cit.is_instance_of::<CitationPageLocationParam>() {
        Ok(TextCitationParam::PageLocation(
            cit.extract::<CitationPageLocationParam>()?,
        ))
    } else if cit.is_instance_of::<CitationContentBlockLocationParam>() {
        Ok(TextCitationParam::ContentBlockLocation(
            cit.extract::<CitationContentBlockLocationParam>()?,
        ))
    } else if cit.is_instance_of::<CitationWebSearchResultLocationParam>() {
        Ok(TextCitationParam::WebSearchResultLocation(
            cit.extract::<CitationWebSearchResultLocationParam>()?,
        ))
    } else if cit.is_instance_of::<CitationSearchResultLocationParam>() {
        Ok(TextCitationParam::SearchResultLocation(
            cit.extract::<CitationSearchResultLocationParam>()?,
        ))
    } else {
        Err(TypeError::InvalidInput(
            "Invalid citation type provided".to_string(),
        ))
    }
}
#[pymethods]
impl TextBlockParam {
    #[new]
    pub fn new(
        text: String,
        cache_control: Option<CacheControl>,
        citations: Option<&Bound<'_, PyAny>>,
    ) -> Result<Self, TypeError> {
        let citations = if let Some(cit) = citations {
            Some(parse_text_citation(cit)?)
        } else {
            None
        };
        Ok(Self {
            text,
            cache_control,
            citations,
            r#type: TEXT_TYPE.to_string(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct Base64ImageSource {
    #[pyo3(get, set)]
    pub media_type: String,
    #[pyo3(get, set)]
    pub data: String,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl Base64ImageSource {
    #[new]
    pub fn new(media_type: String, data: String) -> Result<Self, TypeError> {
        // confirm media_type is an image type, otherwise raise error
        if !get_image_media_types().contains(media_type.as_str()) {
            return Err(TypeError::InvalidMediaType(media_type));
        }
        Ok(Self {
            media_type,
            data,
            r#type: BASE64_TYPE.to_string(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct UrlImageSource {
    #[pyo3(get, set)]
    pub url: String,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl UrlImageSource {
    #[new]
    pub fn new(url: String) -> Self {
        Self {
            url,
            r#type: URL_TYPE.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)] // we need to strip serde type ref
pub enum ImageSource {
    Base64(Base64ImageSource),
    Url(UrlImageSource),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct ImageBlockParam {
    pub source: ImageSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get, set)]
    pub cache_control: Option<CacheControl>,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl ImageBlockParam {
    #[new]
    pub fn new(
        source: &Bound<'_, PyAny>,
        cache_control: Option<CacheControl>,
    ) -> Result<Self, TypeError> {
        let source: ImageSource = if source.is_instance_of::<Base64ImageSource>() {
            ImageSource::Base64(source.extract::<Base64ImageSource>()?)
        } else {
            ImageSource::Url(source.extract::<UrlImageSource>()?)
        };
        Ok(Self {
            source,
            cache_control,
            r#type: IMAGE_TYPE.to_string(),
        })
    }

    #[getter]
    pub fn source<'py>(&self, py: Python<'py>) -> Result<&Bound<'py, PyAny>, TypeError> {
        match &self.source {
            ImageSource::Base64(base64) => {
                let py_obj = base64.clone().into_bound_py_any(py)?;
                Ok(&py_obj)
            }
            ImageSource::Url(url) => {
                let py_obj = url.clone().into_bound_py_any(py)?;
                Ok(&py_obj)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct Base64PDFSource {
    #[pyo3(get, set)]
    pub media_type: String,
    #[pyo3(get, set)]
    pub data: String,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl Base64PDFSource {
    #[new]
    pub fn new(data: String) -> Result<Self, TypeError> {
        Ok(Self {
            media_type: DOCUMENT_BASE64_PDF_TYPE.to_string(),
            data,
            r#type: BASE64_TYPE.to_string(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct UrlPDFSource {
    #[pyo3(get, set)]
    pub url: String,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl UrlPDFSource {
    #[new]
    pub fn new(url: String) -> Self {
        Self {
            url,
            r#type: URL_TYPE.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct PlainTextSource {
    #[pyo3(get, set)]
    pub media_type: String,
    #[pyo3(get, set)]
    pub data: String,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl PlainTextSource {
    #[new]
    pub fn new(data: String) -> Self {
        Self {
            media_type: DOCUMENT_PLAIN_TEXT_TYPE.to_string(),
            data,
            r#type: TEXT_TYPE.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct CitationsConfigParams {
    #[pyo3(get, set)]
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum DocumentSource {
    Base64(Base64PDFSource),
    Url(UrlPDFSource),
    Text(PlainTextSource),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct DocumentBlockParam {
    pub source: DocumentSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get, set)]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get, set)]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get, set)]
    pub context: Option<String>,
    #[serde(rename = "type")]
    #[pyo3(get, set)]
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get, set)]
    pub citations: Option<CitationsConfigParams>,
}

#[pymethods]
impl DocumentBlockParam {
    #[new]
    pub fn new(
        source: &Bound<'_, PyAny>,
        cache_control: Option<CacheControl>,
        title: Option<String>,
        context: Option<String>,
        citations: Option<CitationsConfigParams>,
    ) -> Result<Self, TypeError> {
        let source: DocumentSource = if source.is_instance_of::<Base64PDFSource>() {
            DocumentSource::Base64(source.extract::<Base64PDFSource>()?)
        } else if source.is_instance_of::<UrlPDFSource>() {
            DocumentSource::Url(source.extract::<UrlPDFSource>()?)
        } else {
            DocumentSource::Text(source.extract::<PlainTextSource>()?)
        };

        Ok(Self {
            source,
            cache_control,
            title,
            context,
            r#type: DOCUMENT_TYPE.to_string(),
            citations,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct SearchResultBlockParam {
    #[pyo3(get, set)]
    pub content: Vec<TextBlockParam>,
    #[pyo3(get, set)]
    pub source: String,
    #[pyo3(get, set)]
    pub title: String,
    #[serde(rename = "type")]
    #[pyo3(get, set)]
    pub r#type: String,
    #[pyo3(get, set)]
    pub cache_control: Option<CacheControl>,
    #[pyo3(get, set)]
    pub citations: Option<CitationsConfigParams>,
}

#[pymethods]
impl SearchResultBlockParam {
    #[new]
    pub fn new(
        content: Vec<TextBlockParam>,
        source: String,
        title: String,
        cache_control: Option<CacheControl>,
        citations: Option<CitationsConfigParams>,
    ) -> Self {
        Self {
            content,
            source,
            title,
            r#type: SEARCH_TYPE.to_string(),
            cache_control,
            citations,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct ThinkingBlockParam {
    #[pyo3(get, set)]
    pub thinking: String,
    #[pyo3(get, set)]
    pub signature: Option<String>,
    #[pyo3(get, set)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl ThinkingBlockParam {
    #[new]
    pub fn new(thinking: String, signature: Option<String>) -> Self {
        Self {
            thinking,
            signature,
            r#type: THINKING_TYPE.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct RedactedThinkingBlockParam {
    #[pyo3(get, set)]
    pub data: String,
    #[pyo3(get, set)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl RedactedThinkingBlockParam {
    #[new]
    pub fn new(data: String) -> Self {
        Self {
            data,
            r#type: REDACTED_THINKING_TYPE.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct ToolUseBlockParam {
    #[pyo3(get, set)]
    pub id: String,
    #[pyo3(get, set)]
    pub name: String,
    pub input: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get, set)]
    pub cache_control: Option<CacheControl>,
    #[pyo3(get, set)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl ToolUseBlockParam {
    #[new]
    pub fn new(
        id: String,
        name: String,
        input: &Bound<'_, PyAny>,
        cache_control: Option<CacheControl>,
    ) -> Result<Self, TypeError> {
        let input_value = pyobject_to_json(input)?;
        Ok(Self {
            id,
            name,
            input: input_value,
            cache_control,
            r#type: TOOL_USE_TYPE.to_string(),
        })
    }

    #[getter]
    pub fn input<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        let py_dict = json_to_pyobject(py, &self.input)?.bind(py).clone();
        Ok(py_dict)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ToolResultContentEnum {
    Text(Vec<TextBlockParam>),
    Image(Vec<ImageBlockParam>),
    Document(Vec<DocumentBlockParam>),
    SearchResult(Vec<SearchResultBlockParam>),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct ToolResultBlockParam {
    #[pyo3(get, set)]
    pub tool_use_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get, set)]
    pub cache_control: Option<CacheControl>,
    #[pyo3(get, set)]
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ToolResultContentEnum>,
}

/// Helper function to extract all blocks of a specific type
fn extract_all_blocks<T>(blocks: Vec<Bound<'_, PyAny>>) -> Result<Vec<T>, TypeError>
where
    T: for<'a, 'py> FromPyObject<'a, 'py>,
{
    blocks
        .into_iter()
        .map(|block| {
            block
                .extract::<T>()
                .map_err(|_| TypeError::Error("Failed to extract block".to_string()))
        })
        .collect()
}

#[pymethods]
impl ToolResultBlockParam {
    #[new]
    pub fn new(
        tool_use_id: String,
        is_error: Option<bool>,
        cache_control: Option<CacheControl>,
        content: Option<Vec<Bound<'_, PyAny>>>,
    ) -> Result<Self, TypeError> {
        let content_enum = match content {
            None => None,
            Some(blocks) if blocks.is_empty() => None,

            Some(blocks) => {
                let first_block = &blocks[0];

                if first_block.is_instance_of::<TextBlockParam>() {
                    Some(ToolResultContentEnum::Text(extract_all_blocks(blocks)?))
                } else if first_block.is_instance_of::<ImageBlockParam>() {
                    Some(ToolResultContentEnum::Image(extract_all_blocks(blocks)?))
                } else if first_block.is_instance_of::<DocumentBlockParam>() {
                    Some(ToolResultContentEnum::Document(extract_all_blocks(blocks)?))
                } else if first_block.is_instance_of::<SearchResultBlockParam>() {
                    Some(ToolResultContentEnum::SearchResult(extract_all_blocks(
                        blocks,
                    )?))
                } else {
                    return Err(TypeError::InvalidInput(
                        "Unsupported content block type".to_string(),
                    ));
                }
            }
        };

        Ok(Self {
            tool_use_id,
            is_error,
            cache_control,
            r#type: TOOL_RESULT_TYPE.to_string(),
            content: content_enum,
        })
    }

    #[getter]
    pub fn content<'py>(&self, py: Python<'py>) -> Result<Option<Bound<'py, PyAny>>, TypeError> {
        match &self.content {
            None => Ok(None),
            Some(ToolResultContentEnum::Text(blocks)) => {
                let py_list = blocks
                    .iter()
                    .map(|block| block.clone().into_bound_py_any(py))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Some(py_list.into_bound_py_any(py)?))
            }
            Some(ToolResultContentEnum::Image(blocks)) => {
                let py_list = blocks
                    .iter()
                    .map(|block| block.clone().into_bound_py_any(py))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Some(py_list.into_bound_py_any(py)?))
            }
            Some(ToolResultContentEnum::Document(blocks)) => {
                let py_list = blocks
                    .iter()
                    .map(|block| block.clone().into_bound_py_any(py))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Some(py_list.into_bound_py_any(py)?))
            }
            Some(ToolResultContentEnum::SearchResult(blocks)) => {
                let py_list = blocks
                    .iter()
                    .map(|block| block.clone().into_bound_py_any(py))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Some(py_list.into_bound_py_any(py)?))
            }
        }
    }
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
#[pyclass(name = "AnthropicTool")]
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
#[pyclass(name = "AnthropicThinkingConfig")]
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
#[pyclass(name = "AnthropicToolChoice")]
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
    #[allow(clippy::too_many_arguments)]
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
