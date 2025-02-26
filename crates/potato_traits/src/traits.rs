use futures_core::Stream;
use futures_util::StreamExt;
use potato_client::{AsyncLLMClient, LLMClient};
use potato_error::{PotatoError, PotatoHeadError};
use potato_prompts::ChatPrompt;
use pyo3::prelude::*;
use std::pin::Pin;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[pyclass]
pub struct StreamResponse {
    stream: Pin<Box<dyn Stream<Item = Result<Vec<u8>, PotatoError>> + Send + Sync + 'static>>,
    rt: Arc<Runtime>,
}

impl StreamResponse {
    pub fn new(
        stream: impl Stream<Item = Result<Vec<u8>, PotatoError>> + Send + Sync + 'static,
        rt: Arc<Runtime>,
    ) -> Self {
        StreamResponse {
            stream: Box::pin(stream),
            rt,
        }
    }
}

#[pymethods]
impl StreamResponse {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<Option<Vec<u8>>> {
        let rt = slf.rt.clone();

        rt.block_on(async {
            match slf.stream.next().await {
                Some(Ok(bytes)) => Ok(Some(bytes)),
                Some(Err(_)) => Err(PotatoHeadError::new_err(format!("Stream error"))),
                None => Ok(None),
            }
        })
    }
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
        rt: Arc<Runtime>,
    ) -> PyResult<Bound<'py, PyAny>>
    where
        T: AsyncLLMClient + LLMClient;
}
