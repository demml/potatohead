use crate::ApiHelper;
use crate::OpenAIHelper;
use potato_client::{AsyncLLMClient, LLMClient};
use potato_providers::openai::OpenAIClient;

#[derive(Debug)]
pub enum ApiClient {
    OpenAI(OpenAIClient),
}

impl ApiClient {
    pub fn get_helper(&self) -> impl ApiHelper {
        match self {
            ApiClient::OpenAI(_) => OpenAIHelper::new(),
        }
    }

    pub fn get_client(&self) -> &(impl LLMClient + AsyncLLMClient + '_) {
        match self {
            ApiClient::OpenAI(client) => client,
        }
    }
}
