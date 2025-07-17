use crate::tasklist::TaskList;
use crate::types::Context;
use crate::{
    events::{EventTracker, TaskEvent},
    workflow::error::WorkflowError,
};
pub use potato_agent::agents::{
    agent::{Agent, PyAgent},
    task::{PyTask, Task, TaskStatus},
    types::ChatResponse,
};
use potato_agent::PyAgentResponse;
use potato_prompt::parse_response_format;
use potato_prompt::prompt::types::Role;
use potato_prompt::Message;
use potato_util::{create_uuid7, utils::update_serde_map_with, PyHelperFuncs};
use potato_util::{json_to_pydict, pyobject_to_json};
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
use serde::{
    de::{self, MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::Map;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::RwLock;
use tracing::instrument;
use tracing::{debug, error, info, warn};

/// Python workflows are a work in progress
use pyo3::types::PyDict;

#[derive(Debug)]
#[pyclass]
pub struct WorkflowResult {
    #[pyo3(get)]
    pub tasks: HashMap<String, Py<PyTask>>,

    #[pyo3(get)]
    pub events: Vec<TaskEvent>,
}

impl WorkflowResult {
    pub fn new(
        py: Python,
        tasks: HashMap<String, Task>,
        output_types: &HashMap<String, Arc<PyObject>>,
        events: Vec<TaskEvent>,
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

        Self {
            tasks: py_tasks,
            events,
        }
    }
}

#[pymethods]
impl WorkflowResult {
    pub fn __str__(&self) -> String {
        // serialize tasks to json
        let json = serde_json::json!({
            "tasks": serde_json::to_value(&self.tasks).unwrap_or(Value::Null),
            "events": serde_json::to_value(&self.events).unwrap_or(Value::Null)
        });

        PyHelperFuncs::__str__(&json)
    }
}

/// Rust-specific implementation of a workflow
#[derive(Debug, Clone)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub task_list: TaskList,
    pub agents: HashMap<String, Arc<Agent>>,
    pub event_tracker: Arc<RwLock<EventTracker>>,
    pub global_context: Option<Value>,
}

impl PartialEq for Workflow {
    fn eq(&self, other: &Self) -> bool {
        // Compare by ID and name
        self.id == other.id && self.name == other.name
    }
}

impl Workflow {
    pub fn new(name: &str) -> Self {
        info!("Creating new workflow: {}", name);
        let id = create_uuid7();
        Self {
            id: id.clone(),
            name: name.to_string(),
            task_list: TaskList::new(),
            agents: HashMap::new(),
            event_tracker: Arc::new(RwLock::new(EventTracker::new(id))),
            global_context: None, // Initialize with no global context
        }
    }
    pub fn events(&self) -> Vec<TaskEvent> {
        let tracker = self.event_tracker.read().unwrap();
        let events = tracker.events.read().unwrap().clone();
        events
    }

    pub fn get_new_workflow(&self, global_context: Option<Value>) -> Result<Self, WorkflowError> {
        // set new id for the new workflow
        let id = create_uuid7();

        // create deep copy of the tasklist so we don't clone the arc
        let task_list = self.task_list.deep_clone()?;

        Ok(Workflow {
            id: id.clone(),
            name: self.name.clone(),
            task_list,
            agents: self.agents.clone(), // Agents can be shared since they're read-only during execution
            event_tracker: Arc::new(RwLock::new(EventTracker::new(id))),
            global_context, // Use the provided global context or None
        })
    }

    pub async fn run(
        &self,
        global_context: Option<Value>,
    ) -> Result<Arc<RwLock<Workflow>>, WorkflowError> {
        info!("Running workflow: {}", self.name);

        let run_workflow = Arc::new(RwLock::new(self.get_new_workflow(global_context)?));

        execute_workflow(&run_workflow).await?;

        Ok(run_workflow)
    }

    pub fn is_complete(&self) -> bool {
        self.task_list.is_complete()
    }

    pub fn pending_count(&self) -> usize {
        self.task_list.pending_count()
    }

    pub fn add_task(&mut self, task: Task) -> Result<(), WorkflowError> {
        self.task_list.add_task(task)
    }

    pub fn add_tasks(&mut self, tasks: Vec<Task>) -> Result<(), WorkflowError> {
        for task in tasks {
            self.task_list.add_task(task)?;
        }
        Ok(())
    }

    pub fn add_agent(&mut self, agent: &Agent) {
        self.agents
            .insert(agent.id.clone(), Arc::new(agent.clone()));
    }

    pub fn execution_plan(&self) -> Result<HashMap<i32, HashSet<String>>, WorkflowError> {
        let mut remaining: HashMap<String, HashSet<String>> = self
            .task_list
            .tasks
            .iter()
            .map(|(id, task)| {
                (
                    id.clone(),
                    task.read().unwrap().dependencies.iter().cloned().collect(),
                )
            })
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
            plan.insert(step, ready_set);

            step += 1;
        }

        Ok(plan)
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(&self.task_list)
    }

    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        // reset the workflow
        let json = serde_json::to_string(self).unwrap();
        // Add debug output to see what's being serialized
        Ok(json)
    }

    pub fn from_json(json: &str) -> Result<Self, WorkflowError> {
        // Deserialize the JSON string into a Workflow instance
        Ok(serde_json::from_str(json)?)
    }

    pub fn task_names(&self) -> Vec<String> {
        self.task_list
            .tasks
            .keys()
            .cloned()
            .collect::<Vec<String>>()
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
    match workflow.write().unwrap().task_list.reset_failed_tasks() {
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
fn get_ready_tasks(workflow: &Arc<RwLock<Workflow>>) -> Vec<Arc<RwLock<Task>>> {
    workflow.read().unwrap().task_list.get_ready_tasks()
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
fn mark_task_as_running(task: Arc<RwLock<Task>>, event_tracker: &Arc<RwLock<EventTracker>>) {
    let mut task = task.write().unwrap();
    task.set_status(TaskStatus::Running);
    event_tracker.write().unwrap().record_task_started(&task.id);
}

/// Get an agent for a task
/// # Arguments
/// * `workflow` - A reference to the workflow instance
/// * `task` - A reference to the task for which the agent is needed
fn get_agent_for_task(workflow: &Arc<RwLock<Workflow>>, agent_id: &str) -> Option<Arc<Agent>> {
    let wf = workflow.read().unwrap();
    wf.agents.get(agent_id).cloned()
}

/// Builds the context for a task from its dependencies
/// # Arguments
/// * `workflow` - A reference to the workflow instance
/// * `task` - A reference to the task for which the context is being built
/// # Returns a HashMap containing the context messages for the task
#[instrument(skip_all)]
fn build_task_context(
    workflow: &Arc<RwLock<Workflow>>,
    task_dependencies: &Vec<String>,
) -> Result<Context, WorkflowError> {
    let wf = workflow.read().unwrap();
    let mut ctx = HashMap::new();
    let mut param_ctx: Value = Value::Object(Map::new());

    for dep_id in task_dependencies {
        debug!("Building context for task dependency: {}", dep_id);
        if let Some(dep) = wf.task_list.get_task(dep_id) {
            if let Some(result) = &dep.read().unwrap().result {
                let msg_to_insert = result.response.to_message(Role::Assistant);

                match msg_to_insert {
                    Ok(message) => {
                        ctx.insert(dep_id.clone(), message);
                    }
                    Err(e) => {
                        warn!("Failed to convert response to message: {}", e);
                    }
                }

                if let Some(structure_output) = result.response.extract_structured_data() {
                    // Value should be a serde_json::Value Object type
                    // validate that it's an object
                    if structure_output.is_object() {
                        // extract the Map from the Value
                        update_serde_map_with(&mut param_ctx, &structure_output)?;
                    }
                }
            }
        }
    }

    debug!("Built context for task dependencies: {:?}", ctx);
    let global_context = workflow.read().unwrap().global_context.clone();

    Ok((ctx, param_ctx, global_context))
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
    event_tracker: Arc<RwLock<EventTracker>>,
    task: Arc<RwLock<Task>>,
    task_id: String,
    agent: Option<Arc<Agent>>,
    context: HashMap<String, Vec<Message>>,
    parameter_context: Value,
    global_context: Option<Value>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        if let Some(agent) = agent {
            // (1) Insert any context messages and/or parameters into the task prompt
            // (2) Execute the task with the agent
            // (3) Return the AgentResponse
            let result = agent
                .execute_task_with_context(&task, context, parameter_context, global_context)
                .await;
            match result {
                Ok(response) => {
                    let mut write_task = task.write().unwrap();
                    write_task.set_status(TaskStatus::Completed);
                    write_task.set_result(response.clone());
                    event_tracker.write().unwrap().record_task_completed(
                        &write_task.id,
                        &write_task.prompt,
                        response,
                    );
                }
                Err(e) => {
                    error!("Task {} failed: {}", task_id, e);
                    let mut write_task = task.write().unwrap();
                    write_task.set_status(TaskStatus::Failed);
                    event_tracker.write().unwrap().record_task_failed(
                        &write_task.id,
                        &e.to_string(),
                        &write_task.prompt,
                    );
                }
            }
        } else {
            error!("No agent found for task {}", task_id);
            let mut write_task = task.write().unwrap();
            write_task.set_status(TaskStatus::Failed);
        }
    })
}

fn get_parameters_from_context(task: Arc<RwLock<Task>>) -> (String, Vec<String>, String) {
    let (task_id, dependencies, agent_id) = {
        let task_guard = task.read().unwrap();
        (
            task_guard.id.clone(),
            task_guard.dependencies.clone(),
            task_guard.agent_id.clone(),
        )
    };

    (task_id, dependencies, agent_id)
}

/// Helper for spawning a task execution
/// # Arguments
/// * `workflow` - A reference to the workflow instance
/// * `tasks` - A vector of tasks to be executed
/// # Returns a vector of JoinHandles for the spawned tasks
fn spawn_task_executions(
    workflow: &Arc<RwLock<Workflow>>,
    ready_tasks: Vec<Arc<RwLock<Task>>>,
) -> Result<Vec<tokio::task::JoinHandle<()>>, WorkflowError> {
    let mut handles = Vec::with_capacity(ready_tasks.len());

    // Get the event tracker from the workflow
    let event_tracker = workflow.read().unwrap().event_tracker.clone();

    for task in ready_tasks {
        // Get task parameters
        let (task_id, dependencies, agent_id) = get_parameters_from_context(task.clone());

        // Mark task as running
        // This will also record the task started event
        mark_task_as_running(task.clone(), &event_tracker);

        // Build the context
        // Here we:
        // 1. Get the task dependencies and their results (these will be injected as assistant messages)
        // 2. Parse dependent tasks for any structured outputs and return as a serde_json::Value (this will be task-level context)
        let (context, parameter_context, global_context) =
            build_task_context(workflow, &dependencies)?;

        // Get/clone agent ARC
        let agent = get_agent_for_task(workflow, &agent_id);

        // Spawn task execution and push handle to future vector
        let handle = spawn_task_execution(
            event_tracker.clone(),
            task.clone(),
            task_id,
            agent,
            context,
            parameter_context,
            global_context,
        );
        handles.push(handle);
    }

    Ok(handles)
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
pub async fn execute_workflow(workflow: &Arc<RwLock<Workflow>>) -> Result<(), WorkflowError> {
    // Important to remember that the workflow is an Arc<RwLock<Workflow>> is a new clone of
    // the loaded workflow. This allows us to mutate the workflow without affecting the original
    // workflow instance.
    debug!("Starting workflow execution");

    // Run until workflow is complete
    while !is_workflow_complete(workflow) {
        // Reset any failed tasks
        // This will return an error if any task exceeds its max retries (set at the task level)
        reset_failed_workflow_tasks(workflow)?;

        // Get tasks ready for execution
        // This will return an Arc<RwLock<Task>>
        let ready_tasks = get_ready_tasks(workflow);
        debug!("Found {} ready tasks for execution", ready_tasks.len());

        // Check for circular dependencies
        if ready_tasks.is_empty() {
            if check_for_circular_dependencies(workflow) {
                break;
            }
            continue;
        }

        // Execute tasks asynchronously
        let handles = spawn_task_executions(workflow, ready_tasks)?;

        // Wait for all tasks to complete
        await_task_completions(handles).await;
    }

    debug!("Workflow execution completed");
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
        state.serialize_field("task_list", &self.task_list)?;
        state.serialize_field("agents", &self.agents)?;

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
            TaskList,
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
                let mut task_list_data = None;
                let mut agents: Option<HashMap<String, Agent>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            let value: String = map.next_value().map_err(|e| {
                                error!("Failed to deserialize field 'id': {e}");
                                de::Error::custom(format!("Failed to deserialize field 'id': {e}"))
                            })?;
                            id = Some(value);
                        }
                        Field::TaskList => {
                            // Deserialize as a generic Value first
                            let value: TaskList = map.next_value().map_err(|e| {
                                error!("Failed to deserialize field 'task_list': {e}");
                                de::Error::custom(format!(
                                    "Failed to deserialize field 'task_list': {e}",
                                ))
                            })?;

                            task_list_data = Some(value);
                        }
                        Field::Name => {
                            let value: String = map.next_value().map_err(|e| {
                                error!("Failed to deserialize field 'name': {e}");
                                de::Error::custom(format!(
                                    "Failed to deserialize field 'name': {e}",
                                ))
                            })?;
                            name = Some(value);
                        }
                        Field::Agents => {
                            let value: HashMap<String, Agent> = map.next_value().map_err(|e| {
                                error!("Failed to deserialize field 'agents': {e}");
                                de::Error::custom(format!(
                                    "Failed to deserialize field 'agents': {e}"
                                ))
                            })?;
                            agents = Some(value);
                        }
                    }
                }

                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let task_list_data =
                    task_list_data.ok_or_else(|| de::Error::missing_field("task_list"))?;
                let agents = agents.ok_or_else(|| de::Error::missing_field("agents"))?;

                let event_tracker = Arc::new(RwLock::new(EventTracker::new(create_uuid7())));

                // convert agents to arc
                let agents = agents
                    .into_iter()
                    .map(|(id, agent)| (id, Arc::new(agent)))
                    .collect();

                Ok(Workflow {
                    id,
                    name,
                    task_list: task_list_data,
                    agents,
                    event_tracker,
                    global_context: None, // Initialize with no global context
                })
            }
        }

        const FIELDS: &[&str] = &["id", "name", "task_list", "agents"];
        deserializer.deserialize_struct("Workflow", FIELDS, WorkflowVisitor)
    }
}

