use crate::error::TypeError;
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use pythonize::{depythonize, pythonize};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const SPEC_FILE_EXTENSIONS: [&str; 3] = ["yaml", "yml", "json"];
const PROMPT_RUNTIME_FIELDS: [&str; 9] = [
    "max_iterations",
    "retry_policy",
    "timeout",
    "memory",
    "session_store",
    "callbacks",
    "concurrency",
    "workflow",
    "run_config",
];
const AGENT_RUNTIME_FIELDS: [&str; 12] = [
    "max_iterations",
    "retry_policy",
    "timeout",
    "memory",
    "session_store",
    "callbacks",
    "concurrency",
    "workflow",
    "run_config",
    "max_retries",
    "merge_strategy",
    "pass_output",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InstructionBlock {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VariableSpec {
    pub name: String,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub default: Option<Value>,
    #[serde(default)]
    pub schema: Option<Value>,
}

fn default_true() -> bool {
    true
}

#[pyclass(from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PromptSpec {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub version: String,
    #[pyo3(get)]
    pub title: Option<String>,
    #[pyo3(get)]
    pub description: Option<String>,
    #[serde(default)]
    pub instructions: Vec<InstructionBlock>,
    #[serde(default)]
    pub variables: Vec<VariableSpec>,
    #[serde(default)]
    pub input_schema: Option<Value>,
    #[serde(default)]
    pub output_schema: Option<Value>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
    #[serde(default)]
    pub extensions: BTreeMap<String, Value>,
}

#[pyclass(from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AgentSpec {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub version: String,
    #[pyo3(get)]
    pub name: Option<String>,
    #[pyo3(get)]
    pub description: Option<String>,
    #[serde(default)]
    pub primary_prompt: Option<String>,
    #[serde(default)]
    pub prompt_refs: Vec<String>,
    #[serde(default)]
    pub input_schema: Option<Value>,
    #[serde(default)]
    pub output_schema: Option<Value>,
    #[serde(default)]
    pub tool_refs: Vec<String>,
    #[serde(default)]
    pub agent_refs: Vec<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub provider_hints: Vec<String>,
    #[serde(default)]
    pub framework_hints: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
    #[serde(default)]
    pub extensions: BTreeMap<String, Value>,
}

#[pyclass(from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PortableSpec {
    #[pyo3(get)]
    pub version: String,
    #[serde(default)]
    pub prompts: Vec<PromptSpec>,
    #[serde(default)]
    pub agents: Vec<AgentSpec>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
    #[serde(default)]
    pub extensions: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct FrameworkExport {
    framework: String,
    agent_id: String,
    config: Value,
    losses: Vec<String>,
}

