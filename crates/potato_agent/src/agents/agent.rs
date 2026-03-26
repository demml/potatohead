use crate::agents::{
    callbacks::{AgentCallback, CallbackAction},
    criteria::CompletionCriteria,
    error::AgentError,
    memory::{Memory, MemoryTurn},
    run_context::{AgentRunConfig, AgentRunContext, ResumeContext},
    runner::{AgentRunOutcome, AgentRunResult, AgentRunner},
    session::{SessionSnapshot, SessionState},
    store::{
        app_state_store::AppStateStore, persistent_memory::PersistentMemory,
        session_store::SessionStore, user_state_store::UserStateStore,
    },
    task::Task,
    tool_ext::AgentTool,
    types::{AgentResponse, PyAgentResponse},
};
use async_trait::async_trait;
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

#[derive(Debug)]
pub struct Agent {
    pub id: String,
    client: Arc<GenAiClient>,
    pub provider: Provider,
    pub system_instruction: Vec<MessageNum>,
    pub tools: Arc<RwLock<ToolRegistry>>,
    pub max_iterations: u32,
    // --- new agentic-loop fields ---
    pub run_config: Option<AgentRunConfig>,
    /// If set, overrides the model in any Prompt built by AgentBuilder::run().
    pub model_override: Option<String>,
    pub criteria: Vec<Box<dyn CompletionCriteria>>,
    pub callbacks: Vec<Arc<dyn AgentCallback>>,
    pub memory: Option<Arc<tokio::sync::Mutex<Box<dyn Memory>>>>,
    /// Application name used for store scoping.
    pub app_name: Option<String>,
    /// User identifier used for store scoping.
    pub user_id: Option<String>,
    /// Session identifier used to key store lookups.
    pub session_id: Option<String>,
    /// Optional durable session state store.
    pub session_store: Option<Arc<dyn SessionStore>>,
    /// Optional per-user state store.
    pub user_state_store: Option<Arc<dyn UserStateStore>>,
    /// Optional app-level state store.
    pub app_state_store: Option<Arc<dyn AppStateStore>>,
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
            Provider::Anthropic => {
                GenAiClient::Anthropic(AnthropicClient::new(ServiceType::Generate)?)
            }
            Provider::Google => {
                GenAiClient::Gemini(GeminiClient::new(ServiceType::Generate).await?)
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
            run_config: None,
            model_override: None,
            criteria: Vec::new(),
            callbacks: Vec::new(),
            memory: None,
            app_name: None,
            user_id: None,
            session_id: None,
            session_store: None,
            user_state_store: None,
            app_state_store: None,
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
            Provider::Google => {
                GenAiClient::Gemini(GeminiClient::new(ServiceType::Generate).await?)
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
            run_config: None,
            model_override: None,
            criteria: Vec::new(),
            callbacks: Vec::new(),
            memory: None,
            app_name: None,
            user_id: None,
            session_id: None,
            session_store: None,
            user_state_store: None,
            app_state_store: None,
        })
    }

    pub fn register_tool(&self, tool: Box<dyn Tool + Send + Sync>) {
        self.tools
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .register_tool(tool);
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
        global_context: &Option<Arc<Value>>,
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
    fn prepend_system_instructions(&self, prompt: &mut Prompt) -> Result<(), AgentError> {
        if !self.system_instruction.is_empty() {
            prompt
                .request
                .prepend_system_instructions(self.system_instruction.clone())
                .map_err(|e| AgentError::Error(e.to_string()))?;
        }
        Ok(())
    }
    pub async fn execute_task(&self, task: &Task) -> Result<AgentResponse, AgentError> {
        // Extract the prompt from the task
        debug!("Executing task: {}, count: {}", task.id, task.retry_count);
        let mut prompt = task.prompt.clone();
        self.prepend_system_instructions(&mut prompt)?;

        // Use the client to execute the task
        let chat_response = self.client.generate_content(&prompt).await?;

        Ok(AgentResponse::new(task.id.clone(), chat_response))
    }

    #[instrument(skip_all)]
    pub async fn execute_prompt(&self, prompt: &Prompt) -> Result<AgentResponse, AgentError> {
        // Extract the prompt from the task
        debug!("Executing prompt");
        let mut prompt = prompt.clone();
        self.prepend_system_instructions(&mut prompt)?;

        // Use the client to execute the task
        let chat_response = self.client.generate_content(&prompt).await?;

        Ok(AgentResponse::new(chat_response.id(), chat_response))
    }

    /// Execute task with context without mutating the original task
    /// This method is used by the workflow executor to run individual tasks with context
    #[instrument(skip_all)]
    pub async fn execute_task_with_context(
        &self,
        task: &Arc<RwLock<Task>>,
        context: &Value,
    ) -> Result<AgentResponse, AgentError> {
        // Clone prompt and task_id without holding lock across await
        let (mut prompt, task_id) = {
            let task = task.read().unwrap();
            (task.prompt.clone(), task.id.clone())
        };

        self.bind_context(&mut prompt, context, &None)?;
        self.prepend_system_instructions(&mut prompt)?;

        let chat_response = self.client.generate_content(&prompt).await?;
        Ok(AgentResponse::new(task_id, chat_response))
    }

    pub async fn execute_task_with_context_message(
        &self,
        task: &Arc<RwLock<Task>>,
        context_messages: HashMap<String, Vec<MessageNum>>,
        parameter_context: Value,
        global_context: Option<Arc<Value>>,
    ) -> Result<AgentResponse, AgentError> {
        // Prepare prompt and context before await
        let (prompt, task_id) = {
            let mut task = task.write().unwrap();
            // 1. Add dependency context (should come after system instructions, before user message)
            self.append_task_with_message_dependency_context(&mut task, &context_messages);
            // 2. Bind parameters
            self.bind_context(&mut task.prompt, &parameter_context, &global_context)?;
            // 3. Prepend agent system instructions (add to front)
            self.prepend_system_instructions(&mut task.prompt)?;
            (task.prompt.clone(), task.id.clone())
        };

        // Now do the async work without holding the lock
        let chat_response = self.client.generate_content(&prompt).await?;
        Ok(AgentResponse::new(task_id, chat_response))
    }

    pub fn client_provider(&self) -> &Provider {
        self.client.provider()
    }

    // ── Agentic loop helper ─────────────────────────────────────────────────

    /// Build a minimal one-turn Prompt from a plain input string.
    fn build_input_prompt(&self, input: &str) -> Result<Prompt, AgentError> {
        use potato_type::prompt::builder::to_provider_request;
        use potato_type::prompt::settings::ModelSettings;
        use potato_type::prompt::types::ResponseType;

        let msg = {
            use potato_type::traits::MessageFactory;
            match self.provider {
                Provider::OpenAI => {
                    use potato_type::openai::v1::chat::request::ChatMessage;
                    ChatMessage::from_text(input.to_string(), "user")
                        .map(MessageNum::OpenAIMessageV1)?
                }
                Provider::Anthropic => {
                    use potato_type::anthropic::v1::request::MessageParam;
                    MessageParam::from_text(input.to_string(), "user")
                        .map(MessageNum::AnthropicMessageV1)?
                }
                Provider::Gemini | Provider::Google | Provider::Vertex => {
                    use potato_type::google::v1::generate::request::GeminiContent;
                    GeminiContent::from_text(input.to_string(), "user")
                        .map(MessageNum::GeminiContentV1)?
                }
                _ => {
                    return Err(AgentError::MissingProviderError);
                }
            }
        };

        let model = self.model_override.clone().ok_or_else(|| {
            AgentError::Error("model must be set explicitly via AgentBuilder::model()".into())
        })?;

        let settings = ModelSettings::provider_default_settings(&self.provider);

        let request = to_provider_request(
            vec![msg],
            self.system_instruction.clone(),
            model.clone(),
            settings,
            None,
        )?;

        Ok(Prompt {
            request,
            model,
            provider: self.provider.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            parameters: Vec::new(),
            response_type: ResponseType::Null,
        })
    }

    /// Fire `before_model_call` callbacks. Returns an error if any callback aborts.
    fn fire_before_model(&self, ctx: &AgentRunContext, prompt: &Prompt) -> Result<(), AgentError> {
        for cb in &self.callbacks {
            if let CallbackAction::Abort(msg) = cb.before_model_call(ctx, prompt) {
                return Err(AgentError::CallbackAbort(msg));
            }
        }
        Ok(())
    }

    /// Fire `after_model_call` callbacks. Returns `Some(override_text)` or None.
    fn fire_after_model(
        &self,
        ctx: &AgentRunContext,
        response: &AgentResponse,
    ) -> Result<Option<String>, AgentError> {
        for cb in &self.callbacks {
            match cb.after_model_call(ctx, response) {
                CallbackAction::Abort(msg) => return Err(AgentError::CallbackAbort(msg)),
                CallbackAction::OverrideResponse(text) => return Ok(Some(text)),
                CallbackAction::Continue => {}
            }
        }
        Ok(None)
    }

    /// Fire `before_tool_call` callbacks.
    fn fire_before_tool(
        &self,
        ctx: &AgentRunContext,
        call: &potato_type::tools::ToolCall,
    ) -> Result<(), AgentError> {
        for cb in &self.callbacks {
            if let CallbackAction::Abort(msg) = cb.before_tool_call(ctx, call) {
                return Err(AgentError::CallbackAbort(msg));
            }
        }
        Ok(())
    }

    /// Fire `after_tool_call` callbacks.
    fn fire_after_tool(
        &self,
        ctx: &AgentRunContext,
        call: &potato_type::tools::ToolCall,
        result: &serde_json::Value,
    ) -> Result<(), AgentError> {
        for cb in &self.callbacks {
            if let CallbackAction::Abort(msg) = cb.after_tool_call(ctx, call, result) {
                return Err(AgentError::CallbackAbort(msg));
            }
        }
        Ok(())
    }
}

