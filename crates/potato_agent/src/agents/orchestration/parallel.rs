use crate::agents::{
    error::AgentError,
    run_context::ResumeContext,
    runner::{AgentRunOutcome, AgentRunResult, AgentRunner},
    session::SessionState,
};
use async_trait::async_trait;
use potato_util::create_uuid7;
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;

/// How to combine results from parallel agents.
#[derive(Debug, Clone, Default)]
pub enum MergeStrategy {
    /// Collect every agent's response text into a JSON array.
    #[default]
    CollectAll,
    /// Return the first agent that completes.
    First,
}

/// Runs all agents concurrently; merges results according to `strategy`.
#[derive(Debug)]
pub struct ParallelAgent {
    id: String,
    agents: Vec<Arc<dyn AgentRunner>>,
    strategy: MergeStrategy,
}

#[async_trait]
impl AgentRunner for ParallelAgent {
    fn id(&self) -> &str {
        &self.id
    }

    async fn run(
        &self,
        input: &str,
        session: &mut SessionState,
    ) -> Result<AgentRunOutcome, AgentError> {
        // Each child gets a snapshot clone of the session; we merge back after join.
        let mut handles = Vec::with_capacity(self.agents.len());

        for agent in &self.agents {
            let agent_clone = Arc::clone(agent);
            let input_owned = input.to_string();
            // Give each child a fresh session clone
            let mut child_session = SessionState::new();
            child_session.merge(session.snapshot());

            let handle = tokio::spawn(async move {
                let result = agent_clone.run(&input_owned, &mut child_session).await;
                (result, child_session.snapshot())
            });
            handles.push(handle);
        }

        let mut outcomes: Vec<AgentRunResult> = Vec::new();
        let mut any_needs_input: Option<(String, ResumeContext)> = None;

        for handle in handles {
            let (outcome, child_snapshot) = handle
                .await
                .map_err(|e| AgentError::Error(format!("Parallel join error: {}", e)))?;

            // Merge child session back into parent
            session.merge(child_snapshot);

            match outcome? {
                AgentRunOutcome::Complete(result) => {
                    outcomes.push(*result);
                }
                AgentRunOutcome::NeedsInput {
                    question,
                    resume_context,
                } => {
                    any_needs_input = Some((question, resume_context));
                }
            }
        }

        // If any child needed input, propagate it
        if let Some((question, resume_context)) = any_needs_input {
            return Ok(AgentRunOutcome::NeedsInput {
                question,
                resume_context,
            });
        }

        if outcomes.is_empty() {
            return Err(AgentError::Error(
                "ParallelAgent: no agents produced results".to_string(),
            ));
        }

        match self.strategy {
            MergeStrategy::First => Ok(AgentRunOutcome::complete(
                outcomes.into_iter().next().unwrap(),
            )),
            MergeStrategy::CollectAll => {
                // Combine all text responses into a JSON array
                let texts: Vec<Value> = outcomes
                    .iter()
                    .map(|r| Value::String(r.final_response.response_text()))
                    .collect();
                let combined = Value::Array(texts).to_string();
                // Build a synthetic AgentRunResult wrapping the last agent's response with combined text
                let last = outcomes.into_iter().last().unwrap();
                // We can't mutate ChatResponse easily, so we store the combined text in session
                session.set("__parallel_combined", Value::String(combined));
                Ok(AgentRunOutcome::complete(AgentRunResult {
                    final_response: last.final_response,
                    iterations: last.iterations,
                    completion_reason: "all parallel agents completed".into(),
                }))
            }
        }
    }

    async fn resume(
        &self,
        user_answer: &str,
        ctx: ResumeContext,
        session: &mut SessionState,
    ) -> Result<AgentRunOutcome, AgentError> {
        for agent in &self.agents {
            if agent.id() == ctx.agent_id {
                return agent.resume(user_answer, ctx, session).await;
            }
        }
        Err(AgentError::Error(format!(
            "No agent with id '{}' found in parallel group",
            ctx.agent_id
        )))
    }
}

/// Builder for `ParallelAgent`.
#[derive(Default)]
pub struct ParallelAgentBuilder {
    agents: Vec<Arc<dyn AgentRunner>>,
    strategy: MergeStrategy,
}

impl ParallelAgentBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_agent(mut self, agent: Arc<dyn AgentRunner>) -> Self {
        self.agents.push(agent);
        self
    }

    pub fn merge_strategy(mut self, strategy: MergeStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn build(self) -> Arc<ParallelAgent> {
        Arc::new(ParallelAgent {
            id: create_uuid7(),
            agents: self.agents,
            strategy: self.strategy,
        })
    }
}
