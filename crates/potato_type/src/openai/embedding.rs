use pyo3::{prelude::*, IntoPyObjectExt};
use serde::{Deserialize, Serialize};

use crate::TypeError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[pyclass]
pub struct OpenAIEmbeddingConfig {
    pub model: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[pyclass]
pub struct EmbeddingObject {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[pyclass]
pub struct UsageObject {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[pyclass]
pub struct OpenAIEmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingObject>,

    #[serde(default)]
    pub usage: UsageObject,
}

impl OpenAIEmbeddingResponse {
    pub fn into_py_bound_any<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        let bound = Py::new(py, self.clone())?;
        Ok(bound.into_bound_py_any(py)?)
    }
}
