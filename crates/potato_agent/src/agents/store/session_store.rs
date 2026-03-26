use super::StoreError;
use crate::agents::session::SessionSnapshot;
use async_trait::async_trait;
use std::fmt::Debug;

/// Backend-agnostic key-value session state persistence.
#[async_trait]
pub trait SessionStore: Send + Sync + Debug {
    /// Load session state. Returns `None` if the session does not exist.
    async fn load(
        &self,
        app_name: &str,
        user_id: &str,
        session_id: &str,
    ) -> Result<Option<SessionSnapshot>, StoreError>;

    /// Persist (create or overwrite) session state.
    async fn save(
        &self,
        app_name: &str,
        user_id: &str,
        session_id: &str,
        snapshot: &SessionSnapshot,
    ) -> Result<(), StoreError>;

    /// Remove a session entirely.
    async fn delete(
        &self,
        app_name: &str,
        user_id: &str,
        session_id: &str,
    ) -> Result<(), StoreError>;

    /// Check existence without loading the full snapshot.
    async fn exists(
        &self,
        app_name: &str,
        user_id: &str,
        session_id: &str,
    ) -> Result<bool, StoreError> {
        Ok(self.load(app_name, user_id, session_id).await?.is_some())
    }
}
