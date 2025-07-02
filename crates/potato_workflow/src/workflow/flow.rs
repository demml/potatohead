use crate::workflow::error::WorkflowError;
pub use potato_agent::agents::{
    agent::Agent,
    task::{PyTask, Task, TaskStatus},
    types::ChatResponse,
};
use potato_agent::{AgentResponse, PyAgentResponse};
use potato_util::{create_uuid7, PyHelperFuncs};

use potato_prompt::prompt::types::Role;
use potato_prompt::Message;
use pyo3::prelude::*;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::RwLock;
use tracing::instrument;
use tracing::{debug, error, info, warn};

use serde::{
    de::{self, MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};

#[derive(Debug)]
#[pyclass]
pub struct WorkflowResult {
    #[pyo3(get)]
    pub tasks: HashMap<String, Py<PyTask>>,
}

impl WorkflowResult {
    pub fn new(
        py: Python,
        tasks: HashMap<String, Task>,
        output_types: HashMap<String, Arc<PyObject>>,
    ) -> Self {
        let py_tasks = tasks
            .into_iter()
            .map(|(id, task)| {
                let py_agent_response = if let Some(result) = task.result {
                    let output_type = output_types.get(&id).map(|arc| arc.as_ref().clone_ref(py));
                    Some(PyAgentResponse::new(result, output_type))
                } else {
                    None
                };
                let py_task = PyTask {
                    id: task.id.clone(),
                    prompt: task.prompt,
                    dependencies: task.dependencies,
                    status: task.status,
                    agent_id: task.agent_id,
                    result: py_agent_response,
                    max_retries: task.max_retries,
                    retry_count: task.retry_count,
                };
                (id, Py::new(py, py_task).unwrap())
            })
            .collect::<HashMap<_, _>>();

        Self { tasks: py_tasks }
    }
}

#[pymethods]
impl WorkflowResult {
    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(&self.tasks)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[pyclass]
pub struct TaskList {
    #[pyo3(get)]
    pub tasks: HashMap<String, Task>,
    pub execution_order: Vec<String>,
}

impl TaskList {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            execution_order: Vec::new(),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.tasks
            .values()
            .all(|task| task.status == TaskStatus::Completed || task.status == TaskStatus::Failed)
    }

    pub fn add_task(&mut self, task: Task) -> Result<(), WorkflowError> {
        // assert that task ID is unique
        if self.tasks.contains_key(&task.id) {
            return Err(WorkflowError::TaskAlreadyExists(task.id.clone()));
        }

        // if dependencies are not empty, check if they exist in the task list
        for dep_id in &task.dependencies {
            if !self.tasks.contains_key(dep_id) {
                return Err(WorkflowError::DependencyNotFound(dep_id.clone()));
            }

            // also check that the dependency is not the task itself
            if dep_id == &task.id {
                return Err(WorkflowError::TaskDependsOnItself(task.id.clone()));
            }
        }

        // if all checks pass, insert the task
        self.tasks.insert(task.id.clone(), task);
        self.rebuild_execution_order();
        Ok(())
    }

    pub fn get_task(&self, task_id: &str) -> Option<&Task> {
        self.tasks.get(task_id)
    }

    pub fn remove_task(&mut self, task_id: &str) {
        self.tasks.remove(task_id);
    }

    pub fn pending_count(&self) -> usize {
        self.tasks
            .values()
            .filter(|task| task.status == TaskStatus::Pending)
            .count()
    }

    #[instrument(skip_all)]
    pub fn update_task_status(
        &mut self,
        task_id: &str,
        status: TaskStatus,
        result: Option<AgentResponse>,
    ) {
        debug!(status=?status, result=?result, "Updating task status");
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.status = status;
            task.result = result;
        }
    }

    fn topological_sort(
        &self,
        task_id: &str,
        visited: &mut HashSet<String>,
        temp_visited: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) {
        if temp_visited.contains(task_id) {
            return; // Cycle detected, skip
        }

        if visited.contains(task_id) {
            return;
        }

        temp_visited.insert(task_id.to_string());

        if let Some(task) = self.tasks.get(task_id) {
            for dep_id in &task.dependencies {
                self.topological_sort(dep_id, visited, temp_visited, order);
            }
        }

        temp_visited.remove(task_id);
        visited.insert(task_id.to_string());
        order.push(task_id.to_string());
    }

    fn rebuild_execution_order(&mut self) {
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        for task_id in self.tasks.keys() {
            if !visited.contains(task_id) {
                self.topological_sort(task_id, &mut visited, &mut temp_visited, &mut order);
            }
        }

        self.execution_order = order;
    }

    /// Iterate through all tasks and return those that are ready to be executed
    /// This also checks if all dependencies of the task are completed
    ///
    /// # Returns a vector of references to tasks that are ready to be executed
    pub fn get_ready_tasks(&self) -> Vec<Task> {
        self.tasks
            .values()
            .filter(|task| {
                task.status == TaskStatus::Pending
                    && task.dependencies.iter().all(|dep_id| {
                        self.tasks
                            .get(dep_id)
                            .map(|dep| dep.status == TaskStatus::Completed)
                            .unwrap_or(false)
                    })
            })
            .cloned()
            .collect()
    }

    pub fn reset_failed_tasks(&mut self) -> Result<(), WorkflowError> {
        for task in self.tasks.values_mut() {
            if task.status == TaskStatus::Failed {
                task.status = TaskStatus::Pending;
                task.increment_retry();
                if task.retry_count > task.max_retries {
                    return Err(WorkflowError::MaxRetriesExceeded(task.id.clone()));
                }
            }
        }
        Ok(())
    }
}

