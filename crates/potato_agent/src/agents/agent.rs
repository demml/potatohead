use crate::agents::{
    error::AgentError,
    task::Task,
    types::{AgentResponse, PyAgentResponse},
};
use potato_provider::providers::anthropic::client::AnthropicClient;
use potato_provider::providers::types::ServiceType;
use potato_provider::GeminiClient;
use potato_provider::{providers::google::VertexClient, GenAiClient, OpenAIClient};
use potato_state::block_on;
use potato_type::prompt::Prompt;
use potato_type::prompt::{MessageNum, Role};
use potato_type::Provider;
use potato_type::{
    prompt::extract_system_instructions,
    tools::{Tool, ToolRegistry},
};
use potato_util::create_uuid7;
use pyo3::prelude::*;
use pyo3::types::PyList;
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
use tracing::{debug, instrument, warn};

#[derive(Debug, Clone)]
pub struct Agent {
    pub id: String,
    client: Arc<GenAiClient>,
    pub provider: Provider,
    pub system_instruction: Vec<MessageNum>,
    pub tools: Arc<RwLock<ToolRegistry>>, // Add tool registry
    pub max_iterations: u32,
}

/// Rust method implementation of the Agent
impl Agent {
    /// Helper method to rebuild the client, useful for deserialization
    #[instrument(skip_all)]
    pub async fn rebuild_client(&self) -> Result<Self, AgentError> {
        let client = match self.provider {
            Provider::OpenAI => GenAiClient::OpenAI(OpenAIClient::new(ServiceType::Generate)?),
            Provider::Gemini => {
                GenAiClient::Gemini(GeminiClient::new(ServiceType::Generate).await?)
            }
            Provider::Vertex => {
                GenAiClient::Vertex(VertexClient::new(ServiceType::Generate).await?)
            }
            _ => {
                return Err(AgentError::MissingProviderError);
            } // Add other providers here as needed
        };

        Ok(Self {
            id: self.id.clone(),
            client: Arc::new(client),
            system_instruction: self.system_instruction.clone(),
            provider: self.provider.clone(),
            tools: self.tools.clone(),
            max_iterations: self.max_iterations,
        })
    }
    pub async fn new(
        provider: Provider,
        system_instruction: Option<Vec<MessageNum>>,
    ) -> Result<Self, AgentError> {
        let client = match provider {
            Provider::OpenAI => GenAiClient::OpenAI(OpenAIClient::new(ServiceType::Generate)?),
            Provider::Gemini => {
                GenAiClient::Gemini(GeminiClient::new(ServiceType::Generate).await?)
            }
            Provider::Vertex => {
                GenAiClient::Vertex(VertexClient::new(ServiceType::Generate).await?)
            }
            Provider::Anthropic => {
                GenAiClient::Anthropic(AnthropicClient::new(ServiceType::Generate)?)
            }
            _ => {
                return Err(AgentError::MissingProviderError);
            } // Add other providers here as needed
        };

        Ok(Self {
            client: Arc::new(client),
            id: create_uuid7(),
            system_instruction: system_instruction.unwrap_or_default(),
            provider,
            tools: Arc::new(RwLock::new(ToolRegistry::new())),
            max_iterations: 10,
        })
    }

    pub fn register_tool(&self, tool: Box<dyn Tool + Send + Sync>) {
        self.tools.write().unwrap().register_tool(tool);
    }

    //TODO: add back later
    /// Execute task with agentic reasoning loop
    //pub async fn execute_agentic_task(&self, task: &Task) -> Result<AgentResponse, AgentError> {
    //    let mut prompt = task.prompt.clone();
    //    self.prepend_system_instructions(&mut prompt);

    //    // Add tool definitions to prompt if tools are registered
    //    let tool_definitions = self.tools.read().unwrap().get_definitions();
    //    if !tool_definitions.is_empty() {
    //        // Convert tools to provider-specific format and add to prompt
    //        prompt.add_tools(tool_definitions)?;
    //    }

    //    let mut iteration = 0;
    //    let mut conversation_history = Vec::new();

    //    loop {
    //        if iteration >= self.max_iterations {
    //            return Err(AgentError::Error("Max iterations reached".to_string()));
    //        }

    //        // Generate response
    //        let response = self.client.generate_content(&prompt).await?;

