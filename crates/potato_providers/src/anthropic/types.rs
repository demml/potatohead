use potato_client::{resolve_api_key, resolve_url, resolve_version};
use potato_error::PotatoHeadError;
use pyo3::prelude::*;
use tracing::error;

#[pyclass]
#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub url: String,
    pub anthropic_version: String,
}

#[pymethods]
impl AnthropicConfig {
    #[new]
    #[pyo3(signature = (api_key=None, url=None, version=None))]
    pub fn new(api_key: Option<&str>, url: Option<&str>, version: Option<&str>) -> PyResult<Self> {
        let url = resolve_url(url).map_err(|e| {
            error!("Failed to resolve url: {}", e);
            PotatoHeadError::new_err(e.to_string())
        })?;

        let api_key = resolve_api_key(&url, api_key).map_err(|e| {
            error!("Failed to resolve api key: {}", e);
            PotatoHeadError::new_err(e.to_string())
        })?;

        let version = resolve_version(version)
            .unwrap_or(Some("2023-06-01".to_string()))
            .unwrap();

        Ok(Self {
            api_key: api_key.to_string(),
            url: url.to_string(),
            anthropic_version: version,
        })
    }
}
