pub mod openai;
pub use openai::{ChatCompletion, ParsedChatCompletion};
use pyo3::{prelude::*, IntoPyObjectExt};

pub enum ChatResponse {
    OpenAIChatCompletion(ChatCompletion),
    OpenAIParsedChatCompletion(ParsedChatCompletion),
}
