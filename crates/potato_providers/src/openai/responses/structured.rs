use crate::openai::responses::chat::{ChatCompletion, CompletionUsage};
use potato_error::PotatoError;
use pyo3::prelude::*;

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

pub fn parse_chat_completion(
    chat: &ChatCompletion,
    response_format: &Bound<'_, PyAny>,
) -> Result<ParsedChatCompletion, PotatoError> {
    // Process choices first
    let choices = chat
        .choices
        .iter()
        .map(|choice| match choice.finish_reason.as_str() {
            "length" => Err(PotatoError::Error(format!(
                "Length limit reached - {:?}",
                chat.usage
            ))),
            "content_filter" => Err(PotatoError::Error("Content filter rejection".to_string())),
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
        .collect::<Result<Vec<ParsedChoice>, PotatoError>>()?;

    // Create ParsedChatCompletion with references where possible
    Ok(ParsedChatCompletion {
        id: chat.id.clone(),
        choices,
        created: chat.created,
        model: chat.model.clone(),
        object: chat.object.clone(),
        service_tier: chat.service_tier.clone(),
        system_fingerprint: chat.system_fingerprint.clone(),
        usage: chat.usage.clone(),
    })
}
