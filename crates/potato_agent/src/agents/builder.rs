use crate::agents::{
    agent::Agent,
    callbacks::AgentCallback,
    criteria::{CompletionCriteria, KeywordCriteria, StructuredOutputCriteria},
    error::AgentError,
    memory::{InMemoryMemory, Memory, WindowedMemory},
    run_context::AgentRunConfig,
    store::{
        app_state_store::AppStateStore, persistent_memory::PersistentMemory,
        user_state_store::UserStateStore, MemoryStore, SessionStore,
    },
    tool_ext::{AgentTool, AgentToolPolicy},
};
use potato_type::{
    tools::{AsyncTool, Tool},
    Provider,
};
use std::sync::Arc;

/// Fluent builder for `Agent`.
pub struct AgentBuilder {
    provider: Option<Provider>,
    model: Option<String>,
    system_prompt: Option<String>,
    max_iterations: u32,
    memory: Option<Box<dyn Memory>>,
    tools: Vec<Box<dyn Tool + Send + Sync>>,
    async_tools: Vec<Arc<dyn AsyncTool>>,
    sub_agents: Vec<AgentTool>,
    criteria: Vec<Box<dyn CompletionCriteria>>,
    callbacks: Vec<Arc<dyn AgentCallback>>,
    app_name: Option<String>,
    user_id: Option<String>,
    session_id: Option<String>,
    session_store: Option<Arc<dyn SessionStore>>,
    user_state_store: Option<Arc<dyn UserStateStore>>,
    app_state_store: Option<Arc<dyn AppStateStore>>,
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentBuilder {
    pub fn new() -> Self {
        Self {
            provider: None,
            model: None,
            system_prompt: None,
            max_iterations: 10,
            memory: None,
            tools: Vec::new(),
            async_tools: Vec::new(),
            sub_agents: Vec::new(),
            criteria: Vec::new(),
            callbacks: Vec::new(),
            app_name: None,
            user_id: None,
            session_id: None,
            session_store: None,
            user_state_store: None,
            app_state_store: None,
        }
    }

    pub fn provider(mut self, provider: Provider) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn max_iterations(mut self, n: u32) -> Self {
        self.max_iterations = n;
        self
    }

    /// Use a simple unbounded in-memory memory store.
    pub fn with_in_memory(mut self) -> Self {
        self.memory = Some(Box::new(InMemoryMemory::new()));
        self
    }

    /// Use a sliding-window memory store (keeps last `n` turns).
    pub fn with_windowed_memory(mut self, n: usize) -> Self {
        self.memory = Some(Box::new(WindowedMemory::new(n)));
        self
    }

    /// Register a synchronous tool.
    pub fn with_tool(mut self, tool: impl Tool + 'static) -> Self {
        self.tools.push(Box::new(tool));
        self
    }

    /// Register an asynchronous tool.
    pub fn with_async_tool(mut self, tool: Arc<dyn AsyncTool>) -> Self {
        self.async_tools.push(tool);
        self
    }

    /// Register a sub-agent — wraps it as an `AgentTool` automatically.
    pub fn with_sub_agent(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        runner: Arc<dyn crate::agents::runner::AgentRunner>,
    ) -> Self {
        self.sub_agents
            .push(AgentTool::new(name, description, runner));
        self
    }

    /// Register a sub-agent with a custom `AgentToolPolicy`.
    pub fn with_sub_agent_policy(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        runner: Arc<dyn crate::agents::runner::AgentRunner>,
        policy: AgentToolPolicy,
    ) -> Self {
        self.sub_agents
            .push(AgentTool::new(name, description, runner).with_policy(policy));
        self
    }

    /// Stop when the last response contains `keyword`.
    pub fn stop_on_keyword(mut self, keyword: impl Into<String>) -> Self {
        self.criteria.push(Box::new(KeywordCriteria::new(keyword)));
        self
    }

    /// Stop when the last response is valid JSON (optionally matching `schema`).
    pub fn stop_on_structured_output(mut self, schema: Option<serde_json::Value>) -> Self {
        self.criteria
            .push(Box::new(StructuredOutputCriteria::new(schema)));
        self
    }

    /// Add a lifecycle callback.
    pub fn with_callback(mut self, cb: Arc<dyn AgentCallback>) -> Self {
        self.callbacks.push(cb);
        self
    }

    /// Set the application name for store scoping.
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = Some(name.into());
        self
    }

