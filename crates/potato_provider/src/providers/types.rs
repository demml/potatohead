use crate::error::ProviderError;
use potato_type::anthropic::v1::response::AnthropicMessageResponse;
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

    /// Helper for deserializing a JSON value into the appropriate ChatResponse variant based on its structure.
    /// We won't always know which provider response type we're getting back, so we need to inspect the JSON to determine how to deserialize it.
    pub fn from_response_value(value: Value) -> Result<Self, ProviderError> {
        let obj = value
            .as_object()
            .ok_or(ProviderError::DeserializationError)?;

        if obj.contains_key("choices") {
            serde_json::from_value::<OpenAIChatResponse>(value)
                .map(ChatResponse::OpenAIV1)
                .map_err(|_| ProviderError::DeserializationError)
        } else if obj.contains_key("predictions") {
            serde_json::from_value::<PredictResponse>(value)
                .map(ChatResponse::VertexPredictV1)
                .map_err(|_| ProviderError::DeserializationError)
        } else if obj.contains_key("candidates") {
            // Can't distinguish Gemini vs VertexGenerate from JSON alone
            serde_json::from_value::<GenerateContentResponse>(value)
                .map(ChatResponse::GeminiV1)
                .map_err(|_| ProviderError::DeserializationError)
        } else if obj.contains_key("stop_reason") && obj.contains_key("content") {
            serde_json::from_value::<AnthropicMessageResponse>(value)
                .map(ChatResponse::AnthropicMessageV1)
                .map_err(|_| ProviderError::DeserializationError)
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
        let resp = ChatResponse::from_response_value(openai_json()).unwrap();
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
        let resp = ChatResponse::from_response_value(anthropic_json()).unwrap();
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
        let resp = ChatResponse::from_response_value(gemini_json()).unwrap();
        assert!(matches!(resp, ChatResponse::GeminiV1(_)));
        assert_eq!(resp.response_text(), "Hello from Gemini!");
        assert_eq!(resp.model_name(), Some("gemini-2.0-flash".to_string()));
        assert_eq!(resp.finish_reason_str(), Some("STOP".to_string()));
        assert_eq!(resp.input_tokens(), Some(12));
        assert_eq!(resp.output_tokens(), Some(8));
        assert_eq!(resp.total_tokens(), Some(20));
    }

    #[test]
    fn test_from_response_value_predict() {
        let resp = ChatResponse::from_response_value(predict_json()).unwrap();
        assert!(matches!(resp, ChatResponse::VertexPredictV1(_)));
        assert_eq!(resp.model_name(), Some("gecko@003".to_string()));
        assert!(!resp.is_empty());
    }

    #[test]
    fn test_from_response_value_unknown() {
        let unknown = serde_json::json!({"unknown_field": true});
        assert!(ChatResponse::from_response_value(unknown).is_err());
    }

    #[test]
    fn test_from_response_value_not_object() {
        assert!(ChatResponse::from_response_value(serde_json::json!("string")).is_err());
        assert!(ChatResponse::from_response_value(serde_json::json!(42)).is_err());
    }

    #[test]
    fn test_is_empty_dispatch() {
        let resp = ChatResponse::from_response_value(openai_json()).unwrap();
        assert!(!resp.is_empty());
    }

    #[test]
    fn test_id_dispatch() {
        let resp = ChatResponse::from_response_value(openai_json()).unwrap();
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
        let resp = ChatResponse::from_response_value(json).unwrap();
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
        let resp = ChatResponse::from_response_value(json).unwrap();
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
        let resp = ChatResponse::from_response_value(json).unwrap();
        let calls = resp.get_tool_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "search");
    }

    #[test]
    fn test_get_log_probs_dispatch() {
        let resp = ChatResponse::from_response_value(openai_json()).unwrap();
        assert!(resp.get_log_probs().is_empty());
    }
}
