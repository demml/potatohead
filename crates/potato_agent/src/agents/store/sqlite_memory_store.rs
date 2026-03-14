use super::{MemoryStore, StoreError, StoredMemoryTurn};
use async_trait::async_trait;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SqliteMemoryStore {
    pool: Arc<Pool<Sqlite>>,
}

impl SqliteMemoryStore {
    /// File-backed SQLite store.
    pub async fn new(path: &str) -> Result<Self, StoreError> {
        let url = format!("sqlite:{}?mode=rwc", path);
        let pool = SqlitePool::connect(&url)
            .await
            .map_err(|e| StoreError::Connection(e.to_string()))?;
        let store = Self {
            pool: Arc::new(pool),
        };
        store.init_tables().await?;
        Ok(store)
    }

    /// In-memory SQLite store (for tests).
    pub async fn in_memory() -> Result<Self, StoreError> {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .map_err(|e| StoreError::Connection(e.to_string()))?;
        let store = Self {
            pool: Arc::new(pool),
        };
        store.init_tables().await?;
        Ok(store)
    }

    async fn init_tables(&self) -> Result<(), StoreError> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS memory_turns (
                id TEXT NOT NULL,
                app_name TEXT NOT NULL,
                user_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                invocation_id TEXT NOT NULL,
                user_json TEXT NOT NULL,
                assistant_json TEXT NOT NULL,
                event_data TEXT,
                created_at TEXT NOT NULL,
                PRIMARY KEY (id, app_name, user_id, session_id)
            )",
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| StoreError::Backend(e.to_string()))?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_memory_turns_session
             ON memory_turns(app_name, user_id, session_id)",
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| StoreError::Backend(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl MemoryStore for SqliteMemoryStore {
    async fn load_turns(
        &self,
        app_name: &str,
        user_id: &str,
        session_id: &str,
    ) -> Result<Vec<StoredMemoryTurn>, StoreError> {
        let rows: Vec<(
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            Option<String>,
            String,
        )> = sqlx::query_as(
            "SELECT id, app_name, user_id, session_id, invocation_id,
                    user_json, assistant_json, event_data, created_at
             FROM memory_turns
             WHERE app_name = ? AND user_id = ? AND session_id = ?
             ORDER BY id",
        )
        .bind(app_name)
        .bind(user_id)
        .bind(session_id)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| StoreError::Backend(e.to_string()))?;

        let mut turns = Vec::with_capacity(rows.len());
        for (id, app, uid, sid, inv_id, user_json, asst_json, event_str, created_str) in rows {
            let user = serde_json::from_str(&user_json)?;
            let assistant = serde_json::from_str(&asst_json)?;
            let event_data = event_str.map(|s| serde_json::from_str(&s)).transpose()?;
            let created_at = created_str
                .parse()
                .map_err(|e: chrono::ParseError| StoreError::Backend(e.to_string()))?;

            turns.push(StoredMemoryTurn {
                id,
                session_id: sid,
                app_name: app,
                user_id: uid,
                invocation_id: inv_id,
                user,
                assistant,
                event_data,
                created_at,
            });
        }
        Ok(turns)
    }

    async fn save_turn(&self, turn: &StoredMemoryTurn) -> Result<(), StoreError> {
        let user_json = serde_json::to_string(&turn.user)?;
        let assistant_json = serde_json::to_string(&turn.assistant)?;
        let event_data_json = turn
            .event_data
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let created_at_str = turn.created_at.to_rfc3339();

        sqlx::query(
            "INSERT OR REPLACE INTO memory_turns
             (id, app_name, user_id, session_id, invocation_id, user_json, assistant_json, event_data, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&turn.id)
        .bind(&turn.app_name)
        .bind(&turn.user_id)
        .bind(&turn.session_id)
        .bind(&turn.invocation_id)
        .bind(&user_json)
        .bind(&assistant_json)
        .bind(&event_data_json)
        .bind(&created_at_str)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| StoreError::Backend(e.to_string()))?;

        Ok(())
    }

    async fn clear(
        &self,
        app_name: &str,
        user_id: &str,
        session_id: &str,
    ) -> Result<(), StoreError> {
        sqlx::query(
            "DELETE FROM memory_turns WHERE app_name = ? AND user_id = ? AND session_id = ?",
        )
        .bind(app_name)
        .bind(user_id)
        .bind(session_id)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| StoreError::Backend(e.to_string()))?;

        Ok(())
    }

    async fn count(
        &self,
        app_name: &str,
        user_id: &str,
        session_id: &str,
    ) -> Result<usize, StoreError> {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM memory_turns WHERE app_name = ? AND user_id = ? AND session_id = ?",
        )
        .bind(app_name)
        .bind(user_id)
        .bind(session_id)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| StoreError::Backend(e.to_string()))?;

        Ok(count as usize)
    }
}
