use crate::agents::provider::openai::OpenAIClient;
use crate::agents::provider::types::Provider;
use potato_prompt::{prompt::types::Message, ModelSettings};

use crate::{
    agents::client::GenAiClient, agents::error::AgentError, agents::task::Task,
    agents::types::AgentResponse,
};
use potato_prompt::Prompt;
use potato_util::create_uuid7;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,

    client: GenAiClient,

    pub system_message: Vec<Message>,
}

/// Rust method implementation of the Agent
impl Agent {
    pub fn new(
        provider: Provider,
        system_message: Option<Vec<Message>>,
    ) -> Result<Self, AgentError> {
        let client = match provider {
            Provider::OpenAI => GenAiClient::OpenAI(OpenAIClient::new(None, None, None)?),
            // Add other providers here as needed
        };

        let system_message = system_message.unwrap_or_default();

        Ok(Self {
            client,
            id: create_uuid7(),
            system_message,
        })
    }

    fn get_task_with_context(
        &self,
        task: &Task,
        context_messages: &HashMap<String, Vec<Message>>,
    ) -> Task {
        let mut cloned_task = task.clone();

        if !cloned_task.dependencies.is_empty() {
            for dep in &cloned_task.dependencies {
                if let Some(messages) = context_messages.get(dep) {
                    for message in messages {
                        // prepend the messages from dependencies
                        cloned_task.prompt.user_message.insert(0, message.clone());
                    }
                }
            }
        }

        cloned_task
    }

    fn append_system_messages(&self, prompt: &mut Prompt) {
        if !self.system_message.is_empty() {
            let mut combined_messages = self.system_message.clone();
            combined_messages.extend(prompt.system_message.clone());
            prompt.system_message = combined_messages;
        }
    }
    pub async fn execute_task(&self, task: &Task) -> Result<AgentResponse, AgentError> {
        // Extract the prompt from the task
        debug!("Executing task: {}, count: {}", task.id, task.retry_count);
        let mut prompt = task.prompt.clone();
        self.append_system_messages(&mut prompt);

        // Use the client to execute the task
        let chat_response = self.client.execute(&prompt).await?;

        Ok(AgentResponse::new(task.id.clone(), chat_response))
    }

    pub async fn execute_prompt(&self, prompt: &Prompt) -> Result<AgentResponse, AgentError> {
        // Extract the prompt from the task
        debug!("Executing prompt");
        let mut prompt = prompt.clone();
        self.append_system_messages(&mut prompt);

        // Use the client to execute the task
        let chat_response = self.client.execute(&prompt).await?;

        Ok(AgentResponse::new(chat_response.id(), chat_response))
    }

    pub async fn execute_task_with_context(
        &self,
        task: &Task,
        context_messages: HashMap<String, Vec<Message>>,
    ) -> Result<AgentResponse, AgentError> {
        // Extract the prompt from the task
        debug!("Executing task: {}, count: {}", task.id, task.retry_count);
        let mut prompt = self.get_task_with_context(task, &context_messages).prompt;
        self.append_system_messages(&mut prompt);

        // Use the client to execute the task
        let chat_response = self.client.execute(&prompt).await?;

        Ok(AgentResponse::new(task.id.clone(), chat_response))
    }

    pub fn provider(&self) -> &Provider {
        self.client.provider()
    }

    pub fn from_model_settings(model_settings: &ModelSettings) -> Result<Self, AgentError> {
        let provider = Provider::from_string(&model_settings.provider)?;
        let client = match provider {
            Provider::OpenAI => GenAiClient::OpenAI(OpenAIClient::new(None, None, None)?),
        };

        Ok(Self {
            client,
            id: create_uuid7(),
            system_message: Vec::new(),
        })
    }
}
