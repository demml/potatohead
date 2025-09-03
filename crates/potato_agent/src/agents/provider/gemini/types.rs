use crate::agents::provider::traits::{LogProbExt, ResponseExt, TokenUsage};
use crate::{AgentError, Usage};
use base64::prelude::*;
use potato_prompt::{prompt::types::PromptContent, Message};
use potato_type::google::chat::{
    GenerationConfig, HarmBlockThreshold, HarmCategory, Modality, ModelArmorConfig, SafetySetting,
};
use potato_util::utils::ResponseLogProbs;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

//https://cloud.google.com/vertex-ai/generative-ai/docs/reference/rest/v1beta1/Content

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

impl Part {
    pub fn from_message(message: &Message) -> Result<Self, AgentError> {
        let part = match &message.content {
            PromptContent::Str(text) => Part {
                text: Some(text.clone()),
                ..Default::default()
            },
            PromptContent::Document(doc) => Part {
                file_data: Some(FileData {
                    mime_type: doc.media_type()?,
                    file_uri: doc.url.clone(),
                    display_name: None,
                }),
                ..Default::default()
            },
            PromptContent::Binary(blob) => Part {
                inline_data: Some(Blob {
                    mime_type: blob.media_type.clone(),
                    data: BASE64_STANDARD.encode(blob.data.clone()),
                    display_name: None,
                }),
                ..Default::default()
            },
            // need to implement audio and file for chat
            _ => {
                // Handle other content types as needed
                return Err(AgentError::UnsupportedContentTypeError);
            }
        };

        Ok(part)
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

impl Content {
    pub fn from_message(message: &Message) -> Result<Self, AgentError> {
        let content = match &message.content {
            PromptContent::Str(text) => vec![Part {
                text: Some(text.clone()),
                ..Default::default()
            }],
            PromptContent::Document(doc) => vec![Part {
                file_data: Some(FileData {
                    mime_type: doc.media_type()?,
                    file_uri: doc.url.clone(),
                    display_name: None,
                }),
                ..Default::default()
            }],
            PromptContent::Binary(blob) => vec![Part {
                inline_data: Some(Blob {
                    mime_type: blob.media_type.clone(),
                    data: BASE64_STANDARD.encode(blob.data.clone()),
                    display_name: None,
                }),
                ..Default::default()
            }],
            // need to implement audio and file for chat
            _ => {
                // Handle other content types as needed
                return Err(AgentError::UnsupportedContentTypeError);
            }
        };

        Ok(Content {
            role: Some(message.role.to_string()),
            parts: content,
        })
    }
}

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

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct DataStoreSpec {
    pub data_store: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
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
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiSpecType {
    ApiSpecUnspecified,
    SimpleSearch,
    ElasticSearch,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum ExternalApiParams {
    SimpleSearchParams(SimpleSearchParams),
    ElasticSearchParams(ElasticSearchParams),
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
pub struct GoogleSearch {}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_config: Option<AuthConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EnterpriseWebSearch {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CodeExecution {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UrlContext {}

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
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelRoutingPreference {
    Unknown,
    PrioritizeQuality,
    Balanced,
    PrioritizeCost,
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

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct GeminiGenerateContentRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_armor_config: Option<ModelArmorConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_settings: Option<Vec<SafetySetting>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FinishReason {
    FinishReasonUnspecified,
    Stop,
    MaxTokens,
    Safety,
    Recitation,
    Other,
    Blocklist,
    ProhibitedContent,
    Spii,
    MalformedFunctionCall,
    ImageSafety,
    ImageProhibitedContent,
    ImageRecitation,
    ImageOther,
    UnexpectedToolCall,
}

/// Harm probability levels in the content.
#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmProbability {
    HarmProbabilityUnspecified,
    Negligible,
    Low,
    Medium,
    High,
}

/// Harm severity levels.
#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmSeverity {
    HarmSeverityUnspecified,
    HarmSeverityNegligible,
    HarmSeverityLow,
    HarmSeverityMedium,
    HarmSeverityHigh,
}

/// Blocked reason enumeration.
#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlockedReason {
    BlockedReasonUnspecified,
    Safety,
    Other,
    Blocklist,
    ProhibitedContent,
    ImageSafety,
}

/// Status of the url retrieval.
#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UrlRetrievalStatus {
    UrlRetrievalStatusUnspecified,
    UrlRetrievalStatusSuccess,
    UrlRetrievalStatusError,
}

/// Request traffic type.
#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrafficType {
    TrafficTypeUnspecified,
    OnDemand,
    ProvisionedThroughput,
}

/// Represents a whole or partial calendar date.
#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct GoogleDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

/// Safety rating corresponding to the generated content.
#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct SafetyRating {
    pub category: HarmCategory,
    pub probability: Option<HarmProbability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probability_score: Option<f32>,
    pub severity: Option<HarmSeverity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity_score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overwritten_threshold: Option<HarmBlockThreshold>,
}

/// Source attributions for content.
#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct Citation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_date: Option<GoogleDate>,
}

/// A collection of source attributions for a piece of content.
#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct CitationMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<Citation>>,
}

