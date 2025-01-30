#[derive(Debug, Clone)]
pub enum RequestType {
    Get,
    Post,
}

#[derive(Debug, Clone)]
pub enum ClientURL {
    OpenAI,
    Anthropic,
}

impl ClientURL {
    pub fn as_str(&self) -> &str {
        match self {
            ClientURL::OpenAI => "openai",
            ClientURL::Anthropic => "anthropic",
        }
    }
}
