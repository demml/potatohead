use crate::error::ProviderError;
use crate::providers::embed::EmbeddingResponse;
use crate::providers::google::auth::{GoogleAuth, GoogleUrl};
use crate::providers::google::traits::{ApiConfigExt, RequestClient};
use crate::providers::google::GeminiEmbeddingRequest;
use crate::providers::google::{
    Content, GeminiGenerateContentRequest, GenerateContentResponse, Part,
};
use crate::providers::types::build_http_client;
use crate::providers::types::{add_extra_body_to_prompt, ServiceType};

use potato_prompt::Prompt;
use potato_type::google::EmbeddingConfigTrait;
use potato_type::google::GeminiEmbeddingResponse;
use potato_type::Provider;
use reqwest::Client;
use serde::Serialize;
use tracing::{debug, instrument};

#[derive(Debug)]
pub struct GeminiApiConfig {
    base_url: String,
    service_type: ServiceType,
    auth: GoogleAuth,
}

impl ApiConfigExt for GeminiApiConfig {
    fn new(auth: GoogleAuth, service_type: ServiceType) -> Self {
        let env_base_url = std::env::var("GEMINI_API_URL").ok();
        let base_url = env_base_url.unwrap_or_else(|| GoogleUrl::Gemini.base_url(&auth));

        Self {
            base_url,
            service_type,
            auth,
        }
    }

    fn build_url(&self, model: &str) -> String {
        let endpoint = self.get_endpoint();
        format!("{}/{}:{}", self.base_url, model, endpoint)
    }

    async fn set_auth_header(
        &self,
        req: reqwest::RequestBuilder,
        auth: &GoogleAuth,
    ) -> Result<reqwest::RequestBuilder, ProviderError> {
        match auth {
            GoogleAuth::ApiKey(api_key) => Ok(req.header("x-goog-api-key", api_key)),
            GoogleAuth::GoogleCredentials(_) => Err(ProviderError::MissingAuthenticationError),
            GoogleAuth::NotSet => Err(ProviderError::MissingAuthenticationError),
        }
    }

    fn get_endpoint(&self) -> &'static str {
        // Need to return the gemini endpoint here
        &self.service_type.gemini_endpoint()
    }

    fn auth(&self) -> &GoogleAuth {
        &self.auth
    }
}

struct GeminiRequestClient;
impl RequestClient for GeminiRequestClient {}

#[derive(Debug)]
pub struct GeminiClient {
    client: Client,
    config: GeminiApiConfig,
    pub provider: Provider,
}

impl PartialEq for GeminiClient {
    fn eq(&self, other: &Self) -> bool {
        // Compare auth types without exposing internal details
        matches!(
            (&self.config.auth, &other.config.auth),
            (GoogleAuth::ApiKey(_), GoogleAuth::ApiKey(_))
                | (
                    GoogleAuth::GoogleCredentials(_),
                    GoogleAuth::GoogleCredentials(_)
                )
                | (GoogleAuth::NotSet, GoogleAuth::NotSet)
        ) && self.provider == other.provider
    }
}
impl GeminiClient {
    /// Creates a new GeminiClient embedding instance. This is a shared method that can be used in both Python and Rust.
    ///
    /// # Arguments:
    /// * `api_key`: The API key for authenticating with the Gemini API.
    /// * `base_url`: The base URL for the Gemini API (default is the Gemini API URL).
    /// * `headers`: Optional headers to include in the HTTP requests.
    ///
    /// # Returns:
    /// * `Result<GeminiClient, AgentError>`: Returns a `GeminiClient` instance on success or an `AgentError` on failure.
    pub async fn new(service_type: ServiceType) -> Result<Self, ProviderError> {
        let client = build_http_client(None)?;
        let auth = GoogleAuth::from_env().await?;
        let config = GeminiApiConfig::new(auth, service_type);

        Ok(Self {
            client,
            config,
            provider: Provider::Gemini,
        })
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
    pub async fn generate_content(
        &self,
        prompt: &Prompt,
    ) -> Result<GenerateContentResponse, ProviderError> {
        // Cant make a request without an API key
        if let GoogleAuth::NotSet = self.config.auth {
            return Err(ProviderError::MissingAuthenticationError.into());
        }

        let settings = &prompt.model_settings;

        // get the user messages from the prompt first
        let contents: Vec<Content> = prompt
            .message
            .iter()
            .map(Content::from_message)
            .collect::<Result<Vec<_>, _>>()?;

        // system messages are optional and can only be content with multiple parts
        let system_instruction: Option<Content> = if prompt.system_instruction.is_empty() {
            None
        } else {
            let parts: Result<Vec<Part>, ProviderError> = prompt
                .system_instruction
                .iter()
                .map(Part::from_message)
                .collect();

            Some(Content {
                parts: parts?,
                role: None,
            })
        };

        let mut gemini_settings = settings.get_gemini_settings().unwrap_or_default();

        if prompt.response_json_schema.is_some() {
            gemini_settings.configure_for_structured_output();
        }

        // Create the Gemini generate content request
        let chat_request = GeminiGenerateContentRequest {
            contents,
            system_instruction,
            settings: Some(gemini_settings),
            ..Default::default()
        };

        // serialize the prompt to JSON
        let mut serialized_prompt =
            serde_json::to_value(chat_request).map_err(ProviderError::SerializationError)?;

        // if settings.extra_body is provided, merge it with the prompt
        if let Some(extra_body) = settings.extra_body() {
            add_extra_body_to_prompt(&mut serialized_prompt, extra_body);
        }

        debug!(
            "Sending chat completion request to Gemini API: {:?}",
            serialized_prompt
        );

        let response = GeminiRequestClient::make_request(
            &self.client,
            &self.config,
            &prompt.model,
            &serialized_prompt,
        )
        .await?;

        let chat_response: GenerateContentResponse = response.json().await?;
        debug!("Chat completion successful");

        Ok(chat_response)
    }

    #[instrument(skip_all)]
    pub async fn create_embedding<T>(
        &self,
        inputs: Vec<String>,
        config: &T,
    ) -> Result<EmbeddingResponse, ProviderError>
    where
        T: Serialize + EmbeddingConfigTrait,
    {
        if let GoogleAuth::NotSet = self.config.auth {
            return Err(ProviderError::MissingAuthenticationError.into());
        }

        let model = config.get_model();
        let request = serde_json::to_value(GeminiEmbeddingRequest::new(inputs, config))
            .map_err(ProviderError::SerializationError)?;

        let response =
            GeminiRequestClient::make_request(&self.client, &self.config, &model, &request).await?;

        let embedding_response: GeminiEmbeddingResponse = response.json().await?;

        Ok(EmbeddingResponse::Gemini(embedding_response))
    }
}