/// Rust-specific implementation of a workflow
#[derive(Debug, Clone)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub tasks: TaskList,
    pub agents: HashMap<String, Arc<Agent>>,
}

impl Workflow {
    pub fn new(name: &str) -> Self {
        info!("Creating new workflow: {}", name);
        Self {
            id: create_uuid7(),
            name: name.to_string(),
            tasks: TaskList::new(),
            agents: HashMap::new(),
        }
    }
    pub async fn run(&self) -> Result<(), WorkflowError> {
        info!("Running workflow: {}", self.name);
        let workflow = self.clone();
        let workflow = Arc::new(RwLock::new(workflow));
        execute_workflow(workflow).await
    }

    pub fn is_complete(&self) -> bool {
        self.tasks.is_complete()
    }

    pub fn pending_count(&self) -> usize {
        self.tasks.pending_count()
    }

    pub fn add_task(&mut self, task: Task) -> Result<(), WorkflowError> {
        self.tasks.add_task(task)
    }

    pub fn add_tasks(&mut self, tasks: Vec<Task>) -> Result<(), WorkflowError> {
        for task in tasks {
            self.tasks.add_task(task)?;
        }
        Ok(())
    }

    pub fn add_agent(&mut self, agent: &Agent) {
        self.agents
            .insert(agent.id.clone(), Arc::new(agent.clone()));
    }

    pub fn execution_plan(&self) -> Result<HashMap<String, HashSet<String>>, WorkflowError> {
        let mut remaining: HashMap<String, HashSet<String>> = self
            .tasks
            .tasks
            .iter()
            .map(|(id, task)| (id.clone(), task.dependencies.iter().cloned().collect()))
            .collect();

        let mut executed = HashSet::new();
        let mut plan = HashMap::new();
        let mut step = 1;

        while !remaining.is_empty() {
            // Find all tasks that can be executed in parallel - collect just the keys we need to remove
            let ready_keys: Vec<String> = remaining
                .iter()
                .filter(|(_, deps)| deps.is_subset(&executed))
                .map(|(id, _)| id.to_string())
                .collect();

            if ready_keys.is_empty() {
                // Circular dependency detected
                break;
            }

            // Create the set for the plan (reusing the already allocated Strings)
            let mut ready_set = HashSet::with_capacity(ready_keys.len());

            // Update tracking sets and build the ready set in one pass
            for key in ready_keys {
                executed.insert(key.clone());
                remaining.remove(&key);
                ready_set.insert(key);
            }

            // Add parallel tasks to the current step
            plan.insert(format!("step{step}"), ready_set);

            step += 1;
        }

        Ok(plan)
    }
}

/// Check if the workflow is complete
/// # Arguments
/// * `workflow` - A reference to the workflow instance
/// # Returns true if the workflow is complete, false otherwise
fn is_workflow_complete(workflow: &Arc<RwLock<Workflow>>) -> bool {
    workflow.read().unwrap().is_complete()
}

/// Reset failed tasks in the workflow
/// # Arguments
/// * `workflow` - A reference to the workflow instance
/// # Returns Ok(()) if successful, or an error if the reset fails
fn reset_failed_workflow_tasks(workflow: &Arc<RwLock<Workflow>>) -> Result<(), WorkflowError> {
    match workflow.write().unwrap().tasks.reset_failed_tasks() {
        Ok(_) => Ok(()),
        Err(e) => {
            warn!("Failed to reset failed tasks: {}", e);
            Err(e)
        }
    }
}

