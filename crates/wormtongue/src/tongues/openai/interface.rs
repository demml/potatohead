use crate::client::http::LLMClient;
use crate::client::{HTTPConfig, RequestType};
use crate::error::TongueError;
use crate::tongues::base::Tongue;
use crate::tongues::openai::types::{resolve_api_key, resolve_url};
use crate::tongues::openai::{OpenAIClient, OpenAIPrompt};
use pyo3::prelude::*;
use reqwest::blocking::Client as BlockingClient;

#[pyclass(extends=Tongue, subclass)]
#[derive(Clone)]
pub struct OpenAIInterface {
    pub client: OpenAIClient,

    #[pyo3(get)]
    pub prompt: OpenAIPrompt,
}

#[pymethods]
impl OpenAIInterface {
    #[new]
    #[pyo3(signature = (prompt, url=None, api_key=None))]
    pub fn py_new(
        prompt: OpenAIPrompt,
        url: Option<String>,
        api_key: Option<&str>,
    ) -> PyResult<(Self, Tongue)> {
        let url = resolve_url(url, &prompt.prompt_type)?;
        let api_key = resolve_api_key(&url, api_key)?;
        let config = HTTPConfig::new(url, api_key);
        let client = OpenAIClient::new(config, None)?;

        Ok((Self { client, prompt }, Tongue {}))
    }

    pub fn send(&self) -> PyResult<()> {
        let body = self.prompt.request.to_json()?;
        self.client
            .request_with_retry(RequestType::Post, Some(body), None, None)
            .unwrap();
        Ok(())
    }
}

// Implementing a rust-specific constructor that will allow us to pass a client
// Between different interfaces.
impl OpenAIInterface {
    pub fn new(
        prompt: OpenAIPrompt,
        url: Option<String>,
        api_key: Option<&str>,
        client: Option<BlockingClient>,
    ) -> Result<Self, TongueError> {
        let url = resolve_url(url, &prompt.prompt_type)?;
        let api_key = resolve_api_key(&url, api_key)?;
        let config = HTTPConfig::new(url, api_key);
        let client = OpenAIClient::new(config, client)?;

        Ok(Self { client, prompt })
    }
}
