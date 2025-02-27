use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TokenCount {
    #[serde(default)]
    pub input_tokens: i64,

    #[serde(default)]
    pub cached_tokens: i64,

    #[serde(default)]
    pub output_tokens: i64,
}