#[pyclass(name = "Workflow")]
#[derive(Debug, Clone)]
pub struct PyWorkflow {
    workflow: Workflow,

    // allow adding output types for python tasks (py only)
    // these are provided at runtime by the user and must match the response
    // format of the prompt the task is associated with
    output_types: HashMap<String, Arc<PyObject>>,

    // potatohead version holds a reference to the runtime
    runtime: Arc<tokio::runtime::Runtime>,
}

#[pymethods]
impl PyWorkflow {
    #[new]
    #[pyo3(signature = (name))]
    pub fn new(name: &str) -> Result<Self, WorkflowError> {
        info!("Creating new workflow: {}", name);
        Ok(Self {
            workflow: Workflow::new(name),
            output_types: HashMap::new(),
            runtime: Arc::new(
                tokio::runtime::Runtime::new()
                    .map_err(|e| WorkflowError::RuntimeError(e.to_string()))?,
            ),
        })
    }

    #[getter]
    pub fn name(&self) -> String {
        self.workflow.name.clone()
    }

    #[getter]
    pub fn task_list(&self) -> TaskList {
        self.workflow.task_list.clone()
    }

    #[getter]
    pub fn is_workflow(&self) -> bool {
        true
    }

    #[getter]
    pub fn __workflow__(&self) -> String {
        self.model_dump_json()
    }

