pub mod chat;

use crate::error::{TongueError, WormTongueError};
pub use chat::*;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum OpenAIRequest {
    Chat(ChatCompletionRequest),
}

#[pymethods]
impl OpenAIRequest {
    #[new]
    pub fn py_new(request: &Bound<'_, PyAny>) -> PyResult<Self> {
        if request.is_instance_of::<ChatCompletionRequest>() {
            let extracted = request
                .extract::<ChatCompletionRequest>()
                .map_err(|e| WormTongueError::new_err(e))?;
            return Ok(OpenAIRequest::Chat(extracted));
        } else {
            return Err(WormTongueError::new_err("Invalid request type"));
        }
    }
}

impl OpenAIRequest {
    pub fn add_message(&mut self, message: Message) {
        match self {
            OpenAIRequest::Chat(chat) => chat.messages.push(message),
        }
    }

    pub fn to_json(&self) -> Result<Value, TongueError> {
        match self {
            OpenAIRequest::Chat(chat) => {
                let val =
                    serde_json::to_value(chat).map_err(|e| TongueError::Error(e.to_string()))?;
                Ok(val)
            }
        }
    }
}

impl Default for OpenAIRequest {
    fn default() -> Self {
        OpenAIRequest::Chat(ChatCompletionRequest::default())
    }
}
