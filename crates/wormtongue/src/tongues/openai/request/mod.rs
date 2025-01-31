pub mod chat;

pub use chat::*;

use crate::error::WormTongueError;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum OpenAIRequest {
    Chat(CreateChatCompletionRequest),
}

#[pymethods]
impl OpenAIRequest {
    #[new]
    pub fn py_new(request: Bound<'_, PyAny>) -> PyResult<Self> {
        if request.is_instance_of::<CreateChatCompletionRequest>() {
            let extracted = request
                .extract::<CreateChatCompletionRequest>()
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
}

impl Default for OpenAIRequest {
    fn default() -> Self {
        OpenAIRequest::Chat(CreateChatCompletionRequest::default())
    }
}
