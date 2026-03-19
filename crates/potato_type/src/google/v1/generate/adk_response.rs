use crate::google::v1::generate::request::{
    Blob, CodeExecutionResult, ExecutableCode, FileData, FunctionCall, FunctionResponse,
    GeminiContent, Part,
};
use crate::google::v1::generate::response::{
    CitationMetadata, FinishReason, GroundingMetadata, LogprobsResult,
};
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

/// Python-accessible tool call info — returned from `AdkLlmResponse.get_tool_calls()`.
#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Clone)]
pub struct AdkToolCallInfo {
    pub name: String,
    pub call_id: Option<String>,
    pub arguments_json: String,
}

/// ADK Part — uses individual Option fields to handle Pydantic's wire format which
/// serializes ALL fields including null values (e.g. `"function_call": null`).
/// A flattened enum cannot handle this; individual Option<T> fields can.
#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case", default)]
pub struct AdkPart {
    pub text: Option<String>,
    pub function_call: Option<FunctionCall>,
    pub function_response: Option<FunctionResponse>,
    pub inline_data: Option<Blob>,
    pub file_data: Option<FileData>,
    pub executable_code: Option<ExecutableCode>,
    pub code_execution_result: Option<CodeExecutionResult>,
}

impl AdkPart {
    fn to_data_num(&self) -> DataNum {
        if let Some(fc) = &self.function_call {
            return DataNum::FunctionCall(fc.clone());
        }
        if let Some(fr) = &self.function_response {
            return DataNum::FunctionResponse(fr.clone());
        }
        if let Some(b) = &self.inline_data {
            return DataNum::InlineData(b.clone());
        }
        if let Some(fd) = &self.file_data {
            return DataNum::FileData(fd.clone());
        }
        if let Some(ec) = &self.executable_code {
            return DataNum::ExecutableCode(ec.clone());
        }
        if let Some(cr) = &self.code_execution_result {
            return DataNum::CodeExecutionResult(cr.clone());
        }
        DataNum::Text(self.text.clone().unwrap_or_default())
    }
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AdkContent {
    pub parts: Vec<AdkPart>,
    pub role: String,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AdkUsageMetadata {
    pub prompt_token_count: Option<i32>,
    pub candidates_token_count: Option<i32>,
    pub total_token_count: Option<i32>,
    pub cached_content_token_count: Option<i32>,
    pub thoughts_token_count: Option<i32>,
    pub tool_use_prompt_token_count: Option<i32>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AdkTranscription {
    pub text: Option<String>,
    pub finished: Option<bool>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AdkCacheMetadata {
    pub cache_name: Option<String>,
    pub expire_time: Option<f64>,
    pub fingerprint: String,
    pub invocations_used: Option<i64>,
    pub contents_count: i64,
    pub created_at: Option<f64>,
}

#[pyclass]
#[pyo3(get_all)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AdkLiveSessionResumptionUpdate {
    pub new_handle: Option<String>,
    pub resumable: Option<bool>,
    pub last_consumed_client_message_index: Option<i64>,
}

/// Google ADK `LlmResponse` — flat, snake_case structure.
/// Discriminator: presence of the `"partial"` key (ADK-specific).
#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AdkLlmResponse {
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<AdkContent>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_metadata: Option<GroundingMetadata>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial: Option<bool>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_complete: Option<bool>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupted: Option<bool>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_metadata: Option<AdkUsageMetadata>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_logprobs: Option<f64>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs_result: Option<LogprobsResult>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_metadata: Option<AdkCacheMetadata>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_metadata: Option<CitationMetadata>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interaction_id: Option<String>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live_session_resumption_update: Option<AdkLiveSessionResumptionUpdate>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_transcription: Option<AdkTranscription>,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_transcription: Option<AdkTranscription>,
    // custom_metadata is a freeform JSON blob — not exposed as a Python property
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_metadata: Option<Value>,
}

#[pymethods]
impl AdkLlmResponse {
    #[staticmethod]
    pub fn model_validate_json(json_string: String) -> PyResult<Self> {
        serde_json::from_str(&json_string)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    pub fn model_dump_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| String::new())
    }

    pub fn response_text(&self) -> String {
        self.last_text_part()
    }

    pub fn model_name_str(&self) -> Option<String> {
        self.model_version.clone()
    }

    pub fn finish_reason_str(&self) -> Option<String> {
        self.finish_reason.as_ref().map(|r| r.as_str().to_string())
    }

    pub fn get_tool_calls(&self) -> Vec<AdkToolCallInfo> {
        match &self.content {
            None => vec![],
            Some(content) => content
                .parts
                .iter()
                .filter_map(|p| {
                    p.function_call.as_ref().map(|call| AdkToolCallInfo {
                        name: call.name.clone(),
                        call_id: call.id.clone(),
                        arguments_json: call
                            .args
                            .as_ref()
                            .map(|a| serde_json::to_string(a).unwrap_or_default())
                            .unwrap_or_default(),
                    })
                })
                .collect(),
        }
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

impl MessageResponseExt for AdkContent {
    fn to_message_num(&self) -> Result<MessageNum, TypeError> {
        let parts: Vec<Part> = self
            .parts
            .iter()
            .map(|p| Part {
                data: p.to_data_num(),
                thought: None,
                thought_signature: None,
                part_metadata: None,
                media_resolution: None,
                video_metadata: None,
            })
            .collect();

        Ok(MessageNum::GeminiContentV1(GeminiContent {
            role: self.role.clone(),
            parts,
        }))
    }
}

impl ResponseAdapter for AdkLlmResponse {
    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }

    fn is_empty(&self) -> bool {
        self.content.is_none()
    }

    fn to_bound_py_object<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        Ok(PyHelperFuncs::to_bound_py_object(py, self)?)
    }

    fn id(&self) -> &str {
        self.interaction_id.as_deref().unwrap_or("")
    }

    fn to_message_num(&self) -> Result<Vec<MessageNum>, TypeError> {
        match &self.content {
            Some(content) => Ok(vec![content.to_message_num()?]),
            None => Ok(vec![]),
        }
    }

    fn get_content(&self) -> ResponseContent {
        use crate::google::v1::generate::response::Candidate;
        let content = self.content.as_ref().map(|c| {
            let parts: Vec<Part> = c
                .parts
                .iter()
                .map(|p| Part {
                    data: p.to_data_num(),
                    thought: None,
                    thought_signature: None,
                    part_metadata: None,
                    media_resolution: None,
                    video_metadata: None,
                })
                .collect();
            GeminiContent {
                role: c.role.clone(),
                parts,
            }
        });
        ResponseContent::Google(Candidate {
            index: Some(0),
            content: content.unwrap_or(GeminiContent {
                role: "model".to_string(),
                parts: vec![],
            }),
            avg_logprobs: None,
            logprobs_result: None,
            finish_reason: self.finish_reason.clone(),
            safety_ratings: None,
            citation_metadata: self.citation_metadata.clone(),
            grounding_metadata: self.grounding_metadata.clone(),
            url_context_metadata: None,
            finish_message: None,
        })
    }

    fn tool_call_output(&self) -> Option<Value> {
        let content = self.content.as_ref()?;
        let fc = content
            .parts
            .iter()
            .find_map(|p| p.function_call.as_ref())?;
        serde_json::to_value(fc).ok()
    }

    fn structured_output<'py>(
        &self,
        py: Python<'py>,
        output_model: Option<&Bound<'py, PyAny>>,
    ) -> Result<Bound<'py, PyAny>, TypeError> {
        let text = self.last_text_part();
        if text.is_empty() {
            return Ok(py.None().into_bound_py_any(py)?);
        }
        Ok(construct_structured_response(py, text, output_model)?)
    }

    fn structured_output_value(&self) -> Option<Value> {
        let text = self.last_text_part();
        if text.is_empty() {
            return None;
        }
        serde_json::from_str(&text).ok()
    }

    fn usage<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        Ok(PyHelperFuncs::to_bound_py_object(py, &self.usage_metadata)?)
    }

    fn get_log_probs(&self) -> Vec<TokenLogProbs> {
        let mut probabilities = Vec::new();
        if let Some(logprobs_result) = &self.logprobs_result {
            if let Some(chosen_candidates) = &logprobs_result.chosen_candidates {
                for log_content in chosen_candidates {
                    if let Some(token) = &log_content.token {
                        if let Some(ch) = token.chars().next() {
                            if token.len() == 1 && ch.is_ascii_digit() {
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
        self.last_text_part()
    }

    fn model_name(&self) -> Option<&str> {
        self.model_version.as_deref()
    }

    fn finish_reason(&self) -> Option<&str> {
        self.finish_reason.as_ref().map(|r| r.as_str())
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
        if let Some(content) = &self.content {
            for part in &content.parts {
                if let Some(call) = &part.function_call {
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

impl AdkLlmResponse {
    fn last_text_part(&self) -> String {
        let content = match &self.content {
            Some(c) => c,
            None => return String::new(),
        };
        content
            .parts
            .iter()
            .rev()
            .find_map(|p| p.text.as_ref().filter(|t| !t.is_empty()).cloned())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::ResponseAdapter;

    fn make_text_adk_response(text: &str) -> AdkLlmResponse {
        AdkLlmResponse {
            model_version: Some("google/gemini-3-flash-preview".to_string()),
            content: Some(AdkContent {
                role: "model".to_string(),
                parts: vec![AdkPart {
                    text: Some(text.to_string()),
                    ..Default::default()
                }],
            }),
            grounding_metadata: None,
            partial: Some(false),
            turn_complete: Some(true),
            finish_reason: Some(FinishReason::Stop),
            error_code: None,
            error_message: None,
            interrupted: None,
            usage_metadata: Some(AdkUsageMetadata {
                prompt_token_count: Some(20),
                candidates_token_count: Some(10),
                total_token_count: Some(30),
                cached_content_token_count: None,
                thoughts_token_count: None,
                tool_use_prompt_token_count: None,
            }),
            avg_logprobs: None,
            logprobs_result: None,
            cache_metadata: None,
            citation_metadata: None,
            interaction_id: Some("test-interaction".to_string()),
            live_session_resumption_update: None,
            input_transcription: None,
            output_transcription: None,
            custom_metadata: None,
        }
    }

    fn make_function_call_adk_response() -> AdkLlmResponse {
        use serde_json::Map;
        let mut args = Map::new();
        args.insert(
            "agent_name".to_string(),
            serde_json::json!("MeatRecipeAgent"),
        );
        AdkLlmResponse {
            model_version: Some("google/gemini-3-flash-preview".to_string()),
            content: Some(AdkContent {
                role: "model".to_string(),
                parts: vec![AdkPart {
                    function_call: Some(FunctionCall {
                        name: "transfer_to_agent".to_string(),
                        id: None,
                        args: Some(args),
                        will_continue: None,
                        partial_args: None,
                    }),
                    ..Default::default()
                }],
            }),
            grounding_metadata: None,
            partial: Some(false),
            turn_complete: None,
            finish_reason: Some(FinishReason::Stop),
            error_code: None,
            error_message: None,
            interrupted: None,
            usage_metadata: Some(AdkUsageMetadata {
                prompt_token_count: Some(15),
                candidates_token_count: Some(5),
                total_token_count: Some(20),
                cached_content_token_count: None,
                thoughts_token_count: None,
                tool_use_prompt_token_count: None,
            }),
            avg_logprobs: None,
            logprobs_result: None,
            cache_metadata: None,
            citation_metadata: None,
            interaction_id: None,
            live_session_resumption_update: None,
            input_transcription: None,
            output_transcription: None,
            custom_metadata: None,
        }
    }

    fn make_empty_adk_response() -> AdkLlmResponse {
        AdkLlmResponse {
            model_version: Some("google/gemini-3-flash-preview".to_string()),
            content: None,
            grounding_metadata: None,
            partial: Some(false),
            turn_complete: None,
            finish_reason: None,
            error_code: None,
            error_message: None,
            interrupted: None,
            usage_metadata: None,
            avg_logprobs: None,
            logprobs_result: None,
            cache_metadata: None,
            citation_metadata: None,
            interaction_id: None,
            live_session_resumption_update: None,
            input_transcription: None,
            output_transcription: None,
            custom_metadata: None,
        }
    }

    #[test]
    fn test_deserialize_text_response_from_json() {
        let json = r#"{
            "model_version": "google/gemini-3-flash-preview",
            "content": {
                "role": "model",
                "parts": [{"text": "If you're looking for a great steak recipe, start with a dry-aged ribeye."}]
            },
            "partial": false,
            "turn_complete": true,
            "finish_reason": "STOP",
            "usage_metadata": {
                "prompt_token_count": 20,
                "candidates_token_count": 10,
                "total_token_count": 30
            }
        }"#;
        let resp: AdkLlmResponse = serde_json::from_str(json).unwrap();
        assert!(resp.response_text().starts_with("If you're looking"));
        assert_eq!(resp.model_name(), Some("google/gemini-3-flash-preview"));
        assert_eq!(resp.finish_reason(), Some("STOP"));
        assert_eq!(resp.input_tokens(), Some(20));
        assert_eq!(resp.output_tokens(), Some(10));
        assert_eq!(resp.total_tokens(), Some(30));
    }

    /// Validates the exact Pydantic wire format — all Part fields serialized including nulls.
    #[test]
    fn test_deserialize_pydantic_text_format() {
        let json = r#"{
            "model_version": "google/gemini-3-flash-preview",
            "content": {
                "role": "model",
                "parts": [{
                    "media_resolution": null,
                    "code_execution_result": null,
                    "executable_code": null,
                    "file_data": null,
                    "function_call": null,
                    "function_response": null,
                    "inline_data": null,
                    "text": "hello from pydantic",
                    "thought": null,
                    "thought_signature": null,
                    "video_metadata": null
                }]
            },
            "partial": false,
            "finish_reason": "STOP"
        }"#;
        let resp: AdkLlmResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.response_text(), "hello from pydantic");
        assert_eq!(resp.finish_reason(), Some("STOP"));
    }

    /// Validates function_call deserialization from Pydantic wire format.
    #[test]
    fn test_deserialize_pydantic_function_call_format() {
        let json = r#"{
            "model_version": "google/gemini-3-flash-preview",
            "content": {
                "role": "model",
                "parts": [{
                    "media_resolution": null,
                    "code_execution_result": null,
                    "executable_code": null,
                    "file_data": null,
                    "function_call": {
                        "id": null,
                        "args": {"agent_name": "MeatRecipeAgent"},
                        "name": "transfer_to_agent",
                        "partial_args": null,
                        "will_continue": null
                    },
                    "function_response": null,
                    "inline_data": null,
                    "text": null,
                    "thought": null,
                    "thought_signature": null,
                    "video_metadata": null
                }]
            },
            "partial": false,
            "finish_reason": "STOP"
        }"#;
        let resp: AdkLlmResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.response_text(), "");
        let calls = <AdkLlmResponse as ResponseAdapter>::get_tool_calls(&resp);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "transfer_to_agent");
        assert_eq!(
            calls[0].arguments,
            serde_json::json!({"agent_name": "MeatRecipeAgent"})
        );
    }

    #[test]
    fn test_deserialize_function_call_from_json() {
        let json = r#"{
            "model_version": "google/gemini-3-flash-preview",
            "content": {
                "role": "model",
                "parts": [{
                    "function_call": {
                        "name": "transfer_to_agent",
                        "args": {"agent_name": "MeatRecipeAgent"}
                    }
                }]
            },
            "partial": false,
            "finish_reason": "STOP"
        }"#;
        let resp: AdkLlmResponse = serde_json::from_str(json).unwrap();
        let calls = resp.get_tool_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "transfer_to_agent");
        assert_eq!(resp.response_text(), "");
    }

    #[test]
    fn test_response_text_empty() {
        assert_eq!(make_empty_adk_response().response_text(), "");
    }

    #[test]
    fn test_is_empty() {
        assert!(!make_text_adk_response("hello").is_empty());
        assert!(make_empty_adk_response().is_empty());
    }

    #[test]
    fn test_response_text() {
        assert_eq!(
            make_text_adk_response("hello world").response_text(),
            "hello world"
        );
    }

    #[test]
    fn test_model_name() {
        assert_eq!(
            make_text_adk_response("x").model_name(),
            Some("google/gemini-3-flash-preview")
        );
    }

    #[test]
    fn test_finish_reason() {
        assert_eq!(make_text_adk_response("x").finish_reason(), Some("STOP"));
        assert_eq!(make_empty_adk_response().finish_reason(), None);
    }

    #[test]
    fn test_token_counts() {
        let resp = make_text_adk_response("x");
        assert_eq!(resp.input_tokens(), Some(20));
        assert_eq!(resp.output_tokens(), Some(10));
        assert_eq!(resp.total_tokens(), Some(30));
    }

    #[test]
    fn test_token_counts_no_metadata() {
        let resp = make_empty_adk_response();
        assert_eq!(resp.input_tokens(), None);
        assert_eq!(resp.output_tokens(), None);
        assert_eq!(resp.total_tokens(), None);
    }

    #[test]
    fn test_get_tool_calls() {
        let calls = make_function_call_adk_response().get_tool_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "transfer_to_agent");
    }

    #[test]
    fn test_get_tool_calls_empty() {
        assert!(make_text_adk_response("hello").get_tool_calls().is_empty());
        assert!(make_empty_adk_response().get_tool_calls().is_empty());
    }

    #[test]
    fn test_id() {
        assert_eq!(make_text_adk_response("x").id(), "test-interaction");
        assert_eq!(make_empty_adk_response().id(), "");
    }

    #[test]
    fn test_to_message_num_text() {
        let resp = make_text_adk_response("hello");
        let msgs = resp.to_message_num().unwrap();
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn test_to_message_num_empty() {
        let resp = make_empty_adk_response();
        let msgs = resp.to_message_num().unwrap();
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_structured_output_value_json() {
        let resp = make_text_adk_response(r#"{"key":"value"}"#);
        let val = resp.structured_output_value();
        assert!(val.is_some());
        assert_eq!(val.unwrap()["key"], "value");
    }

    #[test]
    fn test_structured_output_value_plain_text() {
        assert!(make_text_adk_response("not json")
            .structured_output_value()
            .is_none());
    }

    #[test]
    fn test_get_log_probs_snake_case_wire_format() {
        // ADK sends snake_case keys — verify they deserialize correctly into
        // camelCase-annotated shared types via serde aliases.
        let json = r#"{
            "model_version": "google/gemini-3-flash-preview",
            "content": {"role": "model", "parts": [{"text": "4"}]},
            "partial": false,
            "logprobs_result": {
                "chosen_candidates": [
                    {"token": "4", "token_id": 52, "log_probability": -0.1},
                    {"token": "hello", "token_id": 9, "log_probability": -2.5}
                ]
            }
        }"#;
        let resp: AdkLlmResponse = serde_json::from_str(json).unwrap();
        let probs = resp.get_log_probs();
        // Only digit tokens are kept
        assert_eq!(probs.len(), 1);
        assert_eq!(probs[0].token, "4");
        assert!((probs[0].logprob - (-0.1)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_log_probs_digits_only() {
        use crate::google::v1::generate::response::{LogprobsCandidate, LogprobsResult};
        let mut resp = make_text_adk_response("x");
        resp.logprobs_result = Some(LogprobsResult {
            top_candidates: None,
            chosen_candidates: Some(vec![
                LogprobsCandidate {
                    token: Some("3".to_string()),
                    token_id: Some(1),
                    log_probability: Some(-0.5),
                },
                LogprobsCandidate {
                    token: Some("hello".to_string()),
                    token_id: Some(2),
                    log_probability: Some(-1.0),
                },
                LogprobsCandidate {
                    token: Some("7".to_string()),
                    token_id: Some(3),
                    log_probability: Some(-0.2),
                },
            ]),
        });
        let probs = resp.get_log_probs();
        assert_eq!(probs.len(), 2);
        assert_eq!(probs[0].token, "3");
        assert_eq!(probs[1].token, "7");
    }

    #[test]
    fn test_to_data_num_function_call_priority() {
        use crate::google::v1::generate::request::FunctionCall;
        let part = AdkPart {
            text: Some("some text".to_string()),
            function_call: Some(FunctionCall {
                name: "my_fn".to_string(),
                id: None,
                args: None,
                will_continue: None,
                partial_args: None,
            }),
            ..Default::default()
        };
        // function_call takes priority over text
        assert!(matches!(part.to_data_num(), DataNum::FunctionCall(_)));
    }

    #[test]
    fn test_to_data_num_text_fallback() {
        let part = AdkPart {
            text: Some("hello".to_string()),
            ..Default::default()
        };
        assert!(matches!(part.to_data_num(), DataNum::Text(_)));
    }
}
