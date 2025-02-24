pub mod openai;

use crate::tongues::responses::openai::chat::ChatCompletion
use pyo3::prelude::*;

#[pyclass]
pub enum Response {
    OpenAIChat(CompletionResponse),
}

#[pymethods]
impl Response {
    #[getter]
    pub fn response(&self) -> Option<&CompletionResponse> {
        match self {
            Response::OpenAIChat(chat) => Some(chat),
        }
    }
}
