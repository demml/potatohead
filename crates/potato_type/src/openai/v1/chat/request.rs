use crate::openai::v1::chat::settings::OpenAIChatSettings;
use crate::prompt::builder::ProviderRequest;
use crate::prompt::types::MessageNum;
use crate::prompt::ModelSettings;
use crate::traits::{
    get_var_regex, MessageConversion, MessageFactory, PromptMessageExt, RequestAdapter,
};
use crate::{Provider, TypeError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyList, PyString};
use pyo3::IntoPyObjectExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

// anthropic conversion imports
use crate::anthropic::v1::request::{
    ContentBlock, ContentBlockParam, MessageParam, TextBlockParam,
};

// google conversion imports
use crate::google::v1::generate::request::{DataNum, GeminiContent, Part};

pub const OPENAI_CONTENT_PART_TEXT: &str = "text";
pub const OPENAI_CONTENT_PART_IMAGE_URL: &str = "image_url";
pub const OPENAI_CONTENT_PART_INPUT_AUDIO: &str = "input_audio";
pub const OPENAI_CONTENT_PART_FILE: &str = "file";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
    pub fn new(
        file_data: Option<String>,
        file_id: Option<String>,
        filename: Option<String>,
    ) -> Self {
        Self {
            file: File {
                file_data,
                file_id,
                filename,
            },
            r#type: OPENAI_CONTENT_PART_FILE.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
    pub fn new(data: String, format: String) -> Self {
        Self {
            input_audio: InputAudioData::new(data, format),
            r#type: OPENAI_CONTENT_PART_INPUT_AUDIO.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct ImageUrl {
    #[pyo3(get, set)]
    pub url: String,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[pymethods]
impl ImageUrl {
    #[new]
    pub fn new(url: String, detail: Option<String>) -> Self {
        Self { url, detail }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct ImageContentPart {
    #[pyo3(get, set)]
    pub image_url: ImageUrl,
    #[pyo3(get)]
    #[serde(rename = "type")]
    pub r#type: String,
}

#[pymethods]
impl ImageContentPart {
    #[new]
    pub fn new(url: String, detail: Option<String>) -> Self {
        Self {
            image_url: ImageUrl::new(url, detail),
            r#type: OPENAI_CONTENT_PART_IMAGE_URL.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ContentPart {
    Text(TextContentPart),
    ImageUrl(ImageContentPart),
    InputAudio(InputAudioContentPart),
    FileContent(FileContentPart),
}

fn extract_content_from_py_object(content: &Bound<'_, PyAny>) -> PyResult<Vec<ContentPart>> {
    if content.is_instance_of::<PyString>() {
        let text = content.extract::<String>()?;
        return Ok(vec![ContentPart::Text(TextContentPart::new(text))]);
    }

    if content.is_instance_of::<PyList>() {
        let list = content.cast::<PyList>()?;
        let mut parts = Vec::with_capacity(list.len());

        for item in list.iter() {
            // Handle string first (needs custom transformation)
            if item.is_instance_of::<PyString>() {
                parts.push(ContentPart::Text(TextContentPart::new(
                    item.extract::<String>()?,
                )));
                continue;
            }

            if potato_macro::extract_and_push!(
                item => parts,
                ImageContentPart => ContentPart::ImageUrl,
                InputAudioContentPart => ContentPart::InputAudio,
                FileContentPart => ContentPart::FileContent,
            ) {
                continue;
            }

            // If we get here, no type matched
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                "Invalid content part type: {}",
                item.get_type().name()?
            )));
        }
        return Ok(parts);
    }

    Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
        "Content must be a string or a list of content parts",
    ))
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[pyclass]
pub struct ChatMessage {
    #[pyo3(get, set)]
    pub role: String,

    pub content: Vec<ContentPart>,

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
        let content_parts = if content.is_instance_of::<PyString>() {
            let text = content.extract::<String>()?;
            vec![ContentPart::Text(TextContentPart::new(text))]
        } else if content.is_instance_of::<PyList>() {
            extract_content_from_py_object(content)?
        } else {
            let mut result = Vec::with_capacity(1);

            if !potato_macro::extract_and_push!(
                content => result,
                TextContentPart => ContentPart::Text,
                ImageContentPart => ContentPart::ImageUrl,
                InputAudioContentPart => ContentPart::InputAudio,
                FileContentPart => ContentPart::FileContent,
            ) {
                // Macro returned false - no match found
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "Content must be a string, list, or ContentPart type. Got: {}",
                    content.get_type().name()?
                )));
            }

            result
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
                ContentPart::Text(text) => text.clone().into_bound_py_any(py),
                ContentPart::ImageUrl(image) => image.clone().into_bound_py_any(py),
                ContentPart::InputAudio(audio) => audio.clone().into_bound_py_any(py),
                ContentPart::FileContent(file) => file.clone().into_bound_py_any(py),
            })
            .collect()
    }
}

