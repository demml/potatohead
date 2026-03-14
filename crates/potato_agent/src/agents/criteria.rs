use crate::agents::run_context::AgentRunContext;
use std::fmt::Debug;

/// Determines whether an agent run should stop.
pub trait CompletionCriteria: Send + Sync + Debug {
    fn is_complete(&self, ctx: &AgentRunContext) -> bool;
    fn completion_reason(&self, _ctx: &AgentRunContext) -> String {
        "criteria met".into()
    }
}

/// Stop after a fixed number of iterations.
#[derive(Debug)]
pub struct MaxIterationsCriteria {
    pub max: u32,
}

impl MaxIterationsCriteria {
    pub fn new(max: u32) -> Self {
        Self { max }
    }
}

impl CompletionCriteria for MaxIterationsCriteria {
    fn is_complete(&self, ctx: &AgentRunContext) -> bool {
        ctx.iteration >= self.max
    }

    fn completion_reason(&self, _ctx: &AgentRunContext) -> String {
        format!("reached max iterations ({})", self.max)
    }
}

/// Stop when the last response contains a specific keyword.
#[derive(Debug)]
pub struct KeywordCriteria {
    pub keyword: String,
}

impl KeywordCriteria {
    pub fn new(keyword: impl Into<String>) -> Self {
        Self {
            keyword: keyword.into(),
        }
    }
}

impl CompletionCriteria for KeywordCriteria {
    fn is_complete(&self, ctx: &AgentRunContext) -> bool {
        ctx.last_response()
            .map(|r| r.contains(&self.keyword))
            .unwrap_or(false)
    }

    fn completion_reason(&self, _ctx: &AgentRunContext) -> String {
        format!("keyword '{}' found in response", self.keyword)
    }
}

/// Stop when the last response is valid JSON matching a schema.
/// (Schema validation is optional — if no validator is set, any valid JSON stops the loop.)
#[derive(Debug)]
pub struct StructuredOutputCriteria {
    /// Optional JSON schema string for validation.
    pub schema: Option<serde_json::Value>,
}

impl StructuredOutputCriteria {
    pub fn new(schema: Option<serde_json::Value>) -> Self {
        Self { schema }
    }
}

impl CompletionCriteria for StructuredOutputCriteria {
    fn is_complete(&self, ctx: &AgentRunContext) -> bool {
        let Some(last) = ctx.last_response() else {
            return false;
        };
        // Check that last response is valid JSON
        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(last) else {
            return false;
        };
        // If we have a schema, validate against it
        if let Some(schema) = &self.schema {
            return jsonschema::is_valid(schema, &parsed);
        }
        true
    }

    fn completion_reason(&self, _ctx: &AgentRunContext) -> String {
        "structured output received".into()
    }
}
