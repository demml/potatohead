use crate::client::ClientURL;
use crate::common::InteractionType;
use crate::error::TongueError;
use pyo3::prelude::*;
use std::env;

#[allow(non_camel_case_types)]
#[pyclass(eq)]
#[derive(Debug, PartialEq, Clone)]
pub enum OpenAIModels {
    Gpt4o,
    Gpt4oMini,
    o1,
    o1Mini,
    o1Preview,
    Gpt4Turbo,
    ChatGpt4oLatest,
    Gpt4oRealtimePreview,
    Gpt4oMiniRealtimePreview,
    Gpt4oAudioPreview,
}

impl OpenAIModels {
    pub fn as_str(&self) -> &'static str {
        match self {
            OpenAIModels::Gpt4o => "gpt-4o",
            OpenAIModels::Gpt4oMini => "gpt-4o-turbo",
            OpenAIModels::o1 => "o1",
            OpenAIModels::o1Mini => "o1-mini",
            OpenAIModels::o1Preview => "o1-preview",
            OpenAIModels::Gpt4Turbo => "gpt-4o-turbo",
            OpenAIModels::ChatGpt4oLatest => "chat-gpt-4o-latest",
            OpenAIModels::Gpt4oRealtimePreview => "gpt-4o-realtime-preview",
            OpenAIModels::Gpt4oMiniRealtimePreview => "gpt-4o-mini-realtime-preview",
            OpenAIModels::Gpt4oAudioPreview => "gpt-4o-audio-preview",
        }
    }
}

// implement to string for OpenAIModels (for when passing from python to rust)
impl std::fmt::Display for OpenAIModels {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[pyclass(eq)]
#[derive(Debug, PartialEq, Clone)]
pub enum OpenAIEndpoints {
    Chat,
    Batch,
}

impl OpenAIEndpoints {
    pub fn as_str(&self) -> &'static str {
        match self {
            OpenAIEndpoints::Chat => "v1/chat/completions",
            OpenAIEndpoints::Batch => "v1/batches",
        }
    }
}

pub fn resolve_url(
    url: Option<String>,
    interaction_type: &InteractionType,
) -> Result<String, TongueError> {
    let url = url
        .or_else(|| env::var("WORMTONGUE_URL").ok())
        .unwrap_or_else(|| ClientURL::OpenAI.as_str().to_string());

    match interaction_type {
        InteractionType::Text => Ok(format!("{}/v1/chat/completions", url)),
        InteractionType::Image => Ok(format!("{}/v1/images/generations", url)),
        InteractionType::Voice => Ok(format!("{}/v1/audio/speech", url)),
        InteractionType::Batch => Ok(format!("{}/v1/batches", url)),
        InteractionType::Embedding => Ok(format!("{}/v1/embeddings", url)),
        _ => Err(TongueError::UnsupportedInteractionType),
    }
}

pub fn resolve_api_key(url: &str, api_key: Option<&str>) -> Result<String, TongueError> {
    let api_key = api_key
        .map(|s| s.to_string())
        .or_else(|| env::var("WORMTONGUE_API_KEY").ok());

    // if url contains ClientURL::OpenAI.as_str() and api_key is None, return error
    if url.contains(ClientURL::OpenAI.as_str()) && api_key.is_none() {
        return Err(TongueError::MissingAPIKey);
    }

    Ok(api_key.unwrap())
}
