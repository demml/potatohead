use crate::client::{ClientURL, HTTPConfig};
use crate::tongues::base::Tongue;
use crate::tongues::openai::OpenAIClient;
use pyo3::prelude::*;
use std::env;

fn resolve_url(url: Option<String>) -> String {
    url.or_else(|| env::var("WORMTONGUE_URL").ok())
        .unwrap_or_else(|| ClientURL::OpenAI.as_str().to_string())
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
    pub fn new(url: Option<String>, api_key: Option<&str>) -> Self {
        let url = resolve_url(url);
        let api_key = api_key.unwrap_or("REDACTED");

        let mut config = HTTPConfig::new(url, api_key);

        let client = HTTPClient::new(config);
        OpenAI { client }
    }
}
