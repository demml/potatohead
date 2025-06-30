pub mod prompt;

pub use prompt::{
    error::PromptError,
    interface::{ModelSettings, Prompt},
    types::{AudioUrl, BinaryContent, DocumentUrl, ImageUrl, Message, PromptContent, Role},
};
