use crate::agents::error::AgentError;

use crate::agents::provider::openai::{
    OpenAIChatMessage, OpenAIChatRequest, OpenAIChatResponse, OpenAIEmbeddingRequest,
};
use crate::agents::provider::types::add_extra_body_to_prompt;
use crate::agents::provider::types::build_http_client;
use potato_prompt::Prompt;
use potato_type::openai::embedding::OpenAIEmbeddingResponse;
use potato_type::{Common, Provider};
use reqwest::header::AUTHORIZATION;
use reqwest::Client;
use reqwest::Response;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, error, instrument};

enum OpenAIPaths {
    ChatCompletions,
    Embeddings,
}

impl OpenAIPaths {
    fn path(&self) -> &str {
        match self {
            OpenAIPaths::ChatCompletions => "chat/completions",
            OpenAIPaths::Embeddings => "embeddings",
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpenAIClient {
    client: Client,
    api_key: String,
    base_url: String,
    api_key_set: bool,
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
        let (api_key, api_key_set) = match api_key {
            // If api_key is provided, use it
            Some(key) => (key, true),

            // If api_key is None, check the environment variable
            None => match std::env::var("OPENAI_API_KEY") {
                // If the environment variable is set, use it
                Ok(env_key) if !env_key.is_empty() => (env_key, true),

                // If the environment variable is not set, use a placeholder and set api_key_set to false
                _ => (Common::Undefined.to_string(), false),
            },
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
            api_key_set,
        })
    }

    /// Generic helper for executing a request to reduce boilerplate
    async fn make_request(&self, path: &str, object: &Value) -> Result<Response, AgentError> {
        let response = self
            .client
            .post(format!("{}/{}", self.base_url, path))
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .json(object)
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

        Ok(response)
    }

    /// Sends a chat completion request to the OpenAI API. This is a rust-only method
    /// that allows you to interact with the OpenAI API without needing Python.
    ///
    /// # Arguments:
    /// * `messages`: A slice of `Message` objects representing user messages.
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
        // Cant make a request without an API key

        if !self.api_key_set {
            return Err(AgentError::MissingOpenAIApiKeyError);
        }

        let settings = &prompt.model_settings;

        // get the system messages from the prompt first
        let mut messages: Vec<OpenAIChatMessage> = prompt
            .system_instruction
            .iter()
            .map(OpenAIChatMessage::from_message)
            .collect::<Result<Vec<_>, _>>()?;

        // Add user messages to the chat
        messages.extend(
            prompt
                .message
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
            model: prompt.model.clone(),
            messages,
            settings: prompt.model_settings.get_openai_settings(),
            response_format: schema,
        };

        // serialize the prompt to JSON
        let mut serialized_prompt =
            serde_json::to_value(chat_request).map_err(AgentError::SerializationError)?;

        // if settings.extra_body is provided, merge it with the prompt
        if let Some(extra_body) = settings.extra_body() {
            add_extra_body_to_prompt(&mut serialized_prompt, extra_body);
        }

        debug!(
            "Sending chat completion request to OpenAI API: {:?}",
            serialized_prompt
        );

        let response = self
            .make_request(OpenAIPaths::ChatCompletions.path(), &serialized_prompt)
            .await?;

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

    #[instrument(skip_all)]
    pub async fn async_create_embedding<T>(
        &self,
        inputs: Vec<String>,
        config: T,
    ) -> Result<OpenAIEmbeddingResponse, AgentError>
    where
        T: Serialize,
    {
        if !self.api_key_set {
            return Err(AgentError::MissingOpenAIApiKeyError);
        }

        let request = serde_json::to_value(OpenAIEmbeddingRequest::new(inputs, config))
            .map_err(AgentError::SerializationError)?;

        let response = self
            .make_request(OpenAIPaths::Embeddings.path(), &request)
            .await?;

        Ok(response.json().await?)
    }
}
