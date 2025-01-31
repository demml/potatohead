use crate::client::HTTPConfig;
use crate::common::InteractionType;
use crate::tongues::base::Tongue;
use crate::tongues::openai::types::{resolve_api_key, resolve_url};
use crate::tongues::openai::{OpenAIClient, OpenAIPrompt};
use pyo3::prelude::*;
use reqwest::blocking::Client as BlockingClient;

#[pyclass(extends=Tongue, subclass)]
#[derive(Clone)]
pub struct OpenAI {
    pub client: OpenAIClient,

    #[pyo3(get)]
    pub prompt: OpenAIPrompt,

    #[pyo3(get)]
    pub interaction_type: InteractionType,
}

#[pymethods]
impl OpenAI {
    #[new]
    #[pyo3(signature = (url=None, api_key=None, prompt=None, interaction_type=None))]
    pub fn py_new(
        url: Option<String>,
        api_key: Option<&str>,
        prompt: Option<OpenAIPrompt>,
        interaction_type: Option<InteractionType>,
    ) -> PyResult<(Self, Tongue)> {
        let interaction_type = interaction_type.unwrap_or_default();
        let url = resolve_url(url, &interaction_type)?;
        let api_key = resolve_api_key(&url, api_key)?;
        let config = HTTPConfig::new(url, api_key);
        let client = OpenAIClient::new(config, None)?;
        let prompt = prompt.unwrap_or_default();

        Ok((
            Self {
                client,
                prompt,
                interaction_type,
            },
            Tongue {},
        ))
    }
}

// Implementing a rust-specific constructor that will allow us to pass a client
// Between different interfaces.
impl OpenAI {
    pub fn new(
        url: Option<String>,
        api_key: Option<&str>,
        prompt: Option<OpenAIPrompt>,
        interaction_type: Option<InteractionType>,
        client: Option<BlockingClient>,
    ) -> PyResult<(Self, Tongue)> {
        let interaction_type = interaction_type.unwrap_or_default();
        let url = resolve_url(url, &interaction_type)?;
        let api_key = resolve_api_key(&url, api_key)?;
        let config = HTTPConfig::new(url, api_key);
        let client = OpenAIClient::new(config, client)?;
        let prompt = prompt.unwrap_or_default();

        Ok((
            Self {
                client,
                prompt,
                interaction_type,
            },
            Tongue {},
        ))
    }
}
