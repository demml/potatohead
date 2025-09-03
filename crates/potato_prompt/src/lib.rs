pub mod prompt;

pub use prompt::{
    error::PromptError,
    interface::Prompt,
    types::{
        parse_response_to_json, AudioUrl, BinaryContent, DocumentUrl, ImageUrl, Message,
        PromptContent, Role, Score,
    },
};
