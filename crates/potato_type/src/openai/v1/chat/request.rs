use crate::openai::v1::chat::settings::OpenAIChatSettings;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyList, PyString};
use pyo3::IntoPyObjectExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
pub const OPENAI_CONTENT_PART_TEXT: &str = "text";
pub const OPENAI_CONTENT_PART_IMAGE_URL: &str = "image_url";
pub const OPENAI_CONTENT_PART_INPUT_AUDIO: &str = "input_audio";
pub const OPENAI_CONTENT_PART_FILE: &str = "file";

#[derive(Debug, Serialize, Deserialize, Clone)]
#[pyclass]
pub struct File {
    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_data: Option<String>,
    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

#[pymethods]
impl File {
    #[new]
    pub fn new(
        file_data: Option<String>,
        file_id: Option<String>,
        filename: Option<String>,
    ) -> Self {
        Self {
            file_data,
            file_id,
            filename,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[pyclass]
pub struct FileContentPart {
    #[pyo3(get, set)]
    pub file: File,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl FileContentPart {
    #[new]
    pub fn new(file: File) -> Self {
        Self {
            file,
            r#type: OPENAI_CONTENT_PART_FILE.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[pyclass]
pub struct InputAudioData {
    #[pyo3(get, set)]
    pub data: String,
    #[pyo3(get)]
    pub format: String,
}

#[pymethods]
impl InputAudioData {
    #[new]
    pub fn new(data: String, format: String) -> Self {
        Self { data, format }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[pyclass]
pub struct InputAudioContentPart {
    #[pyo3(get, set)]
    pub input_audio: InputAudioData,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl InputAudioContentPart {
    #[new]
    pub fn new(input_audio: InputAudioData) -> Self {
        Self {
            input_audio,
            r#type: OPENAI_CONTENT_PART_INPUT_AUDIO.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[pyclass]
pub struct ImageContentPart {
    #[pyo3(get, set)]
    pub url: String,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl ImageContentPart {
    #[new]
    pub fn new(url: String) -> Self {
        Self {
            url,
            r#type: OPENAI_CONTENT_PART_IMAGE_URL.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[pyclass]
pub struct TextContentPart {
    #[pyo3(get, set)]
    pub text: String,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl TextContentPart {
    #[new]
    pub fn new(text: String) -> Self {
        Self {
            text,
            r#type: OPENAI_CONTENT_PART_TEXT.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum OpenAIContentPart {
    Text(TextContentPart),
    ImageUrl(ImageContentPart),
    InputAudio(InputAudioContentPart),
    FileContent(FileContentPart),
}

fn extract_content_from_py_object(content: &Bound<'_, PyAny>) -> PyResult<Vec<OpenAIContentPart>> {
    if content.is_instance_of::<PyString>() {
        let text = content.extract::<String>()?;
        return Ok(vec![OpenAIContentPart::Text(TextContentPart::new(text))]);
    }

    if content.is_instance_of::<PyList>() {
        let list = content.cast::<PyList>()?;
        let mut parts = Vec::with_capacity(list.len()); // Pre-allocate

        for item in list.iter() {
            // Check native types first (faster)
            if item.is_instance_of::<PyString>() {
                parts.push(OpenAIContentPart::Text(TextContentPart::new(
                    item.extract::<String>()?,
                )));
            } else if item.is_instance_of::<ImageContentPart>() {
                parts.push(OpenAIContentPart::ImageUrl(
                    item.extract::<ImageContentPart>()?,
                ));
            } else if item.is_instance_of::<InputAudioContentPart>() {
                parts.push(OpenAIContentPart::InputAudio(
                    item.extract::<InputAudioContentPart>()?,
                ));
            } else if item.is_instance_of::<FileContentPart>() {
                parts.push(OpenAIContentPart::FileContent(
                    item.extract::<FileContentPart>()?,
                ));
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "Invalid content part type: {}",
                    item.get_type().name()?
                )));
            }
        }
        return Ok(parts);
    }

    Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
        "Content must be a string or a list of content parts",
    ))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[pyclass]
pub struct ChatMessage {
    #[pyo3(get, set)]
    pub role: String,

    pub content: Vec<OpenAIContentPart>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[pymethods]
impl ChatMessage {
    /// Creates an OpenAI message.
    ///
    /// Accepts three input formats:
    /// 1. String: Converted to a text content part
    /// 2. List: Processed as multiple content parts (strings or ContentPart types)
    /// 3. Single ContentPart: Wrapped in a vector
    #[new]
    pub fn new(role: String, content: &Bound<'_, PyAny>, name: Option<String>) -> PyResult<Self> {
        use pyo3::types::{PyList, PyString};

        let content_parts = if content.is_instance_of::<PyString>() {
            let text = content.extract::<String>()?;
            vec![OpenAIContentPart::Text(TextContentPart::new(text))]
        } else if content.is_instance_of::<PyList>() {
            extract_content_from_py_object(content)?
        } else {
            if content.is_instance_of::<TextContentPart>() {
                vec![OpenAIContentPart::Text(
                    content.extract::<TextContentPart>()?,
                )]
            } else if content.is_instance_of::<ImageContentPart>() {
                vec![OpenAIContentPart::ImageUrl(
                    content.extract::<ImageContentPart>()?,
                )]
            } else if content.is_instance_of::<InputAudioContentPart>() {
                vec![OpenAIContentPart::InputAudio(
                    content.extract::<InputAudioContentPart>()?,
                )]
            } else if content.is_instance_of::<FileContentPart>() {
                vec![OpenAIContentPart::FileContent(
                    content.extract::<FileContentPart>()?,
                )]
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "Content must be a string, list, or ContentPart type. Got: {}",
                    content.get_type().name()?
                )));
            }
        };

        Ok(Self {
            role,
            content: content_parts,
            name,
        })
    }

    /// Python getter for content - converts back to PyObject for Python access
    #[getter]
    fn content<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyAny>>> {
        self.content
            .iter()
            .map(|part| match part {
                OpenAIContentPart::Text(text) => text.clone().into_bound_py_any(py),
                OpenAIContentPart::ImageUrl(image) => image.clone().into_bound_py_any(py),
                OpenAIContentPart::InputAudio(audio) => audio.clone().into_bound_py_any(py),
                OpenAIContentPart::FileContent(file) => file.clone().into_bound_py_any(py),
            })
            .collect()
    }
}

#[derive(Debug, Serialize)]
pub struct OpenAIChatRequest<'a> {
    pub model: Cow<'a, str>,
    pub messages: &'a [ChatMessage],

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<&'a Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub settings: Option<&'a OpenAIChatSettings>,
}
