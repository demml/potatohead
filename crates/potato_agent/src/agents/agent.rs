use crate::agents::provider::openai::OpenAIClient;
use crate::agents::provider::types::Provider;
use crate::{
    agents::client::GenAiClient,
    agents::error::AgentError,
    agents::task::Task,
    agents::types::{AgentResponse, PyAgentResponse},
};
use potato_prompt::{
    parse_response_format, prompt::parse_prompt, prompt::types::Message, ModelSettings, Prompt,
    Role,
};
use potato_util::create_uuid7;
use pyo3::{prelude::*, IntoPyObjectExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use tracing::{debug, instrument, warn};

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

    #[instrument(skip_all)]
    fn append_task_with_message_context(
        &self,
        task: &mut Task,
        context_messages: &HashMap<String, Vec<Message>>,
    ) {
        //
        debug!(task.id = %task.id, task.dependencies = ?task.dependencies, context_messages = ?context_messages, "Appending messages");
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
        task: &Arc<RwLock<Task>>,
        context_messages: HashMap<String, Vec<Message>>,
        parameter_context: Value,
    ) -> Result<AgentResponse, AgentError> {
        // Prepare prompt and context before await
        let (prompt, task_id) = {
            let mut task = task.write().unwrap();
            self.append_task_with_message_context(&mut task, &context_messages);
            self.bind_parameters(&mut task.prompt, &parameter_context)?;

            self.append_system_messages(&mut task.prompt);
            (task.prompt.clone(), task.id.clone())
        };

        // Now do the async work without holding the lock
        let chat_response = self.client.execute(&prompt).await?;

        Ok(AgentResponse::new(task_id, chat_response))
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

#[pyclass(name = "Agent")]
#[derive(Debug, Clone)]
pub struct PyAgent {
    pub agent: Arc<Agent>,
    pub runtime: Arc<tokio::runtime::Runtime>,
}

#[pymethods]
impl PyAgent {
    #[new]
    #[pyo3(signature = (provider, system_message = None))]
    /// Creates a new Agent instance.
    ///
    /// # Arguments:
    /// * `provider` - A Python object representing the provider, expected to be an a variant of Provider or a string
    /// that can be mapped to a provider variant
    ///
    pub fn new(
        provider: &Bound<'_, PyAny>,
        system_message: Option<&Bound<'_, PyAny>>,
    ) -> Result<Self, AgentError> {
        let provider = Provider::extract_provider(provider)?;

        let system_message = if let Some(system_message) = system_message {
            Some(
                parse_prompt(system_message)?
                    .into_iter()
                    .map(|mut msg| {
                        msg.role = Role::Developer.to_string();
                        msg
                    })
                    .collect::<Vec<Message>>(),
            )
        } else {
            None
        };

        let agent = Agent::new(provider, system_message)?;

        Ok(Self {
            agent: Arc::new(agent),
            runtime: Arc::new(
                tokio::runtime::Runtime::new()
                    .map_err(|e| AgentError::RuntimeError(e.to_string()))?,
            ),
        })
    }

    #[pyo3(signature = (task, output_type))]
    pub fn execute_task(
        &self,
        py: Python<'_>,
        task: &mut Task,
        output_type: Option<Bound<'_, PyAny>>,
    ) -> Result<PyAgentResponse, AgentError> {
        // Extract the prompt from the task
        debug!("Executing task");
        // if output_type is not None,  mutate task prompt
        if let Some(output_type) = &output_type {
            match parse_response_format(py, output_type) {
                Ok(response_format) => {
                    task.prompt.response_format = response_format;
                }
                Err(_) => {
                    return Err(AgentError::InvalidOutputType(output_type.to_string()));
                }
            }
        }

        let chat_response = self
            .runtime
            .block_on(async { self.agent.execute_task(task).await })?;

        debug!("Task executed successfully");
        let output = output_type.as_ref().map(|obj| obj.clone().unbind());
        let response = PyAgentResponse::new(chat_response, output);

        Ok(response)
    }

    #[pyo3(signature = (prompt, output_type=None))]
    pub fn execute_prompt(
        &self,
        py: Python<'_>,
        prompt: &mut Prompt,
        output_type: Option<Bound<'_, PyAny>>,
    ) -> Result<PyAgentResponse, AgentError> {
        // Extract the prompt from the task
        debug!("Executing task");
        // if output_type is not None,  mutate task prompt
        if let Some(output_type) = &output_type {
            match parse_response_format(py, output_type) {
                Ok(response_format) => {
                    prompt.response_format = response_format;
                }
                Err(_) => {
                    return Err(AgentError::InvalidOutputType(output_type.to_string()));
                }
            }
        }

        let chat_response = self
            .runtime
            .block_on(async { self.agent.execute_prompt(prompt).await })?;

        debug!("Task executed successfully");
        let output = output_type.as_ref().map(|obj| obj.clone().unbind());
        let response = PyAgentResponse::new(chat_response, output);

        Ok(response)
    }

    #[getter]
    pub fn system_message<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        Ok(self.agent.system_message.clone().into_bound_py_any(py)?)
    }

    #[getter]
    pub fn id(&self) -> &str {
        self.agent.id.as_str()
    }
}