    //        // Check if response contains tool calls
    //        if let Some(tool_calls) = response.extract_tool_calls() {
    //            debug!("Agent requesting {} tool calls", tool_calls.len());

    //            // Execute all requested tools
    //            let mut tool_results = Vec::new();
    //            for tool_call in tool_calls {
    //                let result = self.tools.read().unwrap().execute(&tool_call)?;
    //                tool_results.push((tool_call.tool_name.clone(), result));
    //            }

    //            // Add tool results back to conversation
    //            conversation_history.push(response.clone());
    //            prompt.add_tool_results(tool_results)?;

    //            iteration += 1;
    //            continue;
    //        }

    //        // No tool calls - agent has final answer
    //        return Ok(AgentResponse::new(task.id.clone(), response));
    //    }
    //}

    #[instrument(skip_all)]
    fn append_task_with_message_dependency_context(
        &self,
        task: &mut Task,
        context_messages: &HashMap<String, Vec<MessageNum>>,
    ) {
        //
        debug!(task.id = %task.id, task.dependencies = ?task.dependencies, context_messages = ?context_messages, "Appending messages");

        if task.dependencies.is_empty() {
            return;
        }

        let messages = task.prompt.request.messages_mut();
        let first_user_idx = messages.iter().position(|msg| !msg.is_system_message());

        match first_user_idx {
            Some(insert_idx) => {
                // Collect all dependency messages to insert
                let mut dependency_messages = Vec::new();

                for dep_id in &task.dependencies {
                    if let Some(messages) = context_messages.get(dep_id) {
                        debug!(
                            "Adding {} messages from dependency {}",
                            messages.len(),
                            dep_id
                        );
                        dependency_messages.extend(messages.iter().cloned());
                    }
                }

                // Always insert at same index to keep pushing user message forward
                for message in dependency_messages.into_iter() {
                    task.prompt
                        .request
                        .insert_message(message, Some(insert_idx))
                }

                debug!(
                    "Inserted {} dependency messages before user message at index {}",
                    task.dependencies.len(),
                    insert_idx
                );
            }
            None => {
                warn!(
                    "No user message found in task {}, appending dependency context to end",
                    task.id
                );

                for dep_id in &task.dependencies {
                    if let Some(messages) = context_messages.get(dep_id) {
                        for message in messages {
                            task.prompt.request.push_message(message.clone());
                        }
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
                    for message in prompt.request.messages_mut() {
                        if message.role() == Role::User.as_str() {
                            debug!("Binding parameter: {} with value: {}", param, value);
                            message.bind_mut(param, &value.to_string())?;
                        }
                    }
                }

                // If global context is provided, bind it to the user message
                if let Some(global_value) = global_context {
                    if let Some(value) = global_value.get(param) {
                        for message in prompt.request.messages_mut() {
                            if message.role() == Role::User.as_str() {
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

    /// If system instructions are set on the agent, prepend them to the prompt.
    /// Agent system instructions take precedence over task system instructions.
    /// If a user wishes to be more dynamic based on the task, they should set system instructions on the task/prompt
    fn prepend_system_instructions(&self, prompt: &mut Prompt) {
        if !self.system_instruction.is_empty() {
            prompt
                .request
                .prepend_system_instructions(self.system_instruction.clone())
                .unwrap();
        }
    }
    pub async fn execute_task(&self, task: &Task) -> Result<AgentResponse, AgentError> {
        // Extract the prompt from the task
        debug!("Executing task: {}, count: {}", task.id, task.retry_count);
        let mut prompt = task.prompt.clone();
        self.prepend_system_instructions(&mut prompt);

        // Use the client to execute the task
        let chat_response = self.client.generate_content(&prompt).await?;

        Ok(AgentResponse::new(task.id.clone(), chat_response))
    }

    #[instrument(skip_all)]
    pub async fn execute_prompt(&self, prompt: &Prompt) -> Result<AgentResponse, AgentError> {
        // Extract the prompt from the task
        debug!("Executing prompt");
        let mut prompt = prompt.clone();
        self.prepend_system_instructions(&mut prompt);

        // Use the client to execute the task
        let chat_response = self.client.generate_content(&prompt).await?;

        Ok(AgentResponse::new(chat_response.id(), chat_response))
    }

    pub async fn execute_task_with_context(
        &self,
        task: &Arc<RwLock<Task>>,
        context_messages: HashMap<String, Vec<MessageNum>>,
        parameter_context: Value,
        global_context: Option<Value>,
    ) -> Result<AgentResponse, AgentError> {
        // Prepare prompt and context before await
        let (prompt, task_id) = {
            let mut task = task.write().unwrap();
            // 1. Add dependency context (should come after system instructions, before user message)
            self.append_task_with_message_dependency_context(&mut task, &context_messages);
            // 2. Bind parameters
            self.bind_context(&mut task.prompt, &parameter_context, &global_context)?;
            // 3. Prepend agent system instructions (add to front)
            self.prepend_system_instructions(&mut task.prompt);
            (task.prompt.clone(), task.id.clone())
        };

        // Now do the async work without holding the lock
        let chat_response = self.client.generate_content(&prompt).await?;
        Ok(AgentResponse::new(task_id, chat_response))
    }

    pub fn client_provider(&self) -> &Provider {
        self.client.provider()
    }
}

impl PartialEq for Agent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.provider == other.provider
            && self.system_instruction == other.system_instruction
            && self.max_iterations == other.max_iterations
            && self.client == other.client
    }
}

impl Serialize for Agent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Agent", 3)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("provider", &self.provider)?;
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

                // Deserialize is a sync op, so we can't await here (gemini requires async to init)
                // After deserialization, we re-initialize the client based on the provider
                let client = GenAiClient::Undefined;
                Ok(Agent {
                    id,
                    client: Arc::new(client),
                    system_instruction,
                    provider,
                    tools: Arc::new(RwLock::new(ToolRegistry::new())),
                    max_iterations: 10,
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
        let system_instructions = extract_system_instructions(system_instruction, &provider)?;
        let agent = block_on(async { Agent::new(provider, system_instructions).await })?;

        Ok(Self {
            agent: Arc::new(agent),
        })
    }

    #[pyo3(signature = (task, output_type=None))]
    pub fn execute_task(
        &self,
        task: &mut Task,
        output_type: Option<Bound<'_, PyAny>>,
    ) -> Result<PyAgentResponse, AgentError> {
        // Extract the prompt from the task
        debug!("Executing task");

        // agent provider and task.prompt provider must match
        if task.prompt.provider != *self.agent.client_provider() {
            return Err(AgentError::ProviderMismatch(
                task.prompt.provider.to_string(),
                self.agent.client_provider().as_str().to_string(),
            ));
        }

        debug!(
            "Task prompt model identifier: {}",
            task.prompt.model_identifier()
        );

        let chat_response = block_on(async { self.agent.execute_task(task).await })?;

        debug!("Task executed successfully");
        let output = output_type.as_ref().map(|obj| obj.clone().unbind());
        let response = PyAgentResponse::new(chat_response, output);

        Ok(response)
    }

    /// Executes a prompt directly without a task.
    /// # Arguments:
    /// * `prompt` - The prompt to execute.
    /// * `output_type` - An optional Python type to bind the response to. If
    /// provide, it is expected that the output type_object matches the response schema defined in the prompt.
    /// # Returns:
    /// * `PyAgentResponse` - The response from the agent.
    #[pyo3(signature = (prompt, output_type=None))]
    pub fn execute_prompt(
        &self,
        prompt: &mut Prompt,
        output_type: Option<Bound<'_, PyAny>>,
    ) -> Result<PyAgentResponse, AgentError> {
        // Extract the prompt from the task
        debug!("Executing task");

        // agent provider and task.prompt provider must match
        if prompt.provider != *self.agent.client_provider() {
            return Err(AgentError::ProviderMismatch(
                prompt.provider.to_string(),
                self.agent.client_provider().as_str().to_string(),
            ));
        }

        let chat_response = block_on(async { self.agent.execute_prompt(prompt).await })?;

        debug!("Task executed successfully");
        let output = output_type.as_ref().map(|obj| obj.clone().unbind());
        let response = PyAgentResponse::new(chat_response, output);

        Ok(response)
    }

    #[getter]
    pub fn system_instruction<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyList>, AgentError> {
        let instructions = self
            .agent
            .system_instruction
            .iter()
            .map(|msg_num| msg_num.to_bound_py_object(py))
            .collect::<Result<Vec<_>, _>>()
            .map(|instructions| PyList::new(py, &instructions))?;

        Ok(instructions?)
    }

    #[getter]
    pub fn id(&self) -> &str {
        self.agent.id.as_str()
    }
}
