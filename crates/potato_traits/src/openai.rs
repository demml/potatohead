use crate::ApiHelper;
use potato_client::client::{types::RequestType, LLMClient};
use potato_error::{PotatoError, PotatoHeadError};
use potato_prompts::ChatPrompt;
use potato_providers::openai::{convert_pydantic_to_openai_json_schema, resolve_route};
use pyo3::prelude::*;
use serde_json::{json, Value};
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
    ) -> Result<(), PotatoError>
    where
        T: LLMClient,
    {
        let route = resolve_route(client.url(), &request.prompt_type)?;

        let response_format = response_format
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
        if let Some(format) = &response_format {
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
            return Err(PotatoError::Error(format!(
                "Failed to make request: {}, error: {}",
                response.status(),
                response.text().unwrap_or_default()
            )));
        }

        let json_response: serde_json::Value = response.json().map_err(|e| {
            error!("Failed to parse JSON response: {}", e);
            PotatoHeadError::new_err(format!("Failed to parse JSON response: {}", e))
        })?;
        println!("{:?}", json_response);

        Ok(())
        // ...existing OpenAI specific implementation...
    }
}
