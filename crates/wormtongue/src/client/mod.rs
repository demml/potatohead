pub mod http;
pub mod openai;
pub mod types;
pub use http::{AuthStrategy, BaseHTTPClient, HTTPConfig, LLMClient};
pub use openai::{OpenAIClient, OpenAIConfig};
pub use types::{ClientURL, RequestType};

#[derive(Debug)]
pub enum TongueClient {
    OpenAI(OpenAIClient),
}
