pub mod openai;

use crate::tongues::responses::openai::chat::ChatCompletion
use pyo3::prelude::*;

#[pyclass]
pub enum Response {
    OpenAIChat(ChatCompletion),
}

