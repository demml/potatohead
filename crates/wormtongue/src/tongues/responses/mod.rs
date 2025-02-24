pub mod openai;

use crate::tongues::responses::openai::chat::ChatCompletion;
use crate::tongues::responses::openai::structured::ParsedChatCompletion;
use pyo3::prelude::*;

#[pyclass]
pub enum Response {
    OpenAIChat(ChatCompletion),
}
