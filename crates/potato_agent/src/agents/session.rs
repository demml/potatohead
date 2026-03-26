use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Shared mutable state across agents in a run.
/// Clone is cheap (Arc clone). Per-request in Axum handlers.
#[derive(Debug, Clone)]
pub struct SessionState {
    inner: Arc<RwLock<HashMap<String, Value>>>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        self.inner
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(key)
            .cloned()
    }

    pub fn set(&self, key: impl Into<String>, value: Value) {
        self.inner
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert(key.into(), value);
    }

    pub fn remove(&self, key: &str) -> Option<Value> {
        self.inner
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .remove(key)
    }

    pub fn snapshot(&self) -> HashMap<String, Value> {
        self.inner.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Merge another snapshot into this session (later values win).
    pub fn merge(&self, other: HashMap<String, Value>) {
        let mut lock = self.inner.write().unwrap_or_else(|e| e.into_inner());
        for (k, v) in other {
            lock.insert(k, v);
        }
    }

    /// Merge a child snapshot into this session, skipping `__`-prefixed system keys.
    /// Use this when merging child-agent sessions to prevent children from overwriting
    /// system keys such as `__ancestor_ids`.
    pub fn merge_user_data(&self, other: HashMap<String, Value>) {
        let mut lock = self.inner.write().unwrap_or_else(|e| e.into_inner());
        for (k, v) in other {
            if !k.starts_with("__") {
                lock.insert(k, v);
            }
        }
    }

    // ── ancestor tracking (circular call prevention) ──────────────────────

    const ANCESTOR_KEY: &'static str = "__ancestor_ids";

    pub fn push_ancestor(&self, agent_id: &str) {
        let mut lock = self.inner.write().unwrap_or_else(|e| e.into_inner());
        let entry = lock
            .entry(Self::ANCESTOR_KEY.to_string())
            .or_insert_with(|| Value::Array(vec![]));
        if let Value::Array(arr) = entry {
            arr.push(Value::String(agent_id.to_string()));
        }
    }

    pub fn pop_ancestor(&self) {
        let mut lock = self.inner.write().unwrap_or_else(|e| e.into_inner());
        if let Some(Value::Array(arr)) = lock.get_mut(Self::ANCESTOR_KEY) {
            arr.pop();
        }
    }

    pub fn is_ancestor(&self, agent_id: &str) -> bool {
        let lock = self.inner.read().unwrap_or_else(|e| e.into_inner());
        if let Some(Value::Array(arr)) = lock.get(Self::ANCESTOR_KEY) {
            arr.iter().any(|v| v.as_str() == Some(agent_id))
        } else {
            false
        }
    }
}

/// Serializable snapshot of session state — for storage between HTTP requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot(pub HashMap<String, Value>);

impl From<&SessionState> for SessionSnapshot {
    fn from(s: &SessionState) -> Self {
        Self(s.snapshot())
    }
}

impl From<SessionSnapshot> for SessionState {
    fn from(snap: SessionSnapshot) -> Self {
        let s = Self::new();
        s.merge(snap.0);
        s
    }
}