    /// Set the user ID for store scoping.
    pub fn user_id(mut self, id: impl Into<String>) -> Self {
        self.user_id = Some(id.into());
        self
    }

    /// Attach an unbounded `MemoryStore` for persistent conversation history.
    /// Replaces any previously set in-memory memory.
    pub fn with_memory_store(
        mut self,
        session_id: impl Into<String>,
        store: Arc<dyn MemoryStore>,
    ) -> Self {
        let sid = session_id.into();
        let app = self.app_name.clone().unwrap_or_else(|| "default".into());
        let uid = self.user_id.clone().unwrap_or_else(|| "default".into());
        self.memory = Some(Box::new(PersistentMemory::new(
            sid.clone(),
            app,
            uid,
            store,
        )));
        self.session_id = Some(sid);
        self
    }

    /// Attach a windowed `MemoryStore` (keeps only the last `window` turns in cache).
    pub fn with_windowed_memory_store(
        mut self,
        session_id: impl Into<String>,
        store: Arc<dyn MemoryStore>,
        window: usize,
    ) -> Self {
        let sid = session_id.into();
        let app = self.app_name.clone().unwrap_or_else(|| "default".into());
        let uid = self.user_id.clone().unwrap_or_else(|| "default".into());
        self.memory = Some(Box::new(PersistentMemory::windowed(
            sid.clone(),
            app,
            uid,
            store,
            window,
        )));
        self.session_id = Some(sid);
        self
    }

    /// Attach a `SessionStore` for durable key-value session state.
    pub fn with_session_store(
        mut self,
        session_id: impl Into<String>,
        store: Arc<dyn SessionStore>,
    ) -> Self {
        let sid = session_id.into();
        self.session_store = Some(store);
        self.session_id = Some(sid);
        self
    }

    /// Attach a `UserStateStore` for per-user state.
    pub fn with_user_state_store(mut self, store: Arc<dyn UserStateStore>) -> Self {
        self.user_state_store = Some(store);
        self
    }

    /// Attach an `AppStateStore` for app-level state.
    pub fn with_app_state_store(mut self, store: Arc<dyn AppStateStore>) -> Self {
        self.app_state_store = Some(store);
        self
    }

    /// Build the `Agent`.  Async because some providers (Gemini/Vertex) need async init.
    pub async fn build(self) -> Result<Arc<Agent>, AgentError> {
        let provider = self.provider.ok_or(AgentError::MissingProviderError)?;

        // Build system instructions if provided
        let system_instructions = if let Some(ref text) = self.system_prompt {
            use potato_type::prompt::interface::create_system_message_for_provider;
            Some(vec![create_system_message_for_provider(
                text.clone(),
                &provider,
            )?])
        } else {
            None
        };

        // Create the base agent
        let mut agent = Agent::new(provider.clone(), system_instructions).await?;

        // Configure run settings
        agent.run_config = Some(AgentRunConfig {
            max_iterations: self.max_iterations,
        });

        // Set model override
        agent.model_override = self.model;

        // Register tools
        {
            let mut registry = agent.tools.write().unwrap_or_else(|e| e.into_inner());
            for tool in self.tools {
                registry.register_tool(tool);
            }
            for async_tool in self.async_tools {
                registry.register_async_tool(async_tool);
            }
            for sub_agent in self.sub_agents {
                registry.register_async_tool(Arc::new(sub_agent));
            }
        }

        // Attach criteria and callbacks
        agent.criteria = self.criteria;
        agent.callbacks = self.callbacks;

        // Attach memory
        if let Some(mem) = self.memory {
            agent.memory = Some(Arc::new(tokio::sync::Mutex::new(mem)));
        }

        // Attach stores and scoping
        agent.app_name = self.app_name;
        agent.user_id = self.user_id;
        agent.session_id = self.session_id;
        agent.session_store = self.session_store;
        agent.user_state_store = self.user_state_store;
        agent.app_state_store = self.app_state_store;

        Ok(Arc::new(agent))
    }
}
