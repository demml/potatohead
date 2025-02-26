use crate::client::LLMClient;
use crate::error::PotatoError;
use crate::potato_parts::mouth::prompts::chat::ChatPrompt;
use pyo3::prelude::*;

pub trait ApiHelper {
    fn new() -> Self;

    fn execute_chat_request<'py, T>(
        &self,
        py: Python<'py>,
        client: &T,
        request: ChatPrompt,
        response_format: Option<&Bound<'py, PyAny>>,
    ) -> Result<(), PotatoError>
    where
        T: LLMClient;
}
