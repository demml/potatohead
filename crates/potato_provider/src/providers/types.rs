use crate::error::ProviderError;
use potato_type::anthropic::v1::response::AnthropicMessageResponse;
use potato_type::google::v1::generate::adk_response::AdkLlmResponse;
use potato_type::google::v1::generate::GenerateContentResponse;
use potato_type::google::PredictResponse;
use potato_type::openai::v1::OpenAIChatResponse;
use potato_type::prompt::MessageNum;
use potato_type::tools::ToolCallInfo;
use potato_type::traits::ResponseAdapter;
use potato_type::Provider;
use potato_util::utils::TokenLogProbs;
use potatohead_macro::dispatch_response_trait_method;
use pyo3::prelude::*;
use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const GENERATE_CONTENT: &str = "generateContent";
pub const EMBED_CONTENT: &str = "embedContent";
pub const CHAT_COMPLETIONS: &str = "chat/completions";
pub const PREDICT: &str = "predict";
pub const EMBEDDINGS: &str = "embeddings";
pub const MESSAGES: &str = "messages";

#[derive(Debug, PartialEq)]
pub enum ServiceType {
    Generate,
    Embed,
}

impl ServiceType {
    /// Get the service type string
    pub fn gemini_endpoint(&self) -> &'static str {
        match self {
            Self::Generate => GENERATE_CONTENT,
            Self::Embed => EMBED_CONTENT,
        }
    }
    pub fn vertex_endpoint(&self) -> &'static str {
        match self {
            Self::Generate => GENERATE_CONTENT,
            Self::Embed => PREDICT, // vertex uses "predict" for embeddings since it calls models directly
        }
    }

    pub fn openai_endpoint(&self) -> &'static str {
        match self {
            Self::Generate => CHAT_COMPLETIONS,
            Self::Embed => EMBEDDINGS,
        }
    }

    pub fn anthropic_endpoint(&self) -> &'static str {
        match self {
            Self::Generate => MESSAGES,
            Self::Embed => EMBEDDINGS,
        }
    }
}

/// Merges extra_body fields into the serialized prompt JSON object.
///
/// # Arguments
/// * `serialized_prompt` - Mutable reference to the JSON value to modify
/// * `extra_body` - Reference to the extra body JSON object to merge
///
/// # Example
/// ```rust
/// let mut prompt = serde_json::json!({"model": "gpt-4"});
/// let extra = serde_json::json!({"temperature": 0.7});
/// add_extra_body_to_prompt(&mut prompt, &extra);
/// ```
pub fn add_extra_body_to_prompt(serialized_prompt: &mut Value, extra_body: &Value) {
    if let (Some(prompt_obj), Some(extra_obj)) =
        (serialized_prompt.as_object_mut(), extra_body.as_object())
    {
        // Merge the extra_body fields into prompt
        for (key, value) in extra_obj {
            prompt_obj.insert(key.clone(), value.clone());
        }
    }
}

pub fn build_http_client(default_headers: Option<HeaderMap>) -> Result<Client, ProviderError> {
    let headers = default_headers.unwrap_or_default();

    Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(ProviderError::from)
}

/// Unified ChatResponse enum to encapsulate different provider responses
/// Follows  our strategy pattern for dispatching methods to the appropriate inner type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatResponse {
    OpenAIV1(OpenAIChatResponse),
    GeminiV1(GenerateContentResponse),
    VertexGenerateV1(GenerateContentResponse),
    VertexPredictV1(PredictResponse),
    AnthropicMessageV1(AnthropicMessageResponse),
    AdkLlmV1(Box<AdkLlmResponse>),
}

impl ChatResponse {
    /// Returns the token usage as a Python object
    pub fn token_usage<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, ProviderError> {
        Ok(dispatch_response_trait_method!(
            self,
            ResponseAdapter,
            usage(py)
        )?)
    }

    /// Converts the response to a Python object
    pub fn to_bound_py_object<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyAny>, ProviderError> {
        Ok(dispatch_response_trait_method!(
            self,
            ResponseAdapter,
            to_bound_py_object(py)
        )?)
    }

