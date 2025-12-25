use crate::error::ProviderError;
use crate::providers::types::build_http_client;
use crate::providers::types::ServiceType;
use http::{header, HeaderMap};
use potato_type::anthropic::v1::response::AnthropicChatResponse;
use potato_type::prompt::Prompt;
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

    /// Sends a message request to the OpenAI API. This is a rust-only method
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
    pub async fn message(&self, prompt: &Prompt) -> Result<AnthropicChatResponse, ProviderError> {
        if let AnthropicAuth::NotSet = self.config.auth {
            return Err(ProviderError::MissingAuthenticationError);
        }

        let mut additional_headers = HeaderMap::new();

        // get request from prompt.request
        if prompt.request.has_structured_output() {
            additional_headers.insert(
                ANTHROPIC_BETA,
                header::HeaderValue::from_static(ANTHROPIC_STRUCTURED_OUTPUT),
            );
        }

        // in most cases this will be a direct conversion of struct to serde_json::Value
        // however, this allows for extension in the future if we want to allow conversion of one type to another
        // i.e., converting an anthropic request to a gemini request
        let request_body = prompt.request.to_request(&self.provider)?;

        debug!(
            "Sending message request to Anthropic API: {:?}",
            request_body
        );

        let response = self.make_request(&request_body, additional_headers).await?;
        let chat_response: AnthropicChatResponse = response.json().await?;
        debug!("Message successful");

        Ok(chat_response)
    }
}
