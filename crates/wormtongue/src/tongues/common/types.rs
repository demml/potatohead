use chrono::Utc;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

pub enum FileName {
    OpenAIPrompt,
    ClaudePrompt,
    Prompt,
}

impl FileName {
    pub fn to_str(&self) -> String {
        // add current timestamp to filename
        let now = Utc::now().naive_utc().to_string();
        match self {
            FileName::OpenAIPrompt => format!("openai_prompt_{}", now),
            FileName::ClaudePrompt => format!("claude_prompt_{}", now),
            FileName::Prompt => format!("prompt_{}", now),
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
    Text,
    Voice,
    Batch,
    Embedding,
}

#[pyclass(eq)]
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone, Default)]
pub enum TongueType {
    Anthropic,
    #[default]
    OpenAI,
}
