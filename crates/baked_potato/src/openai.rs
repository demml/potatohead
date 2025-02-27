use anyhow::Context;
use anyhow::Result;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Json;
use axum::Router;
use potato_prompts::Message;
use potato_providers::ChatCompletion;
use serde::{Deserialize, Serialize};
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;
use tracing::{debug, instrument};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    messages: Vec<Message>,
    model: String,
    stream: Option<bool>,
    temperature: Option<f32>,
    top_p: Option<f64>,
    user: Option<String>,
    n: Option<i64>,
    max_completion_tokens: Option<i64>,
    logprobs: Option<i64>,
    frequency_penalty: Option<f64>,
}

#[instrument(skip_all)]
pub async fn chat_completion(
    Json(chat_request): Json<ChatCompletion>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    debug!("Chat request: {:?}", chat_request);
    let response = ChatCompletion::default();
    Ok((StatusCode::OK, Json(response)))
}

pub async fn get_openai_router() -> Result<Router> {
    let result = catch_unwind(AssertUnwindSafe(|| {
        Router::new().route(&format!("openai/chat_completion"), post(chat_completion))
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
