pub mod http;
pub mod openai;
pub mod types;
use crate::client::openai::resolve_route;
use crate::error::TongueError;
use crate::tongues::common::PromptType;
pub use http::{AuthStrategy, BaseHTTPClient, HTTPConfig, LLMClient};
pub use openai::{OpenAIClient, OpenAIConfig};
pub use types::{ClientURL, RequestType};

#[derive(Debug)]
pub enum TongueClient {
    OpenAI(OpenAIClient),
}

impl TongueClient {
    pub fn resolve_route(&self, prompt_type: &PromptType) -> Result<String, TongueError> {
        match self {
            TongueClient::OpenAI(client) => resolve_route(client.url(), prompt_type),
        }
    }
}