#[async_trait]
impl AgentRunner for Agent {
    fn id(&self) -> &str {
        &self.id
    }

    async fn run(
        &self,
        input: &str,
        session: &mut SessionState,
    ) -> Result<AgentRunOutcome, AgentError> {
        let max_iter = self
            .run_config
            .as_ref()
            .map(|c| c.max_iterations)
            .unwrap_or(self.max_iterations);

        let mut run_ctx = AgentRunContext::new(self.id.clone(), max_iter);

        let app = self.app_name.as_deref().unwrap_or("default");
        let uid = self.user_id.as_deref().unwrap_or("default");

        // Load app-level state (lowest precedence — overwritten by later loads).
        if let Some(store) = &self.app_state_store {
            if let Some(snapshot) = store.load(app).await? {
                session.merge(snapshot.0);
            }
        }

        // Load user-level state (medium precedence).
        if let Some(store) = &self.user_state_store {
            if let Some(snapshot) = store.load(app, uid).await? {
                session.merge(snapshot.0);
            }
        }

        // Load session state (highest precedence — wins over user and app).
        if let (Some(sid), Some(store)) = (&self.session_id, &self.session_store) {
            if let Some(snapshot) = store.load(app, uid, sid).await? {
                session.merge(snapshot.0);
            }
        }

        // Build the prompt from the input string
        let mut prompt = self.build_input_prompt(input)?;

        // Hydrate PersistentMemory from the backing store (lazy, idempotent).
        if let Some(mem_lock) = &self.memory {
            let mut mem = mem_lock.lock().await;
            if let Some(pm) = mem
                .as_any_mut()
                .and_then(|a| a.downcast_mut::<PersistentMemory>())
            {
                pm.hydrate().await?;
            }
        }

        // Inject memory history in chronological order, after any system messages
        if let Some(mem_lock) = &self.memory {
            let mem = mem_lock.lock().await;
            let history = mem.messages();
            if !history.is_empty() {
                // Find the first non-system message position; insert history before it
                let insert_at = prompt
                    .request
                    .messages()
                    .iter()
                    .position(|m| !m.is_system_message())
                    .unwrap_or(0);
                for (i, msg) in history.into_iter().enumerate() {
                    prompt.request.insert_message(msg, Some(insert_at + i));
                }
            }
        }

        // Attach tool definitions
        {
            let registry = self.tools.read().unwrap_or_else(|e| e.into_inner());
            let defs = registry.get_all_definitions();
            if !defs.is_empty() {
                prompt.request.add_tools(defs)?;
            }
        }

        let mut last_user_msg: Option<MessageNum> = None;
        // Capture the user message for memory storage later
        if let Some(msg) = prompt.request.messages().last().cloned() {
            last_user_msg = Some(msg);
        }

        loop {
            // Check max iterations
            if run_ctx.iteration >= max_iter {
                break;
            }

            // Before-model callbacks
            self.fire_before_model(&run_ctx, &prompt)?;

            // Call the LLM
            let chat_response = self.client.generate_content(&prompt).await?;
            let agent_response = AgentResponse::new(chat_response.id(), chat_response.clone());

            // After-model callbacks
            if let Some(override_text) = self.fire_after_model(&run_ctx, &agent_response)? {
                run_ctx.push_response(override_text.clone());
                return Ok(AgentRunOutcome::complete(AgentRunResult {
                    final_response: agent_response,
                    iterations: run_ctx.iteration,
                    completion_reason: format!("callback override: {}", override_text),
                    combined_text: None,
                }));
            }

            // Check for tool calls
            if let Some(tool_calls) = chat_response.extract_tool_calls() {
                // Append assistant message with tool calls to the prompt
                let assistant_msgs = chat_response.to_message_num(&self.provider)?;
                for msg in assistant_msgs {
                    prompt.request.push_message(msg);
                }

                for call in &tool_calls {
                    self.fire_before_tool(&run_ctx, call)?;

                    // Try async tool first, then sync
                    let result = {
                        let async_tool = {
                            let registry = self.tools.read().unwrap_or_else(|e| e.into_inner());
                            registry.get_async_tool(&call.tool_name)
                        };
                        if let Some(tool) = async_tool {
                            if let Some(agent_tool) =
                                tool.as_any().and_then(|a| a.downcast_ref::<AgentTool>())
                            {
                                // Route AgentTool through dispatch() to propagate ancestor tracking.
                                agent_tool
                                    .dispatch(call.arguments.clone(), session)
                                    .await
                                    .map_err(|e| {
                                        AgentError::Error(format!(
                                            "Tool '{}' failed: {}",
                                            call.tool_name, e
                                        ))
                                    })?
                            } else {
                                tool.execute(call.arguments.clone()).await.map_err(|e| {
                                    AgentError::Error(format!(
                                        "Tool '{}' failed: {}",
                                        call.tool_name, e
                                    ))
                                })?
                            }
                        } else {
                            let registry = self.tools.read().unwrap_or_else(|e| e.into_inner());
                            registry.execute(call).map_err(|e| {
                                AgentError::Error(format!(
                                    "Tool '{}' failed: {}",
                                    call.tool_name, e
                                ))
                            })?
                        }
                    };

                    self.fire_after_tool(&run_ctx, call, &result)?;
                    prompt.request.add_tool_result(call, &result)?;
                }

                run_ctx.increment();
                continue;
            }

            // No tool calls — this is a candidate final response
            let text = chat_response.response_text();

            // Check for ask_user tool pattern (special built-in)
            if text.trim().starts_with("__ask_user__:") {
                let question = text.trim_start_matches("__ask_user__:").trim().to_string();
                let resume_ctx = ResumeContext {
                    agent_id: self.id.clone(),
                    iteration: run_ctx.iteration,
                    session_snapshot: session.snapshot(),
                };
                return Ok(AgentRunOutcome::NeedsInput {
                    question,
                    resume_context: resume_ctx,
                });
            }

            run_ctx.push_response(text);

            // Check completion criteria (any = stop)
            let met = self.criteria.iter().any(|c| c.is_complete(&run_ctx));
            let reason = if met {
                self.criteria
                    .iter()
                    .find(|c| c.is_complete(&run_ctx))
                    .map(|c| c.completion_reason(&run_ctx))
                    .unwrap_or_else(|| "criteria met".into())
            } else {
                String::new()
            };

            if met || run_ctx.iteration + 1 >= max_iter {
                // Store memory turn
                if let Some(mem_lock) = &self.memory {
                    let mut mem = mem_lock.lock().await;
                    if let Some(user_msg) = last_user_msg.take() {
                        let assistant_msgs = chat_response.to_message_num(&self.provider)?;
                        if let Some(asst_msg) = assistant_msgs.into_iter().next() {
                            let turn = MemoryTurn {
                                user: user_msg,
                                assistant: asst_msg,
                            };
                            // Use write-through async path for PersistentMemory.
                            if let Some(pm) = mem
                                .as_any_mut()
                                .and_then(|a| a.downcast_mut::<PersistentMemory>())
                            {
                                pm.push_turn_async(turn).await?;
                            } else {
                                mem.push_turn(turn);
                            }
                        }
                    }
                }

                // Persist session state to backing store.
                if let (Some(sid), Some(store)) = (&self.session_id, &self.session_store) {
                    let snapshot = SessionSnapshot::from(&*session);
                    store.save(app, uid, sid, &snapshot).await?;
                }

                return Ok(AgentRunOutcome::complete(AgentRunResult {
                    final_response: agent_response,
                    iterations: run_ctx.iteration,
                    completion_reason: if met {
                        reason
                    } else {
                        format!("max iterations ({}) reached", max_iter)
                    },
                    combined_text: None,
                }));
            }

            // Not complete yet — append assistant message and continue
            let assistant_msgs = chat_response.to_message_num(&self.provider)?;
            for msg in assistant_msgs {
                prompt.request.push_message(msg);
            }

            run_ctx.increment();
        }

        // Fell out of the loop without a final response — max iterations were all spent on tool calls
        Err(AgentError::MaxIterationsExceeded(max_iter))
    }

