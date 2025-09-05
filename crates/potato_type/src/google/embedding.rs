use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use crate::TypeError;

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
pub struct ContentEmbedding {
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[pyclass]
pub struct GeminiEmbeddingResponse {
    pub embedding: ContentEmbedding,
}
