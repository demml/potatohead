use crate::prompt::error::PromptError;
use mime_guess;
use potato_type::StructuredOutput;
use potato_util::PyHelperFuncs;
use potato_util::{json_to_pyobject, pyobject_to_json};
use pyo3::types::PyAnyMethods;
use pyo3::types::PyDict;
use pyo3::types::PyString;
use pyo3::{prelude::*, IntoPyObjectExt};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fmt::Display;
use std::sync::OnceLock;
use tracing::instrument;
use tracing::{debug, error};
static DOCUMENT_MEDIA_TYPES: OnceLock<HashSet<&'static str>> = OnceLock::new();

pub enum Role {
    User,
    Assistant,
    Developer,
    Tool,
    Model,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
            Role::Developer => write!(f, "developer"),
            Role::Tool => write!(f, "tool"),
            Role::Model => write!(f, "model"),
        }
    }
}

fn get_document_media_types() -> &'static HashSet<&'static str> {
    DOCUMENT_MEDIA_TYPES.get_or_init(|| {
        let mut set = HashSet::new();
        set.insert("application/pdf");
        set.insert("text/plain");
        set.insert("text/csv");
        set.insert("application/vnd.openxmlformats-officedocument.wordprocessingml.document");
        set.insert("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
        set.insert("text/html");
        set.insert("text/markdown");
        set.insert("application/vnd.ms-excel");
        set
    })
}

fn get_audio_media_types() -> &'static HashSet<&'static str> {
    static AUDIO_MEDIA_TYPES: OnceLock<HashSet<&'static str>> = OnceLock::new();
    AUDIO_MEDIA_TYPES.get_or_init(|| {
        let mut set = HashSet::new();
        set.insert("audio/mpeg");
        set.insert("audio/wav");
        set
    })
}

fn get_image_media_types() -> &'static HashSet<&'static str> {
    static IMAGE_MEDIA_TYPES: OnceLock<HashSet<&'static str>> = OnceLock::new();
    IMAGE_MEDIA_TYPES.get_or_init(|| {
        let mut set = HashSet::new();
        set.insert("image/jpeg");
        set.insert("image/png");
        set.insert("image/gif");
        set.insert("image/webp");
        set
    })
}

fn image_format(media_type: &str) -> Result<String, PromptError> {
    let format = match media_type {
        "image/jpeg" => "jpeg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => {
            return Err(PromptError::Error(format!(
                "Unknown image media type: {media_type}"
            )))
        }
    };

    Ok(format.to_string())
}

fn document_format(media_type: &str) -> Result<String, PromptError> {
    let format = match media_type {
        "application/pdf" => "pdf",
        "text/plain" => "txt",
        "text/csv" => "csv",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => "docx",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => "xlsx",
        "text/html" => "html",
        "text/markdown" => "md",
        "application/vnd.ms-excel" => "xls",
        _ => {
            return Err(PromptError::Error(format!(
                "Unknown document media type: {media_type}",
            )))
        }
    };
    Ok(format.to_string())
}

