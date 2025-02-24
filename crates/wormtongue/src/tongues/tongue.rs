use crate::client::http::LLMClient;
use crate::client::{OpenAIClient, OpenAIConfig, RequestType, TongueClient};
use crate::error::WormTongueError;
use crate::tongues::prompts::chat::ChatPrompt;
use crate::tongues::responses::openai::parse_openai_response;
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;

#[pyclass]
#[derive(Debug)]
pub struct Tongue {
    client: TongueClient,
}

#[pymethods]
impl Tongue {
    #[new]
    #[pyo3(signature = (config))]
    pub fn new(config: &Bound<'_, PyAny>) -> PyResult<Self> {
        // if config is subclass of OpenAIConfig then create OpenAIClient
        if config.is_instance_of::<OpenAIConfig>() {
            let config = config.extract::<OpenAIConfig>()?;
            let client = OpenAIClient::new(config)?;
            let tongue_client = TongueClient::OpenAI(client);
            return Ok(Self {
                client: tongue_client,
            });
        }

        Err(WormTongueError::new_err("Invalid config type"))
    }

    #[pyo3(signature = (request, response_format=None))]
    pub fn speak<'py>(
        &self,
        py: Python<'py>,
        request: ChatPrompt,
        response_format: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        match &self.client {
            TongueClient::OpenAI(client) => {
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
                        WormTongueError::new_err(format!("Failed to make request: {}", e))
                    })?;

                let parsed = parse_openai_response(py, response, response_format)?;

                Ok(parsed)
            }
        }
    }
}