    /// Returns the string representation of the response
    pub fn __str__(&self) -> String {
        dispatch_response_trait_method!(self, ResponseAdapter, __str__())
    }

    /// Checks if the response is empty
    pub fn is_empty(&self) -> bool {
        dispatch_response_trait_method!(self, ResponseAdapter, is_empty())
    }

    /// Converts the response to a vector of MessageNum
    pub fn id(&self) -> String {
        dispatch_response_trait_method!(self, ResponseAdapter, id()).to_string()
    }

    /// Converts the response to a vector of MessageNum
    pub fn structured_output<'py>(
        &self,
        py: Python<'py>,
        output_type: Option<&Bound<'py, PyAny>>,
    ) -> Result<Bound<'py, PyAny>, ProviderError> {
        Ok(dispatch_response_trait_method!(
            self,
            ResponseAdapter,
            structured_output(py, output_type)
        )?)
    }

    pub fn get_log_probs(&self) -> Vec<TokenLogProbs> {
        dispatch_response_trait_method!(self, ResponseAdapter, get_log_probs())
    }

    /// Converts the response messages to a vector of MessageNum for requests
    /// If necessary, will convert each message to the appropriate provider format
    pub fn to_message_num(&self, provider: &Provider) -> Result<Vec<MessageNum>, ProviderError> {
        // convert response to MessageNum of existing provider type
        let mut messages =
            dispatch_response_trait_method!(self, ResponseAdapter, to_message_num())?;

        // convert each message to the target provider type if needed
        // if the current message provider type matches the target provider, no conversion done
        for msg in messages.iter_mut() {
            msg.convert_message(provider)?;
        }
        Ok(messages)
    }

    /// Retrieves the structured output value as a serde_json::Value
    /// output is either a structure response or tool call data
    pub fn extract_structured_data(&self) -> Option<Value> {
        if let Some(output) =
            dispatch_response_trait_method!(self, ResponseAdapter, structured_output_value())
        {
            Some(output)
        } else {
            dispatch_response_trait_method!(self, ResponseAdapter, tool_call_output())
        }
    }

    pub fn response_text(&self) -> String {
        dispatch_response_trait_method!(self, ResponseAdapter, response_text())
    }

    /// Helper for deserializing a JSON value into the appropriate ChatResponse variant.
    ///
    /// When a `provider` hint is given, deserialization is attempted directly for that provider's
    /// type — this avoids heuristic misrouting (e.g. ADK responses without `"partial"`).
    /// When `provider` is `None` or `Undefined`, the JSON keys are inspected heuristically.
    pub fn from_response_value(
        value: Value,
        provider: Option<&Provider>,
    ) -> Result<Self, ProviderError> {
        match provider {
            Some(p) if *p != Provider::Undefined => {
                Self::from_response_value_with_provider(value, p)
            }
            _ => Self::from_response_value_heuristic(value),
        }
    }

    /// Provider-hinted path: direct deserialization based on known provider.
    fn from_response_value_with_provider(
        value: Value,
        provider: &Provider,
    ) -> Result<Self, ProviderError> {
        match provider {
            Provider::OpenAI => serde_json::from_value::<OpenAIChatResponse>(value)
                .map(ChatResponse::OpenAIV1)
                .map_err(|e| {
                    tracing::warn!("Failed to deserialize OpenAIChatResponse: {e}");
                    ProviderError::DeserializationError
                }),
            Provider::Gemini => serde_json::from_value::<GenerateContentResponse>(value)
                .map(ChatResponse::GeminiV1)
                .map_err(|e| {
                    tracing::warn!("Failed to deserialize GenerateContentResponse: {e}");
                    ProviderError::DeserializationError
                }),
            Provider::Google | Provider::Vertex => {
                let obj = value
                    .as_object()
                    .ok_or(ProviderError::DeserializationError)?;
                if obj.contains_key("predictions") {
                    serde_json::from_value::<PredictResponse>(value)
                        .map(ChatResponse::VertexPredictV1)
                        .map_err(|e| {
                            tracing::warn!("Failed to deserialize PredictResponse: {e}");
                            ProviderError::DeserializationError
                        })
                } else {
                    serde_json::from_value::<GenerateContentResponse>(value)
                        .map(ChatResponse::VertexGenerateV1)
                        .map_err(|e| {
                            tracing::warn!("Failed to deserialize GenerateContentResponse: {e}");
                            ProviderError::DeserializationError
                        })
                }
            }
            Provider::Anthropic => serde_json::from_value::<AnthropicMessageResponse>(value)
                .map(ChatResponse::AnthropicMessageV1)
                .map_err(|e| {
                    tracing::warn!("Failed to deserialize AnthropicMessageResponse: {e}");
                    ProviderError::DeserializationError
                }),
            Provider::GoogleAdk => serde_json::from_value::<AdkLlmResponse>(value)
                .map(|r| ChatResponse::AdkLlmV1(Box::new(r)))
                .map_err(|e| {
                    tracing::warn!("Failed to deserialize AdkLlmResponse: {e}");
                    ProviderError::DeserializationError
                }),
            // Undefined is guarded at the entry point; reaching here is unreachable in
            // practice, but the pattern must be exhaustive — delegate to heuristic.
            Provider::Undefined => Self::from_response_value_heuristic(value),
        }
    }

    /// ADK-specific keys that distinguish an ADK LlmResponse from other Google responses.
    ///
    /// Includes the unambiguously ADK-exclusive event fields plus the snake_case metadata keys
    /// (`usage_metadata`, `model_version`) that ADK uses instead of Gemini's camelCase
    /// equivalents (`usageMetadata`, `modelVersion`).
    const ADK_SPECIFIC_KEYS: &'static [&'static str] = &[
        "partial",
        "turn_complete",
        "interrupted",
        "error_code",
        "error_message",
        "interaction_id",
        "live_session_resumption_update",
        "input_transcription",
        "output_transcription",
        "custom_metadata",
        "usage_metadata",
        "model_version",
    ];

    /// Heuristic path: inspect JSON keys when provider is unknown.
    fn from_response_value_heuristic(value: Value) -> Result<Self, ProviderError> {
        let obj = value
            .as_object()
            .ok_or(ProviderError::DeserializationError)?;

        if obj.contains_key("choices") {
            serde_json::from_value::<OpenAIChatResponse>(value)
                .map(ChatResponse::OpenAIV1)
                .map_err(|e| {
                    tracing::warn!("Failed to deserialize OpenAIChatResponse: {e}");
                    ProviderError::DeserializationError
                })
        } else if obj.contains_key("predictions") {
            serde_json::from_value::<PredictResponse>(value)
                .map(ChatResponse::VertexPredictV1)
                .map_err(|e| {
                    tracing::warn!("Failed to deserialize PredictResponse: {e}");
                    ProviderError::DeserializationError
                })
        } else if obj.contains_key("stop_reason") && obj.contains_key("content") {
            serde_json::from_value::<AnthropicMessageResponse>(value)
                .map(ChatResponse::AnthropicMessageV1)
                .map_err(|e| {
                    tracing::warn!("Failed to deserialize AnthropicMessageResponse: {e}");
                    ProviderError::DeserializationError
                })
        } else if Self::ADK_SPECIFIC_KEYS.iter().any(|k| obj.contains_key(*k)) {
            // Any ADK-specific key (including snake_case variants like usage_metadata /
            // model_version that differ from Gemini's camelCase equivalents) routes here.
            // This check comes before the Gemini "candidates" check so that ADK responses
            // that also contain "candidates" are not misrouted to GeminiV1.
            serde_json::from_value::<AdkLlmResponse>(value)
                .map(|r| ChatResponse::AdkLlmV1(Box::new(r)))
                .map_err(|e| {
                    tracing::warn!("Failed to deserialize AdkLlmResponse: {e}");
                    ProviderError::DeserializationError
                })
        } else if obj.contains_key("candidates") {
            // Gemini / Vertex generate — can't distinguish from JSON alone
            serde_json::from_value::<GenerateContentResponse>(value)
                .map(ChatResponse::GeminiV1)
                .map_err(|e| {
                    tracing::warn!("Failed to deserialize GenerateContentResponse: {e}");
                    ProviderError::DeserializationError
                })
        } else {
            Err(ProviderError::DeserializationError)
        }
    }

    pub fn model_name(&self) -> Option<String> {
        dispatch_response_trait_method!(self, ResponseAdapter, model_name()).map(|s| s.to_string())
    }
    pub fn finish_reason_str(&self) -> Option<String> {
        dispatch_response_trait_method!(self, ResponseAdapter, finish_reason())
            .map(|s| s.to_string())
    }
    pub fn input_tokens(&self) -> Option<i64> {
        dispatch_response_trait_method!(self, ResponseAdapter, input_tokens())
    }
    pub fn output_tokens(&self) -> Option<i64> {
        dispatch_response_trait_method!(self, ResponseAdapter, output_tokens())
    }
    pub fn total_tokens(&self) -> Option<i64> {
        dispatch_response_trait_method!(self, ResponseAdapter, total_tokens())
    }

    pub fn get_tool_calls(&self) -> Vec<ToolCallInfo> {
        dispatch_response_trait_method!(self, ResponseAdapter, get_tool_calls())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn openai_json() -> Value {
        serde_json::json!({
            "id": "chatcmpl-test",
            "object": "chat.completion",
            "created": 1700000000,
            "model": "gpt-4o",
            "choices": [{
                "message": {"content": "Hello!", "role": "assistant"},
                "finish_reason": "stop"
            }],
            "usage": {
                "completion_tokens": 5,
                "prompt_tokens": 10,
                "total_tokens": 15
            }
        })
    }

    fn anthropic_json() -> Value {
        serde_json::json!({
            "id": "msg_test",
            "model": "claude-sonnet-4-20250514",
            "role": "assistant",
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "type": "message",
            "usage": {"input_tokens": 25, "output_tokens": 10},
            "content": [{"type": "text", "text": "Hello from Claude!"}]
        })
    }

    fn gemini_json() -> Value {
        serde_json::json!({
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [{"text": "Hello from Gemini!"}]
                },
                "finishReason": "STOP"
            }],
            "modelVersion": "gemini-2.0-flash",
            "usageMetadata": {
                "promptTokenCount": 12,
                "candidatesTokenCount": 8,
                "totalTokenCount": 20
            }
        })
    }

    fn predict_json() -> Value {
        serde_json::json!({
            "predictions": [{"embeddings": {"values": [0.1, 0.2]}}],
            "metadata": {},
            "deployedModelId": "dm-1",
            "model": "gecko@003",
            "modelVersionId": "1",
            "modelDisplayName": "Gecko"
        })
    }

    #[test]
    fn test_from_response_value_openai() {
        let resp = ChatResponse::from_response_value(openai_json(), None).unwrap();
        assert!(matches!(resp, ChatResponse::OpenAIV1(_)));
        assert_eq!(resp.response_text(), "Hello!");
        assert_eq!(resp.model_name(), Some("gpt-4o".to_string()));
        assert_eq!(resp.finish_reason_str(), Some("stop".to_string()));
        assert_eq!(resp.input_tokens(), Some(10));
        assert_eq!(resp.output_tokens(), Some(5));
        assert_eq!(resp.total_tokens(), Some(15));
    }

    #[test]
    fn test_from_response_value_anthropic() {
        let resp = ChatResponse::from_response_value(anthropic_json(), None).unwrap();
        assert!(matches!(resp, ChatResponse::AnthropicMessageV1(_)));
        assert_eq!(resp.response_text(), "Hello from Claude!");
        assert_eq!(
            resp.model_name(),
            Some("claude-sonnet-4-20250514".to_string())
        );
        assert_eq!(resp.finish_reason_str(), Some("end_turn".to_string()));
        assert_eq!(resp.input_tokens(), Some(25));
        assert_eq!(resp.output_tokens(), Some(10));
        assert_eq!(resp.total_tokens(), Some(35));
    }

    #[test]
    fn test_from_response_value_gemini() {
        let resp = ChatResponse::from_response_value(gemini_json(), None).unwrap();
        assert!(matches!(resp, ChatResponse::GeminiV1(_)));
        assert_eq!(resp.response_text(), "Hello from Gemini!");
        assert_eq!(resp.model_name(), Some("gemini-2.0-flash".to_string()));
        assert_eq!(resp.finish_reason_str(), Some("STOP".to_string()));
        assert_eq!(resp.input_tokens(), Some(12));
        assert_eq!(resp.output_tokens(), Some(8));
        assert_eq!(resp.total_tokens(), Some(20));
    }

    fn adk_json() -> Value {
        serde_json::json!({
            "model_version": "google/gemini-3-flash-preview",
            "content": {
                "role": "model",
                "parts": [{"text": "Hello from ADK!"}]
            },
            "partial": false,
            "turn_complete": true,
            "finish_reason": "STOP",
            "usage_metadata": {
                "prompt_token_count": 20,
                "candidates_token_count": 10,
                "total_token_count": 30
            }
        })
    }

    #[test]
    fn test_from_response_value_adk() {
        let resp = ChatResponse::from_response_value(adk_json(), None).unwrap();
        assert!(matches!(resp, ChatResponse::AdkLlmV1(_)));
        assert_eq!(resp.response_text(), "Hello from ADK!");
        assert_eq!(
            resp.model_name(),
            Some("google/gemini-3-flash-preview".to_string())
        );
        assert_eq!(resp.finish_reason_str(), Some("STOP".to_string()));
        assert_eq!(resp.input_tokens(), Some(20));
        assert_eq!(resp.output_tokens(), Some(10));
        assert_eq!(resp.total_tokens(), Some(30));
    }

    #[test]
    fn test_from_response_value_adk_partial_null() {
        // Pydantic wire format: "partial" key present but null value
        let json = serde_json::json!({
            "model_version": "google/gemini-3-flash-preview",
            "content": {
                "role": "model",
                "parts": [{"text": "Hello from ADK!"}]
            },
            "partial": null,
            "turn_complete": true,
            "finish_reason": "STOP",
            "usage_metadata": {
                "prompt_token_count": 20,
                "candidates_token_count": 10,
                "total_token_count": 30
            }
        });
        let resp = ChatResponse::from_response_value(json, None).unwrap();
        assert!(matches!(resp, ChatResponse::AdkLlmV1(_)));
        assert_eq!(resp.response_text(), "Hello from ADK!");
    }

    #[test]
    fn test_gemini_provider_hint_overrides_adk_heuristic() {
        // A response with "candidates" + "partial" would route to ADK heuristically (since
        // "partial" is in ADK_SPECIFIC_KEYS), but a Gemini provider hint bypasses the heuristic.
        let json = serde_json::json!({
            "candidates": [{"content": {"role": "model", "parts": [{"text": "hi"}]}}],
            "partial": false
        });
        let resp = ChatResponse::from_response_value(json, Some(&Provider::Gemini)).unwrap();
        assert!(matches!(resp, ChatResponse::GeminiV1(_)));
    }

    #[test]
    fn test_from_response_value_predict() {
        let resp = ChatResponse::from_response_value(predict_json(), None).unwrap();
        assert!(matches!(resp, ChatResponse::VertexPredictV1(_)));
        assert_eq!(resp.model_name(), Some("gecko@003".to_string()));
        assert!(!resp.is_empty());
    }

    #[test]
    fn test_from_response_value_unknown() {
        let unknown = serde_json::json!({"unknown_field": true});
        assert!(ChatResponse::from_response_value(unknown, None).is_err());
    }

    #[test]
    fn test_from_response_value_not_object() {
        assert!(ChatResponse::from_response_value(serde_json::json!("string"), None).is_err());
        assert!(ChatResponse::from_response_value(serde_json::json!(42), None).is_err());
    }

    #[test]
    fn test_is_empty_dispatch() {
        let resp = ChatResponse::from_response_value(openai_json(), None).unwrap();
        assert!(!resp.is_empty());
    }

    #[test]
    fn test_id_dispatch() {
        let resp = ChatResponse::from_response_value(openai_json(), None).unwrap();
        assert_eq!(resp.id(), "chatcmpl-test");
    }

    #[test]
    fn test_extract_structured_data_json() {
        let json = serde_json::json!({
            "id": "chatcmpl-json",
            "object": "chat.completion",
            "created": 1700000000,
            "model": "gpt-4o",
            "choices": [{
                "message": {"content": "{\"name\":\"Alice\"}", "role": "assistant"},
                "finish_reason": "stop"
            }],
            "usage": {"completion_tokens": 5, "prompt_tokens": 10, "total_tokens": 15}
        });
        let resp = ChatResponse::from_response_value(json, None).unwrap();
        let data = resp.extract_structured_data();
        assert!(data.is_some());
        assert_eq!(data.unwrap()["name"], "Alice");
    }

    #[test]
    fn test_extract_structured_data_tool_call() {
        let json = serde_json::json!({
            "id": "chatcmpl-tc",
            "object": "chat.completion",
            "created": 1700000000,
            "model": "gpt-4o",
            "choices": [{
                "message": {
                    "content": null,
                    "role": "assistant",
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {"name": "search", "arguments": "{\"q\":\"rust\"}"}
                    }]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {"completion_tokens": 5, "prompt_tokens": 10, "total_tokens": 15}
        });
        let resp = ChatResponse::from_response_value(json, None).unwrap();
        let data = resp.extract_structured_data();
        assert!(data.is_some());
    }

    #[test]
    fn test_get_tool_calls_dispatch() {
        let json = serde_json::json!({
            "id": "msg_tools",
            "model": "claude-sonnet-4-20250514",
            "role": "assistant",
            "stop_reason": "tool_use",
            "type": "message",
            "usage": {"input_tokens": 10, "output_tokens": 5},
            "content": [
                {"type": "tool_use", "id": "toolu_01", "name": "search", "input": {"q": "test"}}
            ]
        });
        let resp = ChatResponse::from_response_value(json, None).unwrap();
        let calls = resp.get_tool_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "search");
    }

    #[test]
    fn test_get_log_probs_dispatch() {
        let resp = ChatResponse::from_response_value(openai_json(), None).unwrap();
        assert!(resp.get_log_probs().is_empty());
    }

    // --- Provider-hinted tests ---

    #[test]
    fn test_adk_with_provider_hint_no_partial() {
        // The core broken case: valid ADK response without "partial" key
        let json = serde_json::json!({
            "model_version": "google/gemini-3-flash-preview",
            "content": {
                "role": "model",
                "parts": [{"text": "Hello from ADK!"}]
            },
            "turn_complete": true,
            "finish_reason": "STOP",
            "usage_metadata": {
                "prompt_token_count": 20,
                "candidates_token_count": 10,
                "total_token_count": 30
            }
        });
        let resp = ChatResponse::from_response_value(json, Some(&Provider::GoogleAdk)).unwrap();
        assert!(matches!(resp, ChatResponse::AdkLlmV1(_)));
        assert_eq!(resp.response_text(), "Hello from ADK!");
    }

    #[test]
    fn test_adk_heuristic_snake_case_signals_only() {
        // ADK response with only snake_case signals (no ADK-unique keys like partial/turn_complete)
        let json = serde_json::json!({
            "model_version": "google/gemini-3-flash-preview",
            "content": {
                "role": "model",
                "parts": [{"text": "Hello!"}]
            },
            "usage_metadata": {
                "prompt_token_count": 5,
                "candidates_token_count": 3,
                "total_token_count": 8
            }
        });
        let resp = ChatResponse::from_response_value(json, None).unwrap();
        assert!(matches!(resp, ChatResponse::AdkLlmV1(_)));
    }

    #[test]
    fn test_vertex_with_provider_hint_generates_vertex_variant() {
        // With Vertex hint, a generateContent response should yield VertexGenerateV1 (not GeminiV1)
        let json = gemini_json();
        let resp = ChatResponse::from_response_value(json, Some(&Provider::Vertex)).unwrap();
        assert!(matches!(resp, ChatResponse::VertexGenerateV1(_)));
    }

    #[test]
    fn test_undefined_provider_falls_through_to_heuristic() {
        let resp =
            ChatResponse::from_response_value(openai_json(), Some(&Provider::Undefined)).unwrap();
        assert!(matches!(resp, ChatResponse::OpenAIV1(_)));
    }

    #[test]
    fn test_openai_with_provider_hint() {
        let resp =
            ChatResponse::from_response_value(openai_json(), Some(&Provider::OpenAI)).unwrap();
        assert!(matches!(resp, ChatResponse::OpenAIV1(_)));
    }

    #[test]
    fn test_anthropic_with_provider_hint() {
        let resp = ChatResponse::from_response_value(anthropic_json(), Some(&Provider::Anthropic))
            .unwrap();
        assert!(matches!(resp, ChatResponse::AnthropicMessageV1(_)));
    }

    #[test]
    fn test_gemini_with_provider_hint() {
        let resp =
            ChatResponse::from_response_value(gemini_json(), Some(&Provider::Gemini)).unwrap();
        assert!(matches!(resp, ChatResponse::GeminiV1(_)));
    }

    #[test]
    fn test_predict_with_provider_hint() {
        let resp =
            ChatResponse::from_response_value(predict_json(), Some(&Provider::Vertex)).unwrap();
        assert!(matches!(resp, ChatResponse::VertexPredictV1(_)));
    }

    #[test]
    fn test_google_provider_hint_generate() {
        // Provider::Google with a generateContent response → VertexGenerateV1
        let resp =
            ChatResponse::from_response_value(gemini_json(), Some(&Provider::Google)).unwrap();
        assert!(matches!(resp, ChatResponse::VertexGenerateV1(_)));
    }

    #[test]
    fn test_google_provider_hint_predict() {
        // Provider::Google with a predictions response → VertexPredictV1
        let resp =
            ChatResponse::from_response_value(predict_json(), Some(&Provider::Google)).unwrap();
        assert!(matches!(resp, ChatResponse::VertexPredictV1(_)));
    }

    #[test]
    fn test_adk_heuristic_candidates_plus_usage_metadata_routes_adk() {
        // Regression guard: ADK responses can contain "candidates" alongside snake_case keys.
        // The ADK check (usage_metadata ∈ ADK_SPECIFIC_KEYS) must come before the Gemini
        // "candidates" check, or these responses would be silently misrouted to GeminiV1.
        let json = serde_json::json!({
            "candidates": [{"content": {"role": "model", "parts": [{"text": "hi"}]}}],
            "usage_metadata": {
                "prompt_token_count": 5,
                "candidates_token_count": 3,
                "total_token_count": 8
            }
        });
        let resp = ChatResponse::from_response_value(json, None).unwrap();
        assert!(matches!(resp, ChatResponse::AdkLlmV1(_)));
    }

    #[test]
    fn test_provider_hint_error_on_non_object_openai() {
        // A non-object value cannot deserialize into any struct — error path is exercised.
        assert!(
            ChatResponse::from_response_value(serde_json::json!("string"), Some(&Provider::OpenAI))
                .is_err()
        );
    }

    #[test]
    fn test_provider_hint_error_on_non_object_anthropic() {
        assert!(
            ChatResponse::from_response_value(serde_json::json!(42), Some(&Provider::Anthropic))
                .is_err()
        );
    }

    #[test]
    fn test_provider_hint_error_on_non_object_adk() {
        // AdkLlmResponse has all-optional fields so a bare `{}` deserializes successfully;
        // a non-object input must still return Err.
        assert!(
            ChatResponse::from_response_value(
                serde_json::json!("not an object"),
                Some(&Provider::GoogleAdk)
            )
            .is_err()
        );
    }
}
