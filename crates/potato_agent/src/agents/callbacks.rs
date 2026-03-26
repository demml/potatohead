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

#[derive(Debug)]
pub struct LoggingCallback;

impl AgentCallback for LoggingCallback {
    fn before_model_call(&self, ctx: &AgentRunContext, _prompt: &Prompt) -> CallbackAction {
        tracing::info!(agent_id = %ctx.agent_id, iteration = ctx.iteration, "before model call");
        CallbackAction::Continue
    }

    fn after_model_call(&self, ctx: &AgentRunContext, response: &AgentResponse) -> CallbackAction {
        tracing::info!(
            agent_id = %ctx.agent_id,
            iteration = ctx.iteration,
            response_len = response.response_text().len(),
            "after model call"
        );
        CallbackAction::Continue
    }

    fn before_tool_call(&self, ctx: &AgentRunContext, call: &ToolCall) -> CallbackAction {
        tracing::info!(agent_id = %ctx.agent_id, tool = %call.tool_name, "before tool call");
        CallbackAction::Continue
    }

    fn after_tool_call(
        &self,
        ctx: &AgentRunContext,
        call: &ToolCall,
        _result: &Value,
    ) -> CallbackAction {
        tracing::info!(agent_id = %ctx.agent_id, tool = %call.tool_name, "after tool call");
        CallbackAction::Continue
    }
}
