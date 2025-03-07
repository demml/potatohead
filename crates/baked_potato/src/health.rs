use anyhow::Context;
use anyhow::Result;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Json;
use axum::Router;
/// file containing schema for health module
use serde::{Deserialize, Serialize};
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;
use tracing::{error, instrument};

#[derive(Serialize, Deserialize)]
pub struct Alive {
    pub status: String,
}

impl Default for Alive {
    fn default() -> Self {
        Self {
            status: "Alive".to_string(),
        }
    }
}

// Implement IntoResponse for Alive
impl IntoResponse for Alive {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

pub async fn health_check() -> Alive {
    Alive::default()
}

#[instrument(skip_all)]
pub async fn get_health_router() -> Result<Router> {
    let result = catch_unwind(AssertUnwindSafe(|| {
        Router::new().route("/healthcheck", get(health_check))
    }));

    match result {
        Ok(router) => Ok(router),
        Err(e) => {
            // panic
            error!("Failed to create health router: {:?}", e);
            Err(anyhow::anyhow!("Failed to create health router"))
                .context("Panic occurred while creating the router")
        }
    }
}
