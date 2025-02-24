use crate::error::WormTongueError;
use crate::tongues::common::{pyobject_to_json, FileName, PromptType, TongueType, Utils};
use crate::tongues::openai::request::OpenAIRequest;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
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
        // Extract the request and raw_request from the Python objects
        let request = request.map(|r| OpenAIRequest::py_new(r)).transpose()?;
        let raw_json = raw_request.map(|dict| pyobject_to_json(dict)).transpose()?;
        let prompt_type = request
            .as_ref()
            .map(|r| match r {
                OpenAIRequest::Chat(_) => PromptType::Text,
            })
            .unwrap_or_else(|| prompt_type.unwrap_or(PromptType::Text));
        let tongue_type = TongueType::OpenAI;

        Ok(Self {
            request,
            prompt_type,
            tongue_type,
            raw_request: raw_json,
        })
    }

    pub fn build_raw_request(&mut self) -> PyResult<String> {
        // Build the request from the raw JSON
        if let Some(raw) = &self.raw_request {
            Ok(raw.to_string())
        } else {
            Err(WormTongueError::new_err("No raw request found"))
        }
    }

    #[pyo3(signature = (context=None))]
    pub fn build_request(
        &mut self,
        context: Option<Vec<Vec<String>>>,
    ) -> PyResult<serde_json::Value> {
        let request = self
            .request
            .as_mut()
            .ok_or(WormTongueError::new_err("No request found"))?;

        match (&self.prompt_type, request) {
            (PromptType::Text, OpenAIRequest::Chat(chat)) => {
                let mut chat = chat.clone();

                if let Some(context) = context {
                    // Zip messages with their corresponding context if available
                    for (message, ctx) in chat.messages.iter_mut().zip(context.iter()) {
                        // Apply each context string to the message
                        for value in ctx {
                            message.bind(value)?;
                        }
                    }
                }

                Ok(chat.model_dump_json())
            }
            _ => Err(WormTongueError::new_err(
                "Unsupported prompt type or request type",
            )),
        }
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
            request: None,
            prompt_type: PromptType::Text,
            tongue_type: TongueType::OpenAI,
            raw_request: None,
        }
    }
}
