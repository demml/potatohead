use crate::{ApiHelper, StreamResponse};
use futures_util::StreamExt;
use potato_client::client::{types::RequestType, LLMClient};
use potato_client::AsyncLLMClient;
use potato_error::PotatoError;
use potato_error::PotatoHeadError;
use potato_prompts::ChatPrompt;
use potato_providers::{
    openai::{convert_pydantic_to_openai_json_schema, resolve_route},
    parse_openai_response,
};
use pyo3::prelude::*;
use serde_json::{json, Value};

use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::error;

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

    fn execute_stream_chat_request<T>(
        &self,
        client: &T,
        request: ChatPrompt,
        rt: Arc<Runtime>,
    ) -> PyResult<StreamResponse>
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

        Ok(StreamResponse::new(stream, rt))
    }
}
