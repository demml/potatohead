use crate::error::ProviderError;

use crate::providers::embed::EmbeddingResponse;
use crate::providers::openai::{
    OpenAIChatMessage, OpenAIChatRequest, OpenAIChatResponse, OpenAIEmbeddingRequest,
};
use crate::providers::types::add_extra_body_to_prompt;
use crate::providers::types::build_http_client;
use crate::providers::types::ServiceType;
use potato_prompt::Prompt;
use potato_type::openai::embedding::OpenAIEmbeddingResponse;
use potato_type::{Common, Provider};
use reqwest::Client;
use reqwest::Response;
use serde::Serialize;
use serde_json::Value;
use tracing::{debug, error, instrument};

#[derive(Debug, PartialEq)]
pub enum OpenAIAuth {
    ApiKey(String),
    NotSet,
}

impl OpenAIAuth {
    /// Try to create authentication from environment variables
    /// This will first look for a `OPENAI_API_KEY`.
    /// If not found, it will attempt to use Google Application Credentials
    /// to create a token source for authentication.
    ///
    #[instrument(skip_all)]
    pub fn from_env() -> Self {
        // First try API key
        let api_key =
            std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| Common::Undefined.to_string());

        if api_key != Common::Undefined.to_string() {
            debug!("Using OpenAI API key from environment variable");
            return Self::ApiKey(api_key);
        }

        Self::NotSet
    }
}

struct OpenAIPaths {}
impl OpenAIPaths {
    fn base_url() -> String {
        "https://api.openai.com/v1".to_string()
    }
}

#[derive(Debug, PartialEq)]
pub struct OpenAIApiConfig {
    base_url: String,
    service_type: ServiceType,
    auth: OpenAIAuth,
}

impl OpenAIApiConfig {
    fn new(service_type: ServiceType) -> Result<Self, ProviderError> {
        let env_base_url = std::env::var("OPENAI_API_URL").ok();
        let base_url = env_base_url.unwrap_or_else(OpenAIPaths::base_url);
        let auth = OpenAIAuth::from_env();

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

    async fn set_auth_header(
        &self,
        req: reqwest::RequestBuilder,
    ) -> Result<reqwest::RequestBuilder, ProviderError> {
        match &self.auth {
            OpenAIAuth::ApiKey(api_key) => Ok(req.bearer_auth(api_key)),
            OpenAIAuth::NotSet => Ok(req),
        }
    }

    fn get_endpoint(&self) -> &'static str {
        // Need to return the gemini endpoint here
        self.service_type.openai_endpoint()
    }
}

#[derive(Debug)]
pub struct OpenAIClient {
    client: Client,
    config: OpenAIApiConfig,
    pub provider: Provider,
}

impl PartialEq for OpenAIClient {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config && self.provider == other.provider
    }
}

impl OpenAIClient {
    /// Creates a new OpenAIClient instance. This is a shared method that can be used in both Python and Rust.
    /// # Arguments:
    /// * `service_type`: The type of service to use (e.g., Chat, Embed).
    /// # Returns:
    /// * `Result<OpenAIClient, ProviderError>`: Returns an `OpenAIClient` instance on success or an `ProviderError` on failure.
    pub fn new(service_type: ServiceType) -> Result<Self, ProviderError> {
        let client = build_http_client(None)?;
        let config = OpenAIApiConfig::new(service_type)?;
        Ok(Self {
            client,
            config,
            provider: Provider::OpenAI,
        })
    }

    /// Generic helper for executing a request to reduce boilerplate
    async fn make_request(&self, object: &Value) -> Result<Response, ProviderError> {
        let request = self.client.post(self.config.build_url()).json(&object);
        let request = self.config.set_auth_header(request).await?;
        let response = request.send().await.map_err(ProviderError::RequestError)?;

        let status = response.status();
        if !status.is_success() {
            // print the response body for debugging
            error!("OpenAI API request failed with status: {}", status);
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
    #[instrument(skip_all)]
    pub async fn chat_completion(
        &self,
        prompt: &Prompt,
    ) -> Result<OpenAIChatResponse, ProviderError> {
        // Cant make a request without an API key

        if let OpenAIAuth::NotSet = self.config.auth {
            return Err(ProviderError::MissingAuthenticationError);
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
            serde_json::to_value(chat_request).map_err(ProviderError::SerializationError)?;

        // if settings.extra_body is provided, merge it with the prompt
        if let Some(extra_body) = settings.extra_body() {
            add_extra_body_to_prompt(&mut serialized_prompt, extra_body);
        }

        debug!(
            "Sending chat completion request to OpenAI API: {:?}",
            serialized_prompt
        );

        let response = self.make_request(&serialized_prompt).await?;

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
    pub async fn create_embedding<T>(
        &self,
        inputs: Vec<String>,
        config: &T,
    ) -> Result<EmbeddingResponse, ProviderError>
    where
        T: Serialize,
    {
        if let OpenAIAuth::NotSet = self.config.auth {
            return Err(ProviderError::MissingAuthenticationError);
        }

        let request = serde_json::to_value(OpenAIEmbeddingRequest::new(inputs, config))
            .map_err(ProviderError::SerializationError)?;

        debug!("Sending embedding request to OpenAI API: {:?}", request);

        let response = self.make_request(&request).await?;

        let embedding_response: OpenAIEmbeddingResponse = response.json().await?;

        Ok(EmbeddingResponse::OpenAI(embedding_response))
    }
}
