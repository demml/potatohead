use potato_error::PotatoHeadError;
use potato_prompts::ChatPrompt;
use potato_providers::openai::{OpenAIClient, OpenAIConfig};
use potato_traits::{ApiClient, ApiHelper};
use pyo3::prelude::*;
use tracing::{error, instrument};

#[pyclass]
#[derive(Debug)]
pub struct Mouth {
    client: ApiClient,
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
            return Ok(Self { client: client });
        }

        Err(PotatoHeadError::new_err("Invalid config type"))
    }

    #[pyo3(signature = (request,  response_format=None))]
    #[instrument(skip_all)]
    pub fn speak<'py>(
        &self,
        py: Python<'py>,
        request: ChatPrompt,
        response_format: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<()> {
        let helper = self.client.get_helper();
        let client = self.client.get_client();

        helper
            .execute_chat_request(py, client, request, response_format)
            .map_err(|e| {
                error!("Failed to make request: {}", e);
                PotatoHeadError::new_err(format!("Failed to make request: {}", e))
            })
    }
}
