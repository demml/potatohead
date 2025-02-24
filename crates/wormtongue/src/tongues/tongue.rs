use crate::client::http::LLMClient;
use crate::client::{OpenAIClient, OpenAIConfig, RequestType, TongueClient};
use crate::error::WormTongueError;
use crate::tongues::prompts::chat::ChatPrompt;
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;

use super::responses::openai::chat::CompletionResponse;

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

    pub fn speak(&self, py: Python, request: ChatPrompt) -> PyResult<PyObject> {
        match &self.client {
            TongueClient::OpenAI(client) => {
                // build the body of the request

                let response = client
                    .request_with_retry(RequestType::Post, Some(request.to_open_ai_spec()), None)
                    .map_err(|e| {
                        WormTongueError::new_err(format!("Failed to make request: {}", e))
                    })?;

                let key = response
                    .json::<CompletionResponse>()
                    .map_err(|e| WormTongueError::new_err(e.to_string()))?;

                Ok(key.into_py_any(py)?)
            }
        }
    }
}
