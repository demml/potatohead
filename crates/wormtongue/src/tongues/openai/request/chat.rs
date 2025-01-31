use crate::common::Utils;
use crate::error::WormTongueError;
use pyo3::{
    prelude::*,
    types::{PyList, PyString},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<MessageContentPart>),
}

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
pub struct MessageContentPart {
    #[pyo3(get, set)]
    pub r#type: String,
    #[pyo3(get, set)]
    pub text: String,
}

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    #[pyo3(get, set)]
    pub role: String,
    #[pyo3(get, set)]
    pub content: MessageContent,
}

#[pymethods]
impl Message {
    #[new]
    #[pyo3(signature = (role, content))]
    pub fn py_new(role: &str, content: &Bound<'_, PyAny>) -> PyResult<Self> {
        if content.is_instance_of::<PyString>() {
            let content = content
                .extract::<String>()
                .map_err(|e| WormTongueError::new_err(e))?;
            return Ok(Self {
                role: role.to_string(),
                content: MessageContent::Text(content),
            });
        } else if content.is_instance_of::<PyList>() {
            let content = content
                .extract::<Vec<MessageContentPart>>()
                .map_err(|e| WormTongueError::new_err(e))?;
            return Ok(Self {
                role: role.to_string(),
                content: MessageContent::Parts(content),
            });
        } else {
            return Err(WormTongueError::new_err("Invalid content type"));
        }
    }
}

impl Message {
    pub fn new(role: &str, content: MessageContent) -> Self {
        Self {
            role: role.to_string(),
            content,
        }
    }
}

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
pub struct CreateChatCompletionRequest {
    #[pyo3(get, set)]
    pub model: String,
    #[pyo3(get, set)]
    pub messages: Vec<Message>,
    #[pyo3(get, set)]
    pub store: Option<bool>,
    #[pyo3(get, set)]
    pub reasoning_effort: Option<String>,
    #[pyo3(get, set)]
    pub metadata: Option<HashMap<String, String>>,
    #[pyo3(get, set)]
    pub frequency_penalty: Option<f64>,
    #[pyo3(get, set)]
    pub logit_bias: Option<HashMap<String, i32>>,
    #[pyo3(get, set)]
    pub logprobs: Option<bool>,
    #[pyo3(get, set)]
    pub top_logprobs: Option<i32>,
    #[pyo3(get, set)]
    pub max_tokens: Option<i32>,
    #[pyo3(get, set)]
    pub max_completion_tokens: Option<i32>,
    #[pyo3(get, set)]
    pub n: Option<i32>,
    #[pyo3(get, set)]
    pub modalities: Option<Vec<String>>,
    #[pyo3(get, set)]
    pub prediction: Option<PredictionContent>,
    #[pyo3(get, set)]
    pub audio: Option<AudioParameters>,
    #[pyo3(get, set)]
    pub presence_penalty: Option<f64>,
    #[pyo3(get, set)]
    pub response_format: Option<ResponseFormat>,
    #[pyo3(get, set)]
    pub seed: Option<i64>,
    #[pyo3(get, set)]
    pub service_tier: Option<String>,
    #[pyo3(get, set)]
    pub stop: Option<StopSequences>,
    #[pyo3(get, set)]
    pub stream: Option<bool>,
    #[pyo3(get, set)]
    pub stream_options: Option<ChatCompletionStreamOptions>,
    #[pyo3(get, set)]
    pub temperature: Option<f64>,
    #[pyo3(get, set)]
    pub top_p: Option<f64>,
    #[pyo3(get, set)]
    pub tools: Option<Vec<ChatCompletionTool>>,
    #[pyo3(get, set)]
    pub tool_choice: Option<String>,
    #[pyo3(get, set)]
    pub user: Option<String>,
}

