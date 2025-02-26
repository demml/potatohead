use potato_client::LLMClient;
use potato_prompts::ChatPrompt;
use pyo3::prelude::*;

#[pyclass]
struct StreamIter {
    inner: std::vec::IntoIter<String>,
}

#[pymethods]
impl StreamIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<String> {
        slf.inner.next()
    }
}

struct StreamResponse {
    messages: Vec<String>,
}

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

    fn execute_stream_chat_request<'py, T>(
        &self,
        py: Python<'py>,
        client: &T,
        request: ChatPrompt,
    ) -> PyResult<Bound<'py, PyAny>>
    where
        T: LLMClient;
}
