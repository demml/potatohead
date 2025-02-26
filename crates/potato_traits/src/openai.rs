use crate::ApiHelper;
use futures_util::StreamExt;
use potato_client::client::{types::RequestType, LLMClient};
use potato_client::AsyncLLMClient;
use potato_error::PotatoError;
use potato_error::PotatoHeadError;
use potato_prompts::ChatPrompt;
use potato_providers::openai::responses::ChatCompletionChunk;
use potato_providers::{
    openai::{convert_pydantic_to_openai_json_schema, resolve_route},
    parse_openai_response,
};
use pyo3::{prelude::*, IntoPyObjectExt};
use serde_json::{json, Value};

use futures_core::Stream;
use serde::Deserialize;
use std::pin::Pin;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::error;

#[derive(Debug, Deserialize)]
pub struct ServerSentEvent {
    pub event: Option<String>,
    pub data: Vec<String>,
    pub joined_data: String,
    pub id: Option<String>,
    pub retry: Option<i32>,
}

impl ServerSentEvent {
    pub fn parse(data: &str) -> Option<Self> {
        let mut event = None;
        let mut data_lines = Vec::new();
        let mut id = None;
        let mut retry = None;

        for line in data.lines() {
            if line.is_empty() {
                continue;
            }

            //tracing::debug!("Processing line: {}", line);
            if let Some((field, value)) = line.split_once(':') {
                let value = value.trim_start();
                match field {
                    "event" => event = Some(value.to_string()),
                    "data" => {
                        if value.trim() != "[DONE]" {
                            data_lines.push(value.to_string())
                        }
                    }
                    "id" => id = Some(value.to_string()),
                    "retry" => retry = value.parse().ok(),
                    _ => {} // Ignore unknown fields
                }
            }
        }

        if data_lines.is_empty() && event.is_none() && id.is_none() && retry.is_none() {
            tracing::debug!("No valid SSE fields found");
            return None;
        }

        Some(ServerSentEvent {
            event,
            data: data_lines.clone(),
            joined_data: data_lines.join("\n"),
            id,
            retry,
        })
    }
}

#[pyclass]
pub struct OpenAIStreamResponse {
    stream: Pin<Box<dyn Stream<Item = Result<Vec<u8>, PotatoError>> + Send + Sync + 'static>>,
    rt: Arc<Runtime>,
}

impl OpenAIStreamResponse {
    pub fn new(
        stream: impl Stream<Item = Result<Vec<u8>, PotatoError>> + Send + Sync + 'static,
        rt: Arc<Runtime>,
    ) -> Self {
        OpenAIStreamResponse {
            stream: Box::pin(stream),
            rt,
        }
    }
}

#[pymethods]
impl OpenAIStreamResponse {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<Option<ChatCompletionChunk>> {
        let rt = slf.rt.clone();

        rt.block_on(async {
            match slf.stream.next().await {
                Some(Ok(bytes)) => {
                    let text = String::from_utf8_lossy(&bytes);
                    //tracing::debug!("Received bytes: {}", text);

                    if let Some(sse) = ServerSentEvent::parse(&text) {
                        // Check for [DONE] message

                        if sse.joined_data == "[DONE]" {
                            println!("Received DONE message");
                            return Ok(None);
                        }

                        // return data as string
                        let chunk = ChatCompletionChunk::from_sse_events(&sse.data);

                        match chunk {
                            Ok(chunk) => Ok(Some(chunk)),
                            Err(e) => Err(PotatoHeadError::new_err(format!(
                                "Failed to parse chunk: {}",
                                e
                            ))),
                        }
                    } else {
                        Ok(None) // Skip invalid SSE messages
                    }
                }
                Some(Err(_)) => Err(PotatoHeadError::new_err("Stream error")),
                None => Ok(None),
            }
        })
    }
}

pub struct OpenAIHelper;

impl ApiHelper for OpenAIHelper {
    fn new() -> Self {
        OpenAIHelper
    }

    fn execute_chat_request<'py, T>(
        &self,
        py: Python<'py>,
        client: &T,
        request: ChatPrompt,
        response_format: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>>
    where
        T: LLMClient,
    {
        let route = resolve_route(client.url(), &request.prompt_type)?;

        let response_format_spec = response_format
            .map(|format| convert_pydantic_to_openai_json_schema(py, format))
            .transpose()?;

        let msgs = request
            .messages
            .iter()
            .map(|msg| msg.to_spec())
            .collect::<Vec<Value>>();

        let mut spec = json!({
            "model": request.model,
            "messages": msgs,
        });

        if let Some(additional) = &request.additional_data {
            if let Some(spec_obj) = spec.as_object_mut() {
                if let Some(additional_obj) = additional.as_object() {
                    for (key, value) in additional_obj {
                        spec_obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        // if response_format exists, merge it into the spec
        if let Some(format) = response_format_spec {
            if let Some(spec_obj) = spec.as_object_mut() {
                spec_obj.insert("response_format".to_string(), format.clone());
            }
        }

        let response = client
            .request_with_retry(route, RequestType::Post, Some(spec), None, None)
            .map_err(|e| {
                error!("Failed to make request: {}", e);
                PotatoHeadError::new_err(format!("Failed to make request: {}", e))
            })?;

        // check if response was successful
        if !response.status().is_success() {
            return Err(PotatoHeadError::new_err(format!(
                "Failed to make request: {}, error: {}",
                response.status(),
                response.text().unwrap_or_default()
            )));
        }

        parse_openai_response(py, response, response_format).map_err(|e| {
            error!("Failed to parse response: {}", e);
            PotatoHeadError::new_err(format!("Failed to parse response: {}", e))
        })
        // ...existing OpenAI specific implementation...
    }

    fn execute_stream_chat_request<'py, T>(
        &self,
        py: Python<'py>,
        client: &T,
        request: ChatPrompt,
        rt: Arc<Runtime>,
    ) -> PyResult<Bound<'py, PyAny>>
    where
        T: AsyncLLMClient + LLMClient,
    {
        let route = resolve_route(client.url(), &request.prompt_type)?;

        let msgs = request
            .messages
            .iter()
            .map(|msg| msg.to_spec())
            .collect::<Vec<Value>>();

        let mut spec = json!({
            "model": request.model,
            "messages": msgs,
            "stream": true,  // Make sure streaming is enabled
        });

        if let Some(additional) = &request.additional_data {
            if let Some(spec_obj) = spec.as_object_mut() {
                if let Some(additional_obj) = additional.as_object() {
                    for (key, value) in additional_obj {
                        spec_obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        let response = rt
            .block_on(async {
                client
                    .stream_request_with_retry(route, RequestType::Post, Some(spec), None, None)
                    .await
            })
            .map_err(|e| {
                error!("Failed to make request: {}", e);
                PotatoHeadError::new_err(format!("Failed to make request: {}", e))
            })?;

        let stream = response.bytes_stream().map(|result| {
            result
                .map(|bytes| bytes.to_vec())
                .map_err(|e| PotatoError::Error(format!("Failed to read stream: {}", e)))
        });

        Ok(OpenAIStreamResponse::new(stream, rt).into_bound_py_any(py)?)
    }
}
