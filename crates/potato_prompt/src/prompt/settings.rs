use crate::PromptError;
use potato_type::anthropic::v1::message::AnthropicSettings;
use potato_type::{google::chat::GeminiSettings, openai::chat::OpenAIChatSettings};
use potato_type::{Provider, SettingsType};
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::IntoPyObjectExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum ModelSettings {
    OpenAIChat(OpenAIChatSettings),
    GoogleChat(GeminiSettings),
    AnthropicChat(AnthropicSettings),
}

impl Default for ModelSettings {
    fn default() -> Self {
        ModelSettings::OpenAIChat(OpenAIChatSettings::default())
    }
}

#[pymethods]
impl ModelSettings {
    #[new]
    pub fn new(settings: &Bound<'_, PyAny>) -> Result<Self, PromptError> {
        if settings.is_instance_of::<OpenAIChatSettings>() {
            let settings: OpenAIChatSettings = settings.extract()?;
            Ok(ModelSettings::OpenAIChat(settings))
        } else if settings.is_instance_of::<GeminiSettings>() {
            let settings: GeminiSettings = settings.extract()?;
            Ok(ModelSettings::GoogleChat(settings))
        } else if settings.is_instance_of::<AnthropicSettings>() {
            let settings: AnthropicSettings = settings.extract()?;
            Ok(ModelSettings::AnthropicChat(settings))
        } else {
            Err(PromptError::InvalidModelSettings)
        }
    }

    #[getter]
    pub fn settings<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, PromptError> {
        match self {
            ModelSettings::OpenAIChat(settings) => {
                Ok(Py::new(py, settings.clone())?.into_bound_py_any(py)?)
            }
            ModelSettings::GoogleChat(settings) => {
                Ok(Py::new(py, settings.clone())?.into_bound_py_any(py)?)
            }
            ModelSettings::AnthropicChat(settings) => {
                Ok(Py::new(py, settings.clone())?.into_bound_py_any(py)?)
            }
        }
    }

    pub fn model_dump_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn model_dump<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyDict>, PromptError> {
        match self {
            ModelSettings::OpenAIChat(settings) => Ok(settings.model_dump(py)?),
            ModelSettings::GoogleChat(settings) => Ok(settings.model_dump(py)?),
            ModelSettings::AnthropicChat(settings) => Ok(settings.model_dump(py)?),
        }
    }

    pub fn settings_type(&self) -> SettingsType {
        SettingsType::ModelSettings
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

impl ModelSettings {
    pub fn validate_provider(&self, provider: &Provider) -> Result<(), PromptError> {
        match provider {
            Provider::OpenAI => match self {
                ModelSettings::OpenAIChat(_) => Ok(()),
                _ => Err(PromptError::InvalidModelSettings),
            },
            Provider::Gemini => match self {
                ModelSettings::GoogleChat(_) => Ok(()),
                _ => Err(PromptError::InvalidModelSettings),
            },
            Provider::Vertex => match self {
                ModelSettings::GoogleChat(_) => Ok(()),
                _ => Err(PromptError::InvalidModelSettings),
            },
            Provider::Google => match self {
                ModelSettings::GoogleChat(_) => Ok(()),
                _ => Err(PromptError::InvalidModelSettings),
            },
            Provider::Anthropic => match self {
                ModelSettings::AnthropicChat(_) => Ok(()),
                _ => Err(PromptError::InvalidModelSettings),
            },
            Provider::Undefined => match self {
                ModelSettings::OpenAIChat(_) => Ok(()),
                ModelSettings::GoogleChat(_) => Ok(()),
                ModelSettings::AnthropicChat(_) => Ok(()),
            },
        }
    }

    pub fn provider_default_settings(provider: &Provider) -> Self {
        match provider {
            Provider::OpenAI => ModelSettings::OpenAIChat(OpenAIChatSettings::default()),
            Provider::Gemini => ModelSettings::GoogleChat(GeminiSettings::default()),
            _ => ModelSettings::OpenAIChat(OpenAIChatSettings::default()), // Fallback to OpenAI settings
        }
    }

    pub fn get_openai_settings(&self) -> Option<OpenAIChatSettings> {
        match self {
            ModelSettings::OpenAIChat(settings) => {
                let mut cloned_settings = settings.clone();
                // set extra body to None
                cloned_settings.extra_body = None;
                Some(cloned_settings)
            }
            _ => None,
        }
    }

    pub fn get_gemini_settings(&self) -> Option<GeminiSettings> {
        match self {
            ModelSettings::GoogleChat(settings) => {
                let mut cloned_settings = settings.clone();
                // set extra body to None
                cloned_settings.extra_body = None;
                Some(cloned_settings)
            }
            _ => None,
        }
    }

    pub fn get_anthropic_settings(&self) -> AnthropicSettings {
        match self {
            ModelSettings::AnthropicChat(settings) => {
                let mut cloned_settings = settings.clone();
                // set extra body to None
                cloned_settings.extra_body = None;
                cloned_settings
            }
            _ => AnthropicSettings::default(),
        }
    }

    pub fn extra_body(&self) -> Option<&Value> {
        match self {
            ModelSettings::OpenAIChat(settings) => settings.extra_body.as_ref(),
            ModelSettings::GoogleChat(settings) => settings.extra_body.as_ref(),
            ModelSettings::AnthropicChat(settings) => settings.extra_body.as_ref(),
        }
    }

    pub fn provider(&self) -> Provider {
        match self {
            ModelSettings::OpenAIChat(_) => Provider::OpenAI,
            ModelSettings::GoogleChat(_) => Provider::Gemini,
            ModelSettings::AnthropicChat(_) => Provider::Anthropic,
        }
    }
}
