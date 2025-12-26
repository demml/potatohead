use crate::anthropic::v1::request::MessageParam;
use crate::anthropic::v1::request::MessageParam as AnthropicMessage;
use crate::anthropic::v1::request::TextBlockParam;
use crate::anthropic::v1::response::ResponseContentBlock;
use crate::common::document_format;
use crate::common::get_audio_media_types;
use crate::common::get_document_media_types;
use crate::common::get_image_media_types;
use crate::common::image_format;
use crate::google::v1::generate::Candidate;
use crate::google::v1::generate::GeminiContent;
use crate::google::PredictResponse;
use crate::openai::v1::chat::request::ChatMessage as OpenAIChatMessage;
use crate::openai::v1::Choice;
use crate::traits::MessageConversion;
use crate::traits::PromptMessageExt;
use crate::Provider;
use crate::{StructuredOutput, TypeError};
use mime_guess;
use potato_util::PyHelperFuncs;
use potato_util::{json_to_pyobject, pyobject_to_json};
use pyo3::types::PyAnyMethods;
use pyo3::types::PyString;
use pyo3::{prelude::*, IntoPyObjectExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Display;
use tracing::{debug, error};

pub enum Role {
    User,
    Assistant,
    Developer,
    Tool,
    Model,
    System,
}

impl Role {
    /// Returns the string representation of the role
    pub const fn as_str(&self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Developer => "developer",
            Role::Tool => "tool",
            Role::Model => "model",
            Role::System => "system",
        }
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
            Role::Developer => write!(f, "developer"),
            Role::Tool => write!(f, "tool"),
            Role::Model => write!(f, "model"),
            Role::System => write!(f, "system"),
        }
    }
}

impl From<Role> for &str {
    fn from(role: Role) -> Self {
        match role {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Developer => "developer",
            Role::Tool => "tool",
            Role::Model => "model",
            Role::System => "system",
        }
    }
}
fn guess_type(url: &str) -> Result<String, TypeError> {
    // fail if mime type is not found
    let mime_type = mime_guess::from_path(url)
        .first()
        .ok_or_else(|| TypeError::Error(format!("Failed to guess mime type for {url}")))?;

    Ok(mime_type.to_string())
}

pub trait DeserializePromptValExt: for<'de> serde::Deserialize<'de> {
    /// Validates and deserializes a JSON value into its struct type.
    ///
    /// # Arguments
    /// * `value` - The JSON value to deserialize
    ///
    /// # Returns
    /// * `Result<Self, serde_json::Error>` - The deserialized value or error
    fn model_validate_json(value: &Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value.clone())
    }
}

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AudioUrl {
    #[pyo3(get, set)]
    pub url: String,
    #[pyo3(get)]
    pub kind: String,
}

#[pymethods]
impl AudioUrl {
    #[new]
    fn new(url: String) -> PyResult<Self> {
        if !url.ends_with(".mp3") && !url.ends_with(".wav") {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Unknown audio file extension: {url}",
            )));
        }
        Ok(Self {
            url,
            kind: "audio-url".to_string(),
        })
    }

    #[getter]
    fn media_type(&self) -> String {
        if self.url.ends_with(".mp3") {
            "audio/mpeg".to_string()
        } else {
            "audio/wav".to_string()
        }
    }
}

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ImageUrl {
    #[pyo3(get, set)]
    pub url: String,
    #[pyo3(get)]
    pub kind: String,
}

#[pymethods]
impl ImageUrl {
    #[new]
    #[pyo3(signature = (url, kind="image-url"))]
    fn new(url: &str, kind: &str) -> PyResult<Self> {
        if !url.ends_with(".jpg")
            && !url.ends_with(".jpeg")
            && !url.ends_with(".png")
            && !url.ends_with(".gif")
            && !url.ends_with(".webp")
        {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Unknown image file extension: {url}",
            )));
        }
        Ok(Self {
            url: url.to_string(),
            kind: kind.to_string(),
        })
    }

    #[getter]
    fn media_type(&self) -> Result<String, TypeError> {
        if self.url.ends_with(".jpg") || self.url.ends_with(".jpeg") {
            Ok("image/jpeg".to_string())
        } else if self.url.ends_with(".png") {
            Ok("image/png".to_string())
        } else if self.url.ends_with(".gif") {
            Ok("image/gif".to_string())
        } else if self.url.ends_with(".webp") {
            Ok("image/webp".to_string())
        } else {
            Err(TypeError::Error(format!(
                "Unknown image file extension: {}",
                self.url
            )))
        }
    }

    #[getter]
    fn format(&self) -> Result<String, TypeError> {
        let media_type = self.media_type()?;
        image_format(&media_type)
    }
}

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DocumentUrl {
    #[pyo3(get, set)]
    pub url: String,
    #[pyo3(get)]
    pub kind: String,
}

