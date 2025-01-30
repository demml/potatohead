use crate::client::{ClientURL, HTTPConfig};
use crate::error::{TongueError, WormTongueError};
use crate::tongues::base::Tongue;
use crate::tongues::openai::OpenAIClient;
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
}

#[pymethods]
impl OpenAI {
    #[new]
    #[pyo3(signature = (url=None, api_key=None))]
    pub fn new(url: Option<String>, api_key: Option<&str>) -> PyResult<(Self, Tongue)> {
        let url = resolve_url(url);
        let api_key = resolve_api_key(api_key)?;
        let config = HTTPConfig::new(url, api_key);

        let client = OpenAIClient::new(config)?;

        Ok((Self { client }, Tongue {}))
    }
}
