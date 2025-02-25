use crate::mouth::responses::openai::pricing::OpenAIApiPricing;
use crate::mouth::responses::openai::types::TokenCount;
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

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FunctionCall {
    #[pyo3(get)]
    pub arguments: String,
    #[pyo3(get)]
    pub name: String,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Function {
    #[pyo3(get)]
    pub arguments: String,
    #[pyo3(get)]
    pub name: String,
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

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChoiceLogprobs {
    #[pyo3(get)]
    pub content: Option<Vec<ChatCompletionTokenLogprob>>,
    #[pyo3(get)]
    pub refulsal: Option<Vec<ChatCompletionTokenLogprob>>,
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
