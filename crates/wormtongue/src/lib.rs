pub mod client;
pub mod common;
pub mod error;
pub mod tongues;

pub use tongues::openai::{OpenAIInterface, OpenAIModels, OpenAIPrompt};