#[pymethods]
impl DocumentUrl {
    #[new]
    #[pyo3(signature = (url, kind="document-url"))]
    fn new(url: &str, kind: &str) -> Result<Self, TypeError> {
        Ok(Self {
            url: url.to_string(),
            kind: kind.to_string(),
        })
    }

    #[getter]
    pub fn media_type(&self) -> Result<String, TypeError> {
        guess_type(&self.url)
    }

    #[getter]
    fn format(&self) -> Result<String, TypeError> {
        let media_type = self.media_type()?;
        document_format(&media_type)
    }
}

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BinaryContent {
    #[pyo3(get, set)]
    pub data: Vec<u8>,
    #[pyo3(get, set)]
    pub media_type: String,
    #[pyo3(get)]
    pub kind: String,
}

#[pymethods]
impl BinaryContent {
    #[new]
    #[pyo3(signature = (data, media_type, kind="binary"))]
    fn new(data: Vec<u8>, media_type: &str, kind: &str) -> Result<Self, TypeError> {
        // assert that media type is valid, must be audio, image, or document
        let is_audio = get_audio_media_types().contains(media_type);
        let is_image = get_image_media_types().contains(media_type);
        let is_document = get_document_media_types().contains(media_type);

        debug!("Creating BinaryContent with media_type: {media_type}, is_audio: {is_audio}, is_image: {is_image}, is_document: {is_document}");

        if !is_audio && !is_image && !is_document {
            return Err(TypeError::Error(format!(
                "Unknown media type: {media_type}",
            )));
        }

        Ok(Self {
            data,
            media_type: media_type.to_string(),
            kind: kind.to_string(),
        })
    }

    #[getter]
    fn is_audio(&self) -> bool {
        get_audio_media_types().contains(self.media_type.as_str())
    }

    #[getter]
    fn is_image(&self) -> bool {
        get_image_media_types().contains(self.media_type.as_str())
    }

    #[getter]
    fn is_document(&self) -> bool {
        get_document_media_types().contains(self.media_type.as_str())
    }

    #[getter]
    fn format(&self) -> Result<String, TypeError> {
        if self.is_audio() {
            if self.media_type == "audio/mpeg" {
                Ok("mp3".to_string())
            } else if self.media_type == "audio/wav" {
                Ok("wav".to_string())
            } else {
                Err(TypeError::Error(format!(
                    "Unknown media type: {}",
                    self.media_type
                )))
            }
        } else if self.is_image() {
            image_format(&self.media_type)
        } else if self.is_document() {
            document_format(&self.media_type)
        } else {
            Err(TypeError::Error(format!(
                "Unknown media type: {}",
                self.media_type
            )))
        }
    }
}

impl DeserializePromptValExt for AudioUrl {}
impl DeserializePromptValExt for ImageUrl {}
impl DeserializePromptValExt for DocumentUrl {}
impl DeserializePromptValExt for BinaryContent {}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PromptContent {
    Str(String),
    Audio(AudioUrl),
    Image(ImageUrl),
    Document(DocumentUrl),
    Binary(BinaryContent),
    AnthropicMessageContentV1(MessageParam),
}

impl PromptContent {
    pub fn new(prompt: &Bound<'_, PyAny>) -> Result<Self, TypeError> {
        if prompt.is_instance_of::<AudioUrl>() {
            let audio_url = prompt.extract::<AudioUrl>()?;
            Ok(PromptContent::Audio(audio_url))
        } else if prompt.is_instance_of::<ImageUrl>() {
            let image_url = prompt.extract::<ImageUrl>()?;
            Ok(PromptContent::Image(image_url))
        } else if prompt.is_instance_of::<DocumentUrl>() {
            let document_url = prompt.extract::<DocumentUrl>()?;
            Ok(PromptContent::Document(document_url))
        } else if prompt.is_instance_of::<BinaryContent>() {
            let binary_content = prompt.extract::<BinaryContent>()?;
            Ok(PromptContent::Binary(binary_content))
        } else if prompt.is_instance_of::<PyString>() {
            let user_content = prompt.extract::<String>()?;
            Ok(PromptContent::Str(user_content))
        } else if prompt.is_instance_of::<MessageParam>() {
            let message_content = prompt.extract::<MessageParam>()?;
            Ok(PromptContent::AnthropicMessageContentV1(message_content))
        } else {
            Err(TypeError::UnsupportedTypeError)
        }
    }

