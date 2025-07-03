use chrono::{DateTime, Duration, Utc};
use potato_agent::agents::{task::TaskStatus, types::ChatResponse};
use potato_agent::AgentResponse;
use potato_prompt::Prompt;
use potato_util::create_uuid7;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    pub id: String,
    pub workflow_id: String,
    pub task_id: String,
    pub status: TaskStatus,
    pub timestamp: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub details: EventDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<Arc<Prompt>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Arc<ChatResponse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<Duration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EventTracker {
    run_id: String,
    events: Arc<RwLock<Vec<TaskEvent>>>,
    task_start_times: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl EventTracker {
    pub fn new() -> Self {
        Self {
            run_id: create_uuid7(),
            events: Arc::new(RwLock::new(Vec::new())),
            task_start_times: Arc::new(RwLock::new(HashMap::new())),
        }
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
    pub fn record_task_started(&self, workflow_id: &str, task_id: &str) {
        let now = Utc::now();

        let mut start_times = self.task_start_times.write().unwrap();
        start_times.insert(task_id.to_string(), now);

        let event = TaskEvent {
            id: create_uuid7(),
            workflow_id: workflow_id.to_string(),
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
                    event.details.response = Some(Arc::new(response.response.clone()));
                    event.details.duration = duration;
                    event.details.end_time = Some(now);
                    event.details.prompt = Some(Arc::new(prompt.clone()));
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
                    event.details.prompt = Some(Arc::new(prompt.clone()));
                    event.details.error = Some(error_msg.to_string());
                    Some(event)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
    }
}