fn parse_optional_vec<T>(value: Option<&Bound<'_, PyAny>>) -> Result<Vec<T>, TypeError>
where
    T: for<'de> Deserialize<'de>,
{
    match value {
        Some(v) => depythonize(v).map_err(Into::into),
        None => Ok(Vec::new()),
    }
}

fn parse_optional_json(
    value: Option<&Bound<'_, PyAny>>,
    field_name: &str,
) -> Result<Option<Value>, TypeError> {
    match value {
        Some(v) => Ok(Some(depythonize(v).map_err(|e| {
            TypeError::Error(format!("Failed to parse '{field_name}': {e}"))
        })?)),
        None => Ok(None),
    }
}

fn parse_optional_map(
    value: Option<&Bound<'_, PyAny>>,
    field_name: &str,
) -> Result<BTreeMap<String, Value>, TypeError> {
    let json = parse_optional_json(value, field_name)?;
    match json {
        Some(Value::Object(obj)) => Ok(obj.into_iter().collect()),
        Some(_) => Err(TypeError::Error(format!(
            "Field '{field_name}' must be a dictionary/object"
        ))),
        None => Ok(BTreeMap::new()),
    }
}

fn contains_runtime_fields(value: &Value, banned_fields: &[&str]) -> Option<String> {
    let obj = value.as_object()?;
    banned_fields
        .iter()
        .find(|field| obj.contains_key(**field))
        .map(|field| (*field).to_string())
}

fn validate_runtime_fields_in_prompt(value: &Value) -> Result<(), TypeError> {
    if let Some(field) = contains_runtime_fields(value, &PROMPT_RUNTIME_FIELDS) {
        return Err(TypeError::Error(format!(
            "Prompt spec contains runtime-only field '{field}'. Move runtime behavior to framework/runtime configuration."
        )));
    }
    Ok(())
}

fn validate_runtime_fields_in_agent(value: &Value) -> Result<(), TypeError> {
    if let Some(field) = contains_runtime_fields(value, &AGENT_RUNTIME_FIELDS) {
        return Err(TypeError::Error(format!(
            "Agent spec contains runtime-only field '{field}'. Move runtime behavior to framework/runtime configuration."
        )));
    }
    Ok(())
}

fn validate_runtime_fields_in_value(value: &Value) -> Result<(), TypeError> {
    let Some(obj) = value.as_object() else {
        return Ok(());
    };

    if let Some(prompts) = obj.get("prompts") {
        if let Some(list) = prompts.as_array() {
            for prompt in list {
                validate_runtime_fields_in_prompt(prompt)?;
            }
        }
    }

    if let Some(agents) = obj.get("agents") {
        if let Some(list) = agents.as_array() {
            for agent in list {
                validate_runtime_fields_in_agent(agent)?;
            }
        }
    }

    validate_runtime_fields_in_prompt(value)?;
    validate_runtime_fields_in_agent(value)?;
    Ok(())
}

fn merge_extension_fields(
    config: &mut Map<String, Value>,
    extensions: &BTreeMap<String, Value>,
    extension_key: &str,
) {
    if let Some(Value::Object(extension_fields)) = extensions.get(extension_key) {
        for (key, value) in extension_fields {
            if !config.contains_key(key) {
                config.insert(key.clone(), value.clone());
            }
        }
    }
}

fn read_value_from_path(path: &Path) -> Result<Value, TypeError> {
    let content = fs::read_to_string(path)?;
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| TypeError::Error(format!("Invalid spec path: {}", path.display())))?;

    let value: Value = match extension.to_lowercase().as_str() {
        "json" => serde_json::from_str(&content)?,
        "yaml" | "yml" => serde_yaml::from_str(&content)?,
        _ => {
            return Err(TypeError::Error(format!(
                "Unsupported file extension '{extension}'. Expected one of: {}",
                SPEC_FILE_EXTENSIONS.join(", ")
            )))
        }
    };

    validate_runtime_fields_in_value(&value)?;
    Ok(value)
}

fn choose_prompt_from_portable(
    portable: PortableSpec,
    prompt_id: Option<&str>,
) -> Result<PromptSpec, TypeError> {
    if let Some(id) = prompt_id {
        portable
            .prompts
            .into_iter()
            .find(|prompt| prompt.id == id)
            .ok_or_else(|| TypeError::Error(format!("Prompt '{id}' not found in spec file")))
    } else if portable.prompts.len() == 1 {
        portable.prompts.into_iter().next().ok_or_else(|| {
            TypeError::Error("Expected one prompt in spec file but found none".to_string())
        })
    } else {
        Err(TypeError::Error(format!(
            "Spec file contains {} prompts. Provide prompt_id to disambiguate.",
            portable.prompts.len()
        )))
    }
}

fn choose_agent_from_portable(
    portable: PortableSpec,
    agent_id: Option<&str>,
) -> Result<AgentSpec, TypeError> {
    if let Some(id) = agent_id {
        portable
            .agents
            .into_iter()
            .find(|agent| agent.id == id)
            .ok_or_else(|| TypeError::Error(format!("Agent '{id}' not found in spec file")))
    } else if portable.agents.len() == 1 {
        portable.agents.into_iter().next().ok_or_else(|| {
            TypeError::Error("Expected one agent in spec file but found none".to_string())
        })
    } else {
        Err(TypeError::Error(format!(
            "Spec file contains {} agents. Provide agent_id to disambiguate.",
            portable.agents.len()
        )))
    }
}

fn parse_prompt_value(value: Value, prompt_id: Option<&str>) -> Result<PromptSpec, TypeError> {
    if value
        .as_object()
        .is_some_and(|obj| obj.contains_key("prompts") || obj.contains_key("agents"))
    {
        let portable: PortableSpec = serde_json::from_value(value)?;
        return choose_prompt_from_portable(portable, prompt_id);
    }

    Ok(serde_json::from_value(value)?)
}

fn parse_agent_value(value: Value, agent_id: Option<&str>) -> Result<AgentSpec, TypeError> {
    if value
        .as_object()
        .is_some_and(|obj| obj.contains_key("prompts") || obj.contains_key("agents"))
    {
        let portable: PortableSpec = serde_json::from_value(value)?;
        return choose_agent_from_portable(portable, agent_id);
    }

    Ok(serde_json::from_value(value)?)
}

