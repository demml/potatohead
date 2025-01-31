use crate::common::{FileName, PromptType, Utils};
use crate::error::WormTongueError;
use crate::tongues::openai::request::{Message, OpenAIRequest};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
pub struct OpenAIPrompt {
    #[pyo3(get, set)]
    pub request: OpenAIRequest,

    #[pyo3(get)]
    pub prompt_type: PromptType,
}

#[pymethods]
impl OpenAIPrompt {
    #[new]
    #[pyo3(signature = (request))]
    pub fn new(request: &OpenAIRequest) -> PyResult<Self> {
        let prompt_type = match request {
            OpenAIRequest::Chat(_) => PromptType::Text,
        };

        Ok(Self {
            request: request.clone(),
            prompt_type,
        })
    }

    #[pyo3(signature = (message))]
    pub fn add_message(&mut self, message: Message) {
        self.request.add_message(message);
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
        let request = OpenAIRequest::default();
        let prompt_type = match request {
            OpenAIRequest::Chat(_) => PromptType::Text,
        };
        Self {
            request,
            prompt_type,
        }
    }
}
