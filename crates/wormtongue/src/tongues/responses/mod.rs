pub mod openai;

use crate::tongues::responses::openai::chat::CompletionResponse;
use pyo3::prelude::*;

#[pyclass]
pub enum Response {
    OpenAIChat(CompletionResponse),
}
