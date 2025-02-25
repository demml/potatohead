use crate::client::http::LLMClient;
use crate::client::{ApiClient, OpenAIClient, OpenAIConfig, RequestType};
use crate::error::PotatoHeadError;
use crate::mouth::prompts::chat::ChatPrompt;
use crate::mouth::responses::openai::parse_openai_response;
use pyo3::prelude::*;

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
    pub fn speak<'py>(
        &self,
        py: Python<'py>,
        request: ChatPrompt,
        response_format: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        match &self.client {
            ApiClient::OpenAI(client) => {
                // build the body of the request

                let route = self.client.resolve_route(&request.prompt_type)?;
                let response = client
                    .request_with_retry(
                        route,
                        RequestType::Post,
                        Some(request.to_open_ai_spec()),
                        None,
                        None,
                    )
                    .map_err(|e| {
                        PotatoHeadError::new_err(format!("Failed to make request: {}", e))
                    })?;

                parse_openai_response(py, response, response_format)
            }
        }
    }
}