    pub fn to_pyobject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match self {
            PromptContent::Str(s) => s.into_bound_py_any(py),
            PromptContent::Audio(audio_url) => {
                // test pydantic module
                match get_pydantic_module(py, "AudioUrl") {
                    Ok(model_class) => {
                        model_class.call1((audio_url.url.clone(), audio_url.kind.clone()))
                    }
                    Err(_) => audio_url.clone().into_bound_py_any(py),
                }
            }
            PromptContent::Image(image_url) => {
                // test pydantic module
                match get_pydantic_module(py, "ImageUrl") {
                    Ok(model_class) => {
                        model_class.call1((image_url.url.clone(), image_url.kind.clone()))
                    }
                    Err(_) => image_url.clone().into_bound_py_any(py),
                }
            }
            PromptContent::Document(document_url) => {
                // test pydantic module
                match get_pydantic_module(py, "DocumentUrl") {
                    Ok(model_class) => {
                        model_class.call1((document_url.url.clone(), document_url.kind.clone()))
                    }
                    Err(_) => document_url.clone().into_bound_py_any(py),
                }
            }
            PromptContent::Binary(binary_content) => {
                // test pydantic module
                match get_pydantic_module(py, "BinaryContent") {
                    Ok(model_class) => model_class.call1((
                        binary_content.data.clone(),
                        binary_content.media_type.clone(),
                        binary_content.kind.clone(),
                    )),
                    Err(_) => binary_content.clone().into_bound_py_any(py),
                }
            }

            PromptContent::AnthropicMessageContentV1(message_content) => {
                message_content.clone().into_bound_py_any(py)
            }
        }
    }
}

pub fn get_pydantic_module<'py>(py: Python<'py>, module_name: &str) -> PyResult<Bound<'py, PyAny>> {
    py.import("pydantic_ai")?.getattr(module_name)
}

/// Checks if an object is a subclass of a pydantic BaseModel. This is used when validating structured outputs
/// # Arguments
/// * `py` - The Python interpreter instance
/// * `object` - The object to check
/// # Returns
/// A boolean indicating whether the object is a subclass of pydantic.BaseModel
pub fn check_pydantic_model<'py>(
    py: Python<'py>,
    object: &Bound<'_, PyAny>,
) -> Result<bool, TypeError> {
    // check pydantic import. Return false if it fails
    let pydantic = match py.import("pydantic").map_err(|e| {
        error!("Failed to import pydantic: {}", e);
        false
    }) {
        Ok(pydantic) => pydantic,
        Err(_) => return Ok(false),
    };

    // get builtin subclass
    let is_subclass = py.import("builtins")?.getattr("issubclass")?;

    // Need to check if provided object is a basemodel
    let basemodel = pydantic.getattr("BaseModel")?;
    let matched = is_subclass.call1((object, basemodel))?.extract::<bool>()?;

    Ok(matched)
}

/// Generate a JSON schema from a pydantic BaseModel object.
/// # Arguments
/// * `object` - The pydantic BaseModel object to generate the schema from.
/// # Returns
/// A JSON schema as a serde_json::Value.
fn get_json_schema_from_basemodel(object: &Bound<'_, PyAny>) -> Result<Value, TypeError> {
    // call staticmethod .model_json_schema()
    let schema = object.getattr("model_json_schema")?.call1(())?;

    let mut schema = pyobject_to_json(&schema).map_err(|e| {
        error!("Failed to convert schema to JSON: {}", e);
        TypeError::PySerializationError(e.to_string())
    })?;

    // ensure schema as additionalProperties set to false
    if let Some(additional_properties) = schema.get_mut("additionalProperties") {
        *additional_properties = serde_json::json!(false);
    } else {
        schema
            .as_object_mut()
            .unwrap()
            .insert("additionalProperties".to_string(), serde_json::json!(false));
    }

    Ok(schema)
}

fn parse_pydantic_model<'py>(
    py: Python<'py>,
    object: &Bound<'_, PyAny>,
) -> Result<Option<Value>, TypeError> {
    let is_subclass = check_pydantic_model(py, object)?;
    if is_subclass {
        Ok(Some(get_json_schema_from_basemodel(object)?))
    } else {
        Ok(None)
    }
}

pub fn check_response_type(object: &Bound<'_, PyAny>) -> Result<Option<ResponseType>, TypeError> {
    // try calling staticmethod response_type()
    let response_type = match object.getattr("response_type") {
        Ok(method) => {
            if method.is_callable() {
                let response_type: ResponseType = method.call0()?.extract()?;
                Some(response_type)
            } else {
                None
            }
        }
        Err(_) => None,
    };

    Ok(response_type)
}

