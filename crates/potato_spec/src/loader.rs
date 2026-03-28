use crate::error::SpecError;
use crate::spec::*;
use potato_agent::agents::{agent::Agent, runner::AgentRunner};
use potato_agent::{
    AgentBuilder, AgentCallback, LoggingCallback, MergeStrategy, ParallelAgent,
    ParallelAgentBuilder, SequentialAgent, SequentialAgentBuilder,
};
use potato_type::{prompt::Prompt, tools::AsyncTool, Provider};
use potato_workflow::{Task, Workflow};
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

pub(crate) fn topo_sort_tasks(tasks: &[TaskSpec]) -> Result<Vec<&TaskSpec>, SpecError> {
    let mut result: Vec<&TaskSpec> = Vec::with_capacity(tasks.len());
    let mut remaining: Vec<&TaskSpec> = tasks.iter().collect();
    let mut inserted_ids: HashSet<&str> = HashSet::new();

    while !remaining.is_empty() {
        let before = remaining.len();
        remaining.retain(|task| {
            let all_deps_inserted = task
                .dependencies
                .iter()
                .all(|dep| inserted_ids.contains(dep.as_str()));
            if all_deps_inserted {
                inserted_ids.insert(task.id.as_str());
                result.push(task);
                false
            } else {
                true
            }
        });
        if remaining.len() == before {
            let cycle_ids: Vec<_> = remaining.iter().map(|t| t.id.as_str()).collect();
            return Err(SpecError::WorkflowBuild {
                id: "unknown".into(),
                reason: format!("circular dependency or unresolvable tasks: {:?}", cycle_ids),
            });
        }
    }
    Ok(result)
}

pub struct SpecLoader {
    async_tools: HashMap<String, Arc<dyn AsyncTool>>,
    callbacks: HashMap<String, Arc<dyn AgentCallback>>,
}

impl Default for SpecLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl SpecLoader {
    pub fn new() -> Self {
        Self {
            async_tools: HashMap::new(),
            callbacks: HashMap::new(),
        }
    }

    pub fn register_async_tool(mut self, name: &str, tool: Arc<dyn AsyncTool>) -> Self {
        self.async_tools.insert(name.to_owned(), tool);
        self
    }

    pub fn register_callback(mut self, name: &str, cb: Arc<dyn AgentCallback>) -> Self {
        self.callbacks.insert(name.to_owned(), cb);
        self
    }

    /// Load from a YAML string with a default (no-registry) loader.
    pub async fn from_spec(yaml: &str) -> Result<LoadedSpec, SpecError> {
        Self::new().load_str(yaml).await
    }

    /// Load from a YAML file with a default (no-registry) loader.
    pub async fn from_spec_path(path: impl AsRef<Path>) -> Result<LoadedSpec, SpecError> {
        Self::new().load_file(path).await
    }

    pub async fn load_file(&self, path: impl AsRef<Path>) -> Result<LoadedSpec, SpecError> {
        let content = tokio::fs::read_to_string(path).await?;
        self.load_str(&content).await
    }

    pub async fn load_str(&self, yaml: &str) -> Result<LoadedSpec, SpecError> {
        let spec: PotatoSpec = serde_yaml::from_str(yaml)?;
        self.build_spec(spec).await
    }

    async fn build_spec(&self, spec: PotatoSpec) -> Result<LoadedSpec, SpecError> {
        let mut agents: HashMap<String, Arc<Agent>> = HashMap::new();
        for agent_spec in &spec.agents {
            let agent = self.build_agent(agent_spec).await?;
            agents.insert(agent_spec.id.clone(), agent);
        }

        let mut sequential: HashMap<String, Arc<SequentialAgent>> = HashMap::new();
        let mut parallel: HashMap<String, Arc<ParallelAgent>> = HashMap::new();
        let mut workflows: HashMap<String, Workflow> = HashMap::new();

        for wf_spec in &spec.workflows {
            match wf_spec {
                WorkflowSpec::Sequential {
                    id,
                    pass_output,
                    steps,
                } => {
                    let sa = self.build_sequential(*pass_output, steps, &agents).await?;
                    sequential.insert(id.clone(), sa);
                }
                WorkflowSpec::Parallel {
                    id,
                    merge_strategy,
                    steps,
                } => {
                    let pa = self.build_parallel(merge_strategy, steps, &agents).await?;
                    parallel.insert(id.clone(), pa);
                }
                WorkflowSpec::Workflow { id, tasks } => {
                    let wf = self.build_workflow(id, tasks, &agents).await?;
                    workflows.insert(id.clone(), wf);
                }
            }
        }

        Ok(LoadedSpec {
            agents,
            sequential,
            parallel,
            workflows,
        })
    }

