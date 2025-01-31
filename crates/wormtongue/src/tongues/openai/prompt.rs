use crate::common::{FileName, Utils};
use crate::error::WormTongueError;
use crate::tongues::openai::request::{Message, OpenAIRequest};
use crate::tongues::openai::types::OpenAIModels;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
pub struct OpenAIPrompt {
    #[pyo3(get, set)]
    pub request: OpenAIRequest,
}

#[pymethods]
impl OpenAIPrompt {
    #[new]
    #[pyo3(signature = (request))]
    pub fn new(request: &OpenAIRequest) -> PyResult<Self> {
        Ok(Self {
            request: request.clone(),
        })
    }

    #[pyo3(signature = (message))]
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(Message {
            role,
            content,
            name,
        });
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
            store: false,
            reasoning_effort: "medium".to_string(),
            metadata: None,
            frequency_penalty: 0,
            logit_bias: false,
            top_logprobs: None,
            max_completion_tokens: None,
            n: 1,
            modalities: vec!["text".to_string()],
            presence_penalty: 0,
            response_format: None,
        }
    }
}

impl OpenAIPrompt {
    pub fn validate_metadata(metadata: &HashMap<String, String>) -> PyResult<()> {
        if metadata.len() > 16 {
            return Err(WormTongueError::new_err(
                "metadata may not exceed 16 key-value pairs",
            ));
        }
        for (key, value) in metadata {
            if key.len() > 64 {
                return Err(WormTongueError::new_err(
                    "metadata keys cannot exceed 64 characters",
                ));
            }
            if value.len() > 512 {
                return Err(WormTongueError::new_err(
                    "metadata values cannot exceed 512 characters",
                ));
            }
        }
        Ok(())
    }
}
