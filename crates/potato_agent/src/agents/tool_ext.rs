use crate::agents::{
    error::AgentError,
    runner::{AgentRunOutcome, AgentRunner},
    session::SessionState,
};
use async_trait::async_trait;
use potato_type::{tools::AsyncTool, TypeError};
use serde_json::Value;
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Arc;

/// Call-control policy for an `AgentTool`.
#[derive(Debug, Clone, Default)]
pub struct AgentToolPolicy {
    /// Prevent the sub-agent from invoking any `AgentTool` during its run.
    pub disallow_sub_agent_calls: bool,
    /// Explicit set of agent IDs that this sub-agent cannot call.
    pub disallowed_agent_ids: HashSet<String>,
}

/// Wraps any `AgentRunner` as both an `AsyncTool` (preferred) and a blocking `Tool`.
///
/// The LLM calls the sub-agent by passing `{"input": "<some string>"}`.
#[derive(Debug, Clone)]
pub struct AgentTool {
    name: String,
    description: String,
    runner: Arc<dyn AgentRunner>,
    pub policy: AgentToolPolicy,
}

impl AgentTool {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        runner: Arc<dyn AgentRunner>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            runner,
            policy: AgentToolPolicy::default(),
        }
    }

    pub fn with_policy(mut self, policy: AgentToolPolicy) -> Self {
        self.policy = policy;
        self
    }

    pub fn runner(&self) -> &Arc<dyn AgentRunner> {
        &self.runner
    }

    fn fixed_schema() -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" }
            },
            "required": ["input"]
        })
    }

    /// Extract the `input` field from args and run the sub-agent.
    async fn dispatch(&self, args: Value, session: &mut SessionState) -> Result<Value, AgentError> {
        // Check circular call
        if session.is_ancestor(self.runner.id()) {
            return Err(AgentError::CircularAgentCall(self.runner.id().to_string()));
        }
        // Check disallowed list
        if self.policy.disallowed_agent_ids.contains(self.runner.id()) {
            return Err(AgentError::DisallowedAgentCall(
                self.runner.id().to_string(),
            ));
        }

        let input = args
            .get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        session.push_ancestor(self.runner.id());
        let result = self.runner.run(&input, session).await;
        session.pop_ancestor();

        match result? {
            AgentRunOutcome::Complete(r) => Ok(Value::String(r.final_response.response_text())),
            AgentRunOutcome::NeedsInput { question, .. } => {
                // Surfaced as an error — the parent loop should handle NeedsInput at its level
                Err(AgentError::SubAgentNeedsInput(question))
            }
        }
    }
}

#[async_trait]
impl AsyncTool for AgentTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameter_schema(&self) -> Value {
        Self::fixed_schema()
    }

    async fn execute(&self, args: Value) -> Result<Value, TypeError> {
        let mut session = SessionState::new();
        self.dispatch(args, &mut session)
            .await
            .map_err(|e| TypeError::Error(e.to_string()))
    }
}

/// Sync bridge — calls into a new Tokio runtime via `potato_state::block_on`.
impl potato_type::tools::Tool for AgentTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameter_schema(&self) -> Value {
        Self::fixed_schema()
    }

    fn execute(&self, args: Value) -> Result<Value, TypeError> {
        potato_state::block_on(async {
            let mut session = SessionState::new();
            self.dispatch(args, &mut session)
                .await
                .map_err(|e| TypeError::Error(e.to_string()))
        })
    }
}