    async fn build_agent(&self, spec: &AgentSpec) -> Result<Arc<Agent>, SpecError> {
        let provider =
            Provider::from_string(&spec.provider).map_err(|_| SpecError::InvalidProvider {
                value: spec.provider.clone(),
                reason: "unknown provider name".into(),
            })?;

        let mut builder = AgentBuilder::new().provider(provider);

        if let Some(model) = &spec.model {
            builder = builder.model(model.clone());
        }
        if let Some(sp) = &spec.system_prompt {
            builder = builder.system_prompt(sp.clone());
        }
        let has_criteria_max_iterations = spec
            .criteria
            .iter()
            .any(|c| matches!(c, CriteriaSpec::MaxIterations { .. }));

        if let Some(max) = spec.max_iterations {
            if !has_criteria_max_iterations {
                builder = builder.max_iterations(max);
            }
        }

        if let Some(mem) = &spec.memory {
            builder = match mem {
                MemorySpec::InMemory => builder.with_in_memory(),
                MemorySpec::Windowed { window_size } => builder.with_windowed_memory(*window_size),
            };
        }

        for criterion in &spec.criteria {
            builder = match criterion {
                CriteriaSpec::MaxIterations { max } => builder.max_iterations(*max),
                CriteriaSpec::Keyword { keyword } => builder.stop_on_keyword(keyword.clone()),
                CriteriaSpec::StructuredOutput { schema } => {
                    builder.stop_on_structured_output(schema.clone())
                }
            };
        }

        for cb_spec in &spec.callbacks {
            let cb: Arc<dyn AgentCallback> = match cb_spec {
                CallbackSpec::BuiltIn { kind } => match kind.as_str() {
                    "logging" => Arc::new(LoggingCallback),
                    other => return Err(SpecError::UnknownCallback { name: other.into() }),
                },
                CallbackSpec::Named { name } => self
                    .callbacks
                    .get(name)
                    .cloned()
                    .ok_or_else(|| SpecError::UnknownCallback { name: name.clone() })?,
            };
            builder = builder.with_callback(cb);
        }

        for tool_ref in &spec.tools {
            if let Some(tool) = self.async_tools.get(&tool_ref.name) {
                builder = builder.with_async_tool(Arc::clone(tool));
            } else {
                return Err(SpecError::UnknownTool {
                    name: tool_ref.name.clone(),
                });
            }
        }

        Ok(builder.build().await?)
    }

    async fn build_sequential(
        &self,
        pass_output: Option<bool>,
        steps: &[StepSpec],
        agents: &HashMap<String, Arc<Agent>>,
    ) -> Result<Arc<SequentialAgent>, SpecError> {
        let mut sb = SequentialAgentBuilder::new().pass_output(pass_output.unwrap_or(false));
        for step in steps {
            let runner = self.resolve_step(step, agents).await?;
            sb = sb.then(runner);
        }
        Ok(sb.build())
    }

    async fn build_parallel(
        &self,
        merge_strategy: &Option<MergeStrategySpec>,
        steps: &[StepSpec],
        agents: &HashMap<String, Arc<Agent>>,
    ) -> Result<Arc<ParallelAgent>, SpecError> {
        let strategy = match merge_strategy {
            None | Some(MergeStrategySpec::CollectAll) => MergeStrategy::CollectAll,
            Some(MergeStrategySpec::First) => MergeStrategy::First,
        };
        let mut pb = ParallelAgentBuilder::new().merge_strategy(strategy);
        for step in steps {
            let runner = self.resolve_step(step, agents).await?;
            pb = pb.with_agent(runner);
        }
        Ok(pb.build())
    }

