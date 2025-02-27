use potato_error::PotatoError;
use std::env;
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

pub fn resolve_url(url: Option<&str>) -> Result<String, PotatoError> {
    let url = url
        .map(|s| s.to_string())
        .or_else(|| env::var("POTATO_HEAD_URL").ok())
        .unwrap_or_else(|| ClientURL::OpenAI.as_str().to_string());

    Ok(url)
}

pub fn resolve_api_key(url: &str, api_key: Option<&str>) -> Result<String, PotatoError> {
    let api_key = api_key
        .map(|s| s.to_string())
        .or_else(|| env::var("POTATO_HEAD_API_KEY").ok());

    // if url contains ClientURL::OpenAI.as_str() and api_key is None, return error
    if url.contains(ClientURL::OpenAI.as_str()) && api_key.is_none() {
        return Err(PotatoError::MissingAPIKey);
    }

    if api_key.is_none() {
        return Err(PotatoError::MissingAPIKey);
    }

    Ok(api_key.unwrap())
}

pub fn resolve_version(version: Option<&str>) -> Result<Option<String>, PotatoError> {
    let version = version
        .map(|s| s.to_string())
        .or_else(|| env::var("POTATO_HEAD_VERSION").ok());

    Ok(version)
}
