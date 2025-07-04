use pyo3::prelude::*;
use schemars::JsonSchema;
use serde_json::Value;
use std::any::type_name;
use std::fmt;
use std::fmt::Display;
use std::path::{Path, PathBuf};

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
    fn model_validate_json(value: &Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value.clone())
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
        serde_json::json!({
            "type": "json_schema",
            "json_schema": {
                "name": Self::type_name(),
                "schema": schema,
                "strict": true
            }
        })
    }
}
