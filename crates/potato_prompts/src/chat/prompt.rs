use crate::chat::sanitize::SanitizationResult;
use crate::chat::sanitize::{PromptSanitizer, SanitizationConfig};
use crate::Message;
use potato_error::PotatoError;
use potato_error::PotatoHeadError;
use potato_tools::FileName;
use potato_tools::{json_to_pyobject, pyobject_to_json, PromptType, Utils};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::types::PyList;
use pyo3::IntoPyObjectExt;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

fn parse_message_from_pyobject(message: &Bound<'_, PyAny>) -> PyResult<Message> {
    if message.is_instance_of::<Message>() {
        message.extract::<Message>()
    } else if message.is_instance_of::<PyDict>() {
        let dict = message.downcast::<PyDict>()?;
        let role: String = dict.get_item("role")?.unwrap().extract()?;
        let content = dict.get_item("content")?.unwrap();
        let name: Option<String> = match dict.get_item("name")? {
            Some(py_name) => {
                if py_name.is_none() {
                    None
                } else {
                    Some(py_name.extract()?)
                }
            }
            None => None,
        };

        Message::new(&role, &content, name.as_deref())
    } else {
        Err(PotatoHeadError::new_err(
            "messages must contain Message objects or dictionaries
            in the format {'role': str, 'content': Any, 'name': Optional[str]}
            ",
        ))
    }
}

