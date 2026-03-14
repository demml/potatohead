use crate::agents::run_context::AgentRunContext;
use potato_type::prompt::Prompt;
use potato_type::tools::ToolCall;
use serde_json::Value;
use std::fmt::Debug;

use crate::agents::types::AgentResponse;

/// Action returned by callback hooks — controls loop continuation.
#[derive(Debug)]
pub enum CallbackAction {
    /// Continue execution normally.
    Continue,
    /// Override the model response with a fixed string and continue.
    OverrideResponse(String),
    /// Abort the run with the given message (surfaces as `AgentError`).
    Abort(String),
}

/// Hook interface called at key points in the agentic loop.
pub trait AgentCallback: Send + Sync + Debug {
    fn before_model_call(&self, _ctx: &AgentRunContext, _prompt: &Prompt) -> CallbackAction {
        CallbackAction::Continue
    }

    fn after_model_call(
        &self,
        _ctx: &AgentRunContext,
        _response: &AgentResponse,
    ) -> CallbackAction {
        CallbackAction::Continue
    }

    fn before_tool_call(&self, _ctx: &AgentRunContext, _call: &ToolCall) -> CallbackAction {
        CallbackAction::Continue
    }

    fn after_tool_call(
        &self,
        _ctx: &AgentRunContext,
        _call: &ToolCall,
        _result: &Value,
    ) -> CallbackAction {
        CallbackAction::Continue
    }
}
