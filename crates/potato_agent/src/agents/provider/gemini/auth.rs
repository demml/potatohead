use crate::agents::provider::gemini::error::GoogleError;
use base64::prelude::*;
use gcloud_auth::credentials::CredentialsFile;
use gcloud_auth::{project::Config, token::DefaultTokenSourceProvider};
use std::env;

const AUDIENCE: &str = "https://www.aiplatform.googleapis.com/";
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
}

#[allow(unused_imports)]
use token_source::TokenSourceProvider;

pub async fn create_token_provider() -> Result<GoogleCredentials, GoogleError> {
    let creds = CredentialBuilder::new().await?.creds;

    let config = Config::default()
        .with_scopes(&SCOPES)
        .with_audience(AUDIENCE);

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

    async fn build() -> Result<CredentialsFile, GoogleError> {
        if let Ok(base64_creds) = env::var("GOOGLE_ACCOUNT_JSON_BASE64") {
            return Ok(
                CredentialsFile::new_from_str(&Self::decode_base64_str(&base64_creds)?).await?,
            );
        }

        if env::var("GOOGLE_APPLICATION_CREDENTIALS_JSON")
            .or_else(|_| env::var("GOOGLE_APPLICATION_CREDENTIALS"))
            .is_ok()
        {
            return Ok(CredentialsFile::new()
                .await
                .map_err(GoogleError::GCloudAuthError)?);
        }

        Err(GoogleError::NoCredentialsFound)
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
}

impl GeminiAuth {
    /// Try to create authentication from environment variables
    /// This will first look for a `GEMINI_API_KEY`.
    /// If not found, it will attempt to use Google Application Credentials
    /// to create a token source for authentication.
    pub async fn from_env() -> Result<Self, GoogleError> {
        // First try API key
        if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
            if !api_key.is_empty() {
                return Ok(Self::ApiKey(api_key));
            }
        }

        // Then try Google credentials
        match create_token_provider().await {
            Ok(credentials) => Ok(Self::GoogleCredentials(credentials)),
            Err(_) => Err(GoogleError::MissingAuthenticationError),
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
                    "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models",
                    creds.location,
                    creds.project_id,
                    creds.location
                )
            }
        }
    }
}
