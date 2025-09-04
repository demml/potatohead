use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[pyclass]
pub struct OpenAIEmbeddingSettings {
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
    pub usage: Option<UsageObject>,
}
