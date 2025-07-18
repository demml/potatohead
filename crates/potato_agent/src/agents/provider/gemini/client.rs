use crate::agents::error::AgentError;
use crate::agents::provider::gemini::{
    Content, GeminiGenerateContentRequest, GenerateContentResponse, GenerationConfig, Part, Schema,
};
use crate::agents::provider::types::{build_http_client, Provider};
use potato_prompt::Prompt;
use reqwest::Client;
use std::collections::HashMap;
use tracing::{debug, error};

#[derive(Debug, Clone)]
pub struct GeminiClient {
    client: Client,
    api_key: String,
    base_url: String,
    pub provider: Provider,
}

impl PartialEq for GeminiClient {
    fn eq(&self, other: &Self) -> bool {
        self.api_key == other.api_key
            && self.base_url == other.base_url
            && self.provider == other.provider
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
    pub fn new(
        api_key: Option<String>,
        base_url: Option<String>,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Self, AgentError> {
        let client = build_http_client(headers)?;

        //  if optional api_key is None, check the environment variable `GEMINI_API_KEY`
        let api_key = match api_key {
            Some(key) => key,
            None => {
                std::env::var("GEMINI_API_KEY").map_err(AgentError::MissingGeminiApiKeyError)?
            }
        };

        // if optional base_url is None, use the default Gemini API URL
        let env_base_url = std::env::var("GEMINI_API_URL").ok();
        let base_url = base_url
            .unwrap_or_else(|| env_base_url.unwrap_or_else(|| Provider::Gemini.url().to_string()));

        debug!("Creating GeminiClient with base URL with key: {}", base_url);

        Ok(Self {
            client,
            api_key,
            base_url,
            provider: Provider::Gemini,
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
    pub async fn async_generate_content(
        &self,
        prompt: &Prompt,
    ) -> Result<GenerateContentResponse, AgentError> {
        let settings = &prompt.model_settings;

        // get the user messages from the prompt first
        let contents: Vec<Content> = prompt
            .user_message
            .iter()
            .map(Content::from_message)
            .collect::<Result<Vec<_>, _>>()?;

        // system messages are optional and can only be content with multiple parts
        let system_instruction: Option<Content> = if prompt.system_message.is_empty() {
            None
        } else {
            let parts: Result<Vec<Part>, AgentError> = prompt
                .system_message
                .iter()
                .map(Part::from_message)
                .collect();

            Some(Content {
                parts: parts?,
                role: None,
            })
        };

        let response_schema = if prompt.response_format.is_some() {
            let schema: Schema =
                serde_json::from_value(prompt.response_format.clone().unwrap_or_default())?;
            Some(schema)
        } else {
            None
        };

        let generation_config = GenerationConfig {
            temperature: settings.temperature,
            top_p: settings.top_p,
            max_output_tokens: settings.max_tokens.as_ref().map(|v| *v as i32),
            top_k: settings.top_k,
            stop_sequences: settings.stop_sequences.clone(),
            frequency_penalty: settings.frequency_penalty,
            presence_penalty: settings.presence_penalty,
            seed: settings.seed.as_ref().map(|v| *v as i32),
            response_schema,
            ..Default::default()
        };

        // Create the Gemini generate content request
        let chat_request = GeminiGenerateContentRequest {
            contents,
            system_instruction,
            generation_config: Some(generation_config),
            ..Default::default()
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
                    // if key is "generation_config", we need to merge it into the existing generation_config
                    if key == "generation_config" {
                        if let Some(gen_config) = prompt_obj.get_mut("generation_config") {
                            if let (Some(gen_config_obj), Some(extra_gen_config)) =
                                (gen_config.as_object_mut(), value.as_object())
                            {
                                for (gen_key, gen_value) in extra_gen_config {
                                    gen_config_obj.insert(gen_key.clone(), gen_value.clone());
                                }
                            }
                        } else {
                            prompt_obj.insert(key.clone(), value.clone());
                        }
                    } else {
                        prompt_obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        debug!(
            "Sending chat completion request to Gemini API: {:?}",
            serialized_prompt
        );

        let response = self
            .client
            .post(format!(
                "{}/{}:generateContent",
                self.base_url, settings.model
            ))
            .header("x-goog-api-key", &self.api_key)
            .json(&serialized_prompt)
            .send()
            .await
            .map_err(AgentError::RequestError)?;

        let status = response.status();
        if !status.is_success() {
            // print the response body for debugging
            error!("Gemini API request failed with status: {}", status);
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "No response body".to_string());
            return Err(AgentError::ChatCompletionError(body, status));
        }

        let chat_response: GenerateContentResponse = response.json().await?;
        debug!("Chat completion successful");

        Ok(chat_response)
    }
}
