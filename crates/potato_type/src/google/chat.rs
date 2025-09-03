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
#[pyclass]
pub struct ThinkingConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_thoughts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_budget: Option<i32>,
}

#[pymethods]
impl ThinkingConfig {
    #[new]
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
    pub labels: Option<HashMap<String, String>>,
    #[pyo3(get)]
    pub tool_config: Option<ToolConfig>,
    #[pyo3(get)]
    pub generation_config: Option<GenerationConfig>,
    #[pyo3(get)]
    pub safety_settings: Option<Vec<SafetySetting>>,
    #[pyo3(get)]
    pub model_armor_config: Option<ModelArmorConfig>,
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
}
