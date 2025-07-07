use chrono::{DateTime, Duration, Utc};
use potato_agent::agents::{task::TaskStatus, types::ChatResponse};
use potato_agent::AgentResponse;
use potato_prompt::Prompt;
use potato_util::create_uuid7;
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskEvent {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub workflow_id: String,
    #[pyo3(get)]
    pub task_id: String,
    #[pyo3(get)]
    pub status: TaskStatus,
    #[pyo3(get)]
    pub timestamp: DateTime<Utc>,
    #[pyo3(get)]
    pub updated_at: DateTime<Utc>,
    #[pyo3(get)]
    pub details: EventDetails,
}

#[pymethods]
impl TaskEvent {
    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct EventDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub prompt: Option<Prompt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub response: Option<ChatResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub duration: Option<Duration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub end_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get)]
    pub error: Option<String>,
}

#[pymethods]
impl EventDetails {
    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

#[derive(Debug, Clone, Default)]
pub struct EventTracker {
    workflow_id: String,
    pub events: Arc<RwLock<Vec<TaskEvent>>>,
    task_start_times: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl PartialEq for EventTracker {
    fn eq(&self, other: &Self) -> bool {
        // Compare workflow_id and events
        self.workflow_id == other.workflow_id
    }
}

impl EventTracker {
    pub fn new(workflow_id: String) -> Self {
        Self {
            workflow_id,
            events: Arc::new(RwLock::new(Vec::new())),
            task_start_times: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn is_empty(&self) -> bool {
        let events = self.events.read().unwrap();
        events.is_empty()
    }

    pub fn reset(&self) {
        let mut events = self.events.write().unwrap();
        events.clear();
        let mut task_start_times = self.task_start_times.write().unwrap();
        task_start_times.clear();
    }

    /// Creates an event for a task when it is started.
    /// # Arguments
    /// * `workflow_id` - The ID of the workflow to which the task belongs.
    /// * `task_id` - The ID of the task that was started.
    /// # Returns
    /// None
    pub fn record_task_started(&self, task_id: &str) {
        let now = Utc::now();

        let mut start_times = self.task_start_times.write().unwrap();
        start_times.insert(task_id.to_string(), now);

        let event = TaskEvent {
            id: create_uuid7(),
            workflow_id: self.workflow_id.clone(),
            task_id: task_id.to_string(),
            status: TaskStatus::Running,
            timestamp: now,
            updated_at: now,
            details: EventDetails {
                start_time: Some(now),
                ..Default::default()
            },
        };

        let mut events = self.events.write().unwrap();
        events.push(event);
    }

    /// Updates the event for a given task ID when it is completed.
    /// # Arguments
    /// * `task_id` - The ID of the task that was completed.
    /// * `prompt` - The prompt used for the task.
    /// * `response` - The response received from the task.
    /// # Returns
    /// None
    pub fn record_task_completed(&self, task_id: &str, prompt: &Prompt, response: AgentResponse) {
        let now = Utc::now();
        let duration = {
            let start_times = self.task_start_times.read().unwrap();
            start_times
                .get(task_id)
                .map(|start_time| now.signed_duration_since(*start_time))
        };

        // update the event details
        // Update the event
        let mut events = self.events.write().unwrap();

        // filter to find the event with the matching task_id
        // and update it
        let _ = events
            .iter_mut()
            .filter_map(|event| {
                if event.task_id == task_id {
                    event.status = TaskStatus::Completed;
                    event.updated_at = now;
                    event.details.response = Some(response.response.clone());
                    event.details.duration = duration;
                    event.details.end_time = Some(now);
                    event.details.prompt = Some(prompt.clone());
                    Some(event)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
    }

    /// Records a task failure event.
    /// # Arguments
    /// * `task_id` - The ID of the task that failed.
    /// * `error_msg` - The error message associated with the failure.
    /// * `prompt` - The prompt used for the task.
    /// # Returns
    /// None
    pub fn record_task_failed(&self, task_id: &str, error_msg: &str, prompt: &Prompt) {
        let now = Utc::now();
        let duration = {
            let start_times = self.task_start_times.read().unwrap();
            start_times
                .get(task_id)
                .map(|start_time| now.signed_duration_since(*start_time))
        };

        // update the event details
        // Update the event
        let mut events = self.events.write().unwrap();

        // filter to find the event with the matching task_id
        // and update it
        let _ = events
            .iter_mut()
            .filter_map(|event| {
                if event.task_id == task_id {
                    event.status = TaskStatus::Failed;
                    event.updated_at = now;
                    event.details.duration = duration;
                    event.details.end_time = Some(now);
                    event.details.prompt = Some(prompt.clone());
                    event.details.error = Some(error_msg.to_string());
                    Some(event)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
    }
}
