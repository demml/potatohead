use super::StoreError;
use crate::agents::session::SessionSnapshot;
use async_trait::async_trait;
use std::fmt::Debug;

/// Global app-level state that spans all users for a given app_name.
#[async_trait]
pub trait AppStateStore: Send + Sync + Debug {
    async fn load(&self, app_name: &str) -> Result<Option<SessionSnapshot>, StoreError>;

    async fn save(&self, app_name: &str, snapshot: &SessionSnapshot) -> Result<(), StoreError>;

    async fn delete(&self, app_name: &str) -> Result<(), StoreError>;
}
