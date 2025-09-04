use potato_type::Provider;

use crate::agents::provider;

#[derive(Debug, Clone, PartialEq)]
pub struct Embedder {
    provider: Provider,
}

impl Embedder {
    pub fn new(provider: Provider) -> Self {
        Self { provider }
    }
    pub fn create(&self) -> Result<OpenAIEmbeddingResponse, AgentError> {
        // Implementation for creating an embedding
    }
}
