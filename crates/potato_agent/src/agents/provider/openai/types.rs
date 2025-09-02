use crate::agents::error::AgentError;
use crate::agents::provider::traits::{LogProbExt, ResponseExt, TokenUsage};
use potato_prompt::{prompt::types::PromptContent, Message};
use potato_util::utils::ResponseLogProbs;
use potato_util::{pyobject_to_json, PyHelperFuncs, UtilError};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct Function {
    pub arguments: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct ToolCall {
    pub id: String,
    pub type_: String,
    pub function: Function,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct UrlCitation {
    pub end_index: u64,
    pub start_index: u64,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct Annotations {
    pub r#type: String,
    pub url_citations: Vec<UrlCitation>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct Audio {
    pub data: String,
    pub expires_at: u64, // Unix timestamp
    pub id: String,
    pub transcript: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct ChatCompletionMessage {
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    pub role: String,
    pub annotations: Vec<Annotations>,
    pub tool_calls: Vec<ToolCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<Audio>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct TopLogProbs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
    pub logprob: f64,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct LogContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
    pub logprob: f64,
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<Vec<TopLogProbs>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct LogProbs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<LogContent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<Vec<LogContent>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct Choice {
    pub message: ChatCompletionMessage,
    pub finish_reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<LogProbs>,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct CompletionTokenDetails {
    pub accepted_prediction_tokens: u64,
    pub audio_tokens: u64,
    pub reasoning_tokens: u64,
    pub rejected_prediction_tokens: u64,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct PromptTokenDetails {
    pub audio_tokens: u64,
    pub cached_tokens: u64,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct Usage {
    pub completion_tokens: u64,
    pub prompt_tokens: u64,
    pub total_tokens: u64,
    pub completion_tokens_details: CompletionTokenDetails,
    pub prompt_tokens_details: PromptTokenDetails,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

impl TokenUsage for Usage {
    fn total_tokens(&self) -> u64 {
        self.total_tokens
    }

    fn prompt_tokens(&self) -> u64 {
        self.prompt_tokens
    }

    fn completion_tokens(&self) -> u64 {
        self.completion_tokens
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct OpenAIChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

impl ResponseExt for OpenAIChatResponse {
    fn get_content(&self) -> Option<String> {
        self.choices.first().and_then(|c| c.message.content.clone())
    }
}

impl LogProbExt for OpenAIChatResponse {
    fn get_log_probs(&self) -> Vec<ResponseLogProbs> {
        let mut probabilities = Vec::new();
        if let Some(choice) = self.choices.first() {
            if let Some(logprobs) = &choice.logprobs {
                if let Some(content) = &logprobs.content {
                    for log_content in content {
                        // Look for single digit tokens (1, 2, 3, 4, 5)
                        if log_content.token.len() == 1
                            && log_content.token.chars().next().unwrap().is_ascii_digit()
                        {
                            probabilities.push(ResponseLogProbs {
                                token: log_content.token.clone(),
                                logprob: log_content.logprob,
                            });
                        }
                    }
                }
            }
        }

        probabilities
    }
}

#[pymethods]
impl OpenAIChatResponse {
    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenAIChatRequest {
    pub model: String,
    pub messages: Vec<OpenAIChatMessage>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAITextMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAIImageUrl {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAIInputAudio {
    pub data: String,   // base64-encoded audio
    pub format: String, // e.g., "wav"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAIContentPart {
    Text { text: String },
    ImageUrl { image_url: OpenAIImageUrl },
    InputAudio { input_audio: OpenAIInputAudio },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAIChatMessage {
    pub role: String,
    pub content: Vec<OpenAIContentPart>,
}

impl OpenAIChatMessage {
    /// Convert the Prompt Message to an OpenAI multimodal chat message
    pub fn from_message(message: &Message) -> Result<Self, AgentError> {
        let content = match &message.content {
            PromptContent::Str(text) => vec![OpenAIContentPart::Text { text: text.clone() }],
            PromptContent::Image(image) => vec![OpenAIContentPart::ImageUrl {
                image_url: OpenAIImageUrl {
                    url: image.url.clone(),
                },
            }],
            // need to implement audio and file for chat
            _ => {
                // Handle other content types as needed
                return Err(AgentError::UnsupportedContentTypeError);
            }
        };

        Ok(OpenAIChatMessage {
            role: message.role.to_string(),
            content,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[pyclass]
pub struct AudioParam {
    #[pyo3(get, set)]
    pub format: String,
    #[pyo3(get, set)]
    pub voice: String,
}

#[pymethods]
impl AudioParam {
    #[new]
    pub fn new(format: String, voice: String) -> Self {
        AudioParam { format, voice }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[pyclass]
pub struct ContentPart {
    #[pyo3(get, set)]
    pub r#type: String,
    #[pyo3(get, set)]
    pub text: String,
}

#[pymethods]
impl ContentPart {
    #[new]
    pub fn new(r#type: String, text: String) -> Self {
        ContentPart { r#type, text }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[pyclass]
pub enum Content {
    Text(String),
    Array(Vec<ContentPart>),
}

#[pymethods]
impl Content {
    #[new]
    #[pyo3(signature = (text=None, parts=None))]
    pub fn new(text: Option<String>, parts: Option<Vec<ContentPart>>) -> PyResult<Self> {
        match (text, parts) {
            (Some(t), None) => Ok(Content::Text(t)),
            (None, Some(p)) => Ok(Content::Array(p)),
            _ => Err(pyo3::exceptions::PyValueError::new_err(
                "Either text or parts must be provided, but not both.",
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[pyclass]
pub struct Prediction {
    #[pyo3(get, set)]
    pub r#type: String,
    #[pyo3(get, set)]
    pub content: Content,
}

#[pymethods]
impl Prediction {
    #[new]
    pub fn new(r#type: String, content: Content) -> Self {
        Prediction { r#type, content }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[pyclass]
pub struct StreamOptions {
    #[pyo3(get, set)]
    pub include_obfuscation: Option<bool>,
    #[pyo3(get, set)]
    pub include_usage: Option<bool>,
}

#[pymethods]
impl StreamOptions {
    #[new]
    pub fn new(include_obfuscation: Option<bool>, include_usage: Option<bool>) -> Self {
        StreamOptions {
            include_obfuscation,
            include_usage,
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceMode {
    None,
    Auto,
    Required,
}

#[pymethods]
impl ToolChoiceMode {
    #[new]
    #[pyo3(signature = (mode))]
    pub fn new(mode: &str) -> PyResult<Self> {
        match mode.to_lowercase().as_str() {
            "none" => Ok(ToolChoiceMode::None),
            "auto" => Ok(ToolChoiceMode::Auto),
            "required" => Ok(ToolChoiceMode::Required),
            _ => Err(pyo3::exceptions::PyValueError::new_err(
                "Invalid tool choice mode. Must be 'none', 'auto', or 'required'.",
            )),
        }
    }

    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

/// Function specification for function tool choice
#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct FunctionChoice {
    #[pyo3(get, set)]
    pub name: String,
}

#[pymethods]
impl FunctionChoice {
    #[new]
    #[pyo3(signature = (name))]
    pub fn new(name: String) -> Self {
        Self { name }
    }

    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

/// Function tool choice specification
#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct FunctionToolChoice {
    #[pyo3(get)]
    pub r#type: String,
    #[pyo3(get, set)]
    pub function: FunctionChoice,
}

#[pymethods]
impl FunctionToolChoice {
    #[new]
    #[pyo3(signature = (function))]
    pub fn new(function: FunctionChoice) -> Self {
        Self {
            r#type: "function".to_string(),
            function,
        }
    }

    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

/// Custom tool specification
#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CustomChoice {
    #[pyo3(get, set)]
    pub name: String,
}

#[pymethods]
impl CustomChoice {
    #[new]
    #[pyo3(signature = (name))]
    pub fn new(name: String) -> Self {
        Self { name }
    }

    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

/// Custom tool choice specification
#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CustomToolChoice {
    #[pyo3(get)]
    pub r#type: String,
    #[pyo3(get, set)]
    pub custom: CustomChoice,
}

#[pymethods]
impl CustomToolChoice {
    #[new]
    #[pyo3(signature = (custom))]
    pub fn new(custom: CustomChoice) -> Self {
        Self {
            r#type: "custom".to_string(),
            custom,
        }
    }

    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ToolDefinition {
    #[pyo3(get)]
    pub r#type: String,
    #[pyo3(get, set)]
    pub function: FunctionChoice,
}

#[pymethods]
impl ToolDefinition {
    #[new]
    #[pyo3(signature = (function_name))]
    pub fn new(function_name: String) -> Self {
        Self {
            r#type: "function".to_string(),
            function: FunctionChoice::new(function_name),
        }
    }

    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

/// Mode for allowed tools constraint
#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AllowedToolsMode {
    /// Model can pick from allowed tools or generate a message
    Auto,
    /// Model must call one or more of the allowed tools
    Required,
}

#[pymethods]
impl AllowedToolsMode {
    #[new]
    #[pyo3(signature = (mode))]
    pub fn new(mode: &str) -> PyResult<Self> {
        match mode.to_lowercase().as_str() {
            "auto" => Ok(AllowedToolsMode::Auto),
            "required" => Ok(AllowedToolsMode::Required),
            _ => Err(pyo3::exceptions::PyValueError::new_err(
                "Invalid allowed tools mode. Must be 'auto' or 'required'.",
            )),
        }
    }

    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AllowedTools {
    #[pyo3(get)]
    pub r#type: String,
    #[pyo3(get, set)]
    pub mode: AllowedToolsMode,
    #[pyo3(get, set)]
    pub tools: Vec<ToolDefinition>,
}

#[pymethods]
impl AllowedTools {
    #[new]
    #[pyo3(signature = (mode, tools))]
    pub fn new(mode: AllowedToolsMode, tools: Vec<ToolDefinition>) -> Self {
        Self {
            r#type: "allowed_tools".to_string(),
            mode,
            tools,
        }
    }

    /// Create AllowedTools from function names
    #[staticmethod]
    #[pyo3(signature = (mode, function_names))]
    pub fn from_function_names(mode: AllowedToolsMode, function_names: Vec<String>) -> Self {
        let tools = function_names
            .into_iter()
            .map(ToolDefinition::new)
            .collect();

        Self::new(mode, tools)
    }

    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

/// Tool choice configuration - can be a mode string or specific tool object
#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ToolChoice {
    /// Simple mode selection (none, auto, required)
    Mode(ToolChoiceMode),
    /// Specific function tool choice
    Function(FunctionToolChoice),
    /// Custom tool choice
    Custom(CustomToolChoice),
    /// Allowed tools configuration
    Allowed(AllowedTools),
}

#[pymethods]
impl ToolChoice {
    /// Create tool choice from mode string
    #[staticmethod]
    #[pyo3(signature = (mode))]
    pub fn from_mode(mode: &str) -> PyResult<Self> {
        Ok(ToolChoice::Mode(ToolChoiceMode::new(mode)?))
    }

    /// Create tool choice for specific function
    #[staticmethod]
    #[pyo3(signature = (function_name))]
    pub fn from_function(function_name: String) -> Self {
        ToolChoice::Function(FunctionToolChoice::new(FunctionChoice::new(function_name)))
    }

    /// Create tool choice for custom tool
    #[staticmethod]
    #[pyo3(signature = (custom_name))]
    pub fn from_custom(custom_name: String) -> Self {
        ToolChoice::Custom(CustomToolChoice::new(CustomChoice::new(custom_name)))
    }

    /// Create allowed tools configuration
    #[staticmethod]
    #[pyo3(signature = (allowed_tools))]
    pub fn from_allowed_tools(allowed_tools: AllowedTools) -> Self {
        ToolChoice::Allowed(allowed_tools)
    }

    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

impl Default for ToolChoice {
    fn default() -> Self {
        ToolChoice::Mode(ToolChoiceMode::Auto)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FunctionDefinition {
    #[pyo3(get, set)]
    pub name: String,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

#[pymethods]
impl FunctionDefinition {
    #[new]
    #[pyo3(signature = (name, description=None, parameters=None, strict=None))]
    pub fn new(
        name: String,
        description: Option<String>,
        parameters: Option<&Bound<'_, PyAny>>,
        strict: Option<bool>,
    ) -> Result<Self, UtilError> {
        let params = match parameters {
            Some(obj) => Some(pyobject_to_json(obj)?),
            None => None,
        };

        Ok(Self {
            name,
            description,
            parameters: params,
            strict,
        })
    }

    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FunctionTool {
    #[pyo3(get, set)]
    pub function: FunctionDefinition,

    #[pyo3(get, set)]
    pub r#type: String,
}

#[pymethods]
impl FunctionTool {
    #[new]
    #[pyo3(signature = (function, r#type))]
    pub fn new(function: FunctionDefinition, r#type: String) -> Self {
        FunctionTool { function, r#type }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TextFormat {
    #[pyo3(get, set)]
    pub r#type: String,
}

#[pymethods]
impl TextFormat {
    #[new]
    #[pyo3(signature = (r#type))]
    pub fn new(r#type: String) -> Self {
        TextFormat { r#type }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Grammar {
    /// The grammar definition
    #[pyo3(get, set)]
    pub definition: String,

    /// The syntax of the grammar definition (lark or regex)
    #[pyo3(get, set)]
    pub syntax: String,
}

#[pymethods]
impl Grammar {
    #[new]
    #[pyo3(signature = (definition, syntax))]
    pub fn new(definition: String, syntax: String) -> Self {
        Self { definition, syntax }
    }

    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GrammarFormat {
    #[pyo3(get, set)]
    pub grammar: Grammar,
    #[pyo3(get, set)]
    pub r#type: String,
}

#[pymethods]
impl GrammarFormat {
    #[new]
    #[pyo3(signature = (grammar, r#type))]
    pub fn new(grammar: Grammar, r#type: String) -> Self {
        Self { grammar, r#type }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum CustomToolFormat {
    /// Unconstrained free-form text
    Text(TextFormat),
    /// A grammar defined by the user
    Grammar(GrammarFormat),
}

#[pymethods]
impl CustomToolFormat {
    #[new]
    #[pyo3(signature = (r#type=None, grammar=None))]
    pub fn new(r#type: Option<String>, grammar: Option<Grammar>) -> Self {
        match (r#type, grammar) {
            (Some(r#type), None) => CustomToolFormat::Text(TextFormat::new(r#type)),
            (None, Some(grammar)) => {
                CustomToolFormat::Grammar(GrammarFormat::new(grammar, String::new()))
            }
            _ => CustomToolFormat::Text(TextFormat::new(String::new())),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CustomDefinition {
    #[pyo3(get, set)]
    pub name: String,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<CustomToolFormat>,
}

#[pymethods]
impl CustomDefinition {
    #[new]
    #[pyo3(signature = (name, description=None, format=None))]
    pub fn new(
        name: String,
        description: Option<String>,
        format: Option<CustomToolFormat>,
    ) -> Self {
        Self {
            name,
            description,
            format,
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CustomTool {
    #[pyo3(get, set)]
    pub custom: CustomDefinition,

    #[pyo3(get, set)]
    pub r#type: String,
}

#[pymethods]
impl CustomTool {
    #[new]
    #[pyo3(signature = (custom, r#type))]
    pub fn new(custom: CustomDefinition, r#type: String) -> Self {
        CustomTool { custom, r#type }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum Tool {
    Function(FunctionTool),

    Custom(CustomTool),
}

#[pymethods]
impl Tool {
    #[new]
    #[pyo3(signature = (function=None, custom=None))]
    pub fn new(
        function: Option<FunctionTool>,
        custom: Option<CustomTool>,
    ) -> Result<Self, AgentError> {
        match (function, custom) {
            (Some(function), None) => Ok(Tool::Function(function)),
            (None, Some(custom)) => Ok(Tool::Custom(custom)),
            _ => Err(AgentError::InvalidToolDefinitionError),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[pyclass]
pub struct OpenAIChatSettings {
    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<usize>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<f32>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioParam>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<String>>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<usize>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<Prediction>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<i32>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<String>,

    // TODO: add web_search_arg later
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_body: Option<Value>,
}

#[pymethods]
impl OpenAIChatSettings {
    #[new]
    #[pyo3(signature = (max_completion_tokens=None, temperature=None, top_p=None, top_k=None, frequency_penalty=None, timeout=None, parallel_tool_calls=None, seed=None, logit_bias=None, stop_sequences=None, logprobs=None, audio=None, metadata=None, modalities=None, n=None, prediction=None, presence_penalty=None, prompt_cache_key=None, reasoning_effort=None, safety_identifier=None, service_tier=None, store=None, stream=None, stream_options=None, tool_choice=None, tools=None, top_logprobs=None, verbosity=None, extra_body=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        max_completion_tokens: Option<usize>,
        temperature: Option<f32>,
        top_p: Option<f32>,
        top_k: Option<i32>,
        frequency_penalty: Option<f32>,
        timeout: Option<f32>,
        parallel_tool_calls: Option<bool>,
        seed: Option<u64>,
        logit_bias: Option<HashMap<String, i32>>,
        stop_sequences: Option<Vec<String>>,
        logprobs: Option<bool>,
        audio: Option<AudioParam>,
        metadata: Option<HashMap<String, String>>,
        modalities: Option<Vec<String>>,
        n: Option<usize>,
        prediction: Option<Prediction>,
        presence_penalty: Option<f32>,
        prompt_cache_key: Option<String>,
        reasoning_effort: Option<String>,
        safety_identifier: Option<String>,
        service_tier: Option<String>,
        store: Option<bool>,
        stream: Option<bool>,
        stream_options: Option<StreamOptions>,
        tool_choice: Option<ToolChoice>,
        tools: Option<Vec<Tool>>,
        top_logprobs: Option<i32>,
        verbosity: Option<String>,
        extra_body: Option<&Bound<'_, PyAny>>,
    ) -> Result<Self, UtilError> {
        let extra = match extra_body {
            Some(obj) => Some(pyobject_to_json(obj)?),
            None => None,
        };

        Ok(OpenAIChatSettings {
            max_completion_tokens,
            temperature,
            top_p,
            top_k,
            frequency_penalty,
            timeout,
            parallel_tool_calls,
            seed,
            logit_bias,
            stop_sequences,
            logprobs,
            audio,
            metadata,
            modalities,
            n,
            prediction,
            presence_penalty,
            prompt_cache_key,
            reasoning_effort,
            safety_identifier,
            service_tier,
            store,
            stream,
            stream_options,
            tool_choice,
            tools,
            top_logprobs,
            verbosity,
            extra_body: extra,
        })
    }

    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}
