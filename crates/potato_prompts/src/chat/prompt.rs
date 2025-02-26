pub use crate::types::MessageEnum;
use crate::Message;
use potato_error::PotatoHeadError;
use potato_tools::{pyobject_to_json, PromptType, Utils};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::types::PyList;
use pyo3::IntoPyObjectExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatPrompt {
    #[pyo3(get, set)]
    pub model: String,

    pub messages: Vec<MessageEnum>,

    #[pyo3(get)]
    pub prompt_type: PromptType,

    original_messages: Vec<MessageEnum>,

    pub additional_data: Option<Value>,
}

#[pymethods]
impl ChatPrompt {
    #[new]
    #[pyo3(signature = (model, messages, additional_data=None))]
    pub fn new(
        model: &str,
        messages: &Bound<'_, PyAny>,
        additional_data: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let raw_json = additional_data
            .map(|dict| pyobject_to_json(dict))
            .transpose()?;
        let prompt_type = PromptType::Chat;
        let model = model.to_string();

        // Verify messages is a PyList
        if !messages.is_instance_of::<PyList>() {
            return Err(PotatoHeadError::new_err(
                "messages must be a list of Message objects",
            ));
        }

        let py_list = messages.downcast::<PyList>()?;
        let mut result_messages = Vec::new();

        // Iterate through messages list
        for item in py_list.iter() {
            let message = if item.is_instance_of::<Message>() {
                // Direct Message object
                item.extract::<Message>()?
            } else if item.is_instance_of::<PyDict>() {
                // Dictionary that needs to be converted to Message
                let dict = item.downcast::<PyDict>()?;
                let role: String = dict.get_item("role")?.unwrap().extract()?;
                let content = dict.get_item("content")?.unwrap();
                let name: Option<String> = match dict.get_item("name")? {
                    Some(py_name) => Some(py_name.extract()?),
                    None => None,
                };

                Message::new(&role, &content, name.as_deref())?
            } else {
                return Err(PotatoHeadError::new_err(
                    "messages must contain Message objects or dictionaries",
                ));
            };

            result_messages.push(MessageEnum::Base(message));
        }

        Ok(Self {
            model,
            prompt_type,
            messages: result_messages.clone(),  // Working copy
            original_messages: result_messages, // Original state
            additional_data: raw_json,
        })
    }

    #[getter]
    pub fn messages<'py>(&self, py: Python<'py>) -> Vec<PyObject> {
        self.messages
            .iter()
            .filter_map(|msg| match msg {
                MessageEnum::Base(msg) => msg.clone().into_py_any(py).ok(),
            })
            .collect()
    }

    #[getter]
    pub fn additional_data(&self) -> Option<String> {
        self.additional_data
            .clone()
            .map(|data| Utils::__json__(data))
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
        //if let Some(format) = &self.response_format {
        //    if let Some(spec_obj) = spec.as_object_mut() {
        //        spec_obj.insert("response_format".to_string(), format.clone());
        //    }
        //}

        spec
    }
}
