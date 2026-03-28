use potato_agent::AgentError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpecError {
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("IO error reading spec file: {0}")]
    Io(#[from] std::io::Error),
    #[error("unknown tool '{name}' referenced in spec — register it before loading")]
    UnknownTool { name: String },
    #[error("unknown callback '{name}' referenced in spec — register it before loading")]
    UnknownCallback { name: String },
    #[error("unknown agent ref '{id}' — define it in the agents section or inline")]
    UnknownAgentRef { id: String },
    #[error("agent build error: {0}")]
    AgentBuild(#[from] AgentError),
    #[error("invalid provider '{value}': {reason}")]
    InvalidProvider { value: String, reason: String },
    #[error("workflow build error for '{id}': {reason}")]
    WorkflowBuild { id: String, reason: String },
    #[error("failed to load prompt from '{path}': {reason}")]
    PromptLoad { path: String, reason: String },
}
