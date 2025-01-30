use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug, Deserialize)]
pub enum HttpError {
    #[error("Error: {0}")]
    Error(String),
}
