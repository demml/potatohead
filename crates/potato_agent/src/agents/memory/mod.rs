pub mod in_memory;
pub mod windowed;

pub use in_memory::InMemoryMemory;
pub use windowed::WindowedMemory;

use potato_type::prompt::MessageNum;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// A single conversation turn (user + assistant message pair).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTurn {
    pub user: MessageNum,
    pub assistant: MessageNum,
}

/// Trait for agent conversation memory.
pub trait Memory: Send + Sync + Debug {
    fn push_turn(&mut self, turn: MemoryTurn);
    /// Returns messages in chronological order (oldest first).
    fn messages(&self) -> Vec<MessageNum>;
    fn clear(&mut self);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Enables downcasting to concrete types (e.g., `PersistentMemory`).
    /// Override in concrete types to support `downcast_mut`.
    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        None
    }
}
