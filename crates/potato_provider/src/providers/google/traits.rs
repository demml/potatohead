use crate::error::ProviderError;
use crate::providers::google::auth::GoogleAuth;
use crate::providers::types::ServiceType;
use reqwest::Response;
use tracing::{debug, error};

pub trait ApiConfigExt {
    /// Create a new API configuration based on auth and service type
    fn new(auth: GoogleAuth, service_type: ServiceType) -> Self;

    /// Helper for constructing the full URL for a given model
    fn build_url(&self, model: &str) -> String;

    #[allow(async_fn_in_trait)]
    async fn set_auth_header(
        &self,
        req: reqwest::RequestBuilder,
        auth: &GoogleAuth,
    ) -> Result<reqwest::RequestBuilder, ProviderError>;

    /// Get the appropriate endpoint based on service type
    fn get_endpoint(&self) -> &'static str;

    fn auth(&self) -> &GoogleAuth;
}

/// Generic trait for making requests to an either Gemini or Vertex AI API
pub trait RequestClient {
    #[allow(async_fn_in_trait)]
    async fn make_request(
        client: &reqwest::Client,
        config: &impl ApiConfigExt,
        model: &str,
        object: &serde_json::Value,
    ) -> Result<Response, ProviderError> {
        let url = config.build_url(model);
        debug!("Making request to API at URL: {}", url);
        let request = client.post(url).json(&object);
        let request = config.set_auth_header(request, config.auth()).await?;

        let response = request.send().await.map_err(ProviderError::RequestError)?;
        let status = response.status();
        if !status.is_success() {
            error!("API request failed with status: {}", status);

            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "No response body".to_string());

            return Err(ProviderError::CompletionError(body, status));
        }

        Ok(response)
    }
}
