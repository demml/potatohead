use crate::google::v1::generate::request::Modality;
use crate::google::v1::generate::request::{GeminiContent, HarmBlockThreshold, HarmCategory};
use crate::google::v1::generate::DataNum;
use crate::prompt::{MessageNum, ResponseContent};
use crate::traits::{MessageResponseExt, ResponseAdapter};
use crate::TypeError;
use potato_util::utils::{construct_structured_response, TokenLogProbs};
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::fmt::Display;

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrafficType {
    TrafficTypeUnspecified,
    OnDemand,
    ProvisionedThroughput,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModalityTokenCount {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modality: Option<Modality>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_count: Option<i32>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
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

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlockedReason {
    BlockedReasonUnspecified,
    Safety,
    Other,
    Blocklist,
    ModelArmor,
    ProhibitedContent,
    ImageSafety,
    Jailbreak,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PromptFeedback {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_reason: Option<BlockedReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_ratings: Option<Vec<SafetyRating>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_reason_message: Option<String>,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UrlRetrievalStatus {
    UrlRetrievalStatusUnspecified,
    UrlRetrievalStatusSuccess,
    UrlRetrievalStatusError,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UrlContextMetadata {
    /// Output only. List of url context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_metadata: Option<Vec<UrlMetadata>>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SourceFlaggingUri {
    pub source_id: String,
    pub flag_content_uri: String,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalMetadata {
    /// Score indicating how likely information from Google Search could help answer the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search_dynamic_retrieval_score: Option<f64>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PageSpan {
    pub first_page: i32,
    pub last_page: i32,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RagChunk {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_span: Option<PageSpan>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievedContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rag_chunk: Option<RagChunk>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Maps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_id: Option<String>,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum GroundingChunkType {
    Web(Web),
    RetrievedContext(RetrievedContext),
    Maps(Maps),
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GroundingChunk {
    #[serde(flatten)]
    pub chunk_type: GroundingChunkType,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GroundingMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search_queries: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_chunks: Option<Vec<GroundingChunk>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_supports: Option<Vec<GroundingSupport>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_entry_point: Option<SearchEntryPoint>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval_metadata: Option<RetrievalMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_flagging_uris: Option<Vec<SourceFlaggingUri>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_maps_widget_context_token: Option<String>,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmProbability {
    HarmProbabilityUnspecified,
    Negligible,
    Low,
    Medium,
    High,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmSeverity {
    HarmSeverityUnspecified,
    HarmSeverityNegligible,
    HarmSeverityLow,
    HarmSeverityMedium,
    HarmSeverityHigh,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
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

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
    ModelArmor,
    ImageSafety,
    ImageProhibitedContent,
    ImageRecitation,
    ImageOther,
    UnexpectedToolCall,
    NoImage,
}

impl Display for FinishReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let reason_str = match self {
            FinishReason::FinishReasonUnspecified => "FINISH_REASON_UNSPECIFIED",
            FinishReason::Stop => "STOP",
            FinishReason::MaxTokens => "MAX_TOKENS",
            FinishReason::Safety => "SAFETY",
            FinishReason::Recitation => "RECITATION",
            FinishReason::Other => "OTHER",
            FinishReason::Blocklist => "BLOCKLIST",
            FinishReason::ProhibitedContent => "PROHIBITED_CONTENT",
            FinishReason::Spii => "SPII",
            FinishReason::MalformedFunctionCall => "MALFORMED_FUNCTION_CALL",
            FinishReason::ModelArmor => "MODEL_ARMOR",
            FinishReason::ImageSafety => "IMAGE_SAFETY",
            FinishReason::ImageProhibitedContent => "IMAGE_PROHIBITED_CONTENT",
            FinishReason::ImageRecitation => "IMAGE_RECITATION",
            FinishReason::ImageOther => "IMAGE_OTHER",
            FinishReason::UnexpectedToolCall => "UNEXPECTED_TOOL_CALL",
            FinishReason::NoImage => "NO_IMAGE",
        };
        write!(f, "{}", reason_str)
    }
}

impl FinishReason {
    pub fn as_str(&self) -> &str {
        match self {
            Self::FinishReasonUnspecified => "FINISH_REASON_UNSPECIFIED",
            Self::Stop => "STOP",
            Self::MaxTokens => "MAX_TOKENS",
            Self::Safety => "SAFETY",
            Self::Recitation => "RECITATION",
            Self::Other => "OTHER",
            Self::Blocklist => "BLOCKLIST",
            Self::ProhibitedContent => "PROHIBITED_CONTENT",
            Self::Spii => "SPII",
            Self::MalformedFunctionCall => "MALFORMED_FUNCTION_CALL",
            Self::ModelArmor => "MODEL_ARMOR",
            Self::ImageSafety => "IMAGE_SAFETY",
            Self::ImageProhibitedContent => "IMAGE_PROHIBITED_CONTENT",
            Self::ImageRecitation => "IMAGE_RECITATION",
            Self::ImageOther => "IMAGE_OTHER",
            Self::UnexpectedToolCall => "UNEXPECTED_TOOL_CALL",
            Self::NoImage => "NO_IMAGE",
        }
    }
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TopCandidates {
    pub candidates: Option<Vec<LogprobsCandidate>>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LogprobsResult {
    pub top_candidates: Option<Vec<TopCandidates>>,

    pub chosen_candidates: Option<Vec<LogprobsCandidate>>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GoogleDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

/// Source attributions for content.
#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CitationMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<Citation>>,
}

/// A response candidate generated from the model.
#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i32>,
    pub content: GeminiContent,
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

impl MessageResponseExt for Candidate {
    fn to_message_num(&self) -> Result<MessageNum, TypeError> {
        // Convert MessageParam to MessageNum
        Ok(MessageNum::GeminiContentV1(self.content.clone()))
    }
}

/// Response message for GenerateContent.
#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
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

impl ResponseAdapter for GenerateContentResponse {
    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }

    fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    fn to_bound_py_object<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        Ok(PyHelperFuncs::to_bound_py_object(py, self)?)
    }

    fn id(&self) -> &str {
        self.response_id.as_deref().unwrap_or("")
    }

    fn to_message_num(&self) -> Result<Vec<MessageNum>, TypeError> {
        let mut results = Vec::new();
        for choice in &self.candidates {
            match choice.to_message_num() {
                Ok(message_num) => results.push(message_num),
                Err(_) => continue,
            }
        }
        Ok(results)
    }

    fn get_content(&self) -> ResponseContent {
        ResponseContent::Google(self.candidates.first().cloned().unwrap())
    }

    fn tool_call_output(&self) -> Option<Value> {
        let parts = match self.candidates.first().cloned() {
            Some(candidate) => candidate.content.parts,
            None => return None,
        };

        if parts.is_empty() {
            return None;
        }
        let data = parts.first().cloned().unwrap_or_default().data;

        // match on function call data
        match data {
            DataNum::FunctionCall(func_call) => serde_json::to_value(&func_call).ok(),
            _ => None,
        }
    }
    fn structured_output<'py>(
        &self,
        py: Python<'py>,
        output_model: Option<&Bound<'py, PyAny>>,
    ) -> Result<Bound<'py, PyAny>, TypeError> {
        let parts = match self.candidates.first().cloned() {
            Some(candidate) => candidate.content.parts,
            None => return Ok(py.None().into_bound_py_any(py)?),
        };

        if parts.is_empty() {
            return Ok(py.None().into_bound_py_any(py)?);
        }

        let data = parts.first().cloned().unwrap_or_default().data;

        match data {
            DataNum::Text(text) => Ok(construct_structured_response(py, text, output_model)?),
            _ => {
                // Non-text content types can't be converted to structured output
                Ok(py.None().into_bound_py_any(py)?)
            }
        }
    }

    fn structured_output_value(&self) -> Option<Value> {
        let parts = match self.candidates.first().cloned() {
            Some(candidate) => candidate.content.parts,
            None => return None,
        };

        if parts.is_empty() {
            return None;
        }
        let data = parts.first().cloned().unwrap_or_default().data;
        match data {
            DataNum::Text(text) => serde_json::from_str(&text).ok(),
            _ => None,
        }
    }

    /// Returns the total token count across all modalities.
    fn usage<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        Ok(PyHelperFuncs::to_bound_py_object(py, &self.usage_metadata)?)
    }

    fn get_log_probs(&self) -> Vec<TokenLogProbs> {
        let mut probabilities = Vec::new();
        if let Some(choice) = self.candidates.first() {
            if let Some(logprobs_result) = &choice.logprobs_result {
                if let Some(chosen_candidates) = &logprobs_result.chosen_candidates {
                    for log_content in chosen_candidates {
                        // Look for single digit tokens (1, 2, 3, 4, 5)
                        if let Some(token) = &log_content.token {
                            if token.len() == 1 && token.chars().next().unwrap().is_ascii_digit() {
                                probabilities.push(TokenLogProbs {
                                    token: token.clone(),
                                    logprob: log_content.log_probability.unwrap_or(0.0),
                                });
                            }
                        }
                    }
                }
            }
        }

        probabilities
    }

    fn response_text(&self) -> String {
        let parts = match self.candidates.first().cloned() {
            Some(candidate) => candidate.content.parts,
            None => return String::new(),
        };

        if parts.is_empty() {
            return String::new();
        }
        let data = parts.first().cloned().unwrap_or_default().data;

        match data {
            DataNum::Text(text) => text,
            _ => String::new(),
        }
    }

    fn model_name(&self) -> Option<&str> {
        Some(&self.model_version.as_deref().unwrap_or(""))
    }

    fn finish_reason(&self) -> Option<&str> {
        self.candidates
            .first()
            .and_then(|candidate| candidate.finish_reason.as_ref())
            .map(|reason| reason.as_str())
    }

    fn input_tokens(&self) -> Option<i64> {
        Some(
            self.usage_metadata
                .as_ref()?
                .prompt_token_count
                .unwrap_or(0) as i64,
        )
    }

    fn output_tokens(&self) -> Option<i64> {
        Some(
            self.usage_metadata
                .as_ref()?
                .candidates_token_count
                .unwrap_or(0) as i64,
        )
    }

    fn total_tokens(&self) -> Option<i64> {
        Some(self.usage_metadata.as_ref()?.total_token_count.unwrap_or(0) as i64)
    }

    fn get_tool_calls(&self) -> Vec<crate::tools::ToolCallInfo> {
        let mut tool_calls = Vec::new();
        for candidate in &self.candidates {
            for part in &candidate.content.parts {
                if let DataNum::FunctionCall(call) = &part.data {
                    tool_calls.push(crate::tools::ToolCallInfo {
                        name: call.name.clone(),
                        arguments: call
                            .args
                            .as_ref()
                            .map(|a| serde_json::Value::Object(a.clone()))
                            .unwrap_or_default(),
                        call_id: call.id.clone(),
                        result: None,
                    });
                }
            }
        }
        tool_calls
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::google::v1::generate::request::{FunctionCall, Part};
    use crate::traits::ResponseAdapter;
    use serde_json::Map;

    fn make_text_candidate(text: &str, finish: Option<FinishReason>) -> Candidate {
        Candidate {
            index: Some(0),
            content: GeminiContent {
                role: "model".to_string(),
                parts: vec![Part::from_text(text.to_string())],
            },
            avg_logprobs: None,
            logprobs_result: None,
            finish_reason: finish,
            safety_ratings: None,
            citation_metadata: None,
            grounding_metadata: None,
            url_context_metadata: None,
            finish_message: None,
        }
    }

    fn make_text_response(text: &str) -> GenerateContentResponse {
        GenerateContentResponse {
            candidates: vec![make_text_candidate(text, Some(FinishReason::Stop))],
            model_version: Some("gemini-2.0-flash".to_string()),
            create_time: None,
            response_id: Some("resp-123".to_string()),
            prompt_feedback: None,
            usage_metadata: Some(UsageMetadata {
                prompt_token_count: Some(20),
                candidates_token_count: Some(10),
                tool_use_prompt_token_count: None,
                thoughts_token_count: None,
                total_token_count: Some(30),
                cached_content_token_count: None,
                prompt_tokens_details: None,
                cache_tokens_details: None,
                candidates_tokens_details: None,
                tool_use_prompt_tokens_details: None,
                traffic_type: None,
            }),
        }
    }

    fn make_function_call_response() -> GenerateContentResponse {
        let mut args = Map::new();
        args.insert("location".to_string(), serde_json::json!("NYC"));

        let fc_part = Part {
            data: DataNum::FunctionCall(FunctionCall {
                name: "get_weather".to_string(),
                id: Some("fc_01".to_string()),
                args: Some(args),
                will_continue: None,
                partial_args: None,
            }),
            ..Default::default()
        };

        GenerateContentResponse {
            candidates: vec![Candidate {
                index: Some(0),
                content: GeminiContent {
                    role: "model".to_string(),
                    parts: vec![fc_part],
                },
                avg_logprobs: None,
                logprobs_result: None,
                finish_reason: Some(FinishReason::Stop),
                safety_ratings: None,
                citation_metadata: None,
                grounding_metadata: None,
                url_context_metadata: None,
                finish_message: None,
            }],
            model_version: Some("gemini-2.0-flash".to_string()),
            create_time: None,
            response_id: Some("resp-fc".to_string()),
            prompt_feedback: None,
            usage_metadata: Some(UsageMetadata {
                prompt_token_count: Some(15),
                candidates_token_count: Some(5),
                tool_use_prompt_token_count: None,
                thoughts_token_count: None,
                total_token_count: Some(20),
                cached_content_token_count: None,
                prompt_tokens_details: None,
                cache_tokens_details: None,
                candidates_tokens_details: None,
                tool_use_prompt_tokens_details: None,
                traffic_type: None,
            }),
        }
    }

    fn make_empty_response() -> GenerateContentResponse {
        GenerateContentResponse {
            candidates: vec![],
            model_version: Some("gemini-2.0-flash".to_string()),
            create_time: None,
            response_id: None,
            prompt_feedback: None,
            usage_metadata: None,
        }
    }

    #[test]
    fn test_id() {
        assert_eq!(make_text_response("hi").id(), "resp-123");
        assert_eq!(make_empty_response().id(), "");
    }

    #[test]
    fn test_is_empty() {
        assert!(!make_text_response("hi").is_empty());
        assert!(make_empty_response().is_empty());
    }

    #[test]
    fn test_response_text() {
        assert_eq!(
            make_text_response("hello world").response_text(),
            "hello world"
        );
        assert_eq!(make_empty_response().response_text(), "");
    }

    #[test]
    fn test_response_text_function_call() {
        assert_eq!(make_function_call_response().response_text(), "");
    }

    #[test]
    fn test_model_name() {
        assert_eq!(
            make_text_response("x").model_name(),
            Some("gemini-2.0-flash")
        );
    }

    #[test]
    fn test_finish_reason() {
        assert_eq!(make_text_response("x").finish_reason(), Some("STOP"));
        assert_eq!(make_empty_response().finish_reason(), None);
    }

    #[test]
    fn test_finish_reason_variants() {
        let mut resp = make_text_response("x");
        resp.candidates[0].finish_reason = Some(FinishReason::MaxTokens);
        assert_eq!(resp.finish_reason(), Some("MAX_TOKENS"));
        resp.candidates[0].finish_reason = Some(FinishReason::Safety);
        assert_eq!(resp.finish_reason(), Some("SAFETY"));
        resp.candidates[0].finish_reason = Some(FinishReason::MalformedFunctionCall);
        assert_eq!(resp.finish_reason(), Some("MALFORMED_FUNCTION_CALL"));
    }

    #[test]
    fn test_token_counts() {
        let resp = make_text_response("x");
        assert_eq!(resp.input_tokens(), Some(20));
        assert_eq!(resp.output_tokens(), Some(10));
        assert_eq!(resp.total_tokens(), Some(30));
    }

    #[test]
    fn test_token_counts_no_metadata() {
        let resp = make_empty_response();
        assert_eq!(resp.input_tokens(), None);
        assert_eq!(resp.output_tokens(), None);
        assert_eq!(resp.total_tokens(), None);
    }

    #[test]
    fn test_get_tool_calls() {
        let calls = make_function_call_response().get_tool_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "get_weather");
        assert_eq!(calls[0].call_id, Some("fc_01".to_string()));
        assert_eq!(calls[0].arguments, serde_json::json!({"location": "NYC"}));
    }

    #[test]
    fn test_get_tool_calls_empty() {
        assert!(make_text_response("hello").get_tool_calls().is_empty());
        assert!(make_empty_response().get_tool_calls().is_empty());
    }

    #[test]
    fn test_tool_call_output() {
        let resp = make_function_call_response();
        let output = resp.tool_call_output();
        assert!(output.is_some());
    }

    #[test]
    fn test_tool_call_output_none_for_text() {
        assert!(make_text_response("hello").tool_call_output().is_none());
    }

    #[test]
    fn test_structured_output_value_valid_json() {
        let resp = make_text_response(r#"{"key":"value"}"#);
        let val = resp.structured_output_value();
        assert!(val.is_some());
        assert_eq!(val.unwrap()["key"], "value");
    }

    #[test]
    fn test_structured_output_value_plain_text() {
        assert!(make_text_response("not json")
            .structured_output_value()
            .is_none());
    }

    #[test]
    fn test_structured_output_value_empty() {
        assert!(make_empty_response().structured_output_value().is_none());
    }

    #[test]
    fn test_to_message_num() {
        let resp = make_text_response("hello");
        let msgs = resp.to_message_num().unwrap();
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn test_to_message_num_empty() {
        let resp = make_empty_response();
        let msgs = resp.to_message_num().unwrap();
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_get_log_probs_with_digits() {
        let resp = GenerateContentResponse {
            candidates: vec![Candidate {
                index: Some(0),
                content: GeminiContent {
                    role: "model".to_string(),
                    parts: vec![Part::from_text("text".to_string())],
                },
                avg_logprobs: None,
                logprobs_result: Some(LogprobsResult {
                    top_candidates: None,
                    chosen_candidates: Some(vec![
                        LogprobsCandidate {
                            token: Some("4".to_string()),
                            token_id: Some(1),
                            log_probability: Some(-0.3),
                        },
                        LogprobsCandidate {
                            token: Some("hello".to_string()),
                            token_id: Some(2),
                            log_probability: Some(-1.5),
                        },
                    ]),
                }),
                finish_reason: Some(FinishReason::Stop),
                safety_ratings: None,
                citation_metadata: None,
                grounding_metadata: None,
                url_context_metadata: None,
                finish_message: None,
            }],
            model_version: None,
            create_time: None,
            response_id: None,
            prompt_feedback: None,
            usage_metadata: None,
        };
        let probs = resp.get_log_probs();
        assert_eq!(probs.len(), 1);
        assert_eq!(probs[0].token, "4");
        assert!((probs[0].logprob - (-0.3)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_log_probs_empty() {
        assert!(make_text_response("x").get_log_probs().is_empty());
    }

    #[test]
    fn test_deserialize_from_json() {
        let json = serde_json::json!({
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [{"text": "Hello from Gemini!"}]
                },
                "finishReason": "STOP"
            }],
            "modelVersion": "gemini-2.0-flash",
            "responseId": "resp-test",
            "usageMetadata": {
                "promptTokenCount": 12,
                "candidatesTokenCount": 8,
                "totalTokenCount": 20
            }
        });
        let resp: GenerateContentResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.response_text(), "Hello from Gemini!");
        assert_eq!(resp.model_name(), Some("gemini-2.0-flash"));
        assert_eq!(resp.finish_reason(), Some("STOP"));
        assert_eq!(resp.input_tokens(), Some(12));
        assert_eq!(resp.output_tokens(), Some(8));
        assert_eq!(resp.total_tokens(), Some(20));
    }

    #[test]
    fn test_deserialize_tool_calls_from_json() {
        let raw = r#"{
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [
                        {
                            "functionCall": {
                                "name": "get_weather",
                                "id": "fc_001",
                                "args": {"location": "San Francisco", "unit": "celsius"}
                            }
                        },
                        {
                            "functionCall": {
                                "name": "get_stock_price",
                                "id": "fc_002",
                                "args": {"ticker": "AAPL"}
                            }
                        }
                    ]
                },
                "finishReason": "STOP"
            }],
            "modelVersion": "gemini-2.0-flash",
            "responseId": "resp-fc-test",
            "usageMetadata": {
                "promptTokenCount": 50,
                "candidatesTokenCount": 30,
                "totalTokenCount": 80
            }
        }"#;

        let resp: GenerateContentResponse = serde_json::from_str(raw).unwrap();
        let tool_calls = resp.get_tool_calls();

        assert_eq!(tool_calls.len(), 2);

        assert_eq!(tool_calls[0].name, "get_weather");
        assert_eq!(tool_calls[0].call_id, Some("fc_001".to_string()));
        assert_eq!(
            tool_calls[0].arguments,
            serde_json::json!({"location": "San Francisco", "unit": "celsius"})
        );
        assert!(tool_calls[0].result.is_none());

        assert_eq!(tool_calls[1].name, "get_stock_price");
        assert_eq!(tool_calls[1].call_id, Some("fc_002".to_string()));
        assert_eq!(
            tool_calls[1].arguments,
            serde_json::json!({"ticker": "AAPL"})
        );
        assert!(tool_calls[1].result.is_none());
    }

    #[test]
    fn test_function_call_no_args() {
        let fc_part = Part {
            data: DataNum::FunctionCall(FunctionCall {
                name: "no_args_fn".to_string(),
                id: None,
                args: None,
                will_continue: None,
                partial_args: None,
            }),
            ..Default::default()
        };
        let resp = GenerateContentResponse {
            candidates: vec![Candidate {
                index: Some(0),
                content: GeminiContent {
                    role: "model".to_string(),
                    parts: vec![fc_part],
                },
                avg_logprobs: None,
                logprobs_result: None,
                finish_reason: Some(FinishReason::Stop),
                safety_ratings: None,
                citation_metadata: None,
                grounding_metadata: None,
                url_context_metadata: None,
                finish_message: None,
            }],
            model_version: None,
            create_time: None,
            response_id: None,
            prompt_feedback: None,
            usage_metadata: None,
        };
        let calls = resp.get_tool_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "no_args_fn");
        assert_eq!(calls[0].call_id, None);
        assert_eq!(calls[0].arguments, serde_json::Value::Null);
    }
}
