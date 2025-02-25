use crate::client::LLMClient;
use crate::helpers::openai::OpenAIHelper;
use crate::helpers::traits::ApiHelper;
use crate::providers::openai::OpenAIClient;

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

    pub fn get_client(&self) -> &(impl LLMClient + '_) {
        match self {
            ApiClient::OpenAI(client) => client,
        }
    }
}
