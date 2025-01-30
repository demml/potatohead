use crate::client::{ClientURL, HTTPConfig};
use crate::common::InteractionType;
use crate::error::TongueError;
use crate::tongues::base::Tongue;
use crate::tongues::openai::{OpenAIClient, OpenAIPrompt};
use pyo3::prelude::*;
use reqwest::blocking::Client as BlockingClient;
use std::env;

fn resolve_url(
    url: Option<String>,
    interaction_type: &InteractionType,
) -> Result<String, TongueError> {
    let url = url
        .or_else(|| env::var("WORMTONGUE_URL").ok())
        .unwrap_or_else(|| ClientURL::OpenAI.as_str().to_string());

    match interaction_type {
        InteractionType::Text => Ok(format!("{}/v1/chat/completions", url)),
        InteractionType::Image => Ok(format!("{}/v1/images/generations", url)),
        InteractionType::Voice => Ok(format!("{}/v1/audio/speech", url)),
        InteractionType::Batch => Ok(format!("{}/v1/batches", url)),
        InteractionType::Embedding => Ok(format!("{}/v1/embeddings", url)),
        _ => Err(TongueError::UnsupportedInteractionType),
    }
}

fn resolve_api_key(url: &str, api_key: Option<&str>) -> Result<String, TongueError> {
    let api_key = api_key
        .map(|s| s.to_string())
        .or_else(|| env::var("WORMTONGUE_API_KEY").ok());

    // if url contains ClientURL::OpenAI.as_str() and api_key is None, return error
    if url.contains(ClientURL::OpenAI.as_str()) && api_key.is_none() {
        return Err(TongueError::MissingAPIKey);
    }

    Ok(api_key.unwrap())
}

#[pyclass(extends=Tongue, subclass)]
#[derive(Debug)]
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

    #[setter]
    pub fn set_prompt(&mut self, prompt: OpenAIPrompt) {
        self.prompt = prompt;
    }
}

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
