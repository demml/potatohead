use crate::tongues::openai::pricing::OpenAIApiPricing;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
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
    pub index: u32,
    pub message: Message,
    pub logprobs: Option<serde_json::Value>,
    pub finish_reason: String,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub refusal: Option<String>,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub prompt_tokens_details: TokenDetails,
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
    pub cached_tokens: u32,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CompletionTokenDetails {
    pub reasoning_tokens: u32,
    pub accepted_prediction_tokens: u32,
    pub rejected_prediction_tokens: u32,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TokenCount {
    pub input_tokens: u32,
    pub cached_tokens: u32,
    pub output_tokens: u32,
}
