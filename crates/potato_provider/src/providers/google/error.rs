use gcloud_auth::error::Error as GCloudAuthError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GoogleError {
    #[error(transparent)]
    DecodeError(#[from] base64::DecodeError),

    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    GCloudAuthError(#[from] GCloudAuthError),

    #[error("No Google credentials found in environment variables")]
    NoCredentialsFound,

    #[error("No project ID found in credentials or environment variables")]
    NoProjectIdFound,

    #[error("Missing authentication information. Failed to find GEMINI_API_KEY or Google credentials in environment variables.")]
    MissingAuthenticationError,

    #[error("Failed to retrieve access token: {0}")]
    TokenError(String),
}
