use potato_client::LLMClient;
use potato_prompts::ChatPrompt;
use pyo3::prelude::*;

pub trait ApiHelper {
    fn new() -> Self;

    fn execute_chat_request<'py, T>(
        &self,
        py: Python<'py>,
        client: &T,
        request: ChatPrompt,
        response_format: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>>
    where
        T: LLMClient;
}
