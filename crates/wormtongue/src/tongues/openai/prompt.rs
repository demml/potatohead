use crate::common::{FileName, Utils};
use crate::error::WormTongueError;
use crate::tongues::openai::types::OpenAIModels;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextContent {
    pub content: String,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageContent {
    pub url: String,
    pub detail: String,
}

#[pymethods]
impl ImageContent {
    #[new]
    #[pyo3(signature = (url, detail="auto"))]
    pub fn new(url: String, detail: &str) -> Self {
        ImageContent {
            url,
            detail: detail.to_string(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioContent {
    pub data: String,
    pub format: String,
}

#[pymethods]
impl AudioContent {
    #[new]
    #[pyo3(signature = (data, format="mp3"))]
    pub fn new(data: String, format: &str) -> Self {
        AudioContent {
            data,
            format: format.to_string(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Content {
    Text(TextContent),
    Image(ImageContent),
    Audio(AudioContent),
}

#[pymethods]
impl Content {
    #[staticmethod]
    #[pyo3(signature = (content))]
    pub fn text(content: &str) -> Self {
        Content::Text(TextContent {
            content: content.to_string(),
        })
    }

    #[staticmethod]
    #[pyo3(signature = (url, detail="auto"))]
    pub fn image(url: &str, detail: &str) -> Self {
        Content::Image(ImageContent {
            url: url.to_string(),
            detail: detail.to_string(),
        })
    }

    #[staticmethod]
    #[pyo3(signature = (data, format="mp3"))]
    pub fn naudio(data: &str, format: &str) -> Self {
        Content::Audio(AudioContent {
            data: data.to_string(),
            format: format.to_string(),
        })
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: Vec<Content>,
    pub name: Option<String>,
}

#[pymethods]
impl Message {
    #[new]
    #[pyo3(signature = (role, content=vec![], name=None))]
    pub fn new(role: String, content: Vec<Content>, name: Option<String>) -> Self {
        Message {
            role,
            content,
            name,
        }
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

    #[pyo3(get)]
    pub store: bool,

    #[pyo3(get)]
    pub reasoning_effort: String,

    #[pyo3(get)]
    pub metadata: Option<HashMap<String, String>>,

    #[pyo3(get)]
    pub frequency_penalty: i32,

    #[pyo3(get)]
    pub logit_bias: bool,

    #[pyo3(get)]
    pub top_logprobs: Option<i32>,

    #[pyo3(get)]
    pub max_completion_tokens: Option<i32>,

    #[pyo3(get)]
    pub n: i32,

    #[pyo3(get)]
    pub modalities: Vec<String>,

    // not implemented pub prediction

    // not implemented pub audio
    #[pyo3(get)]
    pub presence_penalty: i32,
}

#[pymethods]
impl OpenAIPrompt {
    #[new]
    #[pyo3(signature = (model = OpenAIModels::Gpt4o.as_str(), temperature = 0.7,  messages = vec![], store=false, reasoning_effort="medium", metadata=None, frequency_penalty=0, logit_bias=false, top_logprobs=None, max_completion_tokens=None, n=1, modalities=vec!["text".to_string()], presence_penalty=0))]
    pub fn new(
        model: &str,
        temperature: f32,
        messages: Vec<Message>,
        store: bool,
        reasoning_effort: &str,
        metadata: Option<HashMap<String, String>>,
        frequency_penalty: i32,
        logit_bias: bool,
        top_logprobs: Option<i32>,
        max_completion_tokens: Option<i32>,
        n: i32,
        modalities: Vec<String>,
        presence_penalty: i32,
    ) -> PyResult<Self> {
        // if metadata is provided, validate it
        if let Some(metadata) = &metadata {
            OpenAIPrompt::validate_metadata(metadata)?;
        }

        Ok(OpenAIPrompt {
            messages,
            temperature,
            model: model.to_string(),
            store,
            reasoning_effort: reasoning_effort.to_string(),
            metadata,
            frequency_penalty,
            logit_bias,
            top_logprobs,
            max_completion_tokens,
            n,
            modalities,
            presence_penalty,
        })
    }

    #[pyo3(signature = (role, content, name=None))]
    pub fn add_message(&mut self, role: String, content: Vec<Content>, name: Option<String>) {
        self.messages.push(Message {
            role,
            content,
            name,
        });
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
