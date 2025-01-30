use crate::client::{ClientURL, HTTPConfig};
use crate::error::TongueError;
use crate::tongues::base::Tongue;
use crate::tongues::openai::{Message, OpenAIClient, OpenAIPrompt};
use pyo3::prelude::*;
use std::env;

fn resolve_url(url: Option<String>) -> String {
    url.or_else(|| env::var("WORMTONGUE_URL").ok())
        .unwrap_or_else(|| ClientURL::OpenAI.as_str().to_string())
}

fn resolve_api_key(api_key: Option<&str>) -> Result<String, TongueError> {
    api_key
        .map(|s| s.to_string())
        .or_else(|| env::var("WORMTONGUE_API_KEY").ok())
        .ok_or(TongueError::MissingAPIKey)
}

#[pyclass(extends=Tongue, subclass)]
#[derive(Debug)]
pub struct OpenAI {
    pub client: OpenAIClient,

    #[pyo3(get)]
    pub prompt: OpenAIPrompt,
}

#[pymethods]
impl OpenAI {
    #[new]
    #[pyo3(signature = (url=None, api_key=None, prompt=None))]
    pub fn new(
        url: Option<String>,
        api_key: Option<&str>,
        prompt: Option<OpenAIPrompt>,
    ) -> PyResult<(Self, Tongue)> {
        let url = resolve_url(url);
        let api_key = resolve_api_key(api_key)?;
        let config = HTTPConfig::new(url, api_key);

        let client = OpenAIClient::new(config)?;

        let prompt = prompt.unwrap_or_default();

        Ok((Self { client, prompt }, Tongue {}))
    }

    #[setter]
    pub fn set_prompt(&mut self, prompt: OpenAIPrompt) {
        self.prompt = prompt;
    }

    /// Instantiate a new OpenAIPrompt. This will override the provided default prompt.
    ///
    /// # Arguments
    ///
    /// * `temperature` - The temperature of the model. Must be a float between 0 and 1.
    /// * `model` - The model to use. Must be one of the models from OpenAIModels.
    /// * `messages` - A list of messages to use as the prompt.
    pub fn setup_prompt(
        &mut self,
        temperature: f32,
        model: String,
        messages: Vec<Message>,
    ) -> PyResult<()> {
        self.prompt = OpenAIPrompt::new(model.as_str(), temperature, messages);
        Ok(())
    }
}
