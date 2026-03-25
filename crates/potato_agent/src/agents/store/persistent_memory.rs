use super::{memory_store::StoredMemoryTurn, MemoryStore, StoreError};
use crate::agents::memory::{Memory, MemoryTurn};
use potato_type::prompt::MessageNum;
use potato_util::create_uuid7;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::warn;

/// Write-through memory that persists turns to a `MemoryStore` and caches them in-process.
pub struct PersistentMemory {
    session_id: String,
    app_name: String,
    user_id: String,
    invocation_id: String,
    store: Arc<dyn MemoryStore>,
    cache: Vec<MemoryTurn>,
    /// True once the cache has been hydrated from the backing store.
    loaded: bool,
    /// If `Some(n)`, only the last `n` turns are kept in the cache.
    max_turns: Option<usize>,
}

impl Debug for PersistentMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PersistentMemory")
            .field("session_id", &self.session_id)
            .field("app_name", &self.app_name)
            .field("user_id", &self.user_id)
            .field("loaded", &self.loaded)
            .field("cached_turns", &self.cache.len())
            .field("max_turns", &self.max_turns)
            .finish()
    }
}

impl PersistentMemory {
    /// Unbounded persistent memory.
    pub fn new(
        session_id: impl Into<String>,
        app_name: impl Into<String>,
        user_id: impl Into<String>,
        store: Arc<dyn MemoryStore>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            app_name: app_name.into(),
            user_id: user_id.into(),
            invocation_id: create_uuid7(),
            store,
            cache: Vec::new(),
            loaded: false,
            max_turns: None,
        }
    }

    /// Windowed persistent memory — only the last `n` turns are kept in the in-process cache.
    /// Older turns are still in the backing store but won't be injected into prompts.
    pub fn windowed(
        session_id: impl Into<String>,
        app_name: impl Into<String>,
        user_id: impl Into<String>,
        store: Arc<dyn MemoryStore>,
        n: usize,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            app_name: app_name.into(),
            user_id: user_id.into(),
            invocation_id: create_uuid7(),
            store,
            cache: Vec::new(),
            loaded: false,
            max_turns: Some(n),
        }
    }

    /// Load all turns from the backing store into the in-process cache.
    /// Idempotent — subsequent calls are no-ops once hydrated.
    pub async fn hydrate(&mut self) -> Result<(), StoreError> {
        if self.loaded {
            return Ok(());
        }
        let stored = self
            .store
            .load_turns(&self.app_name, &self.user_id, &self.session_id)
            .await?;
        let turns: Vec<MemoryTurn> = stored.into_iter().map(|t| t.into_memory_turn()).collect();
        self.cache = if let Some(n) = self.max_turns {
            turns.into_iter().rev().take(n).rev().collect()
        } else {
            turns
        };
        self.loaded = true;
        Ok(())
    }

    /// Append a turn to the cache and persist it to the backing store.
    pub async fn push_turn_async(&mut self, turn: MemoryTurn) -> Result<(), StoreError> {
        let stored = StoredMemoryTurn::new(
            &self.session_id,
            &self.app_name,
            &self.user_id,
            &self.invocation_id,
            turn.user.clone(),
            turn.assistant.clone(),
        );
        self.store.save_turn(&stored).await?;

        self.cache.push(turn);
        if let Some(n) = self.max_turns {
            if self.cache.len() > n {
                self.cache.remove(0);
            }
        }
        Ok(())
    }

    /// Clear the in-process cache and delete all turns from the backing store.
    pub async fn clear_store(&mut self) -> Result<(), StoreError> {
        self.store
            .clear(&self.app_name, &self.user_id, &self.session_id)
            .await?;
        self.cache.clear();
        Ok(())
    }
}

impl Memory for PersistentMemory {
    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    /// Synchronous push — appends to the in-process cache only.
    /// **Note**: this does not persist to the backing store. Use `push_turn_async` in async
    /// contexts to ensure write-through persistence.
    fn push_turn(&mut self, turn: MemoryTurn) {
        warn!("PersistentMemory::push_turn called synchronously — turn will not be persisted to the backing store. Use push_turn_async in async contexts.");
        self.cache.push(turn);
        if let Some(n) = self.max_turns {
            if self.cache.len() > n {
                self.cache.remove(0);
            }
        }
    }

    fn messages(&self) -> Vec<MessageNum> {
        let mut msgs = Vec::with_capacity(self.cache.len() * 2);
        for turn in &self.cache {
            msgs.push(turn.user.clone());
            msgs.push(turn.assistant.clone());
        }
        msgs
    }

    fn clear(&mut self) {
        self.cache.clear();
    }

    fn len(&self) -> usize {
        self.cache.len()
    }
}
