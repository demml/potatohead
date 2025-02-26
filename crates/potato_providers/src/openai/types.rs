use potato_client::HTTPConfig;
use potato_client::{resolve_api_key, resolve_url};
use potato_error::PotatoError;
use potato_tools::PromptType;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub url: String,
    pub organization: Option<String>,
    pub project: Option<String>,
}

#[pymethods]
impl OpenAIConfig {
    #[new]
    #[pyo3(signature = (api_key=None, url=None, organization=None, project=None))]
    pub fn new(
        api_key: Option<&str>,
        url: Option<&str>,
        organization: Option<&str>,
        project: Option<&str>,
    ) -> PyResult<Self> {
        let url = resolve_url(url)?;
        let api_key = resolve_api_key(&url, api_key)?;

        Ok(Self {
            api_key: api_key.to_string(),
            url: url.to_string(),
            organization: organization.map(|s| s.to_string()),
            project: project.map(|s| s.to_string()),
        })
    }
}

impl OpenAIConfig {
    pub fn into_http_config(&self) -> HTTPConfig {
        HTTPConfig::new(
            self.url.clone(),
            self.api_key.clone(),
            self.organization.clone(),
            self.project.clone(),
        )
    }
}

pub fn resolve_route(url: &str, prompt_type: &PromptType) -> Result<String, PotatoError> {
    match prompt_type {
        PromptType::Chat => Ok(format!("{}/v1/chat/completions", url)),
        PromptType::Image => Ok(format!("{}/v1/images/generations", url)),
        PromptType::Voice => Ok(format!("{}/v1/audio/speech", url)),
        PromptType::Batch => Ok(format!("{}/v1/batches", url)),
        PromptType::Embedding => Ok(format!("{}/v1/embeddings", url)),
        _ => Err(PotatoError::UnsupportedInteractionType),
    }
}
