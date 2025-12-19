use crate::error::ProviderError;

use crate::providers::anthropic::types::{
    AnthropicChatResponse, AnthropicMessage, AnthropicMessageRequest,
};

use crate::providers::types::add_extra_body_to_prompt;
use crate::providers::types::build_http_client;
use crate::providers::types::ServiceType;
use http::{header, HeaderMap};
use potato_prompt::Prompt;
use potato_type::{Common, Provider};
use reqwest::Client;
use reqwest::Response;
use serde_json::Value;
use tracing::{debug, error, instrument};

const ANTHROPIC_BETA: &str = "anthropic-beta";
const ANTHROPIC_STRUCTURED_OUTPUT: &str = "structured-outputs-2025-11-13";
const ANTHROPIC_VERSION_KEY: &str = "anthropic-version";
const ANTHROPIC_VERSION_VALUE: &str = "2023-06-01";

#[derive(Debug, PartialEq)]
pub enum AnthropicAuth {
    ApiKey(String),
    NotSet,
}

impl AnthropicAuth {
    /// Try to create authentication from environment variables
    /// This will first look for a `OPENAI_API_KEY`.
    /// If not found, it will attempt to use Google Application Credentials
    /// to create a token source for authentication.
    ///
    #[instrument(skip_all)]
    pub fn from_env() -> Self {
        // First try API key
        let api_key =
            std::env::var("ANTHROPIC_API_KEY").unwrap_or_else(|_| Common::Undefined.to_string());

        if api_key != Common::Undefined.to_string() {
            debug!("Using Anthropic API key from environment variable");
            return Self::ApiKey(api_key);
        }

        Self::NotSet
    }
}

struct AnthropicPaths {}
impl AnthropicPaths {
    fn base_url() -> String {
        "https://api.anthropic.com/v1".to_string()
    }
}

#[derive(Debug, PartialEq)]
pub struct AnthropicApiConfig {
    base_url: String,
    service_type: ServiceType,
    auth: AnthropicAuth,
}

impl AnthropicApiConfig {
    fn new(service_type: ServiceType) -> Result<Self, ProviderError> {
        let env_base_url = std::env::var("ANTHROPIC_API_URL").ok();
        let base_url = env_base_url.unwrap_or_else(AnthropicPaths::base_url);
        let auth = AnthropicAuth::from_env();

        Ok(Self {
            base_url,
            service_type,
            auth,
        })
    }

    fn build_url(&self) -> String {
        let endpoint = self.get_endpoint();
        format!("{}/{}", self.base_url, endpoint)
    }

    // uses x-api-key
    async fn set_auth_header(
        &self,
        req: reqwest::RequestBuilder,
    ) -> Result<reqwest::RequestBuilder, ProviderError> {
        match &self.auth {
            AnthropicAuth::ApiKey(api_key) => Ok(req.header("x-api-key", api_key)),
            AnthropicAuth::NotSet => Ok(req),
        }
    }

    fn get_endpoint(&self) -> &'static str {
        self.service_type.anthropic_endpoint()
    }
}

#[derive(Debug)]
pub struct AnthropicClient {
    client: Client,
    config: AnthropicApiConfig,
    pub provider: Provider,
}

impl PartialEq for AnthropicClient {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config && self.provider == other.provider
    }
}

impl AnthropicClient {
    /// Creates a new AnthropicClient instance. This is a shared method that can be used in both Python and Rust.
    /// # Arguments:
    /// * `service_type`: The type of service to use (e.g., Chat, Embed).
    /// # Returns:
    /// * `Result<AnthropicClient, ProviderError>`: Returns an `AnthropicClient` instance on success or an `ProviderError` on failure.
    pub fn new(service_type: ServiceType) -> Result<Self, ProviderError> {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(
            ANTHROPIC_VERSION_KEY,
            header::HeaderValue::from_static(ANTHROPIC_VERSION_VALUE),
        );

        let client = build_http_client(Some(default_headers))?;
        let config = AnthropicApiConfig::new(service_type)?;
        Ok(Self {
            client,
            config,
            provider: Provider::Anthropic,
        })
    }

    /// Generic helper for executing a request to reduce boilerplate
    async fn make_request(
        &self,
        object: &Value,
        additional_headers: HeaderMap,
    ) -> Result<Response, ProviderError> {
        let request = self.client.post(self.config.build_url()).json(&object);
        let request = self.config.set_auth_header(request).await?;
        let request = request.headers(additional_headers);
        let response = request.send().await.map_err(ProviderError::RequestError)?;

        let status = response.status();
        if !status.is_success() {
            // print the response body for debugging
            error!("Anthropic API request failed with status: {}", status);
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "No response body".to_string());
            return Err(ProviderError::CompletionError(body, status));
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
    /// * `Result<ChatResponse, ProviderError>`: Returns a `ChatResponse` on success or an `ProviderError` on failure.
    ///
    #[instrument(name = "anthropic_chat_completion", skip_all)]
    pub async fn chat_completion(
        &self,
        prompt: &Prompt,
    ) -> Result<AnthropicChatResponse, ProviderError> {
        if let AnthropicAuth::NotSet = self.config.auth {
            return Err(ProviderError::MissingAuthenticationError);
        }

        let settings = &prompt.model_settings;
        let mut additional_headers = HeaderMap::new();

        // get the system messages from the prompt first
        let mut messages: Vec<AnthropicMessage> = prompt
            .system_instruction
            .iter()
            .map(AnthropicMessage::from_message)
            .collect::<Result<Vec<_>, _>>()
            .inspect_err(|_| {
                error!("Failed to convert system instructions to Anthropic message")
            })?;

        // Add user messages to the chat
        messages.extend(
            prompt
                .message
                .iter()
                .map(AnthropicMessage::from_message)
                .collect::<Result<Vec<_>, _>>()
                .inspect_err(|_| error!("Failed to convert prompt message to Anthropic message"))?,
        );

        // if prompt has response_json_schema,
        let schema = prompt
            .response_json_schema
            .as_ref()
            .map(|schema| self.create_structured_output_schema(schema));

        // Create the Anthropic chat request
        let chat_request = AnthropicMessageRequest {
            model: prompt.model.clone(),
            messages,
            settings: prompt.model_settings.get_anthropic_settings(),
        };

        // serialize the prompt to JSON
        let mut serialized_prompt =
            serde_json::to_value(chat_request).map_err(ProviderError::SerializationError)?;

        // if schema is provided, add the headers for structured output

        if let Some(schema) = schema {
            // add output_format to the prompt
            additional_headers.insert(
                ANTHROPIC_BETA,
                header::HeaderValue::from_static(ANTHROPIC_STRUCTURED_OUTPUT),
            );
            if let Value::Object(map) = &mut serialized_prompt {
                map.insert("output_format".to_string(), schema);
            }
        }

        // if settings.extra_body is provided, merge it with the prompt
        if let Some(extra_body) = settings.extra_body() {
            add_extra_body_to_prompt(&mut serialized_prompt, extra_body);
        }

        debug!(
            "Sending chat completion request to Anthropic API: {:?}",
            serialized_prompt
        );

        let response = self
            .make_request(&serialized_prompt, additional_headers)
            .await?;

        let chat_response: AnthropicChatResponse = response.json().await?;
        debug!("Chat completion successful");

        Ok(chat_response)
    }

    fn create_structured_output_schema(&self, json_schema: &Value) -> Value {
        serde_json::json!({
            "type": "json_schema",
            "schema": json_schema,

        })
    }
}
