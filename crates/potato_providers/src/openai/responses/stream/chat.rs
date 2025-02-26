use crate::{ChoiceLogprobs, CompletionUsage};
use potato_tools::Utils;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChoiceDeltaToolCallFunction {
    #[pyo3(get)]
    pub arguments: Option<String>,
    #[pyo3(get)]
    pub name: Option<String>,
}

#[pymethods]
impl ChoiceDeltaToolCallFunction {
    fn __str__(&self) -> String {
        Utils::__str__(&self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChoiceDeltaFunctionCall {
    #[pyo3(get)]
    pub arguments: Option<String>,
    #[pyo3(get)]
    pub name: Option<String>,
}

#[pymethods]
impl ChoiceDeltaFunctionCall {
    fn __str__(&self) -> String {
        Utils::__str__(&self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChoiceDeltaToolCall {
    #[pyo3(get)]
    pub index: i64,
    #[pyo3(get)]
    pub id: Option<String>,
    #[pyo3(get)]
    pub function: Option<ChoiceDeltaToolCallFunction>,
    #[pyo3(get)]
    pub r#type: Option<String>,
}

#[pymethods]
impl ChoiceDeltaToolCall {
    fn __str__(&self) -> String {
        Utils::__str__(&self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChoiceDelta {
    #[pyo3(get)]
    pub content: Option<String>,
    #[pyo3(get)]
    pub function_call: Option<ChoiceDeltaFunctionCall>,
    #[pyo3(get)]
    pub refusal: Option<String>,
    #[pyo3(get)]
    pub role: Option<String>,
    #[pyo3(get)]
    pub tool_calls: Option<Vec<ChoiceDeltaToolCall>>,
}

#[pymethods]
impl ChoiceDelta {
    fn __str__(&self) -> String {
        Utils::__str__(&self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChunkChoice {
    #[pyo3(get)]
    pub delta: ChoiceDelta,

    #[pyo3(get)]
    pub finish_reason: Option<String>,

    #[pyo3(get)]
    pub index: i64,

    #[pyo3(get)]
    pub logprobs: Option<ChoiceLogprobs>,
}

#[pymethods]
impl ChunkChoice {
    fn __str__(&self) -> String {
        Utils::__str__(&self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChatCompletionChunk {
    #[pyo3(get)]
    pub id: String,

    #[pyo3(get)]
    pub choices: Vec<ChunkChoice>,

    #[pyo3(get)]
    pub created: i64,

    #[pyo3(get)]
    pub model: String,

    #[pyo3(get)]
    pub object: String,

    #[pyo3(get)]
    pub service_tier: Option<String>,

    #[pyo3(get)]
    pub system_fingerprint: Option<String>,

    #[pyo3(get)]
    pub usage: Option<CompletionUsage>,
}

#[pymethods]
impl ChatCompletionChunk {
    pub fn __str__(&self) -> String {
        Utils::__str__(&self)
    }
}

impl ChatCompletionChunk {
    pub fn from_sse_events(events: &[String]) -> Result<Self, serde_json::Error> {
        // Parse the first event to get our base chunk
        let mut base_chunk: Option<ChatCompletionChunk> = None;
        let mut all_choices = Vec::new();

        for event in events {
            match serde_json::from_str(&event) {
                Ok(chunk) => {
                    if base_chunk.is_none() {
                        base_chunk = Some(chunk);
                    } else {
                        all_choices.extend(chunk.choices);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse event: {}", e);
                    continue;
                }
            };
        }

        // Get our base chunk or create an empty one if parsing failed
        let mut result = base_chunk.unwrap_or_default();

        // Extend the base chunk's choices with all additional choices
        result.choices.extend(all_choices);

        Ok(result)
    }
}
