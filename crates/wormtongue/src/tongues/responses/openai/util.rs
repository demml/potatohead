use crate::error::WormTongueError;
use crate::tongues::responses::openai::chat::ChatCompletion;
use crate::tongues::responses::openai::structured::{parse_chat_completion, ParsedChatCompletion};
use pyo3::{prelude::*, IntoPyObjectExt};
use reqwest::blocking::Response;

pub fn parse_openai_response<'py>(
    py: Python<'py>,
    response: Response,
    response_format: Option<&Bound<'py, PyAny>>,
) -> PyResult<Bound<'py, PyAny>> {
    let chat = response
        .json::<ChatCompletion>()
        .map_err(|e| WormTongueError::new_err(e.to_string()))?;

    let response = match response_format {
        Some(format) => {
            let parse = parse_chat_completion(chat, format)?;
            parse.into_bound_py_any(py)?
        }
        None => chat.into_bound_py_any(py)?,
    };
}
