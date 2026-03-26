use super::StoreError;
use crate::agents::memory::MemoryTurn;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use potato_type::prompt::MessageNum;
use potato_util::create_uuid7;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;

/// Serializable memory turn suitable for database storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMemoryTurn {
    /// UUIDv7 — time-sortable, used for ordering rows from the DB.
    pub id: String,
    pub session_id: String,
    pub app_name: String,
    pub user_id: String,
    /// Groups turns within one agent run invocation.
    pub invocation_id: String,
    pub user: MessageNum,
    pub assistant: MessageNum,
    /// Arbitrary metadata (tool calls, token usage, etc.)
    pub event_data: Option<Value>,
    pub created_at: DateTime<Utc>,
}

impl StoredMemoryTurn {
    pub fn new(
        session_id: &str,
        app_name: &str,
        user_id: &str,
        invocation_id: &str,
        user: MessageNum,
        assistant: MessageNum,
    ) -> Self {
        Self {
            id: create_uuid7(),
            session_id: session_id.to_string(),
            app_name: app_name.to_string(),
            user_id: user_id.to_string(),
            invocation_id: invocation_id.to_string(),
            user,
            assistant,
            event_data: None,
            created_at: Utc::now(),
        }
    }

    /// Set optional event data on this turn.
    pub fn with_event_data(mut self, data: Value) -> Self {
        self.event_data = Some(data);
        self
    }

    pub fn into_memory_turn(self) -> MemoryTurn {
        MemoryTurn {
            user: self.user,
            assistant: self.assistant,
        }
    }
}

/// Backend-agnostic persistent conversation history.
#[async_trait]
pub trait MemoryStore: Send + Sync + Debug {
    /// Load all turns for a session in chronological order.
    async fn load_turns(
        &self,
        app_name: &str,
        user_id: &str,
        session_id: &str,
    ) -> Result<Vec<StoredMemoryTurn>, StoreError>;

    /// Append a single turn. Must be idempotent (upsert on `id`).
    async fn save_turn(&self, turn: &StoredMemoryTurn) -> Result<(), StoreError>;

    /// Delete all turns for a session.
    async fn clear(
        &self,
        app_name: &str,
        user_id: &str,
        session_id: &str,
    ) -> Result<(), StoreError>;

    /// Return the number of stored turns (for windowed eviction decisions).
    async fn count(
        &self,
        app_name: &str,
        user_id: &str,
        session_id: &str,
    ) -> Result<usize, StoreError>;
}
