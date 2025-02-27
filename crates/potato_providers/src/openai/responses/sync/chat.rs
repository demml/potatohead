use crate::openai::responses::pricing::OpenAIApiPricing;
use crate::openai::responses::types::TokenCount;
use potato_tools::Utils;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChatCompletionAudio {
    #[pyo3(get)]
    pub id: String,

    #[pyo3(get)]
    pub data: String,

    #[pyo3(get)]
    pub expires_at: i64,

    #[pyo3(get)]
    pub transcript: String,
}
impl ChatCompletionAudio {
    pub fn __str__(&self) -> String {
        Utils::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FunctionCall {
    #[pyo3(get)]
    pub arguments: String,
    #[pyo3(get)]
    pub name: String,
}

#[pymethods]
impl FunctionCall {
    pub fn __str__(&self) -> String {
        Utils::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Function {
    #[pyo3(get)]
    pub arguments: String,
    #[pyo3(get)]
    pub name: String,
}

#[pymethods]
impl Function {
    pub fn __str__(&self) -> String {
        Utils::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChatCompletionMessageToolCall {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub function: Function,
    #[pyo3(get)]
    pub r#type: String,
}

#[pymethods]
impl ChatCompletionMessageToolCall {
    pub fn __str__(&self) -> String {
        Utils::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChatCompletionMessage {
    #[pyo3(get)]
    pub content: Option<String>,
    #[pyo3(get)]
    pub refusal: Option<String>,
    #[pyo3(get)]
    pub role: String,
    #[pyo3(get)]
    pub audio: Option<ChatCompletionAudio>,
    #[pyo3(get)]
    pub function: Option<FunctionCall>,
    #[pyo3(get)]
    pub tool_calls: Option<Vec<FunctionCall>>,
}

#[pymethods]
impl ChatCompletionMessage {
    pub fn __str__(&self) -> String {
        Utils::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TopLogProb {
    #[pyo3(get)]
    pub token: String,

    #[pyo3(get)]
    pub bytes: Option<Vec<u8>>,

    #[pyo3(get)]
    pub logprob: f64,
}

#[pymethods]
impl TopLogProb {
    pub fn __str__(&self) -> String {
        Utils::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChatCompletionTokenLogprob {
    #[pyo3(get)]
    pub token: String,

    #[pyo3(get)]
    pub bytes: Option<Vec<u8>>,

    #[pyo3(get)]
    pub logprob: f64,

    #[pyo3(get)]
    pub top_logprobs: Vec<TopLogProb>,
}

#[pymethods]
impl ChatCompletionTokenLogprob {
    pub fn __str__(&self) -> String {
        Utils::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChoiceLogprobs {
    #[pyo3(get)]
    pub content: Option<Vec<ChatCompletionTokenLogprob>>,
    #[pyo3(get)]
    pub refusal: Option<Vec<ChatCompletionTokenLogprob>>,
}

impl ChoiceLogprobs {
    pub fn __str__(&self) -> String {
        Utils::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Choice {
    #[pyo3(get)]
    pub finish_reason: String,
    #[pyo3(get)]
    pub index: i32,
    #[pyo3(get)]
    pub logprobs: Option<ChoiceLogprobs>,
    #[pyo3(get)]
    pub message: ChatCompletionMessage,
}

#[pymethods]
impl Choice {
    pub fn __str__(&self) -> String {
        Utils::__str__(self)
    }
}

#[pyclass(subclass)]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChatCompletion {
    #[pyo3(get)]
    pub id: String,

    #[pyo3(get)]
    pub choices: Vec<Choice>,

    #[pyo3(get)]
    pub created: i64,

    #[pyo3(get)]
    pub model: String,

    #[pyo3(get)]
    pub object: String,

    #[pyo3(get)]
    pub service_tier: Option<String>,

    #[pyo3(get)]
    pub system_fingerprint: String,

    #[pyo3(get)]
    pub usage: CompletionUsage,
}

#[pymethods]
impl ChatCompletion {
    #[getter]
    pub fn calculate_cost(&self) -> PyResult<f64> {
        let pricing = OpenAIApiPricing::from_model(&self.model);
        let tokens = self.usage.to_token_count();
        Ok(pricing.calculate_cost(&tokens))
    }

    pub fn __str__(&self) -> String {
        Utils::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CompletionUsage {
    #[pyo3(get)]
    pub completion_tokens: i64,

    #[pyo3(get)]
    pub prompt_tokens: i64,

    #[pyo3(get)]
    pub total_tokens: i64,

    #[pyo3(get)]
    pub completion_tokens_details: Option<CompletionTokensDetails>,

    #[pyo3(get)]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
}

#[pymethods]
impl CompletionUsage {
    pub fn __str__(&self) -> String {
        Utils::__str__(self)
    }
}

impl CompletionUsage {
    fn to_token_count(&self) -> TokenCount {
        let cached_tokens = self
            .prompt_tokens_details
            .as_ref()
            .and_then(|details| details.cached_tokens)
            .unwrap_or(0);

        TokenCount {
            input_tokens: self.prompt_tokens,
            cached_tokens,
            output_tokens: self.completion_tokens,
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PromptTokensDetails {
    #[pyo3(get)]
    pub audio_tokens: Option<i64>,

    #[pyo3(get)]
    pub cached_tokens: Option<i64>,
}

#[pymethods]
impl PromptTokensDetails {
    pub fn __str__(&self) -> String {
        Utils::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CompletionTokensDetails {
    #[pyo3(get)]
    pub accepted_prediction_tokens: Option<i64>,

    #[pyo3(get)]
    pub audio_tokens: Option<i64>,

    #[pyo3(get)]
    pub reasoning_tokens: Option<i64>,

    #[pyo3(get)]
    pub rejected_prediction_tokens: Option<i64>,
}

#[pymethods]
impl CompletionTokensDetails {
    pub fn __str__(&self) -> String {
        Utils::__str__(self)
    }
}
