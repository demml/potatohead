use crate::tongues::responses::openai::pricing::OpenAIApiPricing;
use crate::tongues::responses::openai::types::TokenCount;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CompletionResponse {
    #[serde(default)]
    pub id: String,

    #[serde(default)]
    pub object: String,

    #[serde(default)]
    pub created: u64,

    #[serde(default)]
    pub model: String,

    #[serde(default)]
    pub choices: Vec<Choice>,

    #[serde(default)]
    pub usage: Usage,

    #[serde(default)]
    pub system_fingerprint: String,
}

#[pymethods]
impl CompletionResponse {
    #[getter]
    pub fn calculate_cost(&self) -> PyResult<f64> {
        let pricing = OpenAIApiPricing::from_model(&self.model);
        let tokens = self.usage.to_token_count();
        Ok(pricing.calculate_cost(&tokens))
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Choice {
    #[serde(default)]
    pub index: u32,

    #[serde(default)]
    pub message: Message,

    #[serde(default)]
    pub logprobs: Option<serde_json::Value>,

    #[serde(default)]
    pub finish_reason: String,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Message {
    #[serde(default)]
    pub role: String,

    #[serde(default)]
    pub content: String,

    #[serde(default)]
    pub refusal: Option<String>,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Usage {
    #[serde(default)]
    pub prompt_tokens: u32,

    #[serde(default)]
    pub completion_tokens: u32,

    #[serde(default)]
    pub total_tokens: u32,

    #[serde(default)]
    pub prompt_tokens_details: TokenDetails,

    #[serde(default)]
    pub completion_tokens_details: CompletionTokenDetails,
}

impl Usage {
    fn to_token_count(&self) -> TokenCount {
        TokenCount {
            input_tokens: self.prompt_tokens,
            cached_tokens: self.prompt_tokens_details.cached_tokens,
            output_tokens: self.completion_tokens,
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TokenDetails {
    #[serde(default)]
    pub cached_tokens: u32,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CompletionTokenDetails {
    #[serde(default)]
    pub reasoning_tokens: u32,

    #[serde(default)]
    pub accepted_prediction_tokens: u32,

    #[serde(default)]
    pub rejected_prediction_tokens: u32,
}
