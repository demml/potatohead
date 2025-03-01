use chrono::Utc;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::path::Path;

pub enum FileName {
    OpenAIPrompt,
    ClaudePrompt,
    Prompt,
}

impl FileName {
    pub fn to_str(&self) -> String {
        // add current timestamp to filename
        let now = Utc::now().naive_utc();
        match self {
            FileName::OpenAIPrompt => format!("openai_prompt_{now}"),
            FileName::ClaudePrompt => format!("openai_prompt_{now}"),
            FileName::Prompt => format!("prompt_{now}"),
        }
    }
}

impl AsRef<Path> for FileName {
    fn as_ref(&self) -> &Path {
        match self {
            FileName::OpenAIPrompt => Path::new("openai_prompt"),
            FileName::ClaudePrompt => Path::new("claude_prompt"),
            FileName::Prompt => Path::new("prompt"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    InProgress,
    Completed,
    Failed,
    NotStarted,
}

#[pyclass(eq)]
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone, Default)]
pub enum PromptType {
    Image,
    Vision,
    #[default]
    Chat,
    Voice,
    Batch,
    Embedding,
}

impl Display for PromptType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            PromptType::Chat => write!(f, "chat"),
            PromptType::Image => write!(f, "image"),
            PromptType::Vision => write!(f, "vision"),
            PromptType::Voice => write!(f, "voice"),
            PromptType::Batch => write!(f, "batch"),
            PromptType::Embedding => write!(f, "embedding"),
        }
    }
}

#[pyclass(eq)]
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone, Default)]
pub enum PotatoVendor {
    Anthropic,
    #[default]
    OpenAI,
}
