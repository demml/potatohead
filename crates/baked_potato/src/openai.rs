use anyhow::Context;
use anyhow::Result;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Json;
use axum::Router;
use potato_providers::{ChatCompletion, ChatCompletionChunk};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;
use tracing::{debug, instrument};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "role")]
pub enum ChatRole {
    #[serde(rename = "developer")]
    Developer(DeveloperMessage),
    #[serde(rename = "system")]
    System(SystemMessage),
    #[serde(rename = "user")]
    User(UserMessage),
    #[serde(rename = "assistant")]
    Assistant(AssistantMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentPart {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BaseMessage {
    pub content: MessageContent,
    pub name: Option<String>,
}

// Message type definitions
#[derive(Debug, Serialize, Deserialize)]
pub struct DeveloperMessage(BaseMessage);

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMessage(BaseMessage);

#[derive(Debug, Serialize, Deserialize)]
pub struct UserMessage(BaseMessage);

#[derive(Debug, Serialize, Deserialize)]
pub struct AssistantMessage(BaseMessage);

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    messages: Vec<ChatRole>,
    model: String,
    #[serde(default)]
    stream: bool,
    #[serde(default = "default_temperature")]
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logprobs: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f64>,
}

fn default_temperature() -> f32 {
    1.0
}

#[instrument(skip_all)]
pub async fn chat_completion(
    Json(chat_request): Json<ChatRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if chat_request.messages.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "message": "messages array must not be empty",
                    "type": "invalid_request_error"
                }
            })),
        ));
    }

    debug!("Chat request: {:?}", chat_request);

    if chat_request.stream {
        let value = json!(ChatCompletionChunk::default());
        return Ok((StatusCode::OK, Json(value)));
    } else {
        let value = json!(ChatCompletion::default());
        return Ok((StatusCode::OK, Json(value)));
    }
}

#[instrument(skip_all)]
pub async fn stream_chat_completion(
    Json(chat_request): Json<ChatRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if chat_request.messages.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "message": "messages array must not be empty",
                    "type": "invalid_request_error"
                }
            })),
        ));
    }

    debug!("Chat request: {:?}", chat_request);
    let response = ChatCompletion::default();
    Ok((StatusCode::OK, Json(response)))
}

pub async fn get_openai_router() -> Result<Router> {
    let result = catch_unwind(AssertUnwindSafe(|| {
        Router::new().route(&format!("/v1/chat/completions"), post(chat_completion))
    }));

    match result {
        Ok(router) => Ok(router),
        Err(_) => {
            // panic
            Err(anyhow::anyhow!("Failed to create openai router"))
                .context("Panic occurred while creating the router")
        }
    }
}