fn instruction_text(instruction: &InstructionBlock) -> String {
    instruction.content.trim().to_string()
}

fn merge_prompt_and_agent_instructions(agent: &AgentSpec, prompt: Option<&PromptSpec>) -> String {
    let mut blocks = Vec::new();

    if let Some(prompt) = prompt {
        blocks.extend(
            prompt
                .instructions
                .iter()
                .map(instruction_text)
                .filter(|text| !text.is_empty()),
        );
    }

    if let Some(description) = &agent.description {
        let trimmed = description.trim();
        if !trimmed.is_empty() {
            blocks.push(trimmed.to_string());
        }
    }

    blocks.join("\n\n")
}

impl PortableSpec {
    fn find_prompt(&self, prompt_id: &str) -> Option<&PromptSpec> {
        self.prompts.iter().find(|prompt| prompt.id == prompt_id)
    }

    fn find_agent(&self, agent_id: &str) -> Result<&AgentSpec, TypeError> {
        self.agents
            .iter()
            .find(|agent| agent.id == agent_id)
            .ok_or_else(|| TypeError::Error(format!("Agent '{agent_id}' not found")))
    }

    fn selected_prompt_for_agent<'a>(
        &'a self,
        agent: &'a AgentSpec,
    ) -> Result<(Option<&'a PromptSpec>, Vec<String>), TypeError> {
        let mut losses = Vec::new();
        let prompt_ref = agent
            .primary_prompt
            .as_ref()
            .or_else(|| agent.prompt_refs.first());

        if let Some(prompt_ref) = prompt_ref {
            let prompt = self.find_prompt(prompt_ref);
            if prompt.is_none() {
                losses.push(format!(
                    "Prompt reference '{prompt_ref}' was not found; exported instructions may be incomplete."
                ));
            }
            return Ok((prompt, losses));
        }

        if self.prompts.is_empty() {
            Ok((None, losses))
        } else if self.prompts.len() == 1 {
            Ok((self.prompts.first(), losses))
        } else {
            losses.push(
                "Agent has no prompt reference and spec contains multiple prompts; no prompt body was selected."
                    .to_string(),
            );
            Ok((None, losses))
        }
    }

    fn openai_export(&self, agent_id: &str) -> Result<FrameworkExport, TypeError> {
        let agent = self.find_agent(agent_id)?;
        let (prompt, mut losses) = self.selected_prompt_for_agent(agent)?;

        let mut config = Map::new();
        config.insert(
            "name".to_string(),
            Value::String(agent.name.clone().unwrap_or_else(|| agent.id.clone())),
        );
        config.insert(
            "instructions".to_string(),
            Value::String(merge_prompt_and_agent_instructions(agent, prompt)),
        );
        config.insert(
            "tools".to_string(),
            Value::Array(
                agent
                    .tool_refs
                    .iter()
                    .map(|name| {
                        let mut tool = Map::new();
                        tool.insert("name".to_string(), Value::String(name.clone()));
                        Value::Object(tool)
                    })
                    .collect(),
            ),
        );

        if let Some(schema) = agent
            .output_schema
            .clone()
            .or_else(|| prompt.and_then(|p| p.output_schema.clone()))
        {
            config.insert("output_schema".to_string(), schema);
        } else {
            losses.push("No output schema available for OpenAI export.".to_string());
        }

        if !agent.agent_refs.is_empty() {
            config.insert(
                "handoffs".to_string(),
                Value::Array(
                    agent
                        .agent_refs
                        .iter()
                        .cloned()
                        .map(Value::String)
                        .collect(),
                ),
            );
        }

        merge_extension_fields(&mut config, &agent.extensions, "openai_agents");

        Ok(FrameworkExport {
            framework: "openai_agents".to_string(),
            agent_id: agent.id.clone(),
            config: Value::Object(config),
            losses,
        })
    }

    fn crewai_export(&self, agent_id: &str) -> Result<FrameworkExport, TypeError> {
        let agent = self.find_agent(agent_id)?;
        let (prompt, mut losses) = self.selected_prompt_for_agent(agent)?;
        let merged_instructions = merge_prompt_and_agent_instructions(agent, prompt);

        let mut config = Map::new();
        config.insert(
            "id".to_string(),
            Value::String(agent.name.clone().unwrap_or_else(|| agent.id.clone())),
        );
        config.insert(
            "role".to_string(),
            Value::String(agent.name.clone().unwrap_or_else(|| "Agent".to_string())),
        );
        config.insert(
            "goal".to_string(),
            Value::String(
                agent
                    .description
                    .clone()
                    .unwrap_or_else(|| "Execute assigned tasks".to_string()),
            ),
        );
        config.insert("backstory".to_string(), Value::String(merged_instructions));
        config.insert(
            "tools".to_string(),
            Value::Array(agent.tool_refs.iter().cloned().map(Value::String).collect()),
        );

        if let Some(schema) = agent
            .output_schema
            .clone()
            .or_else(|| prompt.and_then(|p| p.output_schema.clone()))
        {
            config.insert("expected_output_schema".to_string(), schema);
        } else {
            losses.push("No output schema available for CrewAI export.".to_string());
        }

        merge_extension_fields(&mut config, &agent.extensions, "crewai");

        Ok(FrameworkExport {
            framework: "crewai".to_string(),
            agent_id: agent.id.clone(),
            config: Value::Object(config),
            losses,
        })
    }

    fn google_adk_export(&self, agent_id: &str) -> Result<FrameworkExport, TypeError> {
        let agent = self.find_agent(agent_id)?;
        let (prompt, mut losses) = self.selected_prompt_for_agent(agent)?;

        let mut config = Map::new();
        config.insert(
            "name".to_string(),
            Value::String(agent.name.clone().unwrap_or_else(|| agent.id.clone())),
        );
        config.insert(
            "description".to_string(),
            Value::String(agent.description.clone().unwrap_or_default()),
        );
        config.insert(
            "instruction".to_string(),
            Value::String(merge_prompt_and_agent_instructions(agent, prompt)),
        );
        config.insert(
            "tools".to_string(),
            Value::Array(
                agent
                    .tool_refs
                    .iter()
                    .map(|name| {
                        let mut tool = Map::new();
                        tool.insert("name".to_string(), Value::String(name.clone()));
                        Value::Object(tool)
                    })
                    .collect(),
            ),
        );

        if !agent.agent_refs.is_empty() {
            config.insert(
                "sub_agents".to_string(),
                Value::Array(
                    agent
                        .agent_refs
                        .iter()
                        .cloned()
                        .map(Value::String)
                        .collect(),
                ),
            );
        }

        if let Some(model_hint) = agent.provider_hints.first() {
            config.insert("model".to_string(), Value::String(model_hint.clone()));
        } else {
            losses.push("No provider_hints model set for Google ADK export.".to_string());
        }

        if let Some(schema) = agent
            .output_schema
            .clone()
            .or_else(|| prompt.and_then(|p| p.output_schema.clone()))
        {
            config.insert("output_schema".to_string(), schema);
        }

        merge_extension_fields(&mut config, &agent.extensions, "google_adk");

        Ok(FrameworkExport {
            framework: "google_adk".to_string(),
            agent_id: agent.id.clone(),
            config: Value::Object(config),
            losses,
        })
    }
}

