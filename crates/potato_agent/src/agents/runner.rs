use crate::agents::{
    error::AgentError, run_context::ResumeContext, session::SessionState, types::AgentResponse,
};
use async_trait::async_trait;
use std::fmt::Debug;

/// Final result of a completed agent run.
#[derive(Debug, Clone)]
pub struct AgentRunResult {
    pub final_response: AgentResponse,
    pub iterations: u32,
    pub completion_reason: String,
}

/// Outcome of calling `AgentRunner::run()`.
#[derive(Debug)]
pub enum AgentRunOutcome {
    /// The run completed — final answer available.
    Complete(Box<AgentRunResult>),
    /// The agent is paused and needs user input before continuing.
    NeedsInput {
        question: String,
        resume_context: ResumeContext,
    },
}

impl AgentRunOutcome {
    pub fn complete(result: AgentRunResult) -> Self {
        Self::Complete(Box::new(result))
    }
}

/// Core trait implemented by all runnable agents (Agent, SequentialAgent, ParallelAgent, AgentTool).
#[async_trait]
pub trait AgentRunner: Send + Sync + Debug {
    fn id(&self) -> &str;

    async fn run(
        &self,
        input: &str,
        session: &mut SessionState,
    ) -> Result<AgentRunOutcome, AgentError>;

    /// Resume a previously paused run with the user's answer.
    async fn resume(
        &self,
        user_answer: &str,
        ctx: ResumeContext,
        session: &mut SessionState,
    ) -> Result<AgentRunOutcome, AgentError>;
}