    async fn build_workflow(
        &self,
        name: &str,
        tasks: &[TaskSpec],
        agents: &HashMap<String, Arc<Agent>>,
    ) -> Result<Workflow, SpecError> {
        let mut wf = Workflow::new(name);
        let sorted = topo_sort_tasks(tasks)?;

        for task_spec in sorted {
            let agent = agents
                .get(&task_spec.agent)
                .ok_or_else(|| SpecError::UnknownAgentRef {
                    id: task_spec.agent.clone(),
                })?;

            let provider = agent.provider.clone();
            let model = agent
                .model_override
                .clone()
                .ok_or_else(|| SpecError::WorkflowBuild {
                    id: task_spec.id.clone(),
                    reason: format!(
                        "agent '{}' used in task '{}' has no model set",
                        task_spec.agent, task_spec.id
                    ),
                })?;

            let prompt = match &task_spec.prompt {
                PromptRef::Inline(text) => {
                    let config_value = serde_json::json!({
                        "model": model,
                        "provider": provider.as_str(),
                        "messages": [text],
                    });
                    let prompt_config = serde_json::from_value(config_value).map_err(|e| {
                        SpecError::WorkflowBuild {
                            id: task_spec.id.clone(),
                            reason: e.to_string(),
                        }
                    })?;
                    Prompt::from_generic_config(prompt_config).map_err(|e| {
                        SpecError::WorkflowBuild {
                            id: task_spec.id.clone(),
                            reason: e.to_string(),
                        }
                    })?
                }
                PromptRef::File(path) => {
                    if Path::new(path)
                        .components()
                        .any(|c| c == Component::ParentDir)
                    {
                        return Err(SpecError::PromptLoad {
                            path: path.clone(),
                            reason: "path must not contain '..' components".into(),
                        });
                    }
                    let path_owned = path.clone();
                    let task_id = task_spec.id.clone();
                    tokio::task::spawn_blocking(move || {
                        Prompt::from_path(PathBuf::from(&path_owned)).map_err(|e| {
                            SpecError::PromptLoad {
                                path: path_owned,
                                reason: e.to_string(),
                            }
                        })
                    })
                    .await
                    .map_err(|e| SpecError::WorkflowBuild {
                        id: task_id,
                        reason: format!("spawn_blocking failed: {e}"),
                    })??
                }
            };

            let task = Task::new(
                &agent.id,
                prompt,
                &task_spec.id,
                Some(task_spec.dependencies.clone()),
                task_spec.max_retries,
            )
            .map_err(SpecError::AgentBuild)?;

            wf.add_agent(agent);
            wf.add_task(task).map_err(|e| SpecError::WorkflowBuild {
                id: task_spec.id.clone(),
                reason: e.to_string(),
            })?;
        }

        Ok(wf)
    }

    async fn resolve_step(
        &self,
        step: &StepSpec,
        agents: &HashMap<String, Arc<Agent>>,
    ) -> Result<Arc<dyn AgentRunner>, SpecError> {
        match step {
            StepSpec::Ref { agent_ref } => agents
                .get(agent_ref)
                .map(|a| Arc::clone(a) as Arc<dyn AgentRunner>)
                .ok_or_else(|| SpecError::UnknownAgentRef {
                    id: agent_ref.clone(),
                }),
            StepSpec::Inline(agent_spec) => {
                let agent = self.build_agent(agent_spec).await?;
                Ok(agent as Arc<dyn AgentRunner>)
            }
        }
    }
}

pub struct LoadedSpec {
    agents: HashMap<String, Arc<Agent>>,
    sequential: HashMap<String, Arc<SequentialAgent>>,
    parallel: HashMap<String, Arc<ParallelAgent>>,
    workflows: HashMap<String, Workflow>,
}

impl LoadedSpec {
    pub fn agent(&self, id: &str) -> Option<Arc<Agent>> {
        self.agents.get(id).cloned()
    }

    pub fn sequential(&self, id: &str) -> Option<Arc<SequentialAgent>> {
        self.sequential.get(id).cloned()
    }

    pub fn parallel(&self, id: &str) -> Option<Arc<ParallelAgent>> {
        self.parallel.get(id).cloned()
    }

    pub fn workflow(&self, id: &str) -> Option<&Workflow> {
        self.workflows.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(id: &str, deps: Vec<&str>) -> TaskSpec {
        TaskSpec {
            id: id.to_string(),
            agent: "x".to_string(),
            prompt: PromptRef::Inline("p".to_string()),
            dependencies: deps.into_iter().map(|s| s.to_string()).collect(),
            max_retries: None,
        }
    }

    #[test]
    fn test_topo_sort_out_of_order() {
        let tasks = vec![make_task("t2", vec!["t1"]), make_task("t1", vec![])];
        let sorted = topo_sort_tasks(&tasks).unwrap();
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].id, "t1");
        assert_eq!(sorted[1].id, "t2");
    }

    #[test]
    fn test_topo_sort_cycle_returns_error() {
        let tasks = vec![make_task("a", vec!["b"]), make_task("b", vec!["a"])];
        let result = topo_sort_tasks(&tasks);
        assert!(result.is_err());
        match result.unwrap_err() {
            SpecError::WorkflowBuild { reason, .. } => {
                assert!(reason.contains("circular dependency"));
                assert!(reason.contains("a") || reason.contains("b"));
            }
            other => panic!("expected WorkflowBuild, got {:?}", other),
        }
    }
}