#[pymethods]
impl PromptSpec {
    #[new]
    #[pyo3(signature = (id, version, title=None, description=None, instructions=None, variables=None, input_schema=None, output_schema=None, tags=None, metadata=None, extensions=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        version: String,
        title: Option<String>,
        description: Option<String>,
        instructions: Option<&Bound<'_, PyAny>>,
        variables: Option<&Bound<'_, PyAny>>,
        input_schema: Option<&Bound<'_, PyAny>>,
        output_schema: Option<&Bound<'_, PyAny>>,
        tags: Option<&Bound<'_, PyAny>>,
        metadata: Option<&Bound<'_, PyAny>>,
        extensions: Option<&Bound<'_, PyAny>>,
    ) -> Result<Self, TypeError> {
        Ok(Self {
            id,
            version,
            title,
            description,
            instructions: parse_optional_vec(instructions)?,
            variables: parse_optional_vec(variables)?,
            input_schema: parse_optional_json(input_schema, "input_schema")?,
            output_schema: parse_optional_json(output_schema, "output_schema")?,
            tags: parse_optional_vec(tags)?,
            metadata: parse_optional_map(metadata, "metadata")?,
            extensions: parse_optional_map(extensions, "extensions")?,
        })
    }

    #[staticmethod]
    #[pyo3(signature = (path, prompt_id=None))]
    pub fn from_path(path: PathBuf, prompt_id: Option<String>) -> Result<Self, TypeError> {
        let value = read_value_from_path(path.as_path())?;
        parse_prompt_value(value, prompt_id.as_deref())
    }

    #[staticmethod]
    pub fn model_validate_json(json_string: String) -> Result<Self, TypeError> {
        let value: Value = serde_json::from_str(&json_string)?;
        validate_runtime_fields_in_value(&value)?;
        parse_prompt_value(value, None)
    }

    pub fn model_dump<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        let value = serde_json::to_value(self)?;
        Ok(pythonize(py, &value)?)
    }

    pub fn model_dump_json(&self) -> Result<String, TypeError> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