impl PromptMessageExt for ChatMessage {
    fn bind_mut(&mut self, name: &str, value: &str) -> Result<(), TypeError> {
        let placeholder = format!("${{{name}}}");

        for part in &mut self.content {
            if let ContentPart::Text(text_part) = part {
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
            if let ContentPart::Text(text_part) = part {
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
            content: vec![ContentPart::Text(TextContentPart::new(content))],
            name: None,
        })
    }
}

impl MessageFactory for ChatMessage {
    fn from_text(content: String, role: &str) -> Result<Self, TypeError> {
        Ok(Self {
            role: role.to_string(),
            content: vec![ContentPart::Text(TextContentPart::new(content))],
            name: None,
        })
    }
}

impl MessageConversion for ChatMessage {
    fn to_anthropic_message(&self) -> Result<MessageParam, TypeError> {
        // Extract text content from all text content parts
        let mut content_blocks = Vec::new();

        for part in &self.content {
            match part {
                ContentPart::Text(text_part) => {
                    content_blocks.push(ContentBlockParam {
                        inner: ContentBlock::Text(TextBlockParam::new_rs(
                            text_part.text.clone(),
                            None,
                            None,
                        )),
                    });
                }
                _ => {
                    return Err(TypeError::UnsupportedConversion(
                        "Only text content parts are currently supported for conversion"
                            .to_string(),
                    ));
                }
            }
        }

        if content_blocks.is_empty() {
            return Err(TypeError::UnsupportedConversion(
                "Message contains no text content to convert".to_string(),
            ));
        }

        Ok(MessageParam {
            content: content_blocks,
            role: self.role.clone(),
        })
    }

    fn to_google_message(&self) -> Result<GeminiContent, TypeError> {
        // Extract text content from all text content parts
        let mut parts = Vec::new();

        for part in &self.content {
            match part {
                ContentPart::Text(text_part) => {
                    parts.push(Part {
                        data: DataNum::Text(text_part.text.clone()),
                        thought: None,
                        thought_signature: None,
                        part_metadata: None,
                        media_resolution: None,
                        video_metadata: None,
                    });
                }
                _ => {
                    return Err(TypeError::UnsupportedConversion(
                        "Only text content parts are currently supported for conversion"
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

    fn to_openai_message(&self) -> Result<Self, TypeError> {
        // Already an OpenAI message, raise error
        // this method should never be called on OpenAI messages
        Err(TypeError::CantConvertSelf)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct OpenAIChatCompletionRequestV1 {
    pub model: String,
    pub messages: Vec<MessageNum>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub settings: Option<OpenAIChatSettings>,
}

impl RequestAdapter for OpenAIChatCompletionRequestV1 {
    fn messages_mut(&mut self) -> &mut Vec<MessageNum> {
        &mut self.messages
    }
    fn messages(&self) -> &[MessageNum] {
        &self.messages
    }
    fn system_instructions(&self) -> Vec<&MessageNum> {
        self.messages
            .iter()
            .filter(|msg| msg.is_system_message())
            .collect()
    }
    fn response_json_schema(&self) -> Option<&Value> {
        self.response_format.as_ref()
    }
    fn preprend_system_instructions(&mut self, messages: Vec<MessageNum>) -> Result<(), TypeError> {
        let mut combined = messages;
        combined.append(&mut self.messages);
        self.messages = combined;
        Ok(())
    }

    fn get_py_system_instructions<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyList>, TypeError> {
        let py_system_instructions = PyList::empty(py);
        for msg in &self.messages {
            // OpenAI does not have separate system instructions
            // but we can filter them out if needed
            if msg.is_system_message() {
                py_system_instructions.append(msg.to_bound_py_object(py)?)?;
            }
        }

        Ok(py_system_instructions)
    }

    fn model_settings<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        let settings = self.settings.as_ref().cloned().unwrap_or_default();
        Ok(settings.into_bound_py_any(py)?)
    }

    fn to_request_body(&self) -> Result<Value, TypeError> {
        Ok(serde_json::to_value(self)?)
    }

    fn match_provider(&self, provider: &Provider) -> bool {
        *provider == Provider::OpenAI
    }

    fn build_provider_enum(
        messages: Vec<MessageNum>,
        system_instructions: Vec<MessageNum>,
        model: String,
        settings: ModelSettings,
        response_json_schema: Option<Value>,
    ) -> Result<ProviderRequest, TypeError> {
        let mut openai_messages = system_instructions;
        openai_messages.extend(messages);

        let openai_settings = match settings {
            ModelSettings::OpenAIChat(s) => Some(s),
            _ => None,
        };

        let response_json_schema = response_json_schema
            .as_ref()
            .map(create_structured_output_schema);

        Ok(ProviderRequest::OpenAIV1(OpenAIChatCompletionRequestV1 {
            model,
            messages: openai_messages,
            response_format: response_json_schema,
            settings: openai_settings,
        }))
    }

    fn set_response_json_schema(&mut self, response_json_schema: Option<Value>) {
        self.response_format =
            response_json_schema.map(|json_schema| create_structured_output_schema(&json_schema));
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenAIEmbeddingRequest<T>
where
    T: Serialize,
{
    pub input: Vec<String>,

    #[serde(flatten)]
    pub settings: T,
}

impl<T> OpenAIEmbeddingRequest<T>
where
    T: Serialize,
{
    /// Creates a new OpenAI embedding request with generic settings
    ///
    /// # Arguments
    /// * `inputs` - Vector of strings to embed
    /// * `settings` - Any configuration type that implements Serialize
    ///
    /// # Returns
    /// * `Self` - New embedding request instance
    pub fn new(input: Vec<String>, settings: T) -> Self {
        Self { input, settings }
    }
}

pub fn create_structured_output_schema(json_schema: &Value) -> Value {
    // get title from schema
    let title = json_schema
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("StructuredOutput");

    serde_json::json!({
        "type": "json_schema",
        "json_schema": {
            "name": title,
            "schema": json_schema,
            "strict": true
        }
    })
}
