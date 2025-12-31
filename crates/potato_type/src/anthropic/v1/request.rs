use crate::anthropic::v1::response::{
    ResponseContentBlockInner, TextCitation, WebSearchToolResultBlockContent,
};
use crate::common::get_image_media_types;
use crate::google::v1::generate::request::{DataNum, GeminiContent, Part};
use crate::openai::v1::chat::request::{ChatMessage, ContentPart, TextContentPart};
use crate::prompt::builder::ProviderRequest;
use crate::prompt::MessageNum;
use crate::prompt::ModelSettings;
use crate::tools::AgentToolDefinition;
use crate::traits::get_var_regex;
use crate::traits::{MessageConversion, MessageFactory, PromptMessageExt, RequestAdapter};
use crate::TypeError;
use crate::{Provider, SettingsType};
use potato_util::{PyHelperFuncs, UtilError};
use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::IntoPyObjectExt;
use pythonize::{depythonize, pythonize};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

/// common content types used in Anthropic messages
pub const BASE64_TYPE: &str = "base64";
pub const URL_TYPE: &str = "url";
pub const EPHEMERAL_TYPE: &str = "ephemeral";
pub const IMAGE_TYPE: &str = "image";
pub const TEXT_TYPE: &str = "text";
pub const DOCUMENT_TYPE: &str = "document";
pub const DOCUMENT_BASE64_PDF_TYPE: &str = "application/pdf";
pub const DOCUMENT_PLAIN_TEXT_TYPE: &str = "text/plain";
pub const WEB_SEARCH_RESULT_TYPE: &str = "web_search_result";
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
pub const WEB_SEARCH_TOOL_RESULT_ERROR_TYPE: &str = "web_search_tool_result_error";

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
    #[pyo3(signature = (text, cache_control=None, citations=None))]
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

