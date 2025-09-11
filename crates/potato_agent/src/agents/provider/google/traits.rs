use reqwest::Response;

#[derive(Debug)]
pub enum ServiceType {
    Generate,
    Embed,
}

impl ServiceType {
    /// Get the service type string
    pub fn gemini_endpoint(&self) -> &'static str {
        match self {
            Self::Generate => "generateContent",
            Self::Embed => "embedContent",
        }
    }
    pub fn vertex_endpoint(&self) -> &'static str {
        match self {
            Self::Generate => "generateContent",
            Self::Embed => "predict",
        }
    }
}

pub trait ApiConfigTrait {
    /// Create a new API configuration based on auth and service type
    fn new(auth: &GoogleAuth, service_type: ServiceType) -> Self;

    /// Helper for constructing the full URL for a given model
    fn build_url(&self, model: &str) -> String;

    fn set_auth_header(&self, req: &mut reqwest::RequestBuilder, auth: &GoogleAuth);

    /// Get the appropriate endpoint based on service type
    fn get_endpoint(&self) -> &'static str;
}

/// Generic trait for making requests to an either Gemini or Vertex AI API
pub trait RequestClient {
    async fn make_request(
        client: &reqwest::Client,
        config: &impl ApiConfigTrait,
        auth: &GoogleAuth,
        model: &str,
        object: &serde_json::Value,
    ) -> Result<Response, GoogleError> {
        let url = config.build_url(model);
        debug!("Making request to API at URL: {}", url);
        let mut request = client.post(url).json(&object);
        config.set_auth_header(&mut request, auth);

        let response = request.send().await.map_err(AgentError::RequestError)?;
        let status = response.status();
        if !status.is_success() {
            // print the response body for debugging
            error!("API request failed with status: {}", status);

            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "No response body".to_string());

            return Err(AgentError::CompletionError(body, status));
        }

        Ok(response)
    }
}