/// Get all ready tasks in the workflow
/// # Arguments
/// * `workflow` - A reference to the workflow instance
/// # Returns a vector of tasks that are ready to be executed
fn get_ready_tasks(workflow: &Arc<RwLock<Workflow>>) -> Vec<Task> {
    workflow.read().unwrap().tasks.get_ready_tasks()
}

/// Check for circular dependencies
/// # Arguments
/// * `workflow` - A reference to the workflow instance
/// # Returns true if circular dependencies are detected, false otherwise
fn check_for_circular_dependencies(workflow: &Arc<RwLock<Workflow>>) -> bool {
    let pending_count = workflow.read().unwrap().pending_count();

    if pending_count > 0 {
        warn!(
            "No ready tasks found but {} pending tasks remain. Possible circular dependency.",
            pending_count
        );
        return true;
    }

    false
}

/// Mark a task as running
/// # Arguments
/// * `workflow` - A reference to the workflow instance
/// # Returns nothing
fn mark_task_as_running(workflow: &Arc<RwLock<Workflow>>, task_id: &str) {
    let mut wf = workflow.write().unwrap();
    wf.tasks
        .update_task_status(task_id, TaskStatus::Running, None);
}

/// Get an agent for a task
/// # Arguments
/// * `workflow` - A reference to the workflow instance
/// * `task` - A reference to the task for which the agent is needed
fn get_agent_for_task(workflow: &Arc<RwLock<Workflow>>, task: &Task) -> Option<Arc<Agent>> {
    let wf = workflow.read().unwrap();
    wf.agents.get(&task.agent_id).cloned()
}

/// Builds the context for a task from its dependencies
/// # Arguments
/// * `workflow` - A reference to the workflow instance
/// * `task` - A reference to the task for which the context is being built
/// # Returns a HashMap containing the context messages for the task
fn build_task_context(
    workflow: &Arc<RwLock<Workflow>>,
    task: &Task,
) -> HashMap<String, Vec<Message>> {
    let wf = workflow.read().unwrap();
    let mut ctx = HashMap::new();

    for dep_id in &task.dependencies {
        if let Some(dep) = wf.tasks.get_task(dep_id) {
            if let Some(result) = &dep.result {
                if let Ok(message) = result.response.to_message(Role::Assistant) {
                    ctx.insert(dep_id.clone(), message);
                }
            }
        }
    }

    ctx
}

/// Spawns an individual task execution
/// # Arguments
/// * `workflow` - A reference to the workflow instance
/// * `task` - The task to be executed
/// * `task_id` - The ID of the task
/// * `agent` - An optional reference to the agent that will execute the task
/// * `context` - A HashMap containing the context messages for the task
/// # Returns a JoinHandle for the spawned task
fn spawn_task_execution(
    workflow: Arc<RwLock<Workflow>>,
    task: Task,
    task_id: String,
    agent: Option<Arc<Agent>>,
    context: HashMap<String, Vec<Message>>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        if let Some(agent) = agent {
            match agent.execute_async_task_with_context(&task, context).await {
                Ok(response) => {
                    let mut wf = workflow.write().unwrap();
                    wf.tasks
                        .update_task_status(&task_id, TaskStatus::Completed, Some(response));
                }
                Err(e) => {
                    error!("Task {} failed: {}", task_id, e);
                    let mut wf = workflow.write().unwrap();
                    wf.tasks
                        .update_task_status(&task_id, TaskStatus::Failed, None);
                }
            }
        } else {
            error!("No agent found for task {}", task_id);
            let mut wf = workflow.write().unwrap();
            wf.tasks
                .update_task_status(&task_id, TaskStatus::Failed, None);
        }
    })
}

/// Helper for spawning a task execution
/// # Arguments
/// * `workflow` - A reference to the workflow instance
/// * `tasks` - A vector of tasks to be executed
/// # Returns a vector of JoinHandles for the spawned tasks
fn spawn_task_executions(
    workflow: &Arc<RwLock<Workflow>>,
    tasks: Vec<Task>,
) -> Vec<tokio::task::JoinHandle<()>> {
    let mut handles = Vec::with_capacity(tasks.len());

    for task in tasks {
        let task_id = task.id.clone();
        //let workflow_clone = workflow.clone();

        // Mark task as running
        mark_task_as_running(workflow, &task_id);

        // Build the context
        let context = build_task_context(workflow, &task);

        // Get/clone agent ARC
        let agent = get_agent_for_task(workflow, &task);

        // Spawn task execution and push handle to the vector
        let handle = spawn_task_execution(workflow.clone(), task, task_id, agent, context);
        handles.push(handle);
    }

    handles
}

