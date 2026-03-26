use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Static configuration for an agent run (set at build time).
#[derive(Debug, Clone)]
pub struct AgentRunConfig {
    pub max_iterations: u32,
}

impl Default for AgentRunConfig {
    fn default() -> Self {
        Self { max_iterations: 10 }
    }
}

/// Dynamic context available during a single agent run (read-only snapshot per iteration).
#[derive(Debug, Clone)]
pub struct AgentRunContext {
    pub agent_id: String,
    pub iteration: u32,
    pub max_iterations: u32,
    /// Accumulates assistant text responses across iterations.
    pub responses: Vec<String>,
}

impl AgentRunContext {
    pub fn new(agent_id: String, max_iterations: u32) -> Self {
        Self {
            agent_id,
            iteration: 0,
            max_iterations,
            responses: Vec::new(),
        }
    }

    pub fn increment(&mut self) {
        self.iteration += 1;
    }

    pub fn push_response(&mut self, text: String) {
        self.responses.push(text);
    }

    pub fn last_response(&self) -> Option<&str> {
        self.responses.last().map(String::as_str)
    }
}

/// Serializable resume context — persist between HTTP requests for reactive agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeContext {
    pub agent_id: String,
    pub iteration: u32,
    pub session_snapshot: HashMap<String, Value>,
}
