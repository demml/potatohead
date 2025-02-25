use crate::error::WormTongueError;
use crate::tongues::common::{
    convert_pydantic_to_json_schema, pyobject_to_json, PromptType, Utils,
};
pub use crate::tongues::prompts::chat::Message;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatPrompt {
    #[pyo3(get, set)]
    model: String,

    #[pyo3(get, set)]
    messages: Vec<Message>,

    #[pyo3(get)]
    pub prompt_type: PromptType,

    pub response_format: Option<Value>,

    original_messages: Vec<Message>,

    pub additional_data: Option<Value>,
}

#[pymethods]
impl ChatPrompt {
    #[new]
    #[pyo3(signature = (model, messages, additional_data=None, response_format=None))]
    pub fn new(
        py: Python,
        model: &str,
        messages: Vec<Message>,
        additional_data: Option<&Bound<'_, PyDict>>,
        response_format: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let raw_json = additional_data
            .map(|dict| pyobject_to_json(dict))
            .transpose()?;
        let prompt_type = PromptType::Chat;
        let model = model.to_string();

        // If response_format is provided, check if it is a pydantic BaseModel

        let response_format = response_format
            .map(|format| convert_pydantic_to_json_schema(py, format))
            .transpose()?;

        Ok(Self {
            model,
            prompt_type,
            messages: messages.clone(),  // Working copy
            original_messages: messages, // Original state
            additional_data: raw_json,
            response_format,
        })
    }

    #[getter]
    pub fn additional_data(&self) -> Option<String> {
        self.additional_data
            .clone()
            .map(|data| Utils::__json__(data))
    }

    #[getter]
    pub fn response_format(&self) -> Option<String> {
        self.response_format
            .clone()
            .map(|format| Utils::__json__(format))
    }

    #[pyo3(signature = (message))]
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    #[pyo3(signature = (context, index=0,))]
    pub fn bind_context_at(&mut self, context: String, index: usize) -> PyResult<()> {
        if let Some(message) = self.messages.get_mut(index) {
            message.bind(&context)?;

            Ok(())
        } else {
            Err(WormTongueError::new_err(format!(
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
            "response_format": self.response_format,
        });

        Utils::__str__(val)
    }

    pub fn open_ai_spec(&self) -> String {
        Utils::__str__(self.to_open_ai_spec())
    }
}

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
        if let Some(format) = &self.response_format {
            if let Some(spec_obj) = spec.as_object_mut() {
                spec_obj.insert("response_format".to_string(), format.clone());
            }
        }

        spec
    }
}
