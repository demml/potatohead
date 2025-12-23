use crate::error::TypeError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub static VAR_REGEX: OnceLock<Regex> = OnceLock::new();
pub fn get_var_regex() -> &'static Regex {
    VAR_REGEX.get_or_init(|| Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap())
}

/// Core trait that all message types must implement
pub trait PromptMessageExt:
    Send + Sync + Clone + Serialize + for<'de> Deserialize<'de> + PartialEq
{
    /// Bind a variable in the message content, returning a new instance
    fn bind(&self, name: &str, value: &str) -> Result<Self, TypeError>
    where
        Self: Sized;

    /// Bind a variable in-place
    fn bind_mut(&mut self, name: &str, value: &str) -> Result<(), TypeError>;

    /// Extract variables from the message content
    fn extract_variables(&self) -> Vec<String>;
}

pub trait MessageFactory: Sized {
    fn from_text(content: String, role: &str) -> Result<Self, TypeError>;
}
