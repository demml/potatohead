pub mod error;
pub mod providers;

pub use error::ProviderError;
pub use providers::{
    client::GenAiClient,
    embed::{Embedder, EmbeddingConfig, EmbeddingResponse},
    google::{auth::GoogleAuth, GeminiClient},
    openai::client::{OpenAIAuth, OpenAIClient},
    types::ChatResponse,
};