impl TextBlockParam {
    pub fn new_rs(
        text: String,
        cache_control: Option<CacheControl>,
        citations: Option<TextCitationParam>,
    ) -> Self {
        Self {
            text,
            cache_control,
            citations,
            r#type: TEXT_TYPE.to_string(),
        }
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
    #[pyo3(signature = (source, cache_control=None))]
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
    pub fn source<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        match &self.source {
            ImageSource::Base64(base64) => {
                let py_obj = base64.clone().into_bound_py_any(py)?;
                Ok(py_obj.clone())
            }
            ImageSource::Url(url) => {
                let py_obj = url.clone().into_bound_py_any(py)?;
                Ok(py_obj.clone())
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
    #[pyo3(signature = (source, cache_control=None, title=None, context=None, citations=None))]
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
    #[pyo3(signature = (content, source, title, cache_control=None, citations=None))]
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
    #[pyo3(signature = (thinking, signature=None))]
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
    #[pyo3(signature = (id, name, input, cache_control=None))]
    pub fn new(
        id: String,
        name: String,
        input: &Bound<'_, PyAny>,
        cache_control: Option<CacheControl>,
    ) -> Result<Self, TypeError> {
        let input_value = depythonize(input)?;
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
        let py_dict = pythonize(py, &self.input)?;
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
    #[pyo3(signature = (
        tool_use_id,
        is_error=None,
        cache_control=None,
        content=None
    ))]
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
#[pyclass]
pub struct ServerToolUseBlockParam {
    #[pyo3(get, set)]
    pub id: String,
    #[pyo3(get, set)]
    pub name: String,
    pub input: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[pyo3(get, set)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl ServerToolUseBlockParam {
    #[new]
    #[pyo3(signature = (id, name, input, cache_control=None))]
    pub fn new(
        id: String,
        name: String,
        input: &Bound<'_, PyAny>,
        cache_control: Option<CacheControl>,
    ) -> Result<Self, TypeError> {
        let input_value = depythonize(input)?;
        Ok(Self {
            id,
            name,
            input: input_value,
            cache_control,
            r#type: SERVER_TOOL_USE_TYPE.to_string(),
        })
    }
    #[getter]
    pub fn input<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        let py_dict = pythonize(py, &self.input)?;
        Ok(py_dict)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct WebSearchResultBlockParam {
    #[pyo3(get, set)]
    pub encrypted_content: String,
    #[pyo3(get, set)]
    pub title: String,
    #[pyo3(get, set)]
    pub url: String,
    #[pyo3(get, set)]
    pub page_agent: Option<String>,
    #[pyo3(get, set)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl WebSearchResultBlockParam {
    #[new]
    #[pyo3(signature = (encrypted_content, title, url, page_agent=None))]
    pub fn new(
        encrypted_content: String,
        title: String,
        url: String,
        page_agent: Option<String>,
    ) -> Self {
        Self {
            encrypted_content,
            title,
            url,
            page_agent,
            r#type: WEB_SEARCH_RESULT_TYPE.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct WebSearchToolResultBlockParam {
    #[pyo3(get, set)]
    pub tool_use_id: String,
    #[pyo3(get, set)]
    pub content: Vec<WebSearchResultBlockParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get, set)]
    pub cache_control: Option<CacheControl>,
    #[pyo3(get, set)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl WebSearchToolResultBlockParam {
    #[new]
    #[pyo3(signature = (tool_use_id, content, cache_control=None))]
    pub fn new(
        tool_use_id: String,
        content: Vec<WebSearchResultBlockParam>,
        cache_control: Option<CacheControl>,
    ) -> Self {
        Self {
            tool_use_id,
            content,
            cache_control,
            r#type: WEB_SEARCH_TOOL_RESULT_TYPE.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub(crate) enum ContentBlock {
    Text(TextBlockParam),
    Image(ImageBlockParam),
    Document(DocumentBlockParam),
    SearchResult(SearchResultBlockParam),
    Thinking(ThinkingBlockParam),
    RedactedThinking(RedactedThinkingBlockParam),
    ToolUse(ToolUseBlockParam),
    ToolResult(ToolResultBlockParam),
    ServerToolUse(ServerToolUseBlockParam),
    WebSearchResult(WebSearchResultBlockParam),
}

macro_rules! try_extract_content_block {
    ($block:expr, $($variant:ident => $type:ty),+ $(,)?) => {{
        $(
            if $block.is_instance_of::<$type>() {
                return Ok(Self {
                    inner: ContentBlock::$variant($block.extract::<$type>()?),
                });
            }
        )+
        return Err(TypeError::InvalidInput(
            "Unsupported content block type".to_string(),
        ));
    }};
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ContentBlockParam {
    #[serde(flatten)]
    pub(crate) inner: ContentBlock,
}

impl ContentBlockParam {
    pub fn new(block: &Bound<'_, PyAny>) -> Result<Self, TypeError> {
        try_extract_content_block!(
            block,
            Text => TextBlockParam,
            Image => ImageBlockParam,
            Document => DocumentBlockParam,
            SearchResult => SearchResultBlockParam,
            Thinking => ThinkingBlockParam,
            RedactedThinking => RedactedThinkingBlockParam,
            ToolUse => ToolUseBlockParam,
            ToolResult => ToolResultBlockParam,
            ServerToolUse => ServerToolUseBlockParam,
            WebSearchResult => WebSearchResultBlockParam,
        )
    }

    /// Convert the ContentBlockParam back to a PyObject
    /// This is an acceptable clone, as this will really only be used in development/testing scenarios
    pub fn to_pyobject<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        match &self.inner {
            ContentBlock::Text(block) => Ok(block.clone().into_bound_py_any(py)?.clone()),
            ContentBlock::Image(block) => Ok(block.clone().into_bound_py_any(py)?.clone()),
            ContentBlock::Document(block) => Ok(block.clone().into_bound_py_any(py)?.clone()),
            ContentBlock::SearchResult(block) => Ok(block.clone().into_bound_py_any(py)?.clone()),
            ContentBlock::Thinking(block) => Ok(block.clone().into_bound_py_any(py)?.clone()),
            ContentBlock::RedactedThinking(block) => {
                Ok(block.clone().into_bound_py_any(py)?.clone())
            }
            ContentBlock::ToolUse(block) => Ok(block.clone().into_bound_py_any(py)?.clone()),
            ContentBlock::ToolResult(block) => Ok(block.clone().into_bound_py_any(py)?.clone()),
            ContentBlock::ServerToolUse(block) => Ok(block.clone().into_bound_py_any(py)?.clone()),
            ContentBlock::WebSearchResult(block) => {
                Ok(block.clone().into_bound_py_any(py)?.clone())
            }
        }
    }
}

impl ContentBlockParam {
    /// Helper function to create a ContentBlockParam from a ResponseContentBlockInner
    ///
    /// Converts response content blocks (from API responses) into request content blocks
    /// that can be used in subsequent requests. This is useful for multi-turn conversations
    /// where the assistant's response needs to be included in the next request.
    ///
    /// # Arguments
    /// * `block` - A reference to a ResponseContentBlockInner from an API response
    ///
    /// # Returns
    /// * `Result<Self, TypeError>` - The converted ContentBlockParam or an error
    pub(crate) fn from_response_content_block(
        block: &ResponseContentBlockInner,
    ) -> Result<Self, TypeError> {
        match block {
            ResponseContentBlockInner::Text(text_block) => {
                // Convert Vec<TextCitation> to Option<TextCitationParam>
                // Take the first citation if available, as request format uses singular citation
                let citations = text_block.citations.as_ref().and_then(|cits| {
                    cits.first().map(|cit| match cit {
                        TextCitation::CharLocation(c) => {
                            TextCitationParam::CharLocation(CitationCharLocationParam {
                                cited_text: c.cited_text.clone(),
                                document_index: c.document_index,
                                document_title: c.document_title.clone(),
                                end_char_index: c.end_char_index,
                                start_char_index: c.start_char_index,
                                r#type: c.r#type.clone(),
                            })
                        }
                        TextCitation::PageLocation(c) => {
                            TextCitationParam::PageLocation(CitationPageLocationParam {
                                cited_text: c.cited_text.clone(),
                                document_index: c.document_index,
                                document_title: c.document_title.clone(),
                                end_page_number: c.end_page_number,
                                start_page_number: c.start_page_number,
                                r#type: c.r#type.clone(),
                            })
                        }
                        TextCitation::ContentBlockLocation(c) => {
                            TextCitationParam::ContentBlockLocation(
                                CitationContentBlockLocationParam {
                                    cited_text: c.cited_text.clone(),
                                    document_index: c.document_index,
                                    document_title: c.document_title.clone(),
                                    end_block_index: c.end_block_index,
                                    start_block_index: c.start_block_index,
                                    r#type: c.r#type.clone(),
                                },
                            )
                        }
                        TextCitation::WebSearchResultLocation(c) => {
                            TextCitationParam::WebSearchResultLocation(
                                CitationWebSearchResultLocationParam {
                                    cited_text: c.cited_text.clone(),
                                    encrypted_index: c.encrypted_index.clone(),
                                    title: c.title.clone(),
                                    r#type: c.r#type.clone(),
                                    url: c.url.clone(),
                                },
                            )
                        }
                        TextCitation::SearchResultLocation(c) => {
                            TextCitationParam::SearchResultLocation(
                                CitationSearchResultLocationParam {
                                    cited_text: c.cited_text.clone(),
                                    end_block_index: c.end_block_index,
                                    search_result_index: c.search_result_index,
                                    source: c.source.clone(),
                                    start_block_index: c.start_block_index,
                                    title: c.title.clone(),
                                    r#type: c.r#type.clone(),
                                },
                            )
                        }
                    })
                });

                Ok(Self {
                    inner: ContentBlock::Text(TextBlockParam {
                        text: text_block.text.clone(),
                        cache_control: None,
                        citations,
                        r#type: text_block.r#type.clone(),
                    }),
                })
            }
            ResponseContentBlockInner::Thinking(thinking_block) => Ok(Self {
                inner: ContentBlock::Thinking(ThinkingBlockParam {
                    thinking: thinking_block.thinking.clone(),
                    signature: thinking_block.signature.clone(),
                    r#type: thinking_block.r#type.clone(),
                }),
            }),
            ResponseContentBlockInner::RedactedThinking(redacted_thinking_block) => Ok(Self {
                inner: ContentBlock::RedactedThinking(RedactedThinkingBlockParam {
                    data: redacted_thinking_block.data.clone(),
                    r#type: redacted_thinking_block.r#type.clone(),
                }),
            }),
            ResponseContentBlockInner::ToolUse(tool_use_block) => Ok(Self {
                inner: ContentBlock::ToolUse(ToolUseBlockParam {
                    id: tool_use_block.id.clone(),
                    name: tool_use_block.name.clone(),
                    input: tool_use_block.input.clone(),
                    cache_control: None,
                    r#type: tool_use_block.r#type.clone(),
                }),
            }),
            ResponseContentBlockInner::ServerToolUse(server_tool_use_block) => Ok(Self {
                inner: ContentBlock::ServerToolUse(ServerToolUseBlockParam {
                    id: server_tool_use_block.id.clone(),
                    name: server_tool_use_block.name.clone(),
                    input: server_tool_use_block.input.clone(),
                    cache_control: None,
                    r#type: server_tool_use_block.r#type.clone(),
                }),
            }),
            ResponseContentBlockInner::WebSearchToolResult(web_search_tool_result_block) => {
                match &web_search_tool_result_block.content {
                    WebSearchToolResultBlockContent::Results(results) => {
                        // Take the first result and convert it to WebSearchResultBlockParam
                        let first_result = results.first().ok_or_else(|| {
                            TypeError::InvalidInput(
                                "WebSearchToolResult must contain at least one result".to_string(),
                            )
                        })?;

                        Ok(Self {
                            inner: ContentBlock::WebSearchResult(WebSearchResultBlockParam {
                                encrypted_content: first_result.encrypted_content.clone(),
                                title: first_result.title.clone(),
                                url: first_result.url.clone(),
                                page_agent: first_result.page_age.clone(), // Note: page_age -> page_agent mapping
                                r#type: first_result.r#type.clone(),
                            }),
                        })
                    }
                    WebSearchToolResultBlockContent::Error(_) => Err(TypeError::InvalidInput(
                        "Cannot convert WebSearchToolResult error to ContentBlockParam".to_string(),
                    )),
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct MessageParam {
    pub content: Vec<ContentBlockParam>,
    #[pyo3(get)]
    pub role: String,
}

#[pymethods]
impl MessageParam {
    /// Create a new MessageParam
    /// Will accept either a string, Single type of ContentBlockParam, or a list of varying ContentBlockParams
    /// Initial logic determines initial type and processes accordingly
    /// If string, wraps in TextBlockParam --> ContentBlockParam
    /// If single param, pass to ContentBlockParam directly
    /// If list, process each item in list to ContentBlockParam
    #[new]
    pub fn new(content: &Bound<'_, PyAny>, role: String) -> Result<Self, TypeError> {
        let content: Vec<ContentBlockParam> = if content.is_instance_of::<pyo3::types::PyString>() {
            let text = content.extract::<String>()?;
            let text_block = TextBlockParam::new(text, None, None)?;
            let content_block =
                ContentBlockParam::new(&text_block.into_bound_py_any(content.py())?)?;
            vec![content_block]
        } else if content.is_instance_of::<pyo3::types::PyList>() {
            let content_list = content.extract::<Vec<Bound<'_, PyAny>>>()?;
            let mut blocks = Vec::new();
            for item in content_list {
                let content_block = ContentBlockParam::new(&item)?;
                blocks.push(content_block);
            }
            blocks
        } else {
            // pass single ContentBlockParam
            let content_block = ContentBlockParam::new(content)?;
            vec![content_block]
        };

        Ok(Self { content, role })
    }

    #[getter]
    fn content<'py>(&self, py: Python<'py>) -> Result<Vec<Bound<'py, PyAny>>, TypeError> {
        self.content
            .iter()
            .map(|block| block.to_pyobject(py))
            .collect()
    }

    #[pyo3(name = "bind")]
    fn bind_py(&self, name: &str, value: &str) -> Result<Self, TypeError> {
        self.bind(name, value)
    }

    #[pyo3(name = "bind_mut")]
    fn bind_mut_py(&mut self, name: &str, value: &str) -> Result<(), TypeError> {
        self.bind_mut(name, value)
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }

    pub fn model_dump<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        // iterate over each field in model_settings and add to the dict if it is not None
        let json = serde_json::to_value(self)?;
        Ok(pythonize(py, &json)?)
    }

    // Return the text content from the first content part that is text
    pub fn text(&self) -> String {
        self.content
            .iter()
            .find_map(|part| {
                if let ContentBlock::Text(text_part) = &part.inner {
                    Some(text_part.text.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }
}

impl PromptMessageExt for MessageParam {
    fn bind_mut(&mut self, name: &str, value: &str) -> Result<(), TypeError> {
        let placeholder = format!("${{{}}}", name);

        for part in &mut self.content {
            if let ContentBlock::Text(text_part) = &mut part.inner {
                text_part.text = text_part.text.replace(&placeholder, value);
            }
        }

        Ok(())
    }

    fn bind(&self, name: &str, value: &str) -> Result<Self, TypeError>
    where
        Self: Sized,
    {
        let mut new_message = self.clone();
        new_message.bind_mut(name, value)?;
        Ok(new_message)
    }

    fn extract_variables(&self) -> Vec<String> {
        let mut variables = HashSet::new();

        // Lazily initialize regex to avoid recompilation
        let regex = get_var_regex();

        // Extract variables from all text content parts
        for part in &self.content {
            if let ContentBlock::Text(text_part) = &part.inner {
                for captures in regex.captures_iter(&text_part.text) {
                    if let Some(name) = captures.get(1) {
                        variables.insert(name.as_str().to_string());
                    }
                }
            }
        }

        // Convert HashSet to Vec for return
        variables.into_iter().collect()
    }

    fn from_text(content: String, role: &str) -> Result<Self, TypeError> {
        Ok(Self {
            role: role.to_string(),
            content: vec![ContentBlockParam {
                inner: ContentBlock::Text(TextBlockParam::new_rs(content, None, None)),
            }],
        })
    }
}

impl MessageParam {
    /// Helper function to create a MessageParam from a single TextBlockParam
    pub fn to_text_block_param(&self) -> Result<TextBlockParam, TypeError> {
        if self.content.len() != 1 {
            return Err(TypeError::InvalidInput(
                "MessageParam must contain exactly one content block to convert to TextBlockParam"
                    .to_string(),
            ));
        }

        match &self.content[0].inner {
            ContentBlock::Text(text_block) => Ok(text_block.clone()),
            _ => Err(TypeError::InvalidInput(
                "Content block is not of type TextBlockParam".to_string(),
            )),
        }
    }
}

impl MessageFactory for MessageParam {
    fn from_text(content: String, role: &str) -> Result<Self, TypeError> {
        let text_block = TextBlockParam::new_rs(content, None, None);
        let content_block = ContentBlockParam {
            inner: ContentBlock::Text(text_block),
        };

        Ok(Self {
            role: role.to_string(),
            content: vec![content_block],
        })
    }
}

impl MessageConversion for MessageParam {
    fn to_anthropic_message(&self) -> Result<Self, TypeError> {
        // Currently, MessageParam is already in the Anthropic format
        Err(TypeError::CantConvertSelf)
    }

    fn to_google_message(
        &self,
    ) -> Result<crate::google::v1::generate::request::GeminiContent, TypeError> {
        // Extract text content from all text blocks
        let mut parts = Vec::new();

        for content_block in &self.content {
            match &content_block.inner {
                ContentBlock::Text(text_block) => {
                    parts.push(Part {
                        data: DataNum::Text(text_block.text.clone()),
                        thought: None,
                        thought_signature: None,
                        part_metadata: None,
                        media_resolution: None,
                        video_metadata: None,
                    });
                }
                _ => {
                    return Err(TypeError::UnsupportedConversion(
                        "Only text content blocks are currently supported for conversion"
                            .to_string(),
                    ));
                }
            }
        }

        if parts.is_empty() {
            return Err(TypeError::UnsupportedConversion(
                "Message contains no text content to convert".to_string(),
            ));
        }

        Ok(GeminiContent {
            role: self.role.clone(),
            parts,
        })
    }

    fn to_openai_message(
        &self,
    ) -> Result<crate::openai::v1::chat::request::ChatMessage, TypeError> {
        // Extract text content from all text blocks
        let mut content_parts = Vec::new();

        for content_block in &self.content {
            match &content_block.inner {
                ContentBlock::Text(text_block) => {
                    content_parts.push(ContentPart::Text(TextContentPart::new(
                        text_block.text.clone(),
                    )));
                }
                _ => {
                    return Err(TypeError::UnsupportedConversion(
                        "Only text content blocks are currently supported for conversion"
                            .to_string(),
                    ));
                }
            }
        }

        if content_parts.is_empty() {
            return Err(TypeError::UnsupportedConversion(
                "Message contains no text content to convert".to_string(),
            ));
        }

        Ok(ChatMessage {
            role: self.role.clone(),
            content: content_parts,
            name: None,
        })
    }
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
    #[pyo3(signature = (user_id=None))]
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
    #[pyo3(signature = (cache_type, ttl=None))]
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
    #[pyo3(signature = (
        name,
        input_schema,
        description=None,
        cache_control=None
    ))]
    pub fn new(
        name: String,
        input_schema: &Bound<'_, PyAny>,
        description: Option<String>,
        cache_control: Option<CacheControl>,
    ) -> Result<Self, UtilError> {
        Ok(Self {
            name,
            description,
            input_schema: depythonize(input_schema)?,
            cache_control,
        })
    }
}

impl Tool {
    pub fn from_tool_agent_tool_definition(tool: &AgentToolDefinition) -> Result<Self, TypeError> {
        Ok(Self {
            name: tool.name.clone(),
            description: Some(tool.description.clone()),
            input_schema: tool.parameters.clone(),
            cache_control: None,
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
    #[pyo3(signature = (r#type, budget_tokens=None))]
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
    #[pyo3(signature = (r#type, disable_parallel_tool_use=None, name=None))]
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
            Some(obj) => Some(depythonize(obj)?),
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

    pub fn model_dump<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        // iterate over each field in model_settings and add to the dict if it is not None
        let json = serde_json::to_value(self)?;
        Ok(pythonize(py, &json)?)
    }

    pub fn settings_type(&self) -> SettingsType {
        SettingsType::Anthropic
    }
}

impl AnthropicSettings {
    pub fn add_tools(&mut self, tools: Vec<AgentToolDefinition>) -> Result<(), TypeError> {
        let current_tools = self.tools.get_or_insert_with(Vec::new);

        for tool in tools {
            let tool_param = Tool::from_tool_agent_tool_definition(&tool)?;
            current_tools.push(tool_param);
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AnthropicMessageRequestV1 {
    pub model: String,
    pub messages: Vec<MessageNum>,
    pub system: Vec<MessageNum>,
    #[serde(flatten)]
    pub settings: AnthropicSettings,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<Value>,
}

pub(crate) fn create_structured_output_schema(json_schema: &Value) -> Value {
    serde_json::json!({
        "type": "json_schema",
        "schema": json_schema,

    })
}

impl RequestAdapter for AnthropicMessageRequestV1 {
    fn messages_mut(&mut self) -> &mut Vec<MessageNum> {
        &mut self.messages
    }
    fn messages(&self) -> &[MessageNum] {
        &self.messages
    }
    fn system_instructions(&self) -> Vec<&MessageNum> {
        self.system.iter().collect()
    }
    fn response_json_schema(&self) -> Option<&Value> {
        self.output_format.as_ref()
    }

    fn preprend_system_instructions(&mut self, messages: Vec<MessageNum>) -> Result<(), TypeError> {
        let mut combined = messages;
        combined.append(&mut self.system);
        self.system = combined;
        Ok(())
    }

    fn get_py_system_instructions<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyList>, TypeError> {
        let py_system_instructions = PyList::empty(py);
        for system_msg in &self.system {
            py_system_instructions.append(system_msg.to_bound_py_object(py)?)?;
        }

        Ok(py_system_instructions)
    }

    fn model_settings<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        let settings = self.settings.clone();
        Ok(settings.into_bound_py_any(py)?)
    }

    fn to_request_body(&self) -> Result<Value, TypeError> {
        Ok(serde_json::to_value(self)?)
    }
    fn match_provider(&self, provider: &Provider) -> bool {
        *provider == Provider::Anthropic
    }
    fn build_provider_enum(
        messages: Vec<MessageNum>,
        system_instructions: Vec<MessageNum>,
        model: String,
        settings: ModelSettings,
        response_json_schema: Option<Value>,
    ) -> Result<ProviderRequest, TypeError> {
        let anthropic_settings = match settings {
            ModelSettings::AnthropicChat(s) => s,
            _ => AnthropicSettings::default(),
        };

        let output_format =
            response_json_schema.map(|json_schema| create_structured_output_schema(&json_schema));

        Ok(ProviderRequest::AnthropicV1(AnthropicMessageRequestV1 {
            model,
            messages,
            system: system_instructions,
            settings: anthropic_settings,
            output_format,
        }))
    }

    fn set_response_json_schema(&mut self, response_json_schema: Option<Value>) {
        self.output_format =
            response_json_schema.map(|json_schema| create_structured_output_schema(&json_schema));
    }

    fn add_tools(&mut self, tools: Vec<AgentToolDefinition>) -> Result<(), TypeError> {
        self.settings.add_tools(tools)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct SystemPrompt {
    #[pyo3(get)]
    #[serde(flatten)]
    pub content: Vec<TextBlockParam>,
}

#[pymethods]
impl SystemPrompt {
    /// Create a new SystemPrompt
    /// Accepts either a single string or a list of TextBlockParams
    /// # Arguments
    /// * `content` - Either a string or a list of TextBlockParams
    /// # Returns
    /// * `SystemPrompt` - The created SystemPrompt
    /// Errors
    /// * `TypeError` - If the content is not a string or a list of TextBlockParams
    #[new]
    pub fn new(content: &Bound<'_, PyAny>) -> Result<Self, TypeError> {
        let content_blocks: Vec<TextBlockParam> =
            if content.is_instance_of::<pyo3::types::PyString>() {
                let text = content.extract::<String>()?;
                let text_block = TextBlockParam::new(text, None, None)?;
                vec![text_block]
            } else if content.is_instance_of::<pyo3::types::PyList>() {
                let content_list = content.extract::<Vec<Bound<'_, PyAny>>>()?;
                let mut blocks = Vec::new();
                for item in content_list {
                    let text_block = item.extract::<TextBlockParam>().map_err(|_| {
                        TypeError::InvalidInput(
                            "All items in the list must be TextBlockParam".to_string(),
                        )
                    })?;
                    blocks.push(text_block);
                }
                blocks
            } else {
                return Err(TypeError::InvalidInput(
                    "Content must be either a string or a list of TextBlockParam".to_string(),
                ));
            };

        Ok(Self {
            content: content_blocks,
        })
    }
}
