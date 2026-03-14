pub mod app_state_store;
pub mod memory_store;
pub mod persistent_memory;
pub mod session_store;
pub mod user_state_store;

#[cfg(feature = "sqlite")]
pub mod sqlite_app_state_store;
#[cfg(feature = "sqlite")]
pub mod sqlite_memory_store;
#[cfg(feature = "sqlite")]
pub mod sqlite_session_store;
#[cfg(feature = "sqlite")]
pub mod sqlite_user_state_store;

pub use app_state_store::AppStateStore;
pub use memory_store::{MemoryStore, StoredMemoryTurn};
pub use persistent_memory::PersistentMemory;
pub use session_store::SessionStore;
pub use user_state_store::UserStateStore;

#[cfg(feature = "sqlite")]
pub use sqlite_app_state_store::SqliteAppStateStore;
#[cfg(feature = "sqlite")]
pub use sqlite_memory_store::SqliteMemoryStore;
#[cfg(feature = "sqlite")]
pub use sqlite_session_store::SqliteSessionStore;
#[cfg(feature = "sqlite")]
pub use sqlite_user_state_store::SqliteUserStateStore;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("storage backend error: {0}")]
    Backend(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("session not found: {0}")]
    NotFound(String),

    #[error("connection error: {0}")]
    Connection(String),
}
