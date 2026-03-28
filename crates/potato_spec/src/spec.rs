use serde::Deserialize;

#[derive(Debug, Clone)]
pub enum PromptRef {
    Inline(String),
    File(String),
}

impl<'de> Deserialize<'de> for PromptRef {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::{self, Visitor};
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = PromptRef;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "a string or a map with a 'path' key")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(PromptRef::Inline(v.to_owned()))
            }
            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut path: Option<String> = None;
                while let Some(key) = map.next_key::<String>()? {
                    if key == "path" {
                        path = Some(map.next_value()?);
                    } else {
                        map.next_value::<serde::de::IgnoredAny>()?;
                    }
                }
                path.map(PromptRef::File)
                    .ok_or_else(|| de::Error::missing_field("path"))
            }
        }
        d.deserialize_any(V)
    }
}

#[derive(Debug, Deserialize)]
pub struct PotatoSpec {
    #[serde(default)]
    pub agents: Vec<AgentSpec>,
    #[serde(default)]
    pub workflows: Vec<WorkflowSpec>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AgentSpec {
    pub id: String,
    pub provider: String,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub max_iterations: Option<u32>,
    pub memory: Option<MemorySpec>,
    #[serde(default)]
    pub criteria: Vec<CriteriaSpec>,
    #[serde(default)]
    pub callbacks: Vec<CallbackSpec>,
    #[serde(default)]
    pub tools: Vec<ToolRef>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MemorySpec {
    InMemory,
    Windowed { window_size: usize },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CriteriaSpec {
    MaxIterations { max: u32 },
    Keyword { keyword: String },
    StructuredOutput { schema: Option<serde_json::Value> },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum CallbackSpec {
    BuiltIn {
        #[serde(rename = "type")]
        kind: String,
    },
    Named {
        name: String,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub struct ToolRef {
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkflowSpec {
    Sequential {
        id: String,
        pass_output: Option<bool>,
        steps: Vec<StepSpec>,
    },
    Parallel {
        id: String,
        merge_strategy: Option<MergeStrategySpec>,
        steps: Vec<StepSpec>,
    },
    Workflow {
        id: String,
        tasks: Vec<TaskSpec>,
    },
}

impl WorkflowSpec {
    pub fn id(&self) -> &str {
        match self {
            Self::Sequential { id, .. } => id,
            Self::Parallel { id, .. } => id,
            Self::Workflow { id, .. } => id,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum StepSpec {
    Ref {
        #[serde(rename = "ref")]
        agent_ref: String,
    },
    Inline(AgentSpec),
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum MergeStrategySpec {
    #[default]
    CollectAll,
    First,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TaskSpec {
    pub id: String,
    pub agent: String,
    pub prompt: PromptRef,
    #[serde(default)]
    pub dependencies: Vec<String>,
    pub max_retries: Option<u32>,
}