#[pymethods]
impl CreateChatCompletionRequest {
    #[new]
    #[pyo3(signature = (
        model,
        messages,
        store = None,
        reasoning_effort = None,
        metadata = None,
        frequency_penalty = None,
        logit_bias = None,
        logprobs = None,
        top_logprobs = None,
        max_tokens = None,
        max_completion_tokens = None,
        n = Some(1),
        modalities = None,
        prediction = None,
        audio = None,
        presence_penalty = None,
        response_format = None,
        seed = None,
        service_tier = None,
        stop = None,
        stream = None,
        stream_options = None,
        temperature = Some(1.0),
        top_p = Some(1.0),
        tools = None,
        tool_choice = None,
        user = None
    ))]
    fn py_new(
        model: String,
        messages: Vec<Message>,
        store: Option<bool>,
        reasoning_effort: Option<String>,
        metadata: Option<HashMap<String, String>>,
        frequency_penalty: Option<f64>,
        logit_bias: Option<HashMap<String, i32>>,
        logprobs: Option<bool>,
        top_logprobs: Option<i32>,
        max_tokens: Option<i32>,
        max_completion_tokens: Option<i32>,
        n: Option<i32>,
        modalities: Option<Vec<String>>,
        prediction: Option<PredictionContent>,
        audio: Option<AudioParameters>,
        presence_penalty: Option<f64>,
        response_format: Option<ResponseFormat>,
        seed: Option<i64>,
        service_tier: Option<String>,
        stop: Option<StopSequences>,
        stream: Option<bool>,
        stream_options: Option<ChatCompletionStreamOptions>,
        temperature: Option<f64>,
        top_p: Option<f64>,
        tools: Option<Vec<ChatCompletionTool>>,
        tool_choice: Option<String>,
        user: Option<String>,
    ) -> Self {
        Self {
            model,
            messages,
            store,
            reasoning_effort,
            metadata,
            frequency_penalty,
            logit_bias,
            logprobs,
            top_logprobs,
            max_tokens,
            max_completion_tokens,
            n,
            modalities,
            prediction,
            audio,
            presence_penalty,
            response_format,
            seed,
            service_tier,
            stop,
            stream,
            stream_options,
            temperature,
            top_p,
            tools,
            tool_choice,
            user,
        }
    }

    pub fn __str__(&self) -> String {
        // serialize the struct to a string
        Utils::__str__(self)
    }

    pub fn model_dump_json(&self) -> String {
        // serialize the struct to a string
        Utils::__json__(self)
    }
}

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
pub struct PredictionContent {
    #[pyo3(get)]
    pub r#type: String,

    #[pyo3(get)]
    pub content: MessageContent,
}
#[pymethods]
impl PredictionContent {
    #[new]
    #[pyo3(signature = (r#type, content))]
    fn new(r#type: String, content: &Bound<'_, PyAny>) -> PyResult<Self> {
        if content.is_instance_of::<PyString>() {
            let content = content
                .extract::<String>()
                .map_err(|e| WormTongueError::new_err(e))?;
            return Ok(Self {
                r#type,
                content: MessageContent::Text(content),
            });
        } else if content.is_instance_of::<PyList>() {
            let content = content
                .extract::<Vec<MessageContentPart>>()
                .map_err(|e| WormTongueError::new_err(e))?;
            return Ok(Self {
                r#type,
                content: MessageContent::Parts(content),
            });
        } else {
            return Err(WormTongueError::new_err("Invalid content type"));
        }
    }
}

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
pub struct AudioParameters {
    #[pyo3(get, set)]
    pub voice: String,
    #[pyo3(get, set)]
    pub format: String,
}

#[pymethods]
impl AudioParameters {
    #[new]
    #[pyo3(signature = (voice, format))]
    fn new(voice: String, format: String) -> Self {
        Self { voice, format }
    }
}

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ResponseFormat {
    Text(ResponseFormatText),
    JsonObject(ResponseFormatJsonObject),
    JsonSchema(ResponseFormatJsonSchema),
}

#[pymethods]
impl ResponseFormat {
    #[new]
    #[pyo3(signature = (response_format))]
    fn new(response_format: &Bound<'_, PyAny>) -> PyResult<Self> {
        if response_format.is_instance_of::<ResponseFormatText>() {
            let response_format = response_format.extract::<ResponseFormatText>().unwrap();
            return Ok(Self::Text(response_format));
        } else if response_format.is_instance_of::<ResponseFormatJsonObject>() {
            let response_format = response_format
                .extract::<ResponseFormatJsonObject>()
                .unwrap();
            return Ok(Self::JsonObject(response_format));
        } else if response_format.is_instance_of::<ResponseFormatJsonSchema>() {
            let response_format = response_format
                .extract::<ResponseFormatJsonSchema>()
                .unwrap();
            return Ok(Self::JsonSchema(response_format));
        } else {
            return Err(WormTongueError::new_err("Invalid response format"));
        }
    }
}

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
pub struct ResponseFormatText {
    #[pyo3(get)]
    pub r#type: String,
}

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
pub struct ResponseFormatJsonObject {
    #[pyo3(get)]
    pub r#type: String,
}

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
pub struct JsonSchema {
    pub schema: Value,
    pub name: String,
    pub strict: bool,
}

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
pub struct ResponseFormatJsonSchema {
    #[pyo3(get)]
    pub r#type: String,

    #[pyo3(get)]
    pub json_schema: JsonSchema,
}

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum StopSequences {
    Single(String),
    Multiple(Vec<String>),
}