fn guess_type(url: &str) -> Result<String, PromptError> {
    // fail if mime type is not found
    let mime_type = mime_guess::from_path(url)
        .first()
        .ok_or_else(|| PromptError::Error(format!("Failed to guess mime type for {url}")))?;

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
    fn media_type(&self) -> Result<String, PromptError> {
        if self.url.ends_with(".jpg") || self.url.ends_with(".jpeg") {
            Ok("image/jpeg".to_string())
        } else if self.url.ends_with(".png") {
            Ok("image/png".to_string())
        } else if self.url.ends_with(".gif") {
            Ok("image/gif".to_string())
        } else if self.url.ends_with(".webp") {
            Ok("image/webp".to_string())
        } else {
            Err(PromptError::Error(format!(
                "Unknown image file extension: {}",
                self.url
            )))
        }
    }

    #[getter]
    fn format(&self) -> Result<String, PromptError> {
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
    fn new(url: &str, kind: &str) -> Result<Self, PromptError> {
        Ok(Self {
            url: url.to_string(),
            kind: kind.to_string(),
        })
    }

    #[getter]
    pub fn media_type(&self) -> Result<String, PromptError> {
        guess_type(&self.url)
    }

    #[getter]
    fn format(&self) -> Result<String, PromptError> {
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
    fn new(data: Vec<u8>, media_type: &str, kind: &str) -> Result<Self, PromptError> {
        // assert that media type is valid, must be audio, image, or document
        let is_audio = get_audio_media_types().contains(media_type);
        let is_image = get_image_media_types().contains(media_type);
        let is_document = get_document_media_types().contains(media_type);

        debug!("Creating BinaryContent with media_type: {media_type}, is_audio: {is_audio}, is_image: {is_image}, is_document: {is_document}");

        if !is_audio && !is_image && !is_document {
            return Err(PromptError::Error(format!(
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
    fn format(&self) -> Result<String, PromptError> {
        if self.is_audio() {
            if self.media_type == "audio/mpeg" {
                Ok("mp3".to_string())
            } else if self.media_type == "audio/wav" {
                Ok("wav".to_string())
            } else {
                Err(PromptError::Error(format!(
                    "Unknown media type: {}",
                    self.media_type
                )))
            }
        } else if self.is_image() {
            image_format(&self.media_type)
        } else if self.is_document() {
            document_format(&self.media_type)
        } else {
            Err(PromptError::Error(format!(
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
}

impl PromptContent {
    pub fn new(prompt: &Bound<'_, PyAny>) -> Result<Self, PromptError> {
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
        } else {
            Err(PromptError::Error("Unsupported prompt content type".into()))
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
        }
    }

    pub fn from_json_value(value: &Value) -> Result<Self, PromptError> {
        match value {
            Value::String(s) => Ok(PromptContent::Str(s.clone())),
            Value::Object(obj) => {
                if obj.contains_key("audio_url") {
                    AudioUrl::model_validate_json(value)
                        .map(PromptContent::Audio)
                        .map_err(|e| PromptError::Error(format!("Invalid audio_url: {e}")))
                } else if obj.contains_key("image_url") {
                    ImageUrl::model_validate_json(value)
                        .map(PromptContent::Image)
                        .map_err(|e| PromptError::Error(format!("Invalid image_url: {e}")))
                } else if obj.contains_key("document_url") {
                    DocumentUrl::model_validate_json(value)
                        .map(PromptContent::Document)
                        .map_err(|e| PromptError::Error(format!("Invalid document_url: {e}")))
                } else {
                    Err(PromptError::Error(
                        "Unsupported JSON object for PromptContent".into(),
                    ))
                }
            }
            _ => Err(PromptError::Error(
                "Unsupported JSON value for PromptContent".into(),
            )),
        }
    }
}

pub fn get_pydantic_module<'py>(py: Python<'py>, module_name: &str) -> PyResult<Bound<'py, PyAny>> {
    py.import("pydantic_ai")?.getattr(module_name)
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Message {
    pub content: PromptContent,
    pub role: String,
    pub variables: Vec<String>,
}

#[pymethods]
impl Message {
    #[new]
    #[pyo3(signature = (content))]
    pub fn new(content: &Bound<'_, PyAny>) -> PyResult<Self> {
        let content = PromptContent::new(content)?;
        let variables = Self::extract_variables(&content);
        Ok(Self {
            content,
            role: Role::User.to_string(),
            variables,
        })
    }

    pub fn bind(&self, name: &str, value: &str) -> Result<Message, PromptError> {
        let placeholder = format!("${{{name}}}");

        let content = match &self.content {
            PromptContent::Str(content) => {
                let new_content = content.replace(&placeholder, value);
                PromptContent::Str(new_content)
            }
            _ => self.content.clone(),
        };

        Ok(Message {
            content,
            role: self.role.clone(),
            variables: self.variables.clone(),
        })
    }

    #[instrument(skip_all)]
    pub fn bind_mut(&mut self, name: &str, value: &str) -> Result<(), PromptError> {
        debug!("Binding variable: {name} with value: {value}");
        let placeholder = format!("${{{name}}}");

        match &mut self.content {
            PromptContent::Str(content) => {
                *content = content.replace(&placeholder, value);
            }
            _ => return Err(PromptError::Error("Cannot bind non-string content".into())),
        }

        Ok(())
    }

    pub fn unwrap<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.content.to_pyobject(py)
    }

    pub fn model_dump<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let message = PyDict::new(py);

        message.set_item("role", self.role.clone())?;
        message.set_item("content", self.unwrap(py)?)?;
        Ok(message)
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

impl Message {
    pub fn new_rs(content: PromptContent) -> Self {
        let variables = Self::extract_variables(&content);
        Self {
            content,

            role: Role::User.to_string(),
            variables,
        }
    }
    pub fn from(content: PromptContent, role: Role) -> Self {
        let variables = Self::extract_variables(&content);
        Self {
            content,
            role: role.to_string(),
            variables,
        }
    }

    pub fn is_empty(&self) -> bool {
        match &self.content {
            PromptContent::Str(s) => s.is_empty(),
            _ => false,
        }
    }

    pub fn extract_variables(content: &PromptContent) -> Vec<String> {
        let mut variables = HashSet::new();

        if let PromptContent::Str(content) = content {
            // Create regex to find all ${variable_name} patterns
            // This is lazily initialized to avoid recompiling the regex each call
            static VAR_REGEX: OnceLock<Regex> = OnceLock::new();
            let regex = VAR_REGEX.get_or_init(|| {
                Regex::new(r"\$\{([^}]+)\}").expect("Failed to compile variable regex")
            });

            // Find all matches and collect variable names
            for captures in regex.captures_iter(content) {
                if let Some(name) = captures.get(1) {
                    variables.insert(name.as_str().to_string());
                }
            }
        }

        // Convert HashSet to Vec for return
        variables.into_iter().collect()
    }
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
) -> Result<bool, PromptError> {
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
fn get_json_schema_from_basemodel(object: &Bound<'_, PyAny>) -> Result<Value, PromptError> {
    // call staticmethod .model_json_schema()
    let schema = object.getattr("model_json_schema")?.call1(())?;

    let mut schema = pyobject_to_json(&schema).map_err(|e| {
        error!("Failed to convert schema to JSON: {}", e);
        PromptError::PySerializationError(e.to_string())
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
) -> Result<Option<Value>, PromptError> {
    let is_subclass = check_pydantic_model(py, object)?;
    if is_subclass {
        Ok(Some(get_json_schema_from_basemodel(object)?))
    } else {
        Ok(None)
    }
}

pub fn check_response_type(object: &Bound<'_, PyAny>) -> Result<Option<ResponseType>, PromptError> {
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

fn get_json_schema_from_response_type(response_type: &ResponseType) -> Result<Value, PromptError> {
    match response_type {
        ResponseType::Score => Ok(Score::get_structured_output_schema()),
        _ => {
            // If the response type is not recognized, return None
            Err(PromptError::Error(format!(
                "Unsupported response type: {response_type}"
            )))
        }
    }
}

pub fn parse_response_to_json<'py>(
    py: Python<'py>,
    object: &Bound<'_, PyAny>,
) -> Result<(ResponseType, Option<Value>), PromptError> {
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
#[serde(deny_unknown_fields)] // this ensures schemars will not allow additional fields (AdditionalProperties: false)
pub struct Score {
    #[pyo3(get)]
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
    pub fn model_validate_json(json_string: String) -> Result<Score, PromptError> {
        Ok(serde_json::from_str(&json_string)?)
    }

    #[staticmethod]
    pub fn model_json_schema(py: Python<'_>) -> Result<PyObject, PromptError> {
        let schema = Score::get_structured_output_schema();
        Ok(json_to_pyobject(py, &schema)?)
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

impl StructuredOutput for Score {}

//impl Score {
//    pub fn to_json_schema() -> Value {
//        serde_json::json!({
//            "type": "object",
//            "properties": {
//                "score": { "type": "integer" },
//                "reason": { "type": "string" },
//            },
//            "required": ["score", "reason"],
//        })
//    }
//
//    pub fn get_structured_output_schema() -> Value {
//        let json_schema = serde_json::json!({
//            "type": "json_schema",
//            "json_schema": {
//                 "name": "Score",
//                 "schema": Self::to_json_schema(),
//                 "strict": true
//            },
//        });
//
//        json_schema
//    }
//}

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
