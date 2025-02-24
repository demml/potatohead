use crate::error::WormTongueError;
use crate::tongues::responses::openai::chat::ChatCompletion;
use pyo3::prelude::*;

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
    parsed: PyObject,
}

#[pyclass]
pub struct ParsedChoice {
    pub message: ParsedChatCompletionMessage,
}

#[pyclass(extends=ChatCompletion, subclass)]
pub struct ParsedChatCompletion {
    pub choices: Vec<ParsedChoice>,
}

pub fn parse_chat_completion(
    py: Python,
    chat: ChatCompletion,
    response_format: PyObject,
) -> PyResult<ParsedChatCompletion> {
    chat.choices
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
                    .bind(py)
                    .call_method1("model_validate_json", (choice.message.clone(),))?;

                Ok(ParsedChoice {
                    message: ParsedChatCompletionMessage {
                        parsed: structured_object.unbind(),
                    },
                })
            }
        })
        .collect::<PyResult<Vec<ParsedChoice>>>()
        .map(|choices| ParsedChatCompletion { choices })
}