/// Wait for all spawned tasks to complete
/// # Arguments
/// * `handles` - A vector of JoinHandles for the spawned tasks
/// # Returns nothing
async fn await_task_completions(handles: Vec<tokio::task::JoinHandle<()>>) {
    for handle in handles {
        if let Err(e) = handle.await {
            warn!("Task execution failed: {}", e);
        }
    }
}

/// Execute the workflow asynchronously
/// This function will be called to start the workflow execution process and does the following:
/// 1. Iterates over workflow tasks while the shared workflow is not complete.
/// 2. Resets any failed tasks to allow them to be retried. This needs to happen before getting ready tasks.
/// 3. Gets all ready tasks
/// 4. For each ready task:
/// ///    - Marks the task as running
/// ///    - Checks previous tasks for injected context
/// ///    - Gets the agent for the task  
/// ///    - Spawn a new tokio task and execute task with agent
/// ///    - Push task to the handles vector
/// 4. Waits for all spawned tasks to complete
#[instrument(skip_all)]
pub async fn execute_workflow(workflow: Arc<RwLock<Workflow>>) -> Result<(), WorkflowError> {
    info!("Starting workflow execution");

    while !is_workflow_complete(&workflow) {
        // Reset any failed tasks
        // This will return and error if any task exceeds its max retries
        reset_failed_workflow_tasks(&workflow)?;

        // Get tasks ready for execution
        let ready_tasks = get_ready_tasks(&workflow);
        info!("Found {} ready tasks for execution", ready_tasks.len());

        // Check for circular dependencies
        if ready_tasks.is_empty() {
            if check_for_circular_dependencies(&workflow) {
                break;
            }
            continue;
        }

        // Execute tasks asynchronously
        let handles = spawn_task_executions(&workflow, ready_tasks);

        // Wait for all tasks to complete
        await_task_completions(handles).await;
    }

    info!("Workflow execution completed");
    Ok(())
}

impl Serialize for Workflow {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Workflow", 4)?;

        // set session to none
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("tasks", &self.tasks)?;

        // serialize agents by unwrapping the Arc
        let agents: HashMap<String, Agent> = self
            .agents
            .iter()
            .map(|(id, agent)| (id.clone(), (*agent.as_ref()).clone()))
            .collect();

        state.serialize_field("agents", &agents)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Workflow {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Id,
            Name,
            Tasks,
            Agents,
        }

        struct WorkflowVisitor;

        impl<'de> Visitor<'de> for WorkflowVisitor {
            type Value = Workflow;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Workflow")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Workflow, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id = None;
                let mut name = None;
                let mut tasks = None;
                let mut agents: Option<HashMap<String, Agent>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            id = Some(map.next_value()?);
                        }
                        Field::Tasks => {
                            tasks = Some(map.next_value()?);
                        }
                        Field::Agents => {
                            agents = Some(map.next_value()?);
                        }
                        Field::Name => {
                            name = Some(map.next_value()?);
                        }
                    }
                }

                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let tasks = tasks.ok_or_else(|| de::Error::missing_field("tasks"))?;
                let agents = agents.ok_or_else(|| de::Error::missing_field("agents"))?;

                // convert agents to arc
                let agents = agents
                    .into_iter()
                    .map(|(id, agent)| (id, Arc::new(agent)))
                    .collect();

                Ok(Workflow {
                    id,
                    name,
                    tasks,
                    agents,
                })
            }
        }

        const FIELDS: &[&str] = &["id", "name", "tasks", "agents"];
        deserializer.deserialize_struct("Workflow", FIELDS, WorkflowVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use potato_prompt::{prompt::types::PromptContent, Message, Prompt};

    #[test]
    fn test_workflow_creation() {
        let workflow = Workflow::new("Test Workflow");
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.id.len(), 36); // UUID7 length
    }

    #[test]
    fn test_task_list_add_and_get() {
        let mut task_list = TaskList::new();
        let prompt_content = PromptContent::Str("Test prompt".to_string());
        let prompt = Prompt::new_rs(
            vec![Message::new_rs(prompt_content)],
            Some("gpt-4o"),
            Some("openai"),
            vec![],
            None,
            None,
        )
        .unwrap();

        let task = Task::new("task1", prompt, "task1", None, None);
        task_list.add_task(task.clone()).unwrap();
        assert_eq!(task_list.get_task(&task.id).unwrap().id, task.id);
        task_list.reset_failed_tasks().unwrap();
    }
}
