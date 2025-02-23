use crate::error::WormTongueError;
use crate::tongues::common::{prompt, FileName, PromptType, TongueType, Utils};
use crate::tongues::openai::request::{Message, OpenAIRequest};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAIPrompt {
    #[pyo3(get, set)]
    pub request: Option<OpenAIRequest>,

    #[pyo3(get)]
    pub prompt_type: PromptType,

    #[pyo3(get)]
    pub tongue_type: TongueType,

    pub raw_request: Option<serde_json::Value>,
}

#[pymethods]
impl OpenAIPrompt {
    #[new]
    #[pyo3(signature = (request=None, raw_request=None, prompt_type=None))]
    pub fn new(
        request: Option<&Bound<'_, PyAny>>,
        raw_request: Option<&Bound<'_, PyDict>>,
        prompt_type: Option<PromptType>,
    ) -> PyResult<Self> {
        let request = match request {
            Some(request) => Some(request.extract::<OpenAIRequest>()?),
            None => None,
        };

        let prompt_type = match request {
            Some(request) => match request {
                OpenAIRequest::Chat(_) => PromptType::Text,
            },
            None => prompt_type.unwrap_or(PromptType::Text),
        };

        let tongue_type = TongueType::OpenAI;

        Ok(Self {
            request,
            prompt_type,
            tongue_type,
            raw: raw_request.map(|r| r.to_object()),
        })
    }

    #[pyo3(signature = (message), name="add_message")]
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
            tongue_type: TongueType::OpenAI,
        }
    }
}
