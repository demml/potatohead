use crate::{SettingsType, TypeError};
use potato_util::{json_to_pydict, pyobject_to_json, PyHelperFuncs, UtilError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::types::PyString;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use serde_json::Value;
use std::collections::HashMap;

// API Reference:
// Note - This is an attempt at combining both the Gemini and Vertex API specs as they are largely the same
// I'm not sure why google decided on the the anti-pattern of having two separate APIs for what is effectively the same thing, but here we are
//https://cloud.google.com/vertex-ai/generative-ai/docs/reference/rest/v1beta1/Content
//https://docs.cloud.google.com/vertex-ai/generative-ai/docs/reference/rest/v1/projects.locations.endpoints/generateContent

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[pyclass]
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
#[pyclass]
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
    #[serde(rename = "enum")]
    pub r#enum: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Schema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<String>,

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
    pub property_ordering: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
}

#[pymethods]
impl Schema {
    #[new]
    #[pyo3(signature = (type_=None, format=None, title=None, description=None, nullable=None, enum_=None, max_items=None, min_items=None, properties=None, required=None, min_properties=None, max_properties=None, min_length=None, max_length=None, pattern=None, example=None, any_of=None, property_ordering=None, default=None, items=None, minimum=None, maximum=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        type_: Option<SchemaType>,
        format: Option<String>,
        title: Option<String>,
        description: Option<String>,
        nullable: Option<bool>,
        enum_: Option<Vec<String>>,
        max_items: Option<String>,
        min_items: Option<String>,
        properties: Option<HashMap<String, Schema>>,
        required: Option<Vec<String>>,
        min_properties: Option<String>,
        max_properties: Option<String>,
        min_length: Option<String>,
        max_length: Option<String>,
        pattern: Option<String>,
        example: Option<Bound<'_, PyAny>>,
        any_of: Option<Vec<Schema>>,
        property_ordering: Option<Vec<String>>,
        default: Option<Bound<'_, PyAny>>,
        items: Option<Schema>,
        minimum: Option<f64>,
        maximum: Option<f64>,
    ) -> Self {
        let example = match example {
            Some(e) => Some(pyobject_to_json(&e).unwrap_or(Value::Null)),
            None => None,
        };

        let default = match default {
            Some(d) => Some(pyobject_to_json(&d).unwrap_or(Value::Null)),
            None => None,
        };

        // need to add a Box to items
        let items = match items {
            Some(i) => Some(Box::new(i)),
            None => None,
        };

        Schema {
            r#type: type_,
            format,
            title,
            description,
            nullable,
            r#enum: enum_,
            max_items,
            min_items,
            properties,
            required,
            min_properties,
            max_properties,
            min_length,
            max_length,
            pattern,
            example,
            any_of,
            property_ordering,
            default,
            items,
            minimum,
            maximum,
        }
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmCategory {
    #[default]
    HarmCategoryUnspecified,
    HarmCategoryDerogatory,
    HarmCategoryToxicity,
    HarmCategoryViolence,
    HarmCategorySexual,
    HarmCategoryMedical,
    HarmCategoryDangerous,
    HarmCategoryHarassment,
    HarmCategoryHateSpeech,
    HarmCategorySexuallyExplicit,
    HarmCategoryDangerousContent,
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
}

#[pymethods]
impl SafetySetting {
    #[new]
    #[pyo3(signature = (category, threshold))]
    pub fn new(category: HarmCategory, threshold: HarmBlockThreshold) -> Self {
        SafetySetting {
            category,
            threshold,
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
#[pyclass]
pub enum ModelRoutingPreference {
    Unknown,
    PrioritizeQuality,
    Balanced,
    PrioritizeCost,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[pyclass]
pub enum ThinkingLevel {
    ThinkingLevelUnspecified,
    Low,
    High,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
#[pyclass(name = "GeminiThinkingConfig")]
pub struct ThinkingConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_thoughts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_budget: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_level: Option<ThinkingLevel>,
}

#[pymethods]
impl ThinkingConfig {
    #[new]
    #[pyo3(signature = (include_thoughts=None, thinking_budget=None, thinking_level=None))]
    pub fn new(
        include_thoughts: Option<bool>,
        thinking_budget: Option<i32>,
        thinking_level: Option<ThinkingLevel>,
    ) -> Self {
        ThinkingConfig {
            include_thoughts,
            thinking_budget,
            thinking_level,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
#[pyclass(name = "GeminiImageConfig")]
pub struct ImageConfig {
    pub aspect_ratio: Option<String>,
    pub image_size: Option<String>,
}

#[pymethods]
impl ImageConfig {
    #[new]
    #[pyo3(signature = (aspect_ratio=None, image_size=None))]
    pub fn new(aspect_ratio: Option<String>, image_size: Option<String>) -> Self {
        ImageConfig {
            aspect_ratio,
            image_size,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct AutoRoutingMode {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_routing_preference: Option<ModelRoutingPreference>,
}

#[pymethods]
impl AutoRoutingMode {
    #[new]
    #[pyo3(signature = (model_routing_preference=None))]
    pub fn new(model_routing_preference: Option<ModelRoutingPreference>) -> Self {
        AutoRoutingMode {
            model_routing_preference,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct ManualRoutingMode {
    pub model_name: String,
}

#[pymethods]
impl ManualRoutingMode {
    #[new]
    pub fn new(model_name: String) -> Self {
        ManualRoutingMode { model_name }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
#[pyclass]
pub enum RoutingConfigMode {
    AutoMode(AutoRoutingMode),
    ManualMode(ManualRoutingMode),
}

#[pymethods]
impl RoutingConfigMode {
    #[new]
    #[pyo3(signature = (auto_mode=None, manual_mode=None))]
    pub fn new(
        auto_mode: Option<AutoRoutingMode>,
        manual_mode: Option<ManualRoutingMode>,
    ) -> Result<Self, TypeError> {
        match (auto_mode, manual_mode) {
            (Some(auto), None) => Ok(RoutingConfigMode::AutoMode(auto)),
            (None, Some(manual)) => Ok(RoutingConfigMode::ManualMode(manual)),
            _ => Err(TypeError::MissingRoutingConfigMode),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct RoutingConfig {
    #[serde(flatten)]
    pub routing_config: RoutingConfigMode,
}

#[pymethods]
impl RoutingConfig {
    #[new]
    pub fn new(routing_config: RoutingConfigMode) -> Self {
        RoutingConfig { routing_config }
    }
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
#[pyclass]
pub struct VoiceConfig {
    pub prebuilt_voice_config: PrebuiltVoiceConfig,
}

#[pymethods]
impl VoiceConfig {
    #[new]
    pub fn new(prebuilt_voice_config: PrebuiltVoiceConfig) -> Self {
        VoiceConfig {
            prebuilt_voice_config,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct SpeakerVoiceConfig {
    pub speaker: String,
    pub voice_config: VoiceConfig,
}

#[pymethods]
impl SpeakerVoiceConfig {
    #[new]
    pub fn new(speaker: String, voice_config: VoiceConfig) -> Self {
        SpeakerVoiceConfig {
            speaker,
            voice_config,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct MultiSpeakerVoiceConfig {
    pub speaker_voice_configs: Vec<SpeakerVoiceConfig>,
}

#[pymethods]
impl MultiSpeakerVoiceConfig {
    #[new]
    pub fn new(speaker_voice_configs: Vec<SpeakerVoiceConfig>) -> Self {
        MultiSpeakerVoiceConfig {
            speaker_voice_configs,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
#[pyclass]
pub struct SpeechConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_config: Option<VoiceConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_speaker_voice_config: Option<MultiSpeakerVoiceConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

#[pymethods]
impl SpeechConfig {
    #[new]
    pub fn new(
        voice_config: Option<VoiceConfig>,
        multi_speaker_voice_config: Option<MultiSpeakerVoiceConfig>,
        language_code: Option<String>,
    ) -> Self {
        SpeechConfig {
            voice_config,
            multi_speaker_voice_config,
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
    pub response_schema: Option<Schema>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_responseJsonSchema")]
    pub underscore_response_json_schema: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_json_schema: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub response_modalities: Option<Vec<Modality>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub candidate_count: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub max_output_tokens: Option<i32>,

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
    pub presence_penalty: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub frequency_penalty: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub seed: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub response_logprobs: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub logprobs: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_enhanced_civic_answers: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub speech_config: Option<SpeechConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub thinking_config: Option<ThinkingConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub image_config: Option<ImageConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub media_resolution: Option<MediaResolution>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub audio_timestamp: Option<bool>,

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
    Validated,
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
    pub function_calling_config: Option<FunctionCallingConfig>,
    #[pyo3(get)]
    pub retrieval_config: Option<RetrievalConfig>,
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
    pub safety_settings: Option<Vec<SafetySetting>>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_armor_config: Option<ModelArmorConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_body: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_content: Option<String>,
}

#[pymethods]
impl GeminiSettings {
    #[new]
    #[pyo3(signature = (labels=None, tool_config=None, generation_config=None, safety_settings=None, model_armor_config=None, extra_body=None, cached_content=None))]
    pub fn new(
        labels: Option<HashMap<String, String>>,
        tool_config: Option<ToolConfig>,
        generation_config: Option<GenerationConfig>,
        safety_settings: Option<Vec<SafetySetting>>,
        model_armor_config: Option<ModelArmorConfig>,
        extra_body: Option<&Bound<'_, PyAny>>,
        cached_content: Option<String>,
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
            cached_content,
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
pub struct PartialArgs {
    pub json_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_continue: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub null_value: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bool_value: Option<bool>,
}

#[pymethods]
impl PartialArgs {
    #[new]
    #[pyo3(signature = (json_path, will_continue=None, null_value=None, number_value=None, string_value=None, bool_value=None))]
    pub fn new(
        json_path: String,
        will_continue: Option<bool>,
        null_value: Option<bool>,
        number_value: Option<i64>,
        string_value: Option<String>,
        bool_value: Option<bool>,
    ) -> Self {
        PartialArgs {
            json_path,
            will_continue,
            null_value,
            number_value,
            string_value,
            bool_value,
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct FunctionCall {
    /// Required. The name of the function to call.
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Optional. The function parameters and values in JSON object format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_continue: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_args: Option<Vec<PartialArgs>>,
}

#[pymethods]
impl FunctionCall {
    #[new]
    #[pyo3(signature = (name, id=None, args=None, will_continue=None, partial_args=None))]
    pub fn new(
        name: String,
        id: Option<String>,
        args: Option<&Bound<'_, PyDict>>,
        will_continue: Option<bool>,
        partial_args: Option<Vec<PartialArgs>>,
    ) -> Self {
        let args = match args {
            Some(dict) => {
                let json_value = pyobject_to_json(dict).unwrap_or(Value::Null);
                if let Value::Object(map) = json_value {
                    Some(map)
                } else {
                    None
                }
            }
            None => None,
        };
        FunctionCall {
            name,
            id,
            args,
            will_continue,
            partial_args,
        }
    }
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
#[serde(rename_all = "camelCase")]
pub struct ExecutableCode {
    /// Required. Programming language of the code.
    pub language: Language,
    /// Required. The code to be executed.
    pub code: String,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct VideoMetadata {
    /// Optional. The start offset of the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_offset: Option<String>,
    /// Optional. The end offset of the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_offset: Option<String>,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct PartMetadata {
    #[serde(flatten)]
    pub r#struct: Map<String, Value>,
}

#[pymethods]
impl PartMetadata {
    #[new]
    #[pyo3(signature = (struct_=None))]
    pub fn new(struct_: Option<&Bound<'_, PyDict>>) -> Result<Self, TypeError> {
        let struct_map = match struct_ {
            Some(dict) => {
                let json_value = pyobject_to_json(dict)?;
                if let Value::Object(map) = json_value {
                    map
                } else {
                    Map::new()
                }
            }
            None => Map::new(),
        };
        Ok(PartMetadata {
            r#struct: struct_map,
        })
    }
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum DataNum {
    Text(String),
    InlineData(Blob),
    FileData(FileData),
    FunctionCall(FunctionCall),
    FunctionResponse(FunctionResponse),
    ExecutableCode(ExecutableCode),
    CodeExecutionResult(CodeExecutionResult),
}

// helper for extracting data from PyAny to DataNum
fn extract_data_from_py_object(data: &Bound<'_, PyAny>) -> Result<DataNum, TypeError> {
    if data.is_instance_of::<PyString>() {
        return Ok(DataNum::Text(data.extract::<String>()?));
    }

    // Check for native Rust types wrapped in PyO3
    if data.is_instance_of::<Blob>() {
        return Ok(DataNum::InlineData(data.extract::<Blob>()?));
    }

    if data.is_instance_of::<FileData>() {
        return Ok(DataNum::FileData(data.extract::<FileData>()?));
    }

    if data.is_instance_of::<FunctionCall>() {
        return Ok(DataNum::FunctionCall(data.extract::<FunctionCall>()?));
    }

    if data.is_instance_of::<FunctionResponse>() {
        return Ok(DataNum::FunctionResponse(
            data.extract::<FunctionResponse>()?,
        ));
    }

    if data.is_instance_of::<ExecutableCode>() {
        return Ok(DataNum::ExecutableCode(data.extract::<ExecutableCode>()?));
    }

    if data.is_instance_of::<CodeExecutionResult>() {
        return Ok(DataNum::CodeExecutionResult(
            data.extract::<CodeExecutionResult>()?,
        ));
    }

    // If none of the above, return an error
    Err(TypeError::InvalidDataType(
        data.get_type().name()?.to_string(),
    ))
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought_signature: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_metadata: Option<PartMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_resolution: Option<MediaResolution>,

    #[serde(flatten)]
    pub data: DataNum,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_metadata: Option<VideoMetadata>,
}

#[pymethods]
impl Part {
    #[new]
    #[pyo3(signature = (data, thought=None, thought_signature=None, part_metadata=None, media_resolution=None,  video_metadata=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        data: &Bound<'_, PyAny>,
        thought: Option<bool>,
        thought_signature: Option<String>,
        part_metadata: Option<PartMetadata>,
        media_resolution: Option<MediaResolution>,
        video_metadata: Option<VideoMetadata>,
    ) -> Result<Self, TypeError> {
        let data_enum = extract_data_from_py_object(data)?;

        Ok(Part {
            thought,
            thought_signature,
            part_metadata,
            media_resolution,
            data: data_enum,
            video_metadata,
        })
    }
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Content {
    /// Optional. The producer of the content. Must be either 'user' or 'model'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Required. Ordered Parts that constitute a single message.
    pub parts: Vec<Part>,
}

// Helper function to extract parts from a PyAny object
fn extract_parts_from_py_object(parts: &Bound<'_, PyAny>) -> Result<Vec<Part>, TypeError> {
    use pyo3::types::{PyList, PyString};

    // Check for String first - most common case, convert to text Part
    if parts.is_instance_of::<PyString>() {
        let text = parts.extract::<String>()?;
        return Ok(vec![Part {
            data: DataNum::Text(text),
            thought: None,
            thought_signature: None,
            part_metadata: None,
            media_resolution: None,
            video_metadata: None,
        }]);
    }

    // Check for single Part instance
    if parts.is_instance_of::<Part>() {
        return Ok(vec![parts.extract::<Part>()?]);
    }

    // Check for DataNum variants and wrap in Part
    if let Ok(data_num) = extract_data_from_py_object(parts) {
        return Ok(vec![Part {
            data: data_num,
            thought: None,
            thought_signature: None,
            part_metadata: None,
            media_resolution: None,
            video_metadata: None,
        }]);
    }

    // Check for PyList - can contain Parts, DataNum variants, or strings
    if parts.is_instance_of::<PyList>() {
        let list = parts.cast::<PyList>()?;
        let mut part_vec = Vec::with_capacity(list.len());

        for item in list.iter() {
            // Try to extract as Part first
            if item.is_instance_of::<Part>() {
                part_vec.push(item.extract::<Part>()?);
            }
            // Try to extract as String
            else if item.is_instance_of::<PyString>() {
                let text = item.extract::<String>()?;
                part_vec.push(Part {
                    data: DataNum::Text(text),
                    thought: None,
                    thought_signature: None,
                    part_metadata: None,
                    media_resolution: None,
                    video_metadata: None,
                });
            }
            // Try to extract as DataNum variant
            else if let Ok(data_num) = extract_data_from_py_object(&item) {
                part_vec.push(Part {
                    data: data_num,
                    thought: None,
                    thought_signature: None,
                    part_metadata: None,
                    media_resolution: None,
                    video_metadata: None,
                });
            } else {
                return Err(TypeError::InvalidListType(
                    item.get_type().name()?.to_string(),
                ));
            }
        }
        return Ok(part_vec);
    }

    Err(TypeError::InvalidPartType)
}

#[pymethods]
impl Content {
    /// Create a new Content instance.
    ///
    /// # Arguments
    /// * `role` - Optional role string ('user' or 'model')
    /// * `parts` - Can be:
    ///   - A string (converted to a text Part)
    ///   - A single Part instance
    ///   - A DataNum variant (Blob, FileData, FunctionCall, etc.)
    ///   - A list containing any combination of the above
    ///
    /// # Examples
    /// ```python
    /// # Simple text message
    /// content = Content(role="user", parts="Hello, world!")
    ///
    /// # Multiple parts
    /// content = Content(role="user", parts=[
    ///     "Check this image:",
    ///     Blob(mime_type="image/png", data="base64data...")
    /// ])
    ///
    /// # Single Part with metadata
    /// part = Part(data="Hello", thought=True)
    /// content = Content(role="user", parts=part)
    /// ```
    #[new]
    #[pyo3(signature = (parts, role=None))]
    pub fn new(parts: &Bound<'_, PyAny>, role: Option<String>) -> PyResult<Self> {
        let parts_vec = extract_parts_from_py_object(parts)?;
        Ok(Content {
            role,
            parts: parts_vec,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[pyclass]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Behavior {
    #[default]
    Unspecified,
    Blocking,
    NonBlocking,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[pyclass]
#[serde(rename_all = "camelCase", default)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior: Option<Behavior>,
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct Interval {
    #[pyo3(get)]
    pub start_time: String,
    #[pyo3(get)]
    pub end_time: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct GoogleSearch {
    #[pyo3(get)]
    pub time_range_filter: Interval,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[pyclass]
pub enum PhishBlockThreshold {
    PhishBlockThresholdUnspecified,
    BlockLowAndAbove,
    BlockMediumAndAbove,
    BlockHighAndAbove,
    BlockHigherAndAbove,
    BlockVeryHighAndAbove,
    BlockOnlyExtremelyHigh,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct VertexGoogleSearch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_domains: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking_confidence: Option<PhishBlockThreshold>,
}

#[pymethods]
impl VertexGoogleSearch {
    #[new]
    #[pyo3(signature = (exclude_domains=None, blocking_confidence=None))]
    pub fn new(
        exclude_domains: Option<Vec<String>>,
        blocking_confidence: Option<PhishBlockThreshold>,
    ) -> Self {
        VertexGoogleSearch {
            exclude_domains,
            blocking_confidence,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct EnterpriseWebSearch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_domains: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking_confidence: Option<PhishBlockThreshold>,
}

#[pymethods]
impl EnterpriseWebSearch {
    #[new]
    #[pyo3(signature = (exclude_domains=None, blocking_confidence=None))]
    pub fn new(
        exclude_domains: Option<Vec<String>>,
        blocking_confidence: Option<PhishBlockThreshold>,
    ) -> Self {
        EnterpriseWebSearch {
            exclude_domains,
            blocking_confidence,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct ParallelAiSearch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_configs: Option<Map<String, Value>>,
}

#[pymethods]
impl ParallelAiSearch {
    #[new]
    #[pyo3(signature = (api_key=None, custom_configs=None))]
    pub fn new(
        api_key: Option<String>,
        custom_configs: Option<&Bound<'_, PyDict>>,
    ) -> Result<Self, TypeError> {
        let custom_configs_map = match custom_configs {
            Some(dict) => {
                let json_value = pyobject_to_json(dict)?;
                if let Value::Object(map) = json_value {
                    Some(map)
                } else {
                    None
                }
            }
            None => None,
        };
        Ok(ParallelAiSearch {
            api_key,
            custom_configs: custom_configs_map,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
#[pyclass]
pub enum GoogleSearchNum {
    GeminiSearch(GoogleSearch),
    VertexSearch(VertexGoogleSearch),
}

#[pymethods]
impl GoogleSearchNum {
    #[new]
    #[pyo3(signature = (gemini_search=None, vertex_search=None))]
    pub fn new(
        gemini_search: Option<GoogleSearch>,
        vertex_search: Option<VertexGoogleSearch>,
    ) -> Self {
        match (gemini_search, vertex_search) {
            (Some(gemini), None) => GoogleSearchNum::GeminiSearch(gemini),
            (None, Some(vertex)) => GoogleSearchNum::VertexSearch(vertex),
            _ => panic!("Exactly one of gemini_search or vertex_search must be provided"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DynamicRetrievalMode {
    ModeUnspecified,
    ModeDynamic,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct DynamicRetrievalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<DynamicRetrievalMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_threshold: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct GoogleSearchRetrieval {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_retrieval_config: Option<DynamicRetrievalConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct GoogleMaps {
    pub enable_widget: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct CodeExecution {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComputerUseEnvironment {
    EnvironmentUnspecified,
    EnvironmentBrowser,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComputerUse {
    pub environment: ComputerUseEnvironment,
    pub excluded_predefined_functions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UrlContext {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileSearch {
    pub file_search_store_names: Vec<String>,
    pub metadata_filter: String,
    pub top_k: i32,
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
    pub google_search_retrieval: Option<GoogleSearchRetrieval>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_execution: Option<CodeExecution>,

    // for some reason, each api has a different definition of Google Search...cool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search: Option<GoogleSearchNum>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_maps: Option<GoogleMaps>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise_web_search: Option<EnterpriseWebSearch>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_ai_search: Option<ParallelAiSearch>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub computer_use: Option<ComputerUse>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_context: Option<UrlContext>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_search: Option<FileSearch>,
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
#[pyclass]
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
