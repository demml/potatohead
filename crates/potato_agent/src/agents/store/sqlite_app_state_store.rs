use super::{app_state_store::AppStateStore, validate_db_path, StoreError};
use crate::agents::session::SessionSnapshot;
use async_trait::async_trait;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SqliteAppStateStore {
    pool: Arc<Pool<Sqlite>>,
}

impl SqliteAppStateStore {
    pub async fn new(path: &str) -> Result<Self, StoreError> {
        let url = validate_db_path(path)?;
        let pool = SqlitePool::connect(&url)
            .await
            .map_err(|e| StoreError::Connection(e.to_string()))?;
        let store = Self {
            pool: Arc::new(pool),
        };
        store.init_tables().await?;
        Ok(store)
    }

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
            "CREATE TABLE IF NOT EXISTS app_state (
                app_name TEXT NOT NULL PRIMARY KEY,
                state_json TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| StoreError::Backend(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl AppStateStore for SqliteAppStateStore {
    async fn load(&self, app_name: &str) -> Result<Option<SessionSnapshot>, StoreError> {
        let result: Option<(String,)> =
            sqlx::query_as("SELECT state_json FROM app_state WHERE app_name = ?")
                .bind(app_name)
                .fetch_optional(self.pool.as_ref())
                .await
                .map_err(|e| StoreError::Backend(e.to_string()))?;

        match result {
            Some((json,)) => {
                let snapshot: SessionSnapshot = serde_json::from_str(&json)?;
                Ok(Some(snapshot))
            }
            None => Ok(None),
        }
    }

    async fn save(&self, app_name: &str, snapshot: &SessionSnapshot) -> Result<(), StoreError> {
        let json = serde_json::to_string(snapshot)?;
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT OR REPLACE INTO app_state (app_name, state_json, updated_at)
             VALUES (?, ?, ?)",
        )
        .bind(app_name)
        .bind(&json)
        .bind(&now)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| StoreError::Backend(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, app_name: &str) -> Result<(), StoreError> {
        sqlx::query("DELETE FROM app_state WHERE app_name = ?")
            .bind(app_name)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| StoreError::Backend(e.to_string()))?;

        Ok(())
    }
}