/// Content filter results for a prompt sent in the request.
#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct PromptFeedback {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_reason: Option<BlockedReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_ratings: Option<Vec<SafetyRating>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_reason_message: Option<String>,
}

/// Represents token counting info for a single modality.
#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct ModalityTokenCount {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modality: Option<Modality>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_count: Option<i32>,
}

/// Usage metadata about response(s).
#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct UsageMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_token_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates_token_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_prompt_token_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thoughts_token_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_token_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_content_token_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<Vec<ModalityTokenCount>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_tokens_details: Option<Vec<ModalityTokenCount>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates_tokens_details: Option<Vec<ModalityTokenCount>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_prompt_tokens_details: Option<Vec<ModalityTokenCount>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traffic_type: Option<TrafficType>,
}

impl TokenUsage for UsageMetadata {
    /// Returns the total token count across all modalities.
    fn total_tokens(&self) -> u64 {
        let token_count = self.total_token_count.unwrap_or(0);

        token_count as u64
    }

    fn prompt_tokens(&self) -> u64 {
        self.prompt_token_count.unwrap_or(0) as u64
    }

    fn completion_tokens(&self) -> u64 {
        self.candidates_token_count.unwrap_or(0) as u64
    }
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct LogprobsCandidate {
    /// The candidate's token string value.
    pub token: Option<String>,
    /// The candidate's token id value.
    pub token_id: Option<i32>,
    /// The candidate's log probability.
    pub log_probability: Option<f64>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct TopCandidates {
    /// Sorted by log probability in descending order.
    pub candidates: Option<Vec<LogprobsCandidate>>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct LogprobsResult {
    /// Length = total number of decoding steps.
    pub top_candidates: Option<Vec<TopCandidates>>,
    /// Length = total number of decoding steps. The chosen candidates may or may not be in topCandidates.
    pub chosen_candidates: Option<Vec<LogprobsCandidate>>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct SearchEntryPoint {
    /// Optional. Web content snippet that can be embedded in a web page or an app webview.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rendered_content: Option<String>,
    /// Optional. Base64 encoded JSON representing array of <search term, search url> tuple.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdk_blob: Option<String>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct Web {
    /// URI reference of the chunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// title of the chunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// domain of the (original) URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct RagChunk {}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct RetrievedContext {
    /// URI reference of the attribution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// title of the attribution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Text of the attribution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Additional context for the RAG retrieval result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rag_chunk: Option<RagChunk>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct Maps {
    /// URI reference of the chunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// title of the chunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Text of the chunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// This Place's resource name, in places/{placeId} format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_id: Option<String>,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum GroundingChunkType {
    Web(Web),
    RetrievedContext(RetrievedContext),
    Maps(Maps),
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GroundingChunk {
    #[serde(flatten)]
    pub chunk_type: GroundingChunkType,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct Segment {
    /// Output only. The index of a Part object within its parent Content object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_index: Option<i32>,
    /// Output only. Start index in the given Part, measured in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<i32>,
    /// Output only. End index in the given Part, measured in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<i32>,
    /// Output only. The text corresponding to the segment from the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct GroundingSupport {
    /// A list of indices into 'grounding_chunk' specifying the citations associated with the claim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_chunk_indices: Option<Vec<i32>>,
    /// confidence score of the support references.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_scores: Option<Vec<f32>>,
    /// Segment of the content this support belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment: Option<Segment>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct RetrievalMetadata {
    /// Score indicating how likely information from Google Search could help answer the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search_dynamic_retrieval_score: Option<f64>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct GroundingMetadata {
    /// Optional. Web search queries for the following-up web search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search_queries: Option<Vec<String>>,
    /// List of supporting references retrieved from specified grounding source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_chunks: Option<Vec<GroundingChunk>>,
    /// Optional. List of grounding support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_supports: Option<Vec<GroundingSupport>>,
    /// Optional. Google search entry for the following-up web searches.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_entry_point: Option<SearchEntryPoint>,
    /// Optional. Output only. Retrieval metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval_metadata: Option<RetrievalMetadata>,
    /// Optional. Output only. Resource name of the Google Maps widget context token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_maps_widget_context_token: Option<String>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct UrlMetadata {
    /// Retrieved url by the tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieved_url: Option<String>,
    /// status of the url retrieval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_retrieval_status: Option<UrlRetrievalStatus>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct UrlContextMetadata {
    /// Output only. List of url context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_metadata: Option<Vec<UrlMetadata>>,
}

/// A response candidate generated from the model.
#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Candidate {
    pub index: i32,
    pub content: Content,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_logprobs: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs_result: Option<LogprobsResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_ratings: Option<Vec<SafetyRating>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_metadata: Option<CitationMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_metadata: Option<GroundingMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_context_metadata: Option<UrlContextMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_message: Option<String>,
}

/// Response message for GenerateContent.
#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct GenerateContentResponse {
    pub candidates: Vec<Candidate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_feedback: Option<PromptFeedback>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_metadata: Option<UsageMetadata>,
}

impl GenerateContentResponse {
    /// Returns the first candidate's content text if available.
    pub fn get_content(&self) -> Option<String> {
        self.candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .and_then(|part| part.text.clone())
    }

    /// Returns the usage metadata for the response.
    pub fn get_token_usage(&self) -> Usage {
        let usage_metadata = self.usage_metadata.as_ref();

        if let Some(usage_metadata) = usage_metadata {
            Usage {
                completion_tokens: usage_metadata.completion_tokens(),
                prompt_tokens: usage_metadata.prompt_tokens(),
                total_tokens: usage_metadata.total_tokens(),
                ..Default::default()
            }
        } else {
            Usage::default()
        }
    }
}

impl ResponseExt for GenerateContentResponse {
    fn get_content(&self) -> Option<String> {
        self.candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .and_then(|part| part.text.clone())
    }
}

impl LogProbExt for GenerateContentResponse {
    fn get_log_probs(&self) -> Vec<ResponseLogProbs> {
        let mut probabilities = Vec::new();

        if let Some(candidate) = self.candidates.first() {
            if let Some(logprobs_result) = &candidate.logprobs_result {
                if let Some(chosen_candidates) = &logprobs_result.chosen_candidates {
                    for candidate in chosen_candidates {
                        if let Some(token) = &candidate.token {
                            if token.len() == 1 && token.chars().next().unwrap().is_ascii_digit() {
                                if let Some(logprob) = candidate.log_probability {
                                    probabilities.push(ResponseLogProbs {
                                        token: token.clone(),
                                        logprob,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        probabilities
    }
}
