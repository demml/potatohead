use crate::{ChoiceLogprobs, CompletionUsage};
use potato_tools::Utils;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChoiceDeltaToolCallFunction {
    arguments: Option<String>,
    name: Option<String>,
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
    arguments: Option<String>,
    name: Option<String>,
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
    index: i64,
    id: Option<String>,
    function: Option<ChoiceDeltaToolCallFunction>,
    r#type: Option<String>,
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
    content: Option<String>,
    function_call: Option<ChoiceDeltaFunctionCall>,
    refusal: Option<String>,
    role: Option<String>,
    tool_calls: Option<Vec<ChoiceDeltaToolCall>>,
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
    delta: ChoiceDelta,

    finish_reason: Option<String>,

    index: i64,

    logprobs: Option<ChoiceLogprobs>,
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
    id: String,

    choices: Vec<ChunkChoice>,

    created: i64,

    model: String,

    object: String,

    service_tier: Option<String>,

    system_fingerprint: Option<String>,

    usage: Option<CompletionUsage>,
}

#[pymethods]
impl ChatCompletionChunk {
    fn __str__(&self) -> String {
        Utils::__str__(&self)
    }
}