#[pymethods]
impl StopSequences {
    #[new]
    #[pyo3(signature = (stop))]
    fn new(stop: &Bound<'_, PyAny>) -> PyResult<Self> {
        if stop.is_instance_of::<PyString>() {
            let stop = stop.extract::<String>().unwrap();
            return Ok(Self::Single(stop));
        } else if stop.is_instance_of::<PyList>() {
            let stop = stop.extract::<Vec<String>>().unwrap();
            return Ok(Self::Multiple(stop));
        } else {
            return Err(WormTongueError::new_err("Invalid stop sequence"));
        }
    }
}

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
pub struct ChatCompletionStreamOptions {
    #[pyo3(get)]
    include_usage: bool,
}

#[pymethods]
impl ChatCompletionStreamOptions {
    #[new]
    #[pyo3(signature = (include_usage))]
    fn new(include_usage: bool) -> Self {
        Self { include_usage }
    }
}

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
pub struct FunctionObject {
    #[pyo3(get, set)]
    pub name: String,

    #[pyo3(get)]
    pub description: Option<String>,

    pub parameters: Option<Value>,

    #[pyo3(get, set)]
    pub strict: bool,
}

#[pymethods]
impl FunctionObject {
    #[new]
    #[pyo3(signature = (name, description = None, parameters = None, strict = false))]
    fn new(
        name: String,
        description: Option<String>,
        parameters: Option<String>,
        strict: bool,
    ) -> Self {
        let parameters = match parameters {
            Some(parameters) => Some(serde_json::from_str(&parameters).unwrap()),
            None => None,
        };

        Self {
            name,
            description,
            parameters,
            strict,
        }
    }

    #[getter]
    pub fn get_parameters(&self) -> PyResult<Option<String>> {
        match &self.parameters {
            Some(parameters) => Ok(Some(parameters.to_string())),
            None => Ok(None),
        }
    }

    #[setter]
    pub fn set_parameters(&mut self, parameters: Option<String>) {
        match parameters {
            Some(parameters) => self.parameters = Some(serde_json::from_str(&parameters).unwrap()),
            None => self.parameters = None,
        }
    }
}

#[pyclass]
#[derive(Serialize, Deserialize, Clone)]
pub struct ChatCompletionTool {
    #[pyo3(get)]
    pub r#type: String,

    #[pyo3(get, set)]
    pub function: FunctionObject,
}

#[pymethods]
impl ChatCompletionTool {
    #[new]
    #[pyo3(signature = (r#type, function))]
    fn new(r#type: String, function: FunctionObject) -> Self {
        Self { r#type, function }
    }
}

impl CreateChatCompletionRequest {
    pub fn new(
        model: String,
        messages: Vec<Message>,
        store: Option<bool>,
        reasoning_effort: Option<String>,
        metadata: Option<HashMap<String, String>>,
        frequency_penalty: Option<f64>,
        logit_bias: Option<HashMap<String, i32>>,
        logprobs: Option<bool>,
        top_logprobs: Option<i32>,
        max_tokens: Option<i32>,
        max_completion_tokens: Option<i32>,
        n: Option<i32>,
        modalities: Option<Vec<String>>,
        prediction: Option<PredictionContent>,
        audio: Option<AudioParameters>,
        presence_penalty: Option<f64>,
        response_format: Option<ResponseFormat>,
        seed: Option<i64>,
        service_tier: Option<String>,
        stop: Option<StopSequences>,
        stream: Option<bool>,
        stream_options: Option<ChatCompletionStreamOptions>,
        temperature: Option<f64>,
        top_p: Option<f64>,
        tools: Option<Vec<ChatCompletionTool>>,
        tool_choice: Option<String>,
        user: Option<String>,
    ) -> Self {
        Self {
            model,
            messages,
            store,
            reasoning_effort,
            metadata,
            frequency_penalty,
            logit_bias,
            logprobs,
            top_logprobs,
            max_tokens,
            max_completion_tokens,
            n,
            modalities,
            prediction,
            audio,
            presence_penalty,
            response_format,
            seed,
            service_tier,
            stop,
            stream,
            stream_options,
            temperature,
            top_p,
            tools,
            tool_choice,
            user,
        }
    }
}

impl Default for CreateChatCompletionRequest {
    fn default() -> Self {
        Self {
            model: "gpt-4o".to_string(),
            messages: vec![],
            store: None,
            reasoning_effort: None,
            metadata: None,
            frequency_penalty: None,
            logit_bias: None,
            logprobs: None,
            top_logprobs: None,
            max_tokens: None,
            max_completion_tokens: None,
            n: Some(1),
            modalities: None,
            prediction: None,
            audio: None,
            presence_penalty: None,
            response_format: None,
            seed: None,
            service_tier: None,
            stop: None,
            stream: None,
            stream_options: None,
            temperature: Some(1.0),
            top_p: Some(1.0),
            tools: None,
            tool_choice: None,
            user: None,
        }
    }
}
