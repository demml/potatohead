use super::StoreError;
use crate::agents::session::SessionSnapshot;
use async_trait::async_trait;
use std::fmt::Debug;

/// Per-user state that spans all sessions for a given (app_name, user_id).
#[async_trait]
pub trait UserStateStore: Send + Sync + Debug {
    async fn load(
        &self,
        app_name: &str,
        user_id: &str,
    ) -> Result<Option<SessionSnapshot>, StoreError>;

    async fn save(
        &self,
        app_name: &str,
        user_id: &str,
        snapshot: &SessionSnapshot,
    ) -> Result<(), StoreError>;

    async fn delete(&self, app_name: &str, user_id: &str) -> Result<(), StoreError>;
}
