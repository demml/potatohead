use crate::error::TypeError;
use crate::SettingsType;
use potato_util::json_to_pydict;
use potato_util::{pyobject_to_json, PyHelperFuncs, UtilError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

    pub fn model_dump<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyDict>, TypeError> {
        // iterate over each field in model_settings and add to the dict if it is not None
        let pydict = PyDict::new(py);
        pydict.set_item("format", &self.format)?;
        pydict.set_item("voice", &self.voice)?;
        Ok(pydict)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[pyclass]
pub struct PredictionContentPart {
    #[pyo3(get, set)]
    #[serde(rename = "type")]
    pub r#type: String,
    #[pyo3(get, set)]
    pub text: String,
}

#[pymethods]
impl PredictionContentPart {
    #[new]
    pub fn new(r#type: String, text: String) -> Self {
        PredictionContentPart { r#type, text }
    }

    pub fn model_dump<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyDict>, TypeError> {
        // iterate over each field in model_settings and add to the dict if it is not None
        let pydict = PyDict::new(py);

        pydict.set_item("type", &self.r#type)?;
        pydict.set_item("text", &self.text)?;
        Ok(pydict)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
#[pyclass]
pub enum Content {
    Text(String),
    Array(Vec<PredictionContentPart>),
}

#[pymethods]
impl Content {
    #[new]
    #[pyo3(signature = (text=None, parts=None))]
    pub fn new(
        text: Option<String>,
        parts: Option<Vec<PredictionContentPart>>,
    ) -> Result<Self, TypeError> {
        match (text, parts) {
            (Some(t), None) => Ok(Content::Text(t)),
            (None, Some(p)) => Ok(Content::Array(p)),
            _ => Err(TypeError::InvalidInput(
                "Either text or parts must be provided, but not both.".to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    #[serde(rename = "none")]
    NA,
    Auto,
    Required,
}

#[pymethods]
impl ToolChoiceMode {
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
    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct InnerAllowedTools {
    #[pyo3(get)]
    pub mode: AllowedToolsMode,
    #[pyo3(get)]
    pub tools: Vec<ToolDefinition>,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AllowedTools {
    #[pyo3(get)]
    pub r#type: String,
    #[pyo3(get)]
    pub allowed_tools: InnerAllowedTools,
}

#[pymethods]
impl AllowedTools {
    #[new]
    #[pyo3(signature = (mode, tools))]
    pub fn new(mode: AllowedToolsMode, tools: Vec<ToolDefinition>) -> Self {
        Self {
            r#type: "allowed_tools".to_string(),
            allowed_tools: InnerAllowedTools { mode, tools },
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
#[pyclass(name = "OpenAIToolChoice")]
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
    pub fn from_mode(mode: &ToolChoiceMode) -> Self {
        ToolChoice::Mode(mode.clone())
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

#[pyclass(name = "OpenAITool")]
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
    ) -> Result<Self, TypeError> {
        match (function, custom) {
            (Some(function), None) => Ok(Tool::Function(function)),
            (None, Some(custom)) => Ok(Tool::Custom(custom)),
            _ => Err(TypeError::InvalidInput("Invalid tool definition".into())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
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

    #[getter]
    pub fn extra_body<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Option<Bound<'py, PyDict>>, TypeError> {
        // error if extra body is None
        Ok(self
            .extra_body
            .as_ref()
            .map(|v| {
                let pydict = PyDict::new(py);
                json_to_pydict(py, v, &pydict)
            })
            .transpose()?)
    }

    pub fn model_dump<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyDict>, TypeError> {
        // iterate over each field in model_settings and add to the dict if it is not None
        let json = serde_json::to_value(self)?;
        let pydict = PyDict::new(py);
        json_to_pydict(py, &json, &pydict)?;
        Ok(pydict)
    }

    pub fn settings_type(&self) -> SettingsType {
        SettingsType::OpenAIChat
    }
}
