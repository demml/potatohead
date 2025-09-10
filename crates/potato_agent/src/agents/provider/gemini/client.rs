use crate::agents::embed::EmbeddingResponse;
use crate::agents::error::AgentError;
use crate::agents::provider::gemini::auth::GeminiAuth;
use crate::agents::provider::gemini::error::GoogleError;
use crate::agents::provider::gemini::GeminiEmbeddingRequest;
use crate::agents::provider::gemini::{
    Content, GeminiGenerateContentRequest, GenerateContentResponse, Part,
};
use crate::agents::provider::types::add_extra_body_to_prompt;
use crate::agents::provider::types::build_http_client;
use potato_prompt::Prompt;
use potato_type::google::EmbeddingConfigTrait;
use potato_type::google::GeminiEmbeddingResponse;
use potato_type::Provider;
use reqwest::Client;
use reqwest::Response;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, error, instrument};
enum GeminiPaths {
    GenerateContent,
    Embeddings,
}

impl GeminiPaths {
    fn path(&self) -> &str {
        match self {
            GeminiPaths::GenerateContent => "generateContent",
            GeminiPaths::Embeddings => "embedContent",
        }
    }
}

#[derive(Debug)]
pub struct GeminiClient {
    client: Client,
    auth: GeminiAuth,
    base_url: String,
    pub provider: Provider,
}

impl PartialEq for GeminiClient {
    fn eq(&self, other: &Self) -> bool {
        self.base_url == other.base_url && self.provider == other.provider
    }
}

impl GeminiClient {
    /// Creates a new GeminiClient instance. This is a shared method that can be used in both Python and Rust.
    ///
    /// # Arguments:
    /// * `api_key`: The API key for authenticating with the Gemini API.
    /// * `base_url`: The base URL for the Gemini API (default is the Gemini API URL).
    /// * `headers`: Optional headers to include in the HTTP requests.
    ///
    /// # Returns:
    /// * `Result<OpenAIClient, AgentError>`: Returns an `OpenAIClient` instance on success or an `AgentError` on failure.
    pub async fn new(headers: Option<HashMap<String, String>>) -> Result<Self, AgentError> {
        let client = build_http_client(headers)?;

        let auth = GeminiAuth::from_env().await?;
        let base_url = auth.base_url();

        debug!("Creating GeminiClient with base URL: {}", base_url);

        Ok(Self {
            client,
            auth,
            base_url,
            provider: Provider::Gemini,
        })
    }

    async fn make_request(&self, path: &str, object: &Value) -> Result<Response, AgentError> {
        let mut request = self
            .client
            .post(format!("{}/{}", self.base_url, path))
            .json(&object);

        // match on the auth type to set the appropriate headers
        request = match &self.auth {
            GeminiAuth::ApiKey(api_key) => request.header("x-goog-api-key", api_key),
            GeminiAuth::GoogleCredentials(creds) => {
                request.header("Authorization", creds.get_access_token().await?)
            }
            GeminiAuth::NotSet => request,
        };

        let response = request.send().await.map_err(AgentError::RequestError)?;

        let status = response.status();
        if !status.is_success() {
            // print the response body for debugging
            error!("Gemini API request failed with status: {}", status);

            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "No response body".to_string());

            return Err(AgentError::CompletionError(body, status));
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
    pub async fn async_generate_content(
        &self,
        prompt: &Prompt,
    ) -> Result<GenerateContentResponse, AgentError> {
        // Cant make a request without an API key
        if let GeminiAuth::NotSet = self.auth {
            return Err(GoogleError::MissingAuthenticationError.into());
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
            let parts: Result<Vec<Part>, AgentError> = prompt
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
            serde_json::to_value(chat_request).map_err(AgentError::SerializationError)?;

        // if settings.extra_body is provided, merge it with the prompt
        if let Some(extra_body) = settings.extra_body() {
            add_extra_body_to_prompt(&mut serialized_prompt, extra_body);
        }

        debug!(
            "Sending chat completion request to Gemini API: {:?}",
            serialized_prompt
        );

        let response = self
            .make_request(
                &format!("{}:{}", prompt.model, GeminiPaths::GenerateContent.path()),
                &serialized_prompt,
            )
            .await?;
        let chat_response: GenerateContentResponse = response.json().await?;
        debug!("Chat completion successful");

        Ok(chat_response)
    }

    #[instrument(skip_all)]
    pub async fn async_create_embedding<T>(
        &self,
        inputs: Vec<String>,
        config: &T,
    ) -> Result<EmbeddingResponse, AgentError>
    where
        T: Serialize + EmbeddingConfigTrait,
    {
        if let GeminiAuth::NotSet = self.auth {
            return Err(GoogleError::MissingAuthenticationError.into());
        }

        let model = config.get_model();
        let path = format!("{}:{}", model, GeminiPaths::Embeddings.path());

        let request = serde_json::to_value(GeminiEmbeddingRequest::new(inputs, config))
            .map_err(AgentError::SerializationError)?;

        let response = self.make_request(&path, &request).await?;
        let embedding_response: GeminiEmbeddingResponse = response.json().await?;

        Ok(EmbeddingResponse::Gemini(embedding_response))
    }
}
