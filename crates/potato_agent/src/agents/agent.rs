use crate::agents::provider::gemini::GeminiClient;
use crate::agents::provider::openai::OpenAIClient;
use crate::{
    agents::client::GenAiClient,
    agents::error::AgentError,
    agents::task::Task,
    agents::types::{AgentResponse, PyAgentResponse},
};
use potato_prompt::prompt::settings::ModelSettings;
use potato_prompt::{
    parse_response_to_json, prompt::parse_prompt, prompt::types::Message, Prompt, Role,
};
use potato_type::Provider;
use potato_util::create_uuid7;
use pyo3::{prelude::*, IntoPyObjectExt};
use serde::{
    de::{self, MapAccess, Visitor},
    ser::SerializeStruct,
    Deserializer, Serializer,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use tracing::{debug, error, instrument, warn};

#[derive(Debug, PartialEq)]
pub struct Agent {
    pub id: String,

    client: GenAiClient,

    pub system_instruction: Vec<Message>,
}

/// Rust method implementation of the Agent
impl Agent {
    pub fn new(
        provider: Provider,
        system_instruction: Option<Vec<Message>>,
    ) -> Result<Self, AgentError> {
        let client = match provider {
            Provider::OpenAI => GenAiClient::OpenAI(OpenAIClient::new(None, None, None)?),
            Provider::Gemini => GenAiClient::Gemini(GeminiClient::new(None, None, None)?),
            _ => {
                let msg = "No provider specified in ModelSettings";
                error!("{}", msg);
                return Err(AgentError::UndefinedError(msg.to_string()));
            } // Add other providers here as needed
        };

        let system_instruction = system_instruction.unwrap_or_default();

        Ok(Self {
            client,
            id: create_uuid7(),
            system_instruction,
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
                        task.prompt.message.insert(0, message.clone());
                    }
                }
            }
        }
    }

    /// This function will bind dependency-specific context and global context if provided to the user prompt.
    ///
    /// # Arguments:
    /// * `prompt` - The prompt to bind parameters to.
    /// * `parameter_context` - A serde_json::Value containing the parameters to bind.
    /// * `global_context` - An optional serde_json::Value containing global parameters to bind.
    ///
    /// # Returns:
    /// * `Result<(), AgentError>` - Returns Ok(()) if successful, or an `AgentError` if there was an issue binding the parameters.
    #[instrument(skip_all)]
    fn bind_context(
        &self,
        prompt: &mut Prompt,
        parameter_context: &Value,
        global_context: &Option<Value>,
    ) -> Result<(), AgentError> {
        // print user messages
        if !prompt.parameters.is_empty() {
            for param in &prompt.parameters {
                // Bind parameter context to the user message
                if let Some(value) = parameter_context.get(param) {
                    for message in &mut prompt.message {
                        if message.role == "user" {
                            debug!("Binding parameter: {} with value: {}", param, value);
                            message.bind_mut(param, &value.to_string())?;
                        }
                    }
                }

                // If global context is provided, bind it to the user message
                if let Some(global_value) = global_context {
                    if let Some(value) = global_value.get(param) {
                        for message in &mut prompt.message {
                            if message.role == "user" {
                                debug!("Binding global parameter: {} with value: {}", param, value);
                                message.bind_mut(param, &value.to_string())?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn append_system_instructions(&self, prompt: &mut Prompt) {
        if !self.system_instruction.is_empty() {
            let mut combined_messages = self.system_instruction.clone();
            combined_messages.extend(prompt.system_instruction.clone());
            prompt.system_instruction = combined_messages;
        }
    }
    pub async fn execute_task(&self, task: &Task) -> Result<AgentResponse, AgentError> {
        // Extract the prompt from the task
        debug!("Executing task: {}, count: {}", task.id, task.retry_count);
        let mut prompt = task.prompt.clone();
        self.append_system_instructions(&mut prompt);

        // Use the client to execute the task
        let chat_response = self.client.execute(&prompt).await?;

        Ok(AgentResponse::new(task.id.clone(), chat_response))
    }

    #[instrument(skip_all)]
    pub async fn execute_prompt(&self, prompt: &Prompt) -> Result<AgentResponse, AgentError> {
        // Extract the prompt from the task
        debug!("Executing prompt");
        let mut prompt = prompt.clone();
        self.append_system_instructions(&mut prompt);

        // Use the client to execute the task
        let chat_response = self.client.execute(&prompt).await?;

        Ok(AgentResponse::new(chat_response.id(), chat_response))
    }

    pub async fn execute_task_with_context(
        &self,
        task: &Arc<RwLock<Task>>,
        context_messages: HashMap<String, Vec<Message>>,
        parameter_context: Value,
        global_context: Option<Value>,
    ) -> Result<AgentResponse, AgentError> {
        // Prepare prompt and context before await
        let (prompt, task_id) = {
            let mut task = task.write().unwrap();
            self.append_task_with_message_context(&mut task, &context_messages);
            self.bind_context(&mut task.prompt, &parameter_context, &global_context)?;

            self.append_system_instructions(&mut task.prompt);
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
        let provider = model_settings.provider();
        let client = match provider {
            Provider::OpenAI => GenAiClient::OpenAI(OpenAIClient::new(None, None, None)?),
            Provider::Gemini => GenAiClient::Gemini(GeminiClient::new(None, None, None)?),
            Provider::Undefined => {
                let msg = "No provider specified in ModelSettings";
                error!("{}", msg);
                return Err(AgentError::UndefinedError(msg.to_string()));
            }
        };

        Ok(Self {
            client,
            id: create_uuid7(),
            system_instruction: Vec::new(),
        })
    }
}

impl Serialize for Agent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Agent", 3)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("provider", &self.client.provider())?;
        state.serialize_field("system_instruction", &self.system_instruction)?;
        state.end()
    }
}

/// Allows for deserialization of the Agent, re-initializing the client.
impl<'de> Deserialize<'de> for Agent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Id,
            Provider,
            SystemInstruction,
        }

        struct AgentVisitor;

        impl<'de> Visitor<'de> for AgentVisitor {
            type Value = Agent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Agent")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Agent, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id = None;
                let mut provider = None;
                let mut system_instruction = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            id = Some(map.next_value()?);
                        }
                        Field::Provider => {
                            provider = Some(map.next_value()?);
                        }
                        Field::SystemInstruction => {
                            system_instruction = Some(map.next_value()?);
                        }
                    }
                }

                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let provider = provider.ok_or_else(|| de::Error::missing_field("provider"))?;
                let system_instruction = system_instruction
                    .ok_or_else(|| de::Error::missing_field("system_instruction"))?;

                // Re-initialize the client based on the provider
                let client = match provider {
                    Provider::OpenAI => {
                        GenAiClient::OpenAI(OpenAIClient::new(None, None, None).map_err(|e| {
                            de::Error::custom(format!("Failed to initialize OpenAIClient: {e}"))
                        })?)
                    }
                    Provider::Gemini => {
                        GenAiClient::Gemini(GeminiClient::new(None, None, None).map_err(|e| {
                            de::Error::custom(format!("Failed to initialize GeminiClient: {e}"))
                        })?)
                    }

                    Provider::Undefined => {
                        let msg = "No provider specified in ModelSettings";
                        error!("{}", msg);
                        return Err(de::Error::custom(msg));
                    }
                };

                Ok(Agent {
                    id,
                    client,
                    system_instruction,
                })
            }
        }

        const FIELDS: &[&str] = &["id", "provider", "system_instruction"];
        deserializer.deserialize_struct("Agent", FIELDS, AgentVisitor)
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
    #[pyo3(signature = (provider, system_instruction = None))]
    /// Creates a new Agent instance.
    ///
    /// # Arguments:
    /// * `provider` - A Python object representing the provider, expected to be an a variant of Provider or a string
    /// that can be mapped to a provider variant
    ///
    pub fn new(
        provider: &Bound<'_, PyAny>,
        system_instruction: Option<&Bound<'_, PyAny>>,
    ) -> Result<Self, AgentError> {
        let provider = Provider::extract_provider(provider)?;

        let system_instruction = if let Some(system_instruction) = system_instruction {
            Some(
                parse_prompt(system_instruction)?
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

        let agent = Agent::new(provider, system_instruction)?;

        Ok(Self {
            agent: Arc::new(agent),
            runtime: Arc::new(
                tokio::runtime::Runtime::new()
                    .map_err(|e| AgentError::RuntimeError(e.to_string()))?,
            ),
        })
    }

    #[pyo3(signature = (task, output_type=None, model=None))]
    pub fn execute_task(
        &self,
        py: Python<'_>,
        task: &mut Task,
        output_type: Option<Bound<'_, PyAny>>,
        model: Option<&str>,
    ) -> Result<PyAgentResponse, AgentError> {
        // Extract the prompt from the task
        debug!("Executing task");

        // if output_type is not None,  mutate task prompt
        if let Some(output_type) = &output_type {
            match parse_response_to_json(py, output_type) {
                Ok((response_type, response_format)) => {
                    task.prompt.response_type = response_type;
                    task.prompt.response_json_schema = response_format;
                }
                Err(_) => {
                    return Err(AgentError::InvalidOutputType(output_type.to_string()));
                }
            }
        }

        // if model is not None, set task prompt model (this will override the task prompt model)
        if let Some(model) = model {
            task.prompt.model = model.to_string();
        }

        // agent provider and task.prompt provider must match
        if task.prompt.provider != *self.agent.provider() {
            return Err(AgentError::ProviderMismatch(
                task.prompt.provider.to_string(),
                self.agent.provider().as_str().to_string(),
            ));
        }

        debug!(
            "Task prompt model identifier: {}",
            task.prompt.model_identifier()
        );

        let chat_response = self
            .runtime
            .block_on(async { self.agent.execute_task(task).await })?;

        debug!("Task executed successfully");
        let output = output_type.as_ref().map(|obj| obj.clone().unbind());
        let response = PyAgentResponse::new(chat_response, output);

        Ok(response)
    }

    #[pyo3(signature = (prompt, output_type=None, model=None))]
    pub fn execute_prompt(
        &self,
        py: Python<'_>,
        prompt: &mut Prompt,
        output_type: Option<Bound<'_, PyAny>>,
        model: Option<&str>,
    ) -> Result<PyAgentResponse, AgentError> {
        // Extract the prompt from the task
        debug!("Executing task");
        // if output_type is not None,  mutate task prompt
        if let Some(output_type) = &output_type {
            match parse_response_to_json(py, output_type) {
                Ok((response_type, response_format)) => {
                    prompt.response_type = response_type;
                    prompt.response_json_schema = response_format;
                }
                Err(_) => {
                    return Err(AgentError::InvalidOutputType(output_type.to_string()));
                }
            }
        }

        // if model is not None, set task prompt model (this will override the task prompt model)
        if let Some(model) = model {
            prompt.model = model.to_string();
        }

        // agent provider and task.prompt provider must match
        if prompt.provider != *self.agent.provider() {
            return Err(AgentError::ProviderMismatch(
                prompt.provider.to_string(),
                self.agent.provider().as_str().to_string(),
            ));
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
    pub fn system_instruction<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyAny>, AgentError> {
        Ok(self
            .agent
            .system_instruction
            .clone()
            .into_bound_py_any(py)?)
    }

    #[getter]
    pub fn id(&self) -> &str {
        self.agent.id.as_str()
    }
}
