use crate::health::get_health_router;
use crate::openai::get_openai_router;
use anyhow::Ok;
use anyhow::Result;
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    Method,
};
use axum::Router;
use tower_http::cors::CorsLayer;

pub async fn create_router() -> Result<Router> {
    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::PUT,
            Method::DELETE,
            Method::POST,
            Method::PATCH,
        ])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let health_routes = get_health_router().await?;
    let openai_routes = get_openai_router().await?;

    // merge all the routes except the auth routes
    // All routes except the auth routes will be protected by the auth middleware
    let merged_routes = Router::new().merge(health_routes);

    Ok(Router::new()
        .merge(merged_routes)
        .merge(openai_routes)
        .layer(cors))
}

pub async fn create_app() -> Result<Router> {
    // create the router
    let app = create_router().await?;

    Ok(app)
}
