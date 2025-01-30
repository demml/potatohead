use crate::client::http::{HTTPClient, HTTPConfig};
use crate::tongues::base::Tongue;
use pyo3::prelude::*;

#[pyclass(extends=Tongue, subclass)]
#[derive(Debug)]
pub struct OpenAI {
    pub client: HTTPClient,
}

#[pymethods]
impl OpenAI {
    #[new]
    #[pyo3(signature = (url=None, api_key=None))]
    pub fn new(url: Option<&str>, api_key: Option<&str>) -> Self {
        let config = HTTPConfig {
            url: url.to_string(),
            bearer_token: api_key.to_string(),
        };

        let client = HTTPClient::new(config);
        OpenAI { client }
    }
}
