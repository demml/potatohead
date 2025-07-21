use crate::agents::error::AgentError;
use crate::agents::provider::openai::{OpenAIChatMessage, OpenAIChatRequest, OpenAIChatResponse};
use crate::agents::provider::types::build_http_client;
use potato_prompt::Prompt;
use potato_type::Provider;
use reqwest::header::AUTHORIZATION;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, error, instrument};
#[derive(Debug, Clone)]
pub struct OpenAIClient {
    client: Client,
    api_key: String,
    base_url: String,
    pub provider: Provider,
}

impl PartialEq for OpenAIClient {
    fn eq(&self, other: &Self) -> bool {
        self.api_key == other.api_key
            && self.base_url == other.base_url
            && self.provider == other.provider
    }
}

impl OpenAIClient {
    /// Creates a new OpenAIClient instance. This is a shared method that can be used in both Python and Rust.
    ///
    /// # Arguments:
    /// * `api_key`: The API key for authenticating with the OpenAI API.
    /// * `base_url`: The base URL for the OpenAI API (default is the OpenAI API URL).
    /// * `headers`: Optional headers to include in the HTTP requests.
    ///
    /// # Returns:
    /// * `Result<OpenAIClient, AgentError>`: Returns an `OpenAIClient` instance on success or an `AgentError` on failure.
    pub fn new(
        api_key: Option<String>,
        base_url: Option<String>,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Self, AgentError> {
        let client = build_http_client(headers)?;

        //  if optional api_key is None, check the environment variable `OPENAI_API_KEY`
        let api_key = match api_key {
            Some(key) => key,
            None => {
                std::env::var("OPENAI_API_KEY").map_err(AgentError::MissingOpenAIApiKeyError)?
            }
        };

        // if optional base_url is None, use the default OpenAI API URL
        let env_base_url = std::env::var("OPENAI_API_URL").ok();
        let base_url = base_url
            .unwrap_or_else(|| env_base_url.unwrap_or_else(|| Provider::OpenAI.url().to_string()));

        debug!("Creating OpenAIClient with base URL with key: {}", base_url);

        Ok(Self {
            client,
            api_key,
            base_url,
            provider: Provider::OpenAI,
        })
    }

    /// Sends a chat completion request to the OpenAI API. This is a rust-only method
    /// that allows you to interact with the OpenAI API without needing Python.
    ///
    /// # Arguments:
    /// * `user_messages`: A slice of `Message` objects representing user messages.
    /// * `developer_messages`: A slice of `Message` objects representing developer messages.
    /// * `settings`: A reference to `ModelSettings` containing model configuration.
    ///
    /// # Returns:
    /// * `Result<ChatResponse, AgentError>`: Returns a `ChatResponse` on success or an `AgentError` on failure.
    ///
    #[instrument(skip_all)]
    pub async fn async_chat_completion(
        &self,
        prompt: &Prompt,
    ) -> Result<OpenAIChatResponse, AgentError> {
        let settings = &prompt.model_settings;

        // get the system messages from the prompt first
        let mut messages: Vec<OpenAIChatMessage> = prompt
            .system_message
            .iter()
            .map(OpenAIChatMessage::from_message)
            .collect::<Result<Vec<_>, _>>()?;

        // Add user messages to the chat
        messages.extend(
            prompt
                .user_message
                .iter()
                .map(OpenAIChatMessage::from_message)
                .collect::<Result<Vec<_>, _>>()?,
        );

        // if prompt has response_json_schema, format it for openai
        let schema = prompt
            .response_json_schema
            .as_ref()
            .map(|schema| self.create_structured_output_schema(schema));

        // Create the OpenAI chat request
        let chat_request = OpenAIChatRequest {
            model: settings.model.clone(),
            messages,
            max_completion_tokens: settings.max_tokens,
            temperature: settings.temperature,
            top_p: settings.top_p,
            frequency_penalty: settings.frequency_penalty,
            presence_penalty: settings.presence_penalty,
            parallel_tool_calls: settings.parallel_tool_calls,
            logit_bias: settings.logit_bias.clone(),
            seed: settings.seed,
            response_format: schema,
        };

        // serialize the prompt to JSON
        let mut serialized_prompt =
            serde_json::to_value(chat_request).map_err(AgentError::SerializationError)?;

        // if settings.extra_body is provided, merge it with the prompt
        if let Some(extra_body) = &settings.extra_body {
            if let (Some(prompt_obj), Some(extra_obj)) =
                (serialized_prompt.as_object_mut(), extra_body.as_object())
            {
                // Merge the extra_body fields into prompt
                for (key, value) in extra_obj {
                    prompt_obj.insert(key.clone(), value.clone());
                }
            }
        }

        debug!(
            "Sending chat completion request to OpenAI API: {:?}",
            serialized_prompt
        );

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .json(&serialized_prompt)
            .send()
            .await
            .map_err(AgentError::RequestError)?;

        let status = response.status();
        if !status.is_success() {
            // print the response body for debugging
            error!("OpenAI API request failed with status: {}", status);
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "No response body".to_string());
            return Err(AgentError::ChatCompletionError(body, status));
        }

        let chat_response: OpenAIChatResponse = response.json().await?;
        debug!("Chat completion successful");

        Ok(chat_response)
    }

    fn create_structured_output_schema(&self, json_schema: &Value) -> Value {
        // get title from schema
        let title = json_schema
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("StructuredOutput");

        serde_json::json!({
            "type": "json_schema",
            "json_schema": {
                "name": title,
                "schema": json_schema,
                "strict": true
            }
        })
    }
}
