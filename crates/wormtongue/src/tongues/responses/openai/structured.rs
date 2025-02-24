use crate::error::WormTongueError;
use crate::tongues::responses::openai::chat::{ChatCompletion, CompletionUsage};
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;

//{
//    name = response_format.__name__
//    "type": "json_schema",
//    "json_schema": {
//        "schema": model.json_schema(),
//        "name": name,
//        "strict": True,
//    },
//

#[pyclass]
pub struct ParsedChatCompletionMessage {
    #[pyo3(get)]
    parsed: PyObject,
}

impl Clone for ParsedChatCompletionMessage {
    fn clone(&self) -> Self {
        Python::with_gil(|py| Self {
            parsed: self.parsed.clone_ref(py),
        })
    }
}

#[pyclass]
#[derive(Clone)]
pub struct ParsedChoice {
    #[pyo3(get)]
    pub message: ParsedChatCompletionMessage,
}

#[pyclass]
#[derive(Clone)]
pub struct ParsedChatCompletion {
    #[pyo3(get)]
    pub id: String,

    #[pyo3(get)]
    pub choices: Vec<ParsedChoice>,

    #[pyo3(get)]
    pub created: i64,

    #[pyo3(get)]
    pub model: String,

    #[pyo3(get)]
    pub object: String,

    #[pyo3(get)]
    pub service_tier: Option<String>,

    #[pyo3(get)]
    pub system_fingerprint: String,

    #[pyo3(get)]
    pub usage: CompletionUsage,
}

pub fn parse_chat_completion<'py>(
    py: Python<'py>,
    chat: ChatCompletion,
    response_format: &Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyAny>> {
    let parsed = chat
        .choices
        .iter()
        .map(|choice| match choice.finish_reason.as_str() {
            "length" => {
                return Err(WormTongueError::new_err(format!(
                    "Length limit reached - {:?}",
                    chat.usage
                )))
            }
            "content_filter" => return Err(WormTongueError::new_err("Content filter rejection")),
            _ => {
                let structured_object = response_format
                    .call_method1("model_validate_json", (choice.message.clone(),))?;

                Ok(ParsedChoice {
                    message: ParsedChatCompletionMessage {
                        parsed: structured_object.unbind(),
                    },
                })
            }
        })
        .collect::<PyResult<Vec<ParsedChoice>>>()
        .map(|choices| ParsedChatCompletion {
            id: chat.id,
            choices,
            created: chat.created,
            model: chat.model,
            object: chat.object,
            service_tier: chat.service_tier,
            system_fingerprint: chat.system_fingerprint,
            usage: chat.usage,
        })?;

    Ok(parsed.into_bound_py_any(py)?)
}
