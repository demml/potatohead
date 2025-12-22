use crate::{SettingsType, TypeError};
use potato_util::{json_to_pydict, pyobject_to_json, PyHelperFuncs, UtilError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SchemaType {
    TypeUnspecified,
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
    Null,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Schema {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub r#type: Option<SchemaType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_ordering: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<Value>,
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    pub ref_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defs: Option<HashMap<String, Schema>>,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmCategory {
    #[default]
    HarmCategoryUnspecified,
    HarmCategoryHateSpeech,
    HarmCategoryDangerousContent,
    HarmCategoryHarassment,
    HarmCategorySexuallyExplicit,
    HarmCategoryImageHate,
    HarmCategoryImageDangerousContent,
    HarmCategoryImageHarassment,
    HarmCategoryImageSexuallyExplicit,
}

/// Probability-based threshold levels for blocking.
#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmBlockThreshold {
    HarmBlockThresholdUnspecified,
    BlockLowAndAbove,
    BlockMediumAndAbove,
    BlockOnlyHigh,
    BlockNone,
    #[default]
    Off,
}

/// Specifies whether the threshold is used for probability or severity score.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[pyclass]
pub enum HarmBlockMethod {
    HarmBlockMethodUnspecified,
    Severity,
    Probability,
}

/// Safety settings for harm blocking.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct SafetySetting {
    /// Required. The harm category.
    #[pyo3(get)]
    pub category: HarmCategory,
    /// Required. The harm block threshold.
    #[pyo3(get)]
    pub threshold: HarmBlockThreshold,
    /// Optional. Specify if the threshold is used for probability or severity score.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub method: Option<HarmBlockMethod>,
}

#[pymethods]
impl SafetySetting {
    #[new]
    #[pyo3(signature = (category, threshold, method=None))]
    pub fn new(
        category: HarmCategory,
        threshold: HarmBlockThreshold,
        method: Option<HarmBlockMethod>,
    ) -> Self {
        SafetySetting {
            category,
            threshold,
            method,
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Modality {
    ModalityUnspecified,
    Text,
    Image,
    Audio,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[pyclass]
pub enum MediaResolution {
    MediaResolutionUnspecified,
    MediaResolutionLow,
    MediaResolutionMedium,
    MediaResolutionHigh,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelRoutingPreference {
    Unknown,
    PrioritizeQuality,
    Balanced,
    PrioritizeCost,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
#[pyclass(name = "GeminiThinkingConfig")]
pub struct ThinkingConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_thoughts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_budget: Option<i32>,
}

#[pymethods]
impl ThinkingConfig {
    #[new]
    #[pyo3(signature = (include_thoughts=None, thinking_budget=None))]
    pub fn new(include_thoughts: Option<bool>, thinking_budget: Option<i32>) -> Self {
        ThinkingConfig {
            include_thoughts,
            thinking_budget,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AutoRoutingMode {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_routing_preference: Option<ModelRoutingPreference>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ManualRoutingMode {
    pub model_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum RoutingConfigMode {
    AutoMode(AutoRoutingMode),
    ManualMode(ManualRoutingMode),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RoutingConfig {
    #[serde(flatten)]
    pub routing_config: RoutingConfigMode,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct PrebuiltVoiceConfig {
    pub voice_name: String,
}

#[pymethods]
impl PrebuiltVoiceConfig {
    #[new]
    pub fn new(voice_name: String) -> Self {
        PrebuiltVoiceConfig { voice_name }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
#[pyclass]
pub enum VoiceConfigMode {
    PrebuiltVoiceConfig(PrebuiltVoiceConfig),
}

#[pymethods]
impl VoiceConfigMode {
    #[new]
    pub fn new(prebuilt_voice_config: PrebuiltVoiceConfig) -> Self {
        VoiceConfigMode::PrebuiltVoiceConfig(prebuilt_voice_config)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct VoiceConfig {
    #[serde(flatten)]
    pub voice_config: VoiceConfigMode,
}

#[pymethods]
impl VoiceConfig {
    #[new]
    pub fn new(voice_config: VoiceConfigMode) -> Self {
        VoiceConfig { voice_config }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
#[pyclass]
pub struct SpeechConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_config: Option<VoiceConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

#[pymethods]
impl SpeechConfig {
    #[new]
    pub fn new(voice_config: Option<VoiceConfig>, language_code: Option<String>) -> Self {
        SpeechConfig {
            voice_config,
            language_code,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
#[pyclass]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub stop_sequences: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub response_mime_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub response_modalities: Option<Vec<Modality>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub thinking_config: Option<ThinkingConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub top_p: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub top_k: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub candidate_count: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub max_output_tokens: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub response_logprobs: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub logprobs: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub presence_penalty: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub frequency_penalty: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub seed: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_schema: Option<Schema>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_json_schema: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing_config: Option<RoutingConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub audio_timestamp: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub media_resolution: Option<MediaResolution>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub speech_config: Option<SpeechConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub enable_affective_dialog: Option<bool>,
}

#[pymethods]
impl GenerationConfig {
    #[new]
    #[pyo3(signature = (stop_sequences=None, response_mime_type=None, response_modalities=None, thinking_config=None, temperature=None, top_p=None, top_k=None, candidate_count=None, max_output_tokens=None, response_logprobs=None, logprobs=None, presence_penalty=None, frequency_penalty=None, seed=None, audio_timestamp=None, media_resolution=None, speech_config=None, enable_affective_dialog=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        stop_sequences: Option<Vec<String>>,
        response_mime_type: Option<String>,
        response_modalities: Option<Vec<Modality>>,
        thinking_config: Option<ThinkingConfig>,
        temperature: Option<f32>,
        top_p: Option<f32>,
        top_k: Option<i32>,
        candidate_count: Option<i32>,
        max_output_tokens: Option<i32>,
        response_logprobs: Option<bool>,
        logprobs: Option<i32>,
        presence_penalty: Option<f32>,
        frequency_penalty: Option<f32>,
        seed: Option<i32>,
        //TODO: revisit this later
        //response_schema: Option<Schema>,
        //response_json_schema: Option<Value>,
        //routing_config: Option<RoutingConfig>,
        audio_timestamp: Option<bool>,
        media_resolution: Option<MediaResolution>,
        speech_config: Option<SpeechConfig>,
        enable_affective_dialog: Option<bool>,
    ) -> Self {
        Self {
            stop_sequences,
            response_mime_type,
            response_modalities,
            thinking_config,
            temperature,
            top_p,
            top_k,
            candidate_count,
            max_output_tokens,
            response_logprobs,
            logprobs,
            presence_penalty,
            frequency_penalty,
            seed,
            audio_timestamp,
            media_resolution,
            speech_config,
            enable_affective_dialog,
            ..Default::default()
        }
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ModelArmorConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_template_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_template_name: Option<String>,
}

#[pymethods]
impl ModelArmorConfig {
    #[new]
    #[pyo3(signature = (prompt_template_name=None, response_template_name=None))]
    pub fn new(
        prompt_template_name: Option<String>,
        response_template_name: Option<String>,
    ) -> Self {
        ModelArmorConfig {
            prompt_template_name,
            response_template_name,
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Mode {
    ModeUnspecified,
    Any,
    #[default]
    Auto,
    #[pyo3(name = "None_Mode")]
    None,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct FunctionCallingConfig {
    #[pyo3(get)]
    pub mode: Option<Mode>,
    #[pyo3(get)]
    pub allowed_function_names: Option<Vec<String>>,
}

#[pymethods]
impl FunctionCallingConfig {
    #[new]
    pub fn new(mode: Option<Mode>, allowed_function_names: Option<Vec<String>>) -> Self {
        FunctionCallingConfig {
            mode,
            allowed_function_names,
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct LatLng {
    #[pyo3(get)]
    pub latitude: f64,
    #[pyo3(get)]
    pub longitude: f64,
}

#[pymethods]
impl LatLng {
    #[new]
    pub fn new(latitude: f64, longitude: f64) -> Self {
        LatLng {
            latitude,
            longitude,
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct RetrievalConfig {
    #[pyo3(get)]
    pub lat_lng: LatLng,

    #[pyo3(get)]
    pub language_code: String,
}

#[pymethods]
impl RetrievalConfig {
    #[new]
    pub fn new(lat_lng: LatLng, language_code: String) -> Self {
        RetrievalConfig {
            lat_lng,
            language_code,
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ToolConfig {
    #[pyo3(get)]
    function_calling_config: Option<FunctionCallingConfig>,
    #[pyo3(get)]
    retrieval_config: Option<RetrievalConfig>,
}

#[pymethods]
impl ToolConfig {
    #[new]
    #[pyo3(signature = (function_calling_config=None, retrieval_config=None))]
    pub fn new(
        function_calling_config: Option<FunctionCallingConfig>,
        retrieval_config: Option<RetrievalConfig>,
    ) -> Self {
        ToolConfig {
            function_calling_config,
            retrieval_config,
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct GeminiSettings {
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_config: Option<ToolConfig>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_settings: Option<Vec<SafetySetting>>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_armor_config: Option<ModelArmorConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_body: Option<Value>,
}

#[pymethods]
impl GeminiSettings {
    #[new]
    #[pyo3(signature = (labels=None, tool_config=None, generation_config=None, safety_settings=None, model_armor_config=None, extra_body=None))]
    pub fn new(
        labels: Option<HashMap<String, String>>,
        tool_config: Option<ToolConfig>,
        generation_config: Option<GenerationConfig>,
        safety_settings: Option<Vec<SafetySetting>>,
        model_armor_config: Option<ModelArmorConfig>,
        extra_body: Option<&Bound<'_, PyAny>>,
    ) -> Result<Self, UtilError> {
        let extra = match extra_body {
            Some(obj) => Some(pyobject_to_json(obj)?),
            None => None,
        };

        Ok(GeminiSettings {
            labels,
            tool_config,
            generation_config,
            safety_settings,
            model_armor_config,
            extra_body: extra,
        })
    }

    #[getter]
    pub fn extra_body<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Option<Bound<'py, PyDict>>, UtilError> {
        // error if extra body is None
        self.extra_body
            .as_ref()
            .map(|v| {
                let pydict = PyDict::new(py);
                json_to_pydict(py, v, &pydict)
            })
            .transpose()
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }

    pub fn model_dump<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyDict>, TypeError> {
        // iterate over each field in model_settings and add to the dict if it is not None
        let json = serde_json::to_value(self)?;
        let pydict = PyDict::new(py);
        json_to_pydict(py, &json, &pydict)?;
        Ok(pydict)
    }

    pub fn settings_type(&self) -> SettingsType {
        SettingsType::GoogleChat
    }
}

impl GeminiSettings {
    pub fn configure_for_structured_output(&mut self) {
        // Ensure generation_config exists and set response_mime_type
        match self.generation_config.as_mut() {
            Some(generation_config) => {
                generation_config.response_mime_type = Some("application/json".to_string());
            }
            None => {
                self.generation_config = Some(GenerationConfig {
                    response_mime_type: Some("application/json".to_string()),
                    ..Default::default()
                });
            }
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Language {
    LanguageUnspecified,
    Python,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Outcome {
    OutcomeUnspecified,
    OutcomeOk,
    OutcomeFailed,
    OutcomeDeadlineExceeded,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct FileData {
    /// Required. The IANA standard MIME type of the source data.
    pub mime_type: String,
    /// Required. URI.
    pub file_uri: String,
    /// Optional. Display name of the file data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct FunctionCall {
    /// Required. The name of the function to call.
    pub name: String,
    /// Optional. The function parameters and values in JSON object format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<HashMap<String, Value>>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct Blob {
    /// Required. The IANA standard MIME type of the source data.
    pub mime_type: String,
    /// Required. Raw bytes, base64-encoded.
    pub data: String,
    /// Optional. Display name of the blob.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}
#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct FunctionResponse {
    /// Required. The name of the function that was called.
    pub name: String,
    /// Required. The function response in JSON object format.
    pub response: HashMap<String, Value>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExecutableCode {
    /// Required. Programming language of the code.
    pub language: Language,
    /// Required. The code to be executed.
    pub code: String,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CodeExecutionResult {
    /// Required. Outcome of the code execution.
    pub outcome: Outcome,
    /// Optional. Contains stdout or stderr.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct VideoMetadata {
    /// Optional. The start offset of the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_offset: Option<String>,
    /// Optional. The end offset of the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_offset: Option<String>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_data: Option<Blob>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_data: Option<FileData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_response: Option<FunctionResponse>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable_code: Option<ExecutableCode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_execution_result: Option<CodeExecutionResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_metadata: Option<VideoMetadata>,
}

#[pymethods]
impl Part {
    #[new]
    #[pyo3(signature = (text=None, inline_data=None, file_data=None, function_call=None, function_response=None, executable_code=None, code_execution_result=None, video_metadata=None))]
    pub fn new(
        text: Option<String>,
        inline_data: Option<Blob>,
        file_data: Option<FileData>,
        function_call: Option<FunctionCall>,
        function_response: Option<FunctionResponse>,
        executable_code: Option<ExecutableCode>,
        code_execution_result: Option<CodeExecutionResult>,
        video_metadata: Option<VideoMetadata>,
    ) -> Self {
        Part {
            text,
            inline_data,
            file_data,
            function_call,
            function_response,
            executable_code,
            code_execution_result,
            video_metadata,
        }
    }
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct Content {
    /// Optional. The producer of the content. Must be either 'user' or 'model'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Required. Ordered Parts that constitute a single message.
    pub parts: Vec<Part>,
}

#[pymethods]
impl Content {
    #[new]
    #[pyo3(signature = (role=None, parts=vec![]))]
    pub fn new(role: Option<String>, parts: Vec<Part>) -> Self {
        Content { role, parts }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[pyclass]
#[serde(rename_all = "camelCase", default)]
pub struct FunctionDeclaration {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Schema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters_json_schema: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Schema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_json_schema: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct DataStoreSpec {
    pub data_store: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct VertexAISearch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datastore: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_store_specs: Option<Vec<DataStoreSpec>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct VertexRagStore {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rag_resources: Option<Vec<RagResource>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rag_retrieval_config: Option<RagRetrievalConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similarity_top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_distance_threshold: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct RagResource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rag_corpus: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rag_file_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct RagRetrievalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Filter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranking: Option<Ranking>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Filter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_distance_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_similarity_threshold: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct RankService {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct LlmRanker {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum RankingConfig {
    RankService(RankService),
    LlmRanker(LlmRanker),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Ranking {
    #[serde(flatten)]
    pub ranking_config: RankingConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiSpecType {
    ApiSpecUnspecified,
    SimpleSearch,
    ElasticSearch,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SimpleSearchParams {}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct ElasticSearchParams {
    pub index: String,
    pub search_template: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_hits: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum ExternalApiParams {
    SimpleSearchParams(SimpleSearchParams),
    ElasticSearchParams(ElasticSearchParams),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthType {
    AuthTypeUnspecified,
    NoAuth,
    ApiKeyAuth,
    HttpBasicAuth,
    GoogleServiceAccountAuth,
    Oauth,
    OidcAuth,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HttpElementLocation {
    HttpInUnspecified,
    HttpInQuery,
    HttpInHeader,
    HttpInPath,
    HttpInBody,
    HttpInCookie,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct ApiKeyConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_string: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_element_location: Option<HttpElementLocation>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HttpBasicAuthConfig {
    pub credential_secret: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct GoogleServiceAccountConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_account: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum OauthConfigValue {
    AccessToken(String),
    ServiceAccount(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OauthConfig {
    #[serde(flatten)]
    pub oauth_config: OauthConfigValue,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum OidcConfigValue {
    IdToken(String),
    ServiceAccount(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OidcConfig {
    #[serde(flatten)]
    pub oidc_config: OidcConfigValue,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum AuthConfigValue {
    ApiKeyConfig(ApiKeyConfig),
    HttpBasicAuthConfig(HttpBasicAuthConfig),
    GoogleServiceAccountConfig(GoogleServiceAccountConfig),
    OauthConfig(OauthConfig),
    OidcConfig(OidcConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfig {
    pub auth_type: AuthType,
    #[serde(flatten)]
    pub auth_config: AuthConfigValue,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExternalApi {
    pub api_spec: ApiSpecType,
    pub endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_config: Option<AuthConfig>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub params: Option<ExternalApiParams>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum RetrievalSource {
    VertexAiSearch(VertexAISearch),
    VertexRagStore(VertexRagStore),
    ExternalApi(ExternalApi),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Retrieval {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_attribution: Option<bool>,
    #[serde(flatten)]
    pub source: RetrievalSource,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
#[pyclass]
pub struct Tool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_declarations: Option<Vec<FunctionDeclaration>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval: Option<Retrieval>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search: Option<GoogleSearch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search_retrieval: Option<GoogleSearchRetrieval>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_maps: Option<GoogleMaps>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise_web_search: Option<EnterpriseWebSearch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_execution: Option<CodeExecution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_context: Option<UrlContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computer_use: Option<ComputerUse>,
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct GeminiGenerateContentRequest {
    pub contents: Vec<Content>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub settings: Option<GeminiSettings>,
}