    #[getter]
    pub fn agents(&self) -> Result<HashMap<String, PyAgent>, WorkflowError> {
        self.workflow
            .agents
            .iter()
            .map(|(id, agent)| {
                Ok((
                    id.clone(),
                    PyAgent {
                        agent: agent.clone(),
                        runtime: self.runtime.clone(),
                    },
                ))
            })
            .collect::<Result<HashMap<_, _>, _>>()
    }

    #[pyo3(signature = (task_output_types))]
    pub fn add_task_output_types<'py>(
        &mut self,
        task_output_types: Bound<'py, PyDict>,
    ) -> PyResult<()> {
        let converted: HashMap<String, Arc<PyObject>> = task_output_types
            .iter()
            .map(|(k, v)| -> PyResult<(String, Arc<PyObject>)> {
                // Explicitly return a Result from the closure
                let key = k.extract::<String>()?;
                let value = v.clone().unbind();
                Ok((key, Arc::new(value)))
            })
            .collect::<PyResult<_>>()?;
        self.output_types.extend(converted);
        Ok(())
    }

    #[pyo3(signature = (task, output_type = None))]
    pub fn add_task(
        &mut self,
        py: Python<'_>,
        mut task: Task,
        output_type: Option<Bound<'_, PyAny>>,
    ) -> Result<(), WorkflowError> {
        if let Some(output_type) = output_type {
            // Parse and set the response format
            (task.prompt.response_type, task.prompt.response_format) =
                parse_response_format(py, &output_type)
                    .map_err(|e| WorkflowError::InvalidOutputType(e.to_string()))?;

            // Store the output type for later use
            self.output_types
                .insert(task.id.clone(), Arc::new(output_type.unbind()));
        }

        self.workflow.task_list.add_task(task)?;
        Ok(())
    }

    pub fn add_tasks(&mut self, tasks: Vec<Task>) -> Result<(), WorkflowError> {
        for task in tasks {
            self.workflow.task_list.add_task(task)?;
        }
        Ok(())
    }

    pub fn add_agent(&mut self, agent: &Bound<'_, PyAgent>) {
        // extract the arc rust agent from the python agent
        let agent = agent.extract::<PyAgent>().unwrap().agent.clone();
        self.workflow.agents.insert(agent.id.clone(), agent);
    }

    pub fn is_complete(&self) -> bool {
        self.workflow.task_list.is_complete()
    }

    pub fn pending_count(&self) -> usize {
        self.workflow.task_list.pending_count()
    }

    pub fn execution_plan<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyDict>, WorkflowError> {
        let plan = self.workflow.execution_plan()?;
        debug!("Execution plan: {:?}", plan);

        // turn hashmap into a to json
        let json = serde_json::to_value(plan).map_err(|e| {
            error!("Failed to serialize execution plan to JSON: {}", e);
            e
        })?;

        let pydict = PyDict::new(py);
        json_to_pydict(py, &json, &pydict)?;

        Ok(pydict)
    }

    #[pyo3(signature = (global_context=None))]
    pub fn run(
        &self,
        py: Python,
        global_context: Option<Bound<'_, PyDict>>,
    ) -> Result<WorkflowResult, WorkflowError> {
        info!("Running workflow: {}", self.workflow.name);

        // Convert the global context from PyDict to serde_json::Value if provided
        let global_context = if let Some(context) = global_context {
            // Convert PyDict to serde_json::Value
            let json_value = pyobject_to_json(&context.into_bound_py_any(py)?)?;
            Some(json_value)
        } else {
            None
        };

        let workflow: Arc<RwLock<Workflow>> = self
            .runtime
            .block_on(async { self.workflow.run(global_context).await })?;

        // Try to get exclusive ownership of the workflow by unwrapping the Arc if there's only one reference
        let workflow_result = match Arc::try_unwrap(workflow) {
            // If we have exclusive ownership, we can consume the RwLock
            Ok(rwlock) => {
                // Unwrap the RwLock to get the Workflow
                let workflow = rwlock
                    .into_inner()
                    .map_err(|_| WorkflowError::LockAcquireError)?;

                // Get the events before creating WorkflowResult
                let events = workflow
                    .event_tracker
                    .read()
                    .unwrap()
                    .events
                    .read()
                    .unwrap()
                    .clone();

                // Move the tasks out of the workflow
                WorkflowResult::new(py, workflow.task_list.tasks(), &self.output_types, events)
            }
            // If there are other references, we need to clone
            Err(arc) => {
                // Just read the workflow
                error!("Workflow still has other references, reading instead of consuming.");
                let workflow = arc
                    .read()
                    .map_err(|_| WorkflowError::ReadLockAcquireError)?;

                // Get the events before creating WorkflowResult
                let events = workflow
                    .event_tracker
                    .read()
                    .unwrap()
                    .events
                    .read()
                    .unwrap()
                    .clone();

                WorkflowResult::new(py, workflow.task_list.tasks(), &self.output_types, events)
            }
        };

        info!("Workflow execution completed successfully.");
        Ok(workflow_result)
    }

    pub fn model_dump_json(&self) -> String {
        serde_json::to_string(&self.workflow).unwrap()
    }

    #[staticmethod]
    #[pyo3(signature = (json_string, output_types=None))]
    pub fn model_validate_json(
        json_string: String,
        output_types: Option<Bound<'_, PyDict>>,
    ) -> Result<Self, WorkflowError> {
        let workflow: Workflow = serde_json::from_str(&json_string)?;
        let runtime = Arc::new(
            tokio::runtime::Runtime::new()
                .map_err(|e| WorkflowError::RuntimeError(e.to_string()))?,
        );

        let output_types = if let Some(output_types) = output_types {
            output_types
                .iter()
                .map(|(k, v)| -> PyResult<(String, Arc<PyObject>)> {
                    let key = k.extract::<String>()?;
                    let value = v.clone().unbind();
                    Ok((key, Arc::new(value)))
                })
                .collect::<PyResult<HashMap<String, Arc<PyObject>>>>()?
        } else {
            HashMap::new()
        };

        let py_workflow = PyWorkflow {
            workflow,
            output_types,
            runtime,
        };

        Ok(py_workflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use potato_prompt::prompt::ResponseType;
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
            ResponseType::Null,
        )
        .unwrap();

        let task = Task::new("task1", prompt, "task1", None, None);
        task_list.add_task(task.clone()).unwrap();
        assert_eq!(
            task_list.get_task(&task.id).unwrap().read().unwrap().id,
            task.id
        );
        task_list.reset_failed_tasks().unwrap();
    }
}
