pub mod http;
pub mod openai;
pub mod types;
use crate::client::openai::resolve_route;
use crate::common::PromptType;
use crate::error::TongueError;
pub use http::{AuthStrategy, BaseHTTPClient, HTTPConfig, LLMClient};
pub use openai::{OpenAIClient, OpenAIConfig};
pub use types::{ClientURL, RequestType};

#[derive(Debug)]
pub enum ApiClient {
    OpenAI(OpenAIClient),
}

impl ApiClient {
    pub fn resolve_route(&self, prompt_type: &PromptType) -> Result<String, TongueError> {
        match self {
            ApiClient::OpenAI(client) => resolve_route(client.url(), prompt_type),
        }
    }
}
