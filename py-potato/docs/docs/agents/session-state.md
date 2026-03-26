# Session State and Stores

`SessionState` is the carrier for data that flows between agents. It is passed by mutable reference to every `run()` call and every sub-agent tool dispatch. Stores provide durable persistence for state that needs to survive process restarts.

---

## SessionState

`SessionState` wraps an `Arc<RwLock<HashMap<String, Value>>>`. Because it uses an `Arc` internally, cloning it produces a view over the same underlying map — mutations from one clone are visible to all others.

```rust
use potato_agent::SessionState;
use serde_json::json;

let mut session = SessionState::new();

// Write
session.set("user_city", json!("Berlin"));

// Read
if let Some(city) = session.get("user_city") {
    println!("{city}");
}

// Remove
session.remove("user_city");

// Take a snapshot (deep clone of all keys)
let snapshot = session.snapshot();
```

### Reserved keys

Keys prefixed with `__` are reserved for internal use. Do not write to `__`-prefixed keys from application code.

| Key | Used for |
|-----|---------|
| `__ancestor_ids` | Circular sub-agent call detection |

Writing to reserved keys will have no effect in parallel orchestration — `merge_user_data` (used when merging child sessions back to parent) skips all `__`-prefixed keys.

---

## SessionStore — Durable Session State

`SessionStore` persists the entire `SessionState` to a backing database at the end of each `run()` call and loads it at the start of the next. This allows an agent to resume with its state intact after a process restart.

```rust
use potato_agent::{AgentBuilder, SqliteSessionStore, SessionState};
use potato_type::Provider;
use std::sync::Arc;

let store = Arc::new(
    SqliteSessionStore::new("session.db").await?
);

let agent = AgentBuilder::new()
    .provider(Provider::OpenAI)
    .model("gpt-4o")
    .app_name("my_app")
    .user_id("user_42")
    .with_session_store("session_abc", store)
    .build()
    .await?;

let mut session = SessionState::new();
agent.run("Remember that I prefer metric units.", &mut session).await?;
// session is now saved to SQLite under (my_app, user_42, session_abc)
```

On the next `run()` call (even in a new process), the agent loads the session before executing. Any keys you set during the prior run are available.

Session state is keyed by `(app_name, user_id, session_id)` and stores arbitrary key-value data. Memory stores conversation turns. The two are independent.

### Path validation

`SqliteSessionStore::new(path)` rejects paths containing `..`, `?`, or `#`.

---

## UserStateStore — Per-User State

`UserStateStore` stores state scoped to a `(app_name, user_id)` pair, shared across all sessions for that user. Use it for user preferences, profile data, or other cross-session context.

```rust
use potato_agent::{AgentBuilder, SqliteUserStateStore};
use std::sync::Arc;

let store = Arc::new(
    SqliteUserStateStore::new("user_state.db").await?
);

let agent = AgentBuilder::new()
    .provider(Provider::OpenAI)
    .model("gpt-4o")
    .app_name("my_app")
    .user_id("user_42")
    .with_user_state_store(store)
    .build()
    .await?;
```

The `SessionState` passed to `run()` is populated with the user state snapshot at the start of each call. Changes to `session` during the run are **not** automatically written back to `UserStateStore`. To persist changes, your tool or callback code would need to call the store directly.

---

## AppStateStore — Application-Level State

`AppStateStore` stores state scoped to `app_name` only, shared across all users and sessions. Use it for configuration, shared counters, or global context that all agents in your application need.

```rust
use potato_agent::{AgentBuilder, SqliteAppStateStore};
use std::sync::Arc;

let store = Arc::new(
    SqliteAppStateStore::new("app_state.db").await?
);

let agent = AgentBuilder::new()
    .provider(Provider::OpenAI)
    .model("gpt-4o")
    .app_name("my_app")
    .with_app_state_store(store)
    .build()
    .await?;
```

---

## Store Load Order at run() Start

At the start of every `agent.run()` call, stores are loaded in this order:

1. `AppStateStore` — loads `(app_name)` snapshot, merges into `session`
2. `UserStateStore` — loads `(app_name, user_id)` snapshot, merges into `session`
3. `SessionStore` — loads `(app_name, user_id, session_id)` snapshot, merges into `session`

Later stores overwrite earlier stores for conflicting keys. Session-level state takes precedence over user-level, which takes precedence over app-level.

At the end of the run (after memory is saved), only `SessionStore` is written back automatically. `AppStateStore` and `UserStateStore` are read-only from the agent's perspective during a run.

---

## Store Trait Summary

All three stores follow the same pattern: `load`, `save`, `delete`. Implement any of these traits to use a custom database backend.

### `SessionStore`

```rust
#[async_trait]
pub trait SessionStore: Send + Sync + Debug {
    async fn load(&self, app_name: &str, user_id: &str, session_id: &str) -> Result<Option<SessionSnapshot>, StoreError>;
    async fn save(&self, app_name: &str, user_id: &str, session_id: &str, snapshot: &SessionSnapshot) -> Result<(), StoreError>;
    async fn delete(&self, app_name: &str, user_id: &str, session_id: &str) -> Result<(), StoreError>;
}
```

### `UserStateStore`

```rust
#[async_trait]
pub trait UserStateStore: Send + Sync + Debug {
    async fn load(&self, app_name: &str, user_id: &str) -> Result<Option<SessionSnapshot>, StoreError>;
    async fn save(&self, app_name: &str, user_id: &str, snapshot: &SessionSnapshot) -> Result<(), StoreError>;
    async fn delete(&self, app_name: &str, user_id: &str) -> Result<(), StoreError>;
}
```

### `AppStateStore`

```rust
#[async_trait]
pub trait AppStateStore: Send + Sync + Debug {
    async fn load(&self, app_name: &str) -> Result<Option<SessionSnapshot>, StoreError>;
    async fn save(&self, app_name: &str, snapshot: &SessionSnapshot) -> Result<(), StoreError>;
    async fn delete(&self, app_name: &str) -> Result<(), StoreError>;
}
```

`SessionSnapshot` is a `HashMap<String, Value>` newtype that serializes to/from JSON.
