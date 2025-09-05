use crate::TypeError;
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[pyclass(eq, eq_int)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EmbeddingTaskType {
    TaskTypeUnspecified,
    RetrievalQuery,
    RetrievalDocument,
    SemanticSimilarity,
    Classification,
    Clustering,
    QuestionAnswering,
    FactVerification,
    CodeRetrievalQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[pyclass]
pub struct GeminiEmbeddingConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_dimensionality: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<EmbeddingTaskType>,
}

#[pymethods]
impl GeminiEmbeddingConfig {
    #[new]
    #[pyo3(signature = (model=None, output_dimensionality=None, task_type=None))]
    pub fn new(
        model: Option<String>,
        output_dimensionality: Option<i32>,
        task_type: Option<EmbeddingTaskType>,
    ) -> Result<Self, TypeError> {
        if model.is_none() && task_type.is_none() {
            return Err(TypeError::GeminiEmbeddingConfigError(
                "Either 'model' or 'task_type' must be provided.".to_string(),
            ));
        }
        Ok(Self {
            model,
            output_dimensionality,
            task_type,
        })
    }
}

pub trait EmbeddingConfigTrait {
    fn get_model(&self) -> &str;
}

impl EmbeddingConfigTrait for GeminiEmbeddingConfig {
    fn get_model(&self) -> &str {
        self.model.as_deref().unwrap_or("embedding-001")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[pyclass]
pub struct ContentEmbedding {
    pub values: Vec<f32>,
}

#[pymethods]
impl ContentEmbedding {
    #[getter]
    pub fn values(&self) -> &Vec<f32> {
        &self.values
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[pyclass]
pub struct GeminiEmbeddingResponse {
    #[pyo3(get)]
    pub embedding: ContentEmbedding,
}

impl GeminiEmbeddingResponse {
    pub fn into_py_bound_any<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        let bound = Py::new(py, self.clone())?;
        Ok(bound.into_bound_py_any(py)?)
    }
}