#[pymethods]
impl AgentSpec {
    #[new]
    #[pyo3(signature = (id, version, name=None, description=None, primary_prompt=None, prompt_refs=None, input_schema=None, output_schema=None, tool_refs=None, agent_refs=None, capabilities=None, provider_hints=None, framework_hints=None, tags=None, metadata=None, extensions=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        version: String,
        name: Option<String>,
        description: Option<String>,
        primary_prompt: Option<String>,
        prompt_refs: Option<&Bound<'_, PyAny>>,
        input_schema: Option<&Bound<'_, PyAny>>,
        output_schema: Option<&Bound<'_, PyAny>>,
        tool_refs: Option<&Bound<'_, PyAny>>,
        agent_refs: Option<&Bound<'_, PyAny>>,
        capabilities: Option<&Bound<'_, PyAny>>,
        provider_hints: Option<&Bound<'_, PyAny>>,
        framework_hints: Option<&Bound<'_, PyAny>>,
        tags: Option<&Bound<'_, PyAny>>,
        metadata: Option<&Bound<'_, PyAny>>,
        extensions: Option<&Bound<'_, PyAny>>,
    ) -> Result<Self, TypeError> {
        let spec = Self {
            id,
            version,
            name,
            description,
            primary_prompt,
            prompt_refs: parse_optional_vec(prompt_refs)?,
            input_schema: parse_optional_json(input_schema, "input_schema")?,
            output_schema: parse_optional_json(output_schema, "output_schema")?,
            tool_refs: parse_optional_vec(tool_refs)?,
            agent_refs: parse_optional_vec(agent_refs)?,
            capabilities: parse_optional_vec(capabilities)?,
            provider_hints: parse_optional_vec(provider_hints)?,
            framework_hints: parse_optional_vec(framework_hints)?,
            tags: parse_optional_vec(tags)?,
            metadata: parse_optional_map(metadata, "metadata")?,
            extensions: parse_optional_map(extensions, "extensions")?,
        };
        validate_runtime_fields_in_agent(&serde_json::to_value(&spec)?)?;
        Ok(spec)
    }

    #[staticmethod]
    #[pyo3(signature = (path, agent_id=None))]
    pub fn from_path(path: PathBuf, agent_id: Option<String>) -> Result<Self, TypeError> {
        let value = read_value_from_path(path.as_path())?;
        parse_agent_value(value, agent_id.as_deref())
    }

    #[staticmethod]
    pub fn model_validate_json(json_string: String) -> Result<Self, TypeError> {
        let value: Value = serde_json::from_str(&json_string)?;
        validate_runtime_fields_in_value(&value)?;
        parse_agent_value(value, None)
    }

    pub fn model_dump<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        let value = serde_json::to_value(self)?;
        Ok(pythonize(py, &value)?)
    }

    pub fn model_dump_json(&self) -> Result<String, TypeError> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

#[pymethods]
impl PortableSpec {
    #[new]
    #[pyo3(signature = (version, prompts=None, agents=None, metadata=None, extensions=None))]
    pub fn new(
        version: String,
        prompts: Option<&Bound<'_, PyAny>>,
        agents: Option<&Bound<'_, PyAny>>,
        metadata: Option<&Bound<'_, PyAny>>,
        extensions: Option<&Bound<'_, PyAny>>,
    ) -> Result<Self, TypeError> {
        let spec = Self {
            version,
            prompts: parse_optional_vec(prompts)?,
            agents: parse_optional_vec(agents)?,
            metadata: parse_optional_map(metadata, "metadata")?,
            extensions: parse_optional_map(extensions, "extensions")?,
        };
        validate_runtime_fields_in_value(&serde_json::to_value(&spec)?)?;
        Ok(spec)
    }

    #[staticmethod]
    pub fn from_path(path: PathBuf) -> Result<Self, TypeError> {
        let value = read_value_from_path(path.as_path())?;
        Ok(serde_json::from_value(value)?)
    }

