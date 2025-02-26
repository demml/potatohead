use potato_error::PotatoHeadError;
use potato_prompts::ChatPrompt;
use potato_providers::openai::{OpenAIClient, OpenAIConfig};
use potato_traits::{ApiClient, ApiHelper, StreamResponse};
use pyo3::prelude::*;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[pyclass]
#[derive(Debug)]
pub struct Mouth {
    client: ApiClient,
    rt: Arc<Runtime>,
}

#[pymethods]
impl Mouth {
    #[new]
    #[pyo3(signature = (config))]
    pub fn new(config: &Bound<'_, PyAny>) -> PyResult<Self> {
        // if config is subclass of OpenAIConfig then create OpenAIClient
        if config.is_instance_of::<OpenAIConfig>() {
            let config = config.extract::<OpenAIConfig>()?;
            let client = OpenAIClient::new(config)?;
            let client = ApiClient::OpenAI(client);

            let rt = Arc::new(Runtime::new().unwrap());

            return Ok(Self { client, rt });
        }

        Err(PotatoHeadError::new_err("Invalid config type"))
    }

    #[pyo3(signature = (request,  response_format=None))]
    pub fn speak<'py>(
        &self,
        py: Python<'py>,
        request: ChatPrompt,
        response_format: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let helper = self.client.get_helper();
        let client = self.client.get_client();
        helper.execute_chat_request(py, client, request, response_format)
    }

    #[pyo3(signature = (request))]
    pub fn stream_speak<'py>(&self, request: ChatPrompt) -> PyResult<StreamResponse> {
        let helper = self.client.get_helper();
        let client = self.client.get_client();
        let response = helper.execute_stream_chat_request(client, request, self.rt.clone())?;
        Ok(response)
    }
}
