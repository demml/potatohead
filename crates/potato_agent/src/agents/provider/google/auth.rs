use crate::agents::provider::google::error::GoogleError;
use base64::prelude::*;
use gcloud_auth::credentials::CredentialsFile;
use gcloud_auth::{project::Config, token::DefaultTokenSourceProvider};
use std::env;
use tracing::{debug, instrument};

const SCOPES: [&str; 1] = ["https://www.googleapis.com/auth/cloud-platform"];

#[derive(Debug)]
pub struct GoogleCredentials {
    pub project_id: String,
    pub location: String,
    pub token_provider: DefaultTokenSourceProvider,
}

impl GoogleCredentials {
    pub fn from_token_provider(
        token_provider: DefaultTokenSourceProvider,
    ) -> Result<Self, GoogleError> {
        // attempt to retrieve project_id from token_provider, if None attempt to load from env var
        let project_id = match &token_provider.project_id {
            Some(id) => id.clone(),
            // attempt to load GOOGLE_CLOUD_PROJECT env var if project_id is None
            None => match env::var("GOOGLE_CLOUD_PROJECT") {
                Ok(val) => val,
                Err(_) => Err(GoogleError::NoProjectIdFound)?,
            },
        };

        let location =
            env::var("GOOGLE_CLOUD_LOCATION").unwrap_or_else(|_| "us-central1".to_string());

        Ok(GoogleCredentials {
            project_id,
            location,
            token_provider,
        })
    }

    pub async fn get_access_token(&self) -> Result<String, GoogleError> {
        let token = self
            .token_provider
            .token_source()
            .token()
            .await
            .map_err(|e| GoogleError::TokenError(e.to_string()))?;

        Ok(token)
    }
}

#[allow(unused_imports)]
use token_source::TokenSourceProvider;

pub async fn create_token_provider() -> Result<GoogleCredentials, GoogleError> {
    let creds = CredentialBuilder::new().await?.creds;

    let config = Config::default().with_scopes(&SCOPES);

    let provider =
        DefaultTokenSourceProvider::new_with_credentials(config, Box::new(creds)).await?;

    GoogleCredentials::from_token_provider(provider)
}

pub struct CredentialBuilder {
    pub creds: CredentialsFile,
}

impl CredentialBuilder {
    pub async fn new() -> Result<Self, GoogleError> {
        let creds = Self::build().await?;
        let creds = CredentialBuilder { creds };

        Ok(creds)
    }

    #[instrument(skip_all)]
    async fn build() -> Result<CredentialsFile, GoogleError> {
        if let Ok(base64_creds) = env::var("GOOGLE_ACCOUNT_JSON_BASE64") {
            debug!("Using GOOGLE_ACCOUNT_JSON_BASE64 for credentials");
            let decoded_creds = Self::decode_base64_str(&base64_creds)?;

            return Ok(CredentialsFile::new_from_str(&decoded_creds).await?);
        }

        debug!("Using GOOGLE_APPLICATION_CREDENTIALS for credentials",);
        return CredentialsFile::new()
            .await
            .map_err(GoogleError::GCloudAuthError);
    }

    fn decode_base64_str(service_base64_creds: &str) -> Result<String, GoogleError> {
        let decoded = BASE64_STANDARD.decode(service_base64_creds)?;

        Ok(String::from_utf8(decoded)?)
    }
}

#[derive(Debug)]
pub enum GeminiAuth {
    ApiKey(String),
    GoogleCredentials(GoogleCredentials),
    NotSet,
}

impl GeminiAuth {
    /// Try to create authentication from environment variables
    /// This will first look for a `GEMINI_API_KEY`.
    /// If not found, it will attempt to use Google Application Credentials
    /// to create a token source for authentication.
    ///
    #[instrument(skip_all)]
    pub async fn from_env() -> Result<Self, GoogleError> {
        // First try API key
        if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
            if !api_key.is_empty() {
                debug!("Using GEMINI_API_KEY for authentication");
                return Ok(Self::ApiKey(api_key));
            }
        }

        // Then try Google credentials
        match create_token_provider().await {
            Ok(credentials) => {
                debug!("Using Google Application Credentials for authentication");
                Ok(Self::GoogleCredentials(credentials))
            }
            Err(e) => {
                debug!("Failed to create Google token provider: {}", e);
                Ok(Self::NotSet)
            }
        }
    }

    /// Get the appropriate base URL for this auth method
    pub fn base_url(&self) -> String {
        match self {
            Self::ApiKey(_) => {
                "https://generativelanguage.googleapis.com/v1beta/models".to_string()
            }
            Self::GoogleCredentials(creds) => {
                format!(
                    "https://{}-aiplatform.googleapis.com/v1beta1/projects/{}/locations/{}/publishers/google/models",
                    creds.location,
                    creds.project_id,
                    creds.location
                )
            }
            Self::NotSet => "https://generativelanguage.googleapis.com/v1beta/models".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct GeminiApiConfig {
    base_url: String,
    service_type: GeminiServiceType,
}

impl GeminiApiConfig {
    /// Create a new API configuration based on auth and service type
    pub fn new(auth: &GeminiAuth, service_type: GeminiServiceType) -> Self {
        let base_url = match auth {
            GeminiAuth::ApiKey(_) => {
                "https://generativelanguage.googleapis.com/v1beta/models".to_string()
            }
            GeminiAuth::GoogleCredentials(creds) => {
                // use the vertex AI endpoint format
                format!(
                    "https://{}-aiplatform.googleapis.com/v1beta1/projects/{}/locations/{}/publishers/google/models",
                    creds.location, creds.project_id, creds.location
                )
            }
            GeminiAuth::NotSet => {
                "https://generativelanguage.googleapis.com/v1beta/models".to_string()
            }
        };

        Self {
            base_url,
            service_type,
        }
    }

    /// Helper for constructing the full URL for a given model and auth method
    pub fn build_url(&self, model: &str, auth: &GeminiAuth) -> String {
        let endpoint = self.get_endpoint(auth);
        match auth {
            GeminiAuth::GoogleCredentials(_)
                if matches!(self.service_type, GeminiServiceType::Embed) =>
            {
                // Vertex AI embedding uses models/{model}:predict
                format!("{}/models/{}:{}", self.base_url, model, endpoint.path())
            }
            _ => {
                // Standard format: {base_url}/{model}:{endpoint}
                format!("{}/{}:{}", self.base_url, model, endpoint.path())
            }
        }
    }

    /// Get the appropriate endpoint based on service type and auth method
    /// Currently generateContent is supported for both gemini and vertex AI
    /// embedContent is supported for API key auth
    /// predict is used for Vertex AI embeddings with Google credentials
    /// # Arguments
    /// * `auth`: The authentication method being used
    /// # Returns
    /// * `GeminiEndpoint`: The appropriate endpoint for the request
    fn get_endpoint(&self, auth: &GeminiAuth) -> GeminiEndpoint {
        match (&self.service_type, auth) {
            (GeminiServiceType::Generate, _) => GeminiEndpoint::GenerateContent,
            (GeminiServiceType::Embed, GeminiAuth::ApiKey(_)) => GeminiEndpoint::EmbedContent,
            (GeminiServiceType::Embed, GeminiAuth::GoogleCredentials(_)) => GeminiEndpoint::Predict,
            (GeminiServiceType::Embed, GeminiAuth::NotSet) => GeminiEndpoint::EmbedContent,
        }
    }
}
