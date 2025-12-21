pub mod error;
pub mod providers;

pub use error::ProviderError;
pub use providers::{
    client::GenAiClient,
    embed::{Embedder, EmbeddingConfig, EmbeddingResponse},
    google::{auth::GoogleAuth, types::*, GeminiClient, GenerateContentResponse},
    openai::{
        client::{OpenAIAuth, OpenAIClient},
        CompletionTokenDetails, OpenAIChatMessage, OpenAIChatResponse, PromptTokenDetails,
        ToolCall, Usage,
    },
    types::ChatResponse,
};
