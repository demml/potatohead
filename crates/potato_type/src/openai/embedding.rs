use crate::TypeError;
use pyo3::types::PyList;
use pyo3::{prelude::*, IntoPyObjectExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[pyclass]
pub struct OpenAIEmbeddingConfig {
    #[pyo3(get)]
    pub model: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub dimensions: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub encoding_format: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub user: Option<String>,
}

#[pymethods]
impl OpenAIEmbeddingConfig {
    #[new]
    #[pyo3(signature = (model, dimensions=None, encoding_format=None, user=None))]
    fn new(
        model: String,
        dimensions: Option<i32>,
        encoding_format: Option<String>,
        user: Option<String>,
    ) -> Self {
        OpenAIEmbeddingConfig {
            model,
            dimensions,
            encoding_format,
            user,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[pyclass]
pub struct EmbeddingObject {
    #[pyo3(get)]
    pub object: String,
    pub embedding: Vec<f32>,
    #[pyo3(get)]
    pub index: u32,
}

#[pymethods]
impl EmbeddingObject {
    #[getter]
    pub fn embedding(&self) -> &Vec<f32> {
        &self.embedding
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[pyclass]
pub struct UsageObject {
    #[pyo3(get)]
    pub prompt_tokens: u32,
    #[pyo3(get)]
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[pyclass]
pub struct OpenAIEmbeddingResponse {
    #[pyo3(get)]
    pub object: String,
    #[pyo3(get)]
    pub data: Vec<EmbeddingObject>,
    #[pyo3(get)]
    #[serde(default)]
    pub usage: UsageObject,
}

impl OpenAIEmbeddingResponse {
    pub fn into_py_bound_any<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        let bound = Py::new(py, self.clone())?;
        Ok(bound.into_bound_py_any(py)?)
    }
}