    async fn resume(
        &self,
        user_answer: &str,
        ctx: ResumeContext,
        session: &mut SessionState,
    ) -> Result<AgentRunOutcome, AgentError> {
        // Restore session from the snapshot in ResumeContext
        session.merge(ctx.session_snapshot);
        // Re-run with the user's answer as new input
        self.run(user_answer, session).await
    }
}

/// Manual Clone: clones the provider-level fields; criteria/callbacks/memory are NOT cloned.
/// This preserves backward compatibility with the workflow layer which stores `Arc<Agent>`.
impl Clone for Agent {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            client: self.client.clone(),
            provider: self.provider.clone(),
            system_instruction: self.system_instruction.clone(),
            tools: self.tools.clone(),
            max_iterations: self.max_iterations,
            run_config: self.run_config.clone(),
            model_override: self.model_override.clone(),
            // Non-clonable fields — intentionally reset on clone
            criteria: Vec::new(),
            callbacks: Vec::new(),
            memory: None,
            app_name: None,
            user_id: None,
            session_id: None,
            session_store: None,
            user_state_store: None,
            app_state_store: None,
        }
    }
}

impl PartialEq for Agent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.provider == other.provider
            && self.system_instruction == other.system_instruction
            && self.max_iterations == other.max_iterations
            && self.client == other.client
        // criteria / callbacks / memory intentionally excluded
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
                    run_config: None,
                    model_override: None,
                    criteria: Vec::new(),
                    callbacks: Vec::new(),
                    memory: None,
                    app_name: None,
                    user_id: None,
                    session_id: None,
                    session_store: None,
                    user_state_store: None,
                    app_state_store: None,
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
