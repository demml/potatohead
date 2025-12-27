use crate::workflow::error::WorkflowError;

pub use potato_agent::agents::{
    agent::Agent,
    task::{Task, TaskStatus, WorkflowTask},
};
use potato_agent::AgentResponse;
use pyo3::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::RwLock;
use tracing::instrument;
use tracing::{debug, warn};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[pyclass]
pub struct TaskList {
    pub tasks: HashMap<String, Arc<RwLock<Task>>>,
    pub execution_order: Vec<String>,
}

impl PartialEq for TaskList {
    fn eq(&self, other: &Self) -> bool {
        // Compare tasks by their IDs and execution order
        self.tasks.keys().eq(other.tasks.keys()) && self.execution_order == other.execution_order
    }
}

#[pymethods]
impl TaskList {
    /// This is mainly a utility function to help with python interoperability.
    pub fn tasks(&self) -> HashMap<String, Task> {
        self.tasks
            .iter()
            .map(|(id, task)| {
                let cloned_task = task.read().unwrap().clone();
                (id.clone(), cloned_task)
            })
            .collect()
    }

    /// Helper for creating a new TaskList by cloning each task in the current TaskList out of the Arc<RwLock<Task>> wrapper.
    pub fn deep_clone(&self) -> Result<Self, WorkflowError> {
        let mut new_task_list = TaskList::new();

        // Clone each task individually to create new Arc<RwLock<Task>> instances
        for (task_id, task_arc) in &self.tasks {
            let task = task_arc.read().unwrap();
            let cloned_task = task.clone(); // This should clone the Task struct itself
            new_task_list
                .tasks
                .insert(task_id.clone(), Arc::new(RwLock::new(cloned_task)));
        }

        // Copy execution order
        new_task_list.execution_order = self.execution_order.clone();

        Ok(new_task_list)
    }
}

impl TaskList {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            execution_order: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn is_complete(&self) -> bool {
        self.tasks.values().all(|task| {
            task.read().unwrap().status == TaskStatus::Completed
                || task.read().unwrap().status == TaskStatus::Failed
        })
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
        self.tasks
            .insert(task.id.clone(), Arc::new(RwLock::new(task)));
        self.rebuild_execution_order();
        Ok(())
    }

    pub fn get_task(&self, task_id: &str) -> Option<Arc<RwLock<Task>>> {
        self.tasks.get(task_id).cloned()
    }

    pub fn remove_task(&mut self, task_id: &str) {
        self.tasks.remove(task_id);
    }

    pub fn pending_count(&self) -> usize {
        self.tasks
            .values()
            .filter(|task| task.read().unwrap().status == TaskStatus::Pending)
            .count()
    }

    #[instrument(skip_all)]
    pub fn update_task_status(
        &mut self,
        task_id: &str,
        status: TaskStatus,
        result: Option<&AgentResponse>,
    ) {
        debug!(status=?status, result=?result, "Updating task status");
        if let Some(task) = self.tasks.get_mut(task_id) {
            let mut task = task.write().unwrap();
            task.status = status;
            task.result = result.cloned();
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
            for dep_id in &task.read().unwrap().dependencies {
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
    pub fn get_ready_tasks(&self) -> Vec<Arc<RwLock<Task>>> {
        self.tasks
            .values()
            .filter(|task_arc| {
                let task = task_arc.read().unwrap();
                task.status == TaskStatus::Pending
                    && task.dependencies.iter().all(|dep_id| {
                        self.tasks
                            .get(dep_id)
                            .map(|dep| dep.read().unwrap().status == TaskStatus::Completed)
                            .unwrap_or(false)
                    })
            })
            .cloned()
            .collect()
    }

    pub fn reset_failed_tasks(&mut self) -> Result<(), WorkflowError> {
        for task in self.tasks.values_mut() {
            let mut task = task.write().unwrap();
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