fn parse_messages_from_pyobject(messages: &Bound<'_, PyAny>) -> PyResult<Vec<Message>> {
    // Verify messages is a PyList
    if !messages.is_instance_of::<PyList>() {
        return Err(PotatoHeadError::new_err(
            "messages must be a list of Message objects",
        ));
    }

    let messages = messages.downcast::<PyList>()?;
    let result_messages = messages
        .iter()
        .map(|item| parse_message_from_pyobject(&item))
        .collect::<PyResult<Vec<Message>>>()?;

    Ok(result_messages)
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatPrompt {
    #[pyo3(get, set)]
    pub model: String,

    pub messages: Vec<Message>,

    #[pyo3(get)]
    pub prompt_type: PromptType,

    original_messages: Vec<Message>,

    pub additional_data: Option<Value>,

    pub version: String,

    #[pyo3(get)]
    pub sanitization_config: Option<SanitizationConfig>,

    #[serde(skip)] // skip serialization and deserialization (added when loading from json)
    pub sanitizer: Option<PromptSanitizer>,

    #[serde(serialize_with = "serialize_as_empty_vec", default = "Vec::new")]
    #[pyo3(get)]
    pub sanitized_results: Vec<SanitizationResult>,

    #[pyo3(get)]
    pub has_sanitize_error: bool,
}

fn serialize_as_empty_vec<S>(_: &Vec<SanitizationResult>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // Always serialize as an empty vector
    serializer.serialize_seq(Some(0))?.end()
}

#[pymethods]
impl ChatPrompt {
    #[new]
    #[pyo3(signature = (model, messages, sanitization_config=None, **kwargs))]
    pub fn new(
        model: &str,
        messages: &Bound<'_, PyAny>,
        sanitization_config: Option<SanitizationConfig>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        // Convert additional_data to JSON format for serialization
        let raw_json = kwargs.map(|dict| pyobject_to_json(dict)).transpose()?;

        // extract messages
        let messages = parse_messages_from_pyobject(messages)?;

        // get version from crate
        let version = env!("CARGO_PKG_VERSION").to_string();

        // Create a sanitizer if sanitization_config is provided
        let sanitizer = sanitization_config
            .clone()
            .map(|config| PromptSanitizer::new(config));

        Ok(Self {
            model: model.to_string(),
            prompt_type: PromptType::Chat,
            original_messages: messages.clone(),
            messages,
            additional_data: raw_json,
            version,
            sanitization_config,
            sanitizer,
            sanitized_results: Vec::new(),
            has_sanitize_error: false,
        })
    }

    #[getter]
    pub fn messages<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.messages.clone().into_bound_py_any(py)
    }

    #[getter]
    pub fn additional_data(&self) -> Option<String> {
        self.additional_data.clone().map(Utils::__json__)
    }

    #[pyo3(signature = (message))]
    pub fn add_message(&mut self, message: Bound<'_, PyAny>) -> PyResult<()> {
        let message = parse_message_from_pyobject(&message)?;
        self.messages.push(message);
        Ok(())
    }

    #[pyo3(signature = (context, index=0,))]
    pub fn bind_context_at(&mut self, context: String, index: usize) -> PyResult<()> {
        let new_context = self.sanitize_message(&context)?;

        if let Some(message) = self.messages.get_mut(index) {
            message.bind(&new_context)?;

            Ok(())
        } else {
            Err(PotatoHeadError::new_err(format!(
                "Message index {} out of bounds",
                index
            )))
        }
    }

    pub fn deep_copy(&mut self, py: Python) -> PyResult<Py<Self>> {
        // Create new object with current state
        Py::new(py, self.clone())
    }

    pub fn reset(&mut self) -> PyResult<()> {
        self.messages = self.original_messages.clone();
        for message in &mut self.messages {
            message.reset_binding();
        }
        self.has_sanitize_error = false;
        self.sanitized_results.clear();

        Ok(())
    }

    pub fn __str__(&self) -> String {
        // iterate over messages and create json() object

        let msgs = self
            .messages
            .iter()
            .map(|msg| {
                json!({
                    "role": msg.role,
                    "content": msg.content,
                })
            })
            .collect::<Vec<Value>>();

        let val = json!({
            "model": self.model,
            "messages": msgs,
            "additional_data": self.additional_data,
        });

        Utils::__str__(val)
    }

    pub fn open_ai_spec(&self) -> String {
        Utils::__str__(self.to_open_ai_spec())
    }

    pub fn model_dump_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn to_open_ai_request<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let spec = self.to_open_ai_spec();
        let pydict = PyDict::new(py);
        json_to_pyobject(py, &spec, &pydict)
    }

    #[staticmethod]
    pub fn model_validate_json(json_string: String) -> PyResult<Self> {
        let json_value: Value = serde_json::from_str(&json_string)
            .map_err(|e| PotatoHeadError::new_err(format!("Failed to parse JSON string: {}", e)))?;
        let mut model: Self = serde_json::from_value(json_value)
            .map_err(|e| PotatoHeadError::new_err(format!("Failed to parse JSON value: {}", e)))?;

        // if model has sanitization_config, create a sanitizer
        if let Some(config) = &model.sanitization_config {
            model.sanitizer = Some(PromptSanitizer::new(config.clone()));
        }

        Ok(model)
    }

    #[pyo3(signature = (path = None))]
    pub fn save_prompt(&self, path: Option<PathBuf>) -> PyResult<PathBuf> {
        let path = Utils::save_to_json(self, path, &FileName::Prompt.to_str())?;
        Ok(path)
    }

    #[staticmethod]
    pub fn load_from_path(path: PathBuf) -> PyResult<Self> {
        // Load the JSON file from the path
        let file = std::fs::read_to_string(&path)
            .map_err(|e| PotatoHeadError::new_err(format!("Failed to read file: {}", e)))?;

        // Parse the JSON file into a ChatPrompt
        serde_json::from_str(&file)
            .map_err(|e| PotatoHeadError::new_err(format!("Failed to parse JSON: {}", e)))
    }
}
use std::borrow::Cow;
impl ChatPrompt {
    pub fn to_open_ai_spec(&self) -> Value {
        let msgs = self
            .messages
            .iter()
            .map(|msg| msg.to_spec())
            .collect::<Vec<Value>>();

        let mut spec = json!({
            "model": self.model,
            "messages": msgs,
        });

        // If additional_data exists, merge it into the spec
        if let Some(additional) = &self.additional_data {
            if let Some(spec_obj) = spec.as_object_mut() {
                if let Some(additional_obj) = additional.as_object() {
                    for (key, value) in additional_obj {
                        spec_obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        // if response_format exists, merge it into the spec
        //if let Some(format) = &self.response_format {
        //    if let Some(spec_obj) = spec.as_object_mut() {
        //        spec_obj.insert("response_format".to_string(), format.clone());
        //    }
        //}

        spec
    }

    /// Sanitize the message using the provided sanitizer
    /// If no sanitizer is provided, return the original message
    ///
    /// Returns the sanitized message
    ///
    fn sanitize_message<'a>(&mut self, message: &'a str) -> Result<Cow<'a, str>, PotatoError> {
        if let Some(sanitizer) = &self.sanitizer {
            let result = if sanitizer.config.sanitize {
                sanitizer.sanitize(message)?
            } else {
                sanitizer.assess_risk(message)?
            };

            if result.risk_level >= sanitizer.config.risk_threshold {
                self.has_sanitize_error = true;
            }

            let sanitized = result.sanitized_text.clone();
            self.sanitized_results.push(result);

            return Ok(Cow::Owned(sanitized));
        }

        // No need to clone the original message
        Ok(Cow::Borrowed(message))
    }
}
