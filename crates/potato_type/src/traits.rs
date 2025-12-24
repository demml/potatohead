use crate::{
    error::TypeError,
    prompt::{MessageNum, ModelSettings},
    Provider,
};
use pyo3::prelude::*;
use pyo3::types::PyList;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;
pub static VAR_REGEX: OnceLock<Regex> = OnceLock::new();
pub fn get_var_regex() -> &'static Regex {
    VAR_REGEX.get_or_init(|| Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap())
}
use crate::prompt::builder::ProviderRequest;
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

    fn from_text(content: String, role: &str) -> Result<Self, TypeError>;
}

pub trait RequestAdapter {
    fn messages(&self) -> &[MessageNum];
    fn messages_mut(&mut self) -> &mut Vec<MessageNum>;
    fn system_instructions(&self) -> Vec<&MessageNum>;
    fn response_json_schema(&self) -> Option<&Value>;
    fn insert_message(&mut self, message: MessageNum, idx: Option<usize>) -> () {
        self.messages_mut().insert(idx.unwrap_or(0), message);
    }
    fn preprend_system_instructions(&mut self, messages: Vec<MessageNum>) -> Result<(), TypeError>;
    fn get_py_system_instructions<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyList>, TypeError>;
    fn model_settings<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError>;
    fn to_request_body(&self) -> Result<Value, TypeError>;
    fn match_provider(&self, provider: &Provider) -> bool;
    fn build_provider_enum(
        messages: Vec<MessageNum>,
        system_instructions: Vec<MessageNum>,
        model: String,
        settings: ModelSettings,
        response_json_schema: Option<Value>,
    ) -> Result<ProviderRequest, TypeError>;
}

pub trait ResponseAdapter {
    fn __str__(&self) -> String;
    fn is_empty(&self) -> bool;
    fn to_bound_py_object<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError>;
    fn id(&self) -> &str;
    fn to_message_num(&self) -> Result<Vec<MessageNum>, TypeError>;
}

pub trait MessageResponseExt {
    fn to_message_num(&self) -> Result<Option<MessageNum>, TypeError>;
}

pub trait MessageFactory: Sized {
    fn from_text(content: String, role: &str) -> Result<Self, TypeError>;
}
