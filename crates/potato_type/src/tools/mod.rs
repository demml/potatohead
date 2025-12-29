use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub arguments: serde_json::Value,
}

#[derive(Debug)]
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool + Send + Sync>>,
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
        }
    }

    pub fn register_tool(&mut self, tool: Box<dyn Tool + Send + Sync>) {
        self.tools.insert(tool.name().to_string(), tool);
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

    /// Get the number of registered tools
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