    #[staticmethod]
    pub fn model_validate_json(json_string: String) -> Result<Self, TypeError> {
        let value: Value = serde_json::from_str(&json_string)?;
        validate_runtime_fields_in_value(&value)?;
        Ok(serde_json::from_value(value)?)
    }

    pub fn model_dump<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        let value = serde_json::to_value(self)?;
        Ok(pythonize(py, &value)?)
    }

    pub fn model_dump_json(&self) -> Result<String, TypeError> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn prompt(&self, prompt_id: String) -> Result<PromptSpec, TypeError> {
        self.find_prompt(&prompt_id)
            .cloned()
            .ok_or_else(|| TypeError::Error(format!("Prompt '{prompt_id}' not found")))
    }

    pub fn agent(&self, agent_id: String) -> Result<AgentSpec, TypeError> {
        self.find_agent(&agent_id).cloned()
    }

    pub fn to_openai_agent_config<'py>(
        &self,
        py: Python<'py>,
        agent_id: String,
    ) -> Result<Bound<'py, PyAny>, TypeError> {
        let export = self.openai_export(&agent_id)?;
        Ok(pythonize(py, &serde_json::to_value(export)?)?)
    }

    pub fn to_crewai_agent_config<'py>(
        &self,
        py: Python<'py>,
        agent_id: String,
    ) -> Result<Bound<'py, PyAny>, TypeError> {
        let export = self.crewai_export(&agent_id)?;
        Ok(pythonize(py, &serde_json::to_value(export)?)?)
    }

    pub fn to_google_adk_agent_config<'py>(
        &self,
        py: Python<'py>,
        agent_id: String,
    ) -> Result<Bound<'py, PyAny>, TypeError> {
        let export = self.google_adk_export(&agent_id)?;
        Ok(pythonize(py, &serde_json::to_value(export)?)?)
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_spec_deserializes_and_exports_openai() {
        let yaml = r#"
version: "1.0"
prompts:
  - id: support_prompt
    version: "1.0.0"
    instructions:
      - role: system
        content: You classify support tickets.
    output_schema:
      type: object
      properties:
        category:
          type: string
agents:
  - id: support_agent
    version: "1.0.0"
    name: Support Agent
    description: Routes support tickets to the right queue.
    primary_prompt: support_prompt
    tool_refs: ["ticket_lookup"]
    agent_refs: ["escalation_agent"]
    extensions:
      openai_agents:
        model: gpt-4o-mini
"#;

        let value: Value = serde_yaml::from_str(yaml).unwrap();
        let spec: PortableSpec = serde_json::from_value(value).unwrap();
        let export = spec.openai_export("support_agent").unwrap();

        assert_eq!(export.framework, "openai_agents");
        assert_eq!(export.agent_id, "support_agent");
        assert_eq!(
            export.config["name"],
            Value::String("Support Agent".to_string())
        );
        assert_eq!(
            export.config["model"],
            Value::String("gpt-4o-mini".to_string())
        );
        assert_eq!(
            export.config["handoffs"],
            Value::Array(vec![Value::String("escalation_agent".to_string())])
        );
    }

    #[test]
    fn runtime_field_is_rejected_in_agent() {
        let yaml = r#"
version: "1.0"
agents:
  - id: support_agent
    version: "1.0.0"
    max_iterations: 4
"#;

        let value: Value = serde_yaml::from_str(yaml).unwrap();
        let result = PortableSpec::model_validate_json(serde_json::to_string(&value).unwrap());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("runtime-only field 'max_iterations'"));
    }

    #[test]
    fn extension_does_not_override_canonical_core_fields() {
        let yaml = r#"
version: "1.0"
prompts:
  - id: support_prompt
    version: "1.0.0"
    instructions:
      - role: system
        content: You classify support tickets.
agents:
  - id: support_agent
    version: "1.0.0"
    name: Canonical Name
    primary_prompt: support_prompt
    extensions:
      openai_agents:
        name: Wrong Override Name
        model: gpt-4o-mini
"#;

        let value: Value = serde_yaml::from_str(yaml).unwrap();
        let spec: PortableSpec = serde_json::from_value(value).unwrap();
        let export = spec.openai_export("support_agent").unwrap();

        assert_eq!(
            export.config["name"],
            Value::String("Canonical Name".to_string())
        );
        assert_eq!(
            export.config["model"],
            Value::String("gpt-4o-mini".to_string())
        );
    }
}
