pub mod async_tool;
pub use async_tool::AsyncTool;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::TypeError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema for parameters
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    /// Optional call ID — set by OpenAI (tool_calls[].id) and Anthropic (ToolUseBlock.id)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
    pub arguments: serde_json::Value,
}

#[derive(Debug)]
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool + Send + Sync>>,
    async_tools: HashMap<String, Arc<dyn AsyncTool>>,
}

pub trait Tool: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameter_schema(&self) -> serde_json::Value;
    fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value, TypeError>;
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            async_tools: HashMap::new(),
        }
    }

    pub fn register_tool(&mut self, tool: Box<dyn Tool + Send + Sync>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn register_async_tool(&mut self, tool: Arc<dyn AsyncTool>) {
        self.async_tools.insert(tool.name().to_string(), tool);
    }

    pub fn execute(&self, call: &ToolCall) -> Result<serde_json::Value, TypeError> {
        self.tools
            .get(&call.tool_name)
            .ok_or_else(|| TypeError::Error(format!("Tool {} not found", call.tool_name)))?
            .execute(call.arguments.clone())
    }

    pub fn get_definitions(&self) -> Vec<AgentToolDefinition> {
        self.tools
            .values()
            .map(|tool| AgentToolDefinition {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                parameters: tool.parameter_schema(),
            })
            .collect()
    }

    /// Get definitions for both sync and async tools (used by the agentic loop)
    pub fn get_all_definitions(&self) -> Vec<AgentToolDefinition> {
        let mut defs: Vec<AgentToolDefinition> = self
            .tools
            .values()
            .map(|tool| AgentToolDefinition {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                parameters: tool.parameter_schema(),
            })
            .collect();

        for tool in self.async_tools.values() {
            defs.push(AgentToolDefinition {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                parameters: tool.parameter_schema(),
            });
        }

        defs
    }

    /// Try to get an async tool clone (Arc) for the given name.
    /// Returns None if not found as async tool.
    pub fn get_async_tool(&self, name: &str) -> Option<Arc<dyn AsyncTool>> {
        self.async_tools.get(name).cloned()
    }

    /// Check if a sync tool exists by name.
    pub fn has_sync_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get the number of registered tools (sync + async)
    pub fn len(&self) -> usize {
        self.tools.len() + self.async_tools.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty() && self.async_tools.is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ToolCallInfo {
    pub name: String,
    pub arguments: serde_json::Value,
    pub call_id: Option<String>,
    pub result: Option<serde_json::Value>,
}
