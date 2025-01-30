use crate::common::{FileName, Utils};
use crate::error::WormTongueError;
use crate::tongues::openai::types::OpenAIModels;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[pymethods]
impl Message {
    #[new]
    pub fn new(role: String, content: String) -> Self {
        Message { role, content }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAIPrompt {
    #[pyo3(get, set)]
    pub messages: Vec<Message>,

    #[pyo3(get)]
    pub temperature: f32,

    #[pyo3(get)]
    pub model: String,
}

#[pymethods]
impl OpenAIPrompt {
    #[new]
    #[pyo3(signature = (model = OpenAIModels::Gpt4o.as_str(), temperature = 0.7,  messages = vec![]))]
    pub fn new(model: &str, temperature: f32, messages: Vec<Message>) -> Self {
        OpenAIPrompt {
            messages,
            temperature,
            model: model.to_string(),
        }
    }

    pub fn add_message(&mut self, role: String, content: String) {
        self.messages.push(Message { role, content });
    }

    #[setter]
    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature = temperature;
    }

    #[setter]
    pub fn model(&mut self, model: String) {
        self.model = model;
    }

    pub fn __str__(&self) -> String {
        // serialize the struct to a string
        Utils::__str__(self)
    }

    pub fn model_dump_json(&self) -> String {
        // serialize the struct to a string
        Utils::__json__(self)
    }

    #[pyo3(signature = (path=None))]
    pub fn save_to_json(&self, path: Option<PathBuf>) -> PyResult<()> {
        Utils::save_to_json(self, path, &FileName::OpenAIPrompt.to_str())
            .map_err(|e| WormTongueError::new_err(e.to_string()))
    }
}

impl Default for OpenAIPrompt {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            temperature: 0.7,
            model: OpenAIModels::Gpt4o.to_string(),
        }
    }
}