fn get_json_schema_from_response_type(response_type: &ResponseType) -> Result<Value, TypeError> {
    match response_type {
        ResponseType::Score => Ok(Score::get_structured_output_schema()),
        _ => {
            // If the response type is not recognized, return None
            Err(TypeError::Error(format!(
                "Unsupported response type: {response_type}"
            )))
        }
    }
}

pub fn parse_response_to_json<'py>(
    py: Python<'py>,
    object: &Bound<'_, PyAny>,
) -> Result<(ResponseType, Option<Value>), TypeError> {
    // check if object is a pydantic model
    let is_pydantic_model = check_pydantic_model(py, object)?;
    if is_pydantic_model {
        return Ok((ResponseType::Pydantic, parse_pydantic_model(py, object)?));
    }

    // check if object has response_type method
    let response_type = check_response_type(object)?;
    if let Some(response_type) = response_type {
        return Ok((
            response_type.clone(),
            Some(get_json_schema_from_response_type(&response_type)?),
        ));
    }

    Ok((ResponseType::Null, None))
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)] // ensure strict validation
pub struct Score {
    #[pyo3(get)]
    #[schemars(range(min = 1, max = 5))]
    pub score: i64,

    #[pyo3(get)]
    pub reason: String,
}
#[pymethods]
impl Score {
    #[staticmethod]
    pub fn response_type() -> ResponseType {
        ResponseType::Score
    }

    #[staticmethod]
    pub fn model_validate_json(json_string: String) -> Result<Score, TypeError> {
        Ok(serde_json::from_str(&json_string)?)
    }

    #[staticmethod]
    pub fn model_json_schema(py: Python<'_>) -> Result<Py<PyAny>, TypeError> {
        let schema = Score::get_structured_output_schema();
        Ok(json_to_pyobject(py, &schema)?)
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

impl StructuredOutput for Score {}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResponseType {
    Score,
    Pydantic,
    Null, // This is used when no response type is specified
}

impl Display for ResponseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseType::Score => write!(f, "Score"),
            ResponseType::Pydantic => write!(f, "Pydantic"),
            ResponseType::Null => write!(f, "Null"),
        }
    }
}

// add conversion logic based on message conversion trait

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum MessageNum {
    OpenAIMessageV1(OpenAIChatMessage),
    AnthropicMessageV1(AnthropicMessage),
    GeminiContentV1(GeminiContent),

    // this is a special case for Anthropic system messages
    AnthropicSystemMessageV1(TextBlockParam),
}

impl MessageNum {
    /// Checks if the message type matches the given provider
    fn matches_provider(&self, provider: &Provider) -> bool {
        matches!(
            (self, provider),
            (MessageNum::OpenAIMessageV1(_), Provider::OpenAI)
                | (MessageNum::AnthropicMessageV1(_), Provider::Anthropic)
                | (MessageNum::AnthropicSystemMessageV1(_), Provider::Anthropic)
                | (MessageNum::GeminiContentV1(_), Provider::Google)
                | (MessageNum::GeminiContentV1(_), Provider::Vertex)
                | (MessageNum::GeminiContentV1(_), Provider::Gemini)
        )
    }

    /// Converts the message to an openai message
    /// This is only done for anthropic and gemini messages
    /// openai message will return a failure if called on an openai message
    /// Control flow should ensure this is only called on non-openai messages
    fn to_openai_message(&self) -> Result<MessageNum, TypeError> {
        match self {
            MessageNum::AnthropicMessageV1(msg) => {
                Ok(MessageNum::OpenAIMessageV1(msg.to_openai_message()?))
            }
            MessageNum::GeminiContentV1(msg) => {
                Ok(MessageNum::OpenAIMessageV1(msg.to_openai_message()?))
            }
            _ => Err(TypeError::CantConvertSelf),
        }
    }

    /// Converts to Anthropic message format
    fn to_anthropic_message(&self) -> Result<MessageNum, TypeError> {
        match self {
            MessageNum::OpenAIMessageV1(msg) => {
                Ok(MessageNum::AnthropicMessageV1(msg.to_anthropic_message()?))
            }
            MessageNum::GeminiContentV1(msg) => {
                Ok(MessageNum::AnthropicMessageV1(msg.to_anthropic_message()?))
            }
            _ => Err(TypeError::CantConvertSelf),
        }
    }

    /// Converts to Google Gemini message format
    fn to_google_message(&self) -> Result<MessageNum, TypeError> {
        match self {
            MessageNum::OpenAIMessageV1(msg) => {
                Ok(MessageNum::GeminiContentV1(msg.to_google_message()?))
            }
            MessageNum::AnthropicMessageV1(msg) => {
                Ok(MessageNum::GeminiContentV1(msg.to_google_message()?))
            }
            _ => Err(TypeError::CantConvertSelf),
        }
    }

