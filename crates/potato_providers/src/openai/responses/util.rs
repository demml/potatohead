use crate::openai::responses::chat::ChatCompletion;
use crate::openai::responses::structured::parse_chat_completion;
use potato_error::PotatoError;
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
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
        .map_err(|e| PotatoError::Error(e.to_string()))?;

    match response_format {
        Some(format) => {
            let parsed = parse_chat_completion(&chat, format)?;
            Ok(parsed.into_bound_py_any(py)?)
        }
        None => Ok(chat.into_bound_py_any(py)?),
    }
}
