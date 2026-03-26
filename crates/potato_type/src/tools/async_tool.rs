use crate::TypeError;
use async_trait::async_trait;
use serde_json::Value;
use std::fmt::Debug;

/// Async version of the Tool trait for use in the agentic loop.
/// Implements async execute for non-blocking tool execution.
#[async_trait]
pub trait AsyncTool: Send + Sync + Debug {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameter_schema(&self) -> Value;
    async fn execute(&self, args: Value) -> Result<Value, TypeError>;

    /// Optional downcast hook. Override in concrete types that need session-aware dispatch.
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        None
    }
}
