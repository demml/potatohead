use crate::error::ProviderError;
use base64::prelude::*;
use gcloud_auth::credentials::CredentialsFile;
use gcloud_auth::{project::Config, token::DefaultTokenSourceProvider};
use std::env;
use tracing::{debug, instrument};

const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/generative-language",
];

#[derive(Debug)]
pub struct GoogleCredentials {
    pub project_id: String,
    pub location: String,
    pub token_provider: DefaultTokenSourceProvider,
}

impl GoogleCredentials {
    pub fn from_token_provider(
        token_provider: DefaultTokenSourceProvider,
    ) -> Result<Self, ProviderError> {
        // attempt to retrieve project_id from token_provider, if None attempt to load from env var
        let project_id = match &token_provider.project_id {
            Some(id) => id.clone(),
            // attempt to load GOOGLE_CLOUD_PROJECT env var if project_id is None
            None => match env::var("GOOGLE_CLOUD_PROJECT") {
                Ok(val) => val,
                Err(_) => Err(ProviderError::NoProjectIdFound)?,
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

    pub async fn get_access_token(&self) -> Result<String, ProviderError> {
        let token = self
            .token_provider
            .token_source()
            .token()
            .await
            .map_err(|e| ProviderError::TokenError(e.to_string()))?;

        Ok(token)
    }
}

#[allow(unused_imports)]
use token_source::TokenSourceProvider;

pub async fn create_token_provider() -> Result<GoogleCredentials, ProviderError> {
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
    pub async fn new() -> Result<Self, ProviderError> {
        let creds = Self::build().await?;
        let creds = CredentialBuilder { creds };

        Ok(creds)
    }

    #[instrument(skip_all)]
    async fn build() -> Result<CredentialsFile, ProviderError> {
        if let Ok(base64_creds) = env::var("GOOGLE_ACCOUNT_JSON_BASE64") {
            debug!("Using GOOGLE_ACCOUNT_JSON_BASE64 for credentials");
            let decoded_creds = Self::decode_base64_str(&base64_creds)?;

            return Ok(CredentialsFile::new_from_str(&decoded_creds).await?);
        }

        debug!("Using GOOGLE_APPLICATION_CREDENTIALS for credentials",);
        return CredentialsFile::new()
            .await
            .map_err(ProviderError::GCloudAuthError);
    }

    fn decode_base64_str(service_base64_creds: &str) -> Result<String, ProviderError> {
        let decoded = BASE64_STANDARD.decode(service_base64_creds)?;

        Ok(String::from_utf8(decoded)?)
    }
}

#[derive(Debug)]
pub enum GoogleAuth {
    ApiKey(String),
    GoogleCredentials(GoogleCredentials),
    NotSet,
}

impl GoogleAuth {
    /// Try to create authentication from environment variables
    /// This will first look for a `GEMINI_API_KEY`.
    /// If not found, it will attempt to use Google Application Credentials
    /// to create a token source for authentication.
    ///
    #[instrument(skip_all)]
    pub async fn from_env() -> Self {
        // First try API key (GEMINI_API_KEY or GOOGLE_API_KEY)
        if let Ok(api_key) = std::env::var("GEMINI_API_KEY").or(std::env::var("GOOGLE_API_KEY")) {
            if !api_key.is_empty() {
                debug!("Using GEMINI_API_KEY for authentication");
                return Self::ApiKey(api_key);
            }
        }

        // Then try Google credentials
        match create_token_provider().await {
            Ok(credentials) => {
                debug!("Using Google Application Credentials for authentication");
                Self::GoogleCredentials(credentials)
            }
            Err(e) => {
                debug!("Failed to create Google token provider: {}", e);
                Self::NotSet
            }
        }
    }
}

enum GoogleApiVersion {
    V1Beta,
    V1Beta1, // This is super annoying that google can't keep this consistent
    V1,
}

impl GoogleApiVersion {
    fn from_env() -> Option<Self> {
        match std::env::var("GOOGLE_API_VERSION") {
            Ok(ver) => match ver.to_lowercase().as_str() {
                "v1beta" => Some(GoogleApiVersion::V1Beta),
                "v1beta1" => Some(GoogleApiVersion::V1Beta1),
                "v1" => Some(GoogleApiVersion::V1),
                _ => None,
            },
            Err(_) => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            GoogleApiVersion::V1Beta => "v1beta",
            GoogleApiVersion::V1 => "v1",
            GoogleApiVersion::V1Beta1 => "v1beta1",
        }
    }
}

pub enum GoogleUrl {
    Gemini,
    Vertex,
}

impl GoogleUrl {
    /// Helper to get the root URL based on auth type
    pub fn base_url(&self, auth: &GoogleAuth) -> String {
        match self {
            GoogleUrl::Gemini => {
                let version = GoogleApiVersion::from_env().unwrap_or(GoogleApiVersion::V1Beta);
                format!(
                    "https://generativelanguage.googleapis.com/{}/models",
                    version.as_str()
                )
            }
            GoogleUrl::Vertex => match auth {
                GoogleAuth::GoogleCredentials(creds) => {
                    let version = GoogleApiVersion::from_env().unwrap_or(GoogleApiVersion::V1Beta1);
                    format!(
                        "https://{}-aiplatform.googleapis.com/{}/projects/{}/locations/{}/publishers/google/models",
                        creds.location,
                        version.as_str(),
                        creds.project_id,
                        creds.location
                    )
                }
                _ => "https://generativelanguage.googleapis.com/v1beta/models".to_string(),
            },
        }
    }
}