    fn convert_message_to_provider_type(
        &self,
        provider: &Provider,
    ) -> Result<MessageNum, TypeError> {
        match provider {
            Provider::OpenAI => self.to_openai_message(),
            Provider::Anthropic => self.to_anthropic_message(),
            Provider::Google => self.to_google_message(),
            Provider::Vertex => self.to_google_message(),
            Provider::Gemini => self.to_google_message(),
            _ => Err(TypeError::UnsupportedProviderError),
        }
    }

    pub fn convert_message(&mut self, provider: &Provider) -> Result<(), TypeError> {
        // if message already matches provider, return Ok
        if self.matches_provider(provider) {
            return Ok(());
        }
        let converted = self.convert_message_to_provider_type(provider)?;
        *self = converted;
        Ok(())
    }

    pub fn anthropic_message_to_system_message(&mut self) -> Result<(), TypeError> {
        match self {
            MessageNum::AnthropicMessageV1(msg) => {
                let text_param = msg.to_text_block_param()?;
                *self = MessageNum::AnthropicSystemMessageV1(text_param);
                Ok(())
            }
            _ => Err(TypeError::Error(
                "Cannot convert non-AnthropicMessageV1 to system message".to_string(),
            )),
        }
    }
    pub fn role(&self) -> &str {
        match self {
            MessageNum::OpenAIMessageV1(msg) => &msg.role,
            MessageNum::AnthropicMessageV1(msg) => &msg.role,
            MessageNum::GeminiContentV1(msg) => &msg.role,
            _ => "system",
        }
    }
    pub fn bind(&self, name: &str, value: &str) -> Result<Self, TypeError> {
        match self {
            MessageNum::OpenAIMessageV1(msg) => {
                let bound_msg = msg.bind(name, value)?;
                Ok(MessageNum::OpenAIMessageV1(bound_msg))
            }
            MessageNum::AnthropicMessageV1(msg) => {
                let bound_msg = msg.bind(name, value)?;
                Ok(MessageNum::AnthropicMessageV1(bound_msg))
            }
            MessageNum::GeminiContentV1(msg) => {
                let bound_msg = msg.bind(name, value)?;
                Ok(MessageNum::GeminiContentV1(bound_msg))
            }
            _ => Ok(self.clone()),
        }
    }
    pub fn bind_mut(&mut self, name: &str, value: &str) -> Result<(), TypeError> {
        match self {
            MessageNum::OpenAIMessageV1(msg) => msg.bind_mut(name, value),
            MessageNum::AnthropicMessageV1(msg) => msg.bind_mut(name, value),
            MessageNum::GeminiContentV1(msg) => msg.bind_mut(name, value),
            _ => Ok(()),
        }
    }

    pub(crate) fn extract_variables(&self) -> Vec<String> {
        match self {
            MessageNum::OpenAIMessageV1(msg) => msg.extract_variables(),
            MessageNum::AnthropicMessageV1(msg) => msg.extract_variables(),
            MessageNum::GeminiContentV1(msg) => msg.extract_variables(),
            _ => vec![],
        }
    }

    pub fn to_bound_py_object<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        match self {
            MessageNum::OpenAIMessageV1(msg) => {
                let bound_msg = msg.clone().into_bound_py_any(py)?;
                Ok(bound_msg)
            }
            MessageNum::AnthropicMessageV1(msg) => {
                let bound_msg = msg.clone().into_bound_py_any(py)?;
                Ok(bound_msg)
            }
            MessageNum::GeminiContentV1(msg) => {
                let bound_msg = msg.clone().into_bound_py_any(py)?;
                Ok(bound_msg)
            }
            MessageNum::AnthropicSystemMessageV1(msg) => {
                let bound_msg = msg.clone().into_bound_py_any(py)?;
                Ok(bound_msg)
            }
        }
    }

    pub fn is_system_message(&self) -> bool {
        match self {
            MessageNum::OpenAIMessageV1(msg) => {
                msg.role == Role::Developer.to_string() || msg.role == Role::System.to_string()
            }
            MessageNum::AnthropicMessageV1(msg) => msg.role == Role::Assistant.to_string(),
            MessageNum::GeminiContentV1(msg) => msg.role == Role::Model.to_string(),
            MessageNum::AnthropicSystemMessageV1(_) => true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResponseContent {
    OpenAI(Choice),
    Google(Candidate),
    Anthropic(ResponseContentBlock),
    PredictResponse(PredictResponse),
}
