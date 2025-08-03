pub mod error;

use crate::error::TypeError;
use pyo3::prelude::*;
use schemars::JsonSchema;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::type_name;
use std::fmt;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[pyclass]
pub enum Model {
    Undefined,
}

impl Model {
    pub fn as_str(&self) -> &str {
        match self {
            Model::Undefined => "undefined",
        }
    }

    pub fn from_string(s: &str) -> Result<Self, TypeError> {
        match s.to_lowercase().as_str() {
            "undefined" => Ok(Model::Undefined),
            _ => Err(TypeError::UnknownModelError(s.to_string())),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[pyclass]
pub enum Provider {
    OpenAI,
    Gemini,
    Undefined, // Added Undefined for better error handling
}

impl Provider {
    pub fn url(&self) -> &str {
        match self {
            Provider::OpenAI => "https://api.openai.com/v1",
            Provider::Gemini => "https://generativelanguage.googleapis.com/v1beta/models",
            Provider::Undefined => {
                error!("Undefined provider URL requested");
                "https://undefined.provider.url"
            }
        }
    }

    pub fn from_string(s: &str) -> Result<Self, TypeError> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(Provider::OpenAI),
            "gemini" => Ok(Provider::Gemini),
            "undefined" => Ok(Provider::Undefined), // Handle undefined case
            _ => Err(TypeError::UnknownProviderError(s.to_string())),
        }
    }

    /// Extract provider from a PyAny object
    ///
    /// # Arguments
    /// * `provider` - PyAny object
    ///
    /// # Returns
    /// * `Result<Provider, AgentError>` - Result
    ///
    /// # Errors
    /// * `AgentError` - Error
    pub fn extract_provider(provider: &Bound<'_, PyAny>) -> Result<Provider, TypeError> {
        match provider.is_instance_of::<Provider>() {
            true => Ok(provider.extract::<Provider>().inspect_err(|e| {
                error!("Failed to extract provider: {}", e);
            })?),
            false => {
                let provider = provider.extract::<String>().unwrap();
                Ok(Provider::from_string(&provider).inspect_err(|e| {
                    error!("Failed to convert string to provider: {}", e);
                })?)
            }
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Provider::OpenAI => "openai",
            Provider::Gemini => "gemini",
            Provider::Undefined => "undefined", // Added Undefined case
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Clone)]
pub enum SaveName {
    Prompt,
}

#[pymethods]
impl SaveName {
    #[staticmethod]
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "prompt" => Some(SaveName::Prompt),

            _ => None,
        }
    }

    pub fn as_string(&self) -> &str {
        match self {
            SaveName::Prompt => "prompt",
        }
    }

    pub fn __str__(&self) -> String {
        self.to_string()
    }
}

impl Display for SaveName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl AsRef<Path> for SaveName {
    fn as_ref(&self) -> &Path {
        match self {
            SaveName::Prompt => Path::new("prompt"),
        }
    }
}

// impl PathBuf: From<SaveName>
impl From<SaveName> for PathBuf {
    fn from(save_name: SaveName) -> Self {
        PathBuf::from(save_name.as_ref())
    }
}

/// A trait for structured output types that can be used with potatohead prompts agents and workflows.
///
/// # Example
/// ```rust
/// use potato_macros::StructureOutput;
/// use serde::{Serialize, Deserialize};
/// use schemars::JsonSchema;
///
/// #[derive(Serialize, Deserialize, JsonSchema)]
/// struct MyOutput {
///     message: String,
///     value: i32,
/// }
///
/// impl StructuredOutput for MyOutput {}
///
/// let schema = MyOutput::get_structured_output_schema();
/// ```
pub trait StructuredOutput: for<'de> serde::Deserialize<'de> + JsonSchema {
    fn type_name() -> &'static str {
        type_name::<Self>().rsplit("::").next().unwrap_or("Unknown")
    }

    /// Validates and deserializes a JSON value into its struct type.
    ///
    /// # Arguments
    /// * `value` - The JSON value to deserialize
    ///
    /// # Returns
    /// * `Result<Self, serde_json::Error>` - The deserialized value or error
    fn model_validate_json_value(value: &Value) -> Result<Self, serde_json::Error> {
        match &value {
            Value::String(json_str) => Self::model_validate_json_str(json_str),
            Value::Object(_) => {
                // Content is already a JSON object
                serde_json::from_value(value.clone())
            }
            _ => {
                // If the value is not a string or object, we cannot deserialize it
                Err(Error::custom("Expected a JSON string or object"))
            }
        }
    }

    fn model_validate_json_str(value: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(value)
    }

    /// Generates an OpenAI-compatible JSON schema.
    ///
    /// # Returns
    /// * `Value` - The JSON schema wrapped in OpenAI's format
    fn get_structured_output_schema() -> Value {
        let schema = ::schemars::schema_for!(Self);
        schema.into()
    }
    // add fallback parsing logic
}
