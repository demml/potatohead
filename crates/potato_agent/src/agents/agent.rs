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
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, instrument};

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

    fn append_task_with_message_context(
        &self,
        task: &mut Task,
        context_messages: &HashMap<String, Vec<Message>>,
    ) {
        if !task.dependencies.is_empty() {
            for dep in &task.dependencies {
                if let Some(messages) = context_messages.get(dep) {
                    for message in messages {
                        // prepend the messages from dependencies
                        task.prompt.user_message.insert(0, message.clone());
                    }
                }
            }
        }
    }

    #[instrument(skip_all)]
    fn bind_parameters(
        &self,
        prompt: &mut Prompt,
        context_messages: &Value,
    ) -> Result<(), AgentError> {
        // print user messages
        if !prompt.parameters.is_empty() {
            for param in &prompt.parameters {
                if let Some(value) = context_messages.get(param) {
                    for message in &mut prompt.user_message {
                        if message.role == "user" {
                            debug!("Binding parameter: {} with value: {}", param, value);
                            message.bind_mut(param, &value.to_string())?;
                        }
                    }
                }
            }
        }
        Ok(())
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
        task: &mut Task,
        task_id: &str,
        context_messages: HashMap<String, Vec<Message>>,
        parameter_context: Value,
    ) -> Result<AgentResponse, AgentError> {
        // Record event started

        // Extract the prompt from the task
        debug!("Executing task: {}, count: {}", task_id, task.retry_count);
        self.append_task_with_message_context(task, &context_messages);

        // Bind parameters if any
        self.bind_parameters(&mut task.prompt, &parameter_context)?;
        self.append_system_messages(&mut task.prompt);

        // Use the client to execute the task
        let chat_response = self.client.execute(&task.prompt).await?;

        Ok(AgentResponse::new(task_id.to_string(), chat_response))
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
