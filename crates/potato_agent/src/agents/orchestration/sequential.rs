use crate::agents::{
    error::AgentError,
    run_context::ResumeContext,
    runner::{AgentRunOutcome, AgentRunResult, AgentRunner},
    session::SessionState,
};
use async_trait::async_trait;
use potato_util::create_uuid7;
use std::fmt::Debug;
use std::sync::Arc;

/// Runs agents in sequence: A → B → C.
/// When `pass_output` is `true` each agent receives the previous agent's text response as input.
#[derive(Debug)]
pub struct SequentialAgent {
    id: String,
    agents: Vec<Arc<dyn AgentRunner>>,
    pass_output: bool,
}

impl SequentialAgent {
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[async_trait]
impl AgentRunner for SequentialAgent {
    fn id(&self) -> &str {
        &self.id
    }

    async fn run(
        &self,
        input: &str,
        session: &mut SessionState,
    ) -> Result<AgentRunOutcome, AgentError> {
        let mut current_input = input.to_string();
        let mut last_result: Option<AgentRunResult> = None;

        for agent in &self.agents {
            match agent.run(&current_input, session).await? {
                AgentRunOutcome::Complete(result) => {
                    if self.pass_output {
                        current_input = result.final_response.response_text();
                    }
                    last_result = Some(*result);
                }
                AgentRunOutcome::NeedsInput {
                    question,
                    resume_context,
                } => {
                    // Propagate NeedsInput upward immediately
                    return Ok(AgentRunOutcome::NeedsInput {
                        question,
                        resume_context,
                    });
                }
            }
        }

        match last_result {
            Some(result) => Ok(AgentRunOutcome::complete(result)),
            None => Err(AgentError::Error(
                "SequentialAgent has no agents".to_string(),
            )),
        }
    }

    async fn resume(
        &self,
        user_answer: &str,
        ctx: ResumeContext,
        session: &mut SessionState,
    ) -> Result<AgentRunOutcome, AgentError> {
        // Find the agent that owns this resume context and delegate
        for agent in &self.agents {
            if agent.id() == ctx.agent_id {
                return agent.resume(user_answer, ctx, session).await;
            }
        }
        Err(AgentError::Error(format!(
            "No agent with id '{}' found in sequential pipeline",
            ctx.agent_id
        )))
    }
}

/// Builder for `SequentialAgent`.
#[derive(Default)]
pub struct SequentialAgentBuilder {
    agents: Vec<Arc<dyn AgentRunner>>,
    pass_output: bool,
}

impl SequentialAgentBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn then(mut self, agent: Arc<dyn AgentRunner>) -> Self {
        self.agents.push(agent);
        self
    }

    /// If `true`, each agent in the chain receives the previous agent's text output as its input.
    pub fn pass_output(mut self, yes: bool) -> Self {
        self.pass_output = yes;
        self
    }

    pub fn build(self) -> Arc<SequentialAgent> {
        Arc::new(SequentialAgent {
            id: create_uuid7(),
            agents: self.agents,
            pass_output: self.pass_output,
        })
    }
}
