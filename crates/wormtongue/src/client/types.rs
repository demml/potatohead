use std::fmt::Display;

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
            ClientURL::OpenAI => "https://api.openai.com",
            ClientURL::Anthropic => "https://api.anthropic.com",
        }
    }
}

impl Display for ClientURL {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
