use crate::error::WormTongueError;
use crate::tongues::responses::openai::chat::ChatCompletion;
use crate::tongues::responses::openai::structured::{parse_chat_completion, ParsedChatCompletion};
use pyo3::{prelude::*, IntoPyObjectExt};
use reqwest::blocking::Response;

/// Parse an OpenAI response into either a structured format or raw ChatCompletion
///
/// # Arguments
/// * `py` - Python interpreter token
/// * `response` - Raw HTTP response from OpenAI
/// * `response_format` - Optional format specification for structured parsing
///
/// # Returns
/// * `PyResult<Bound<'py, PyAny>>` - Parsed response as a Python object
pub fn parse_openai_response<'py>(
    py: Python<'py>,
    response: Response,
    response_format: Option<&Bound<'py, PyAny>>,
) -> PyResult<Bound<'py, PyAny>> {
    let chat: ChatCompletion = response
        .json()
        .map_err(|e| WormTongueError::new_err(format!("Failed to parse ChatCompletion: {}", e)))?;

    response_format
        .map(|format| parse_chat_completion(py, &chat, format))
        .unwrap_or_else(|| chat.into_bound_py_any(py))
}
