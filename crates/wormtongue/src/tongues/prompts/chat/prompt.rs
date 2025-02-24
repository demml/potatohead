use crate::error::WormTongueError;
use crate::tongues::common::{pyobject_to_json, PromptType};
pub use crate::tongues::prompts::chat::Message;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatPrompt {
    #[pyo3(get)]
    model: String,

    #[pyo3(get)]
    messages: Vec<Message>,

    #[pyo3(get)]
    original_messages: Vec<Message>,

    pub additional_data: Option<Value>,

    #[pyo3(get)]
    pub prompt_type: PromptType,
}

#[pymethods]
impl ChatPrompt {
    #[new]
    #[pyo3(signature = (model, messages, additional_data=None))]
    pub fn new(
        model: &str,
        messages: Vec<Message>,
        additional_data: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let raw_json = additional_data
            .map(|dict| pyobject_to_json(dict))
            .transpose()?;
        let prompt_type = PromptType::Text;
        let model = model.to_string();

        Ok(Self {
            model,
            prompt_type,
            messages: messages.clone(),  // Working copy
            original_messages: messages, // Original state
            additional_data: raw_json,
        })
    }

    #[getter]
    pub fn get_messages(&self) -> Vec<Message> {
        self.messages.clone()
    }

    #[setter]
    pub fn set_messages(&mut self, messages: Vec<Message>) {
        self.messages = messages;
    }

    #[pyo3(signature = (message))]
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    #[pyo3(signature = (message_idx, context))]
    pub fn bind_context_at(&mut self, message_idx: usize, context: String) -> PyResult<()> {
        if let Some(message) = self.messages.get_mut(message_idx) {
            message.bind(&context)?;

            Ok(())
        } else {
            Err(WormTongueError::new_err(format!(
                "Message index {} out of bounds",
                message_idx
            )))
        }
    }

    pub fn build(&mut self, py: Python) -> PyResult<Py<Self>> {
        // Create new object with current state
        let result = Py::new(py, self.clone())?;

        // Reset working copy back to original state
        self.messages = self.original_messages.clone();

        // Reset next_param counter in messages
        for message in &mut self.messages {
            message.reset_binding();
        }

        Ok(result)
    }

    pub fn reset(&mut self) -> PyResult<()> {
        self.messages = self.original_messages.clone();
        for message in &mut self.messages {
            message.reset_binding();
        }
        Ok(())
    }
}

impl ChatPrompt {
    pub fn to_open_ai_spec(&self) -> Value {
        let mut spec = json!({
            "model": self.model,
            "messages": self.messages
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

        spec
    }
}
