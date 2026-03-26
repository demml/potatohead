# Memory

Memory injects prior conversation turns into the prompt at the start of each `run()` call. Without it, every call to `agent.run()` is stateless — the model has no knowledge of prior exchanges.

A "turn" is one user message plus one assistant message. Memory stores turns and replays them as conversation history.

---

## How Memory Is Injected

When `run()` is called:

1. If using `PersistentMemory`, the store is hydrated from the backing database (idempotent — only reads once per agent instance).
2. The cached turns are flattened into a `[user, assistant, user, assistant, ...]` message sequence.
3. Those messages are inserted into the prompt **after any system messages and before the current user turn**.

At the end of the loop, after the model produces a final text response, the completed turn is appended to memory. For `PersistentMemory`, this triggers a write to the backing store. For in-memory variants, it stays in the process heap.

---

## In-Memory (No Persistence)

`with_in_memory()` attaches an unbounded in-memory store. All turns accumulate for the lifetime of the `Agent` instance. They are lost when the process ends.

```rust
use potato_agent::{AgentBuilder, AgentRunOutcome, AgentRunner, SessionState};
use potato_type::Provider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = AgentBuilder::new()
        .provider(Provider::OpenAI)
        .model("gpt-4o")
        .system_prompt("You are a helpful assistant.")
        .with_in_memory()
        .build()
        .await?;

    let mut session = SessionState::new();

    // Turn 1
    agent.run("My name is Alice.", &mut session).await?;

    // Turn 2 — memory from turn 1 is injected into the prompt
    if let AgentRunOutcome::Complete(result) =
        agent.run("What is my name?", &mut session).await?
    {
        println!("{}", result.final_response.response_text()); // "Your name is Alice."
    }

    Ok(())
}
```

Use `with_windowed_memory(n)` to keep only the last `n` turns. `n = 0` disables memory without disabling the memory struct (all pushes are no-ops).

```rust
.with_windowed_memory(10)   // keep last 10 turns in prompt
```

---

## Persistent Memory (SQLite)

Persistent memory writes each completed turn to a SQLite database and rehydrates from it on the first `run()` call of each agent instance. Conversation history survives process restarts.

```rust
use potato_agent::{
    AgentBuilder, AgentRunOutcome, AgentRunner, SessionState, SqliteMemoryStore,
};
use potato_type::Provider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store = Arc::new(
        SqliteMemoryStore::new("conversations.db").await?
    );

    let agent = AgentBuilder::new()
        .provider(Provider::OpenAI)
        .model("gpt-4o")
        .system_prompt("You are a helpful assistant.")
        .app_name("my_app")            // optional: scope the history namespace
        .user_id("user_42")            // optional: per-user isolation
        .with_memory_store("session_abc", store)
        .build()
        .await?;

    let mut session = SessionState::new();

    agent.run("My name is Alice.", &mut session).await?;

    Ok(())
}
```

History is keyed by `(app_name, user_id, session_id)`. Two agents sharing the same triple share the same history. `app_name` and `user_id` default to `"default"` if not set.

For a windowed persistent store (persist all turns, but only inject the last `n` into the prompt):

```rust
.with_windowed_memory_store("session_abc", store, 10)
```

**Path validation:** `SqliteMemoryStore::new(path)` rejects paths containing `..`, `?`, or `#`. Pass a plain relative or absolute file path.

---

## Memory Variants Comparison

| Variant | Persistence | Grows unbounded | Survives restart |
|---------|-------------|-----------------|-----------------|
| `with_in_memory()` | Process heap | Yes | No |
| `with_windowed_memory(n)` | Process heap | No (last n turns) | No |
| `with_memory_store(sid, store)` | SQLite | Yes | Yes |
| `with_windowed_memory_store(sid, store, n)` | SQLite | No (last n turns) | Yes |

---

## Custom Memory Store Backends

Implement `MemoryStore` to use a different database or storage system.

```rust
use async_trait::async_trait;
use potato_agent::store::{MemoryStore, StoredMemoryTurn, StoreError};

#[derive(Debug)]
struct MyPostgresStore {
    pool: sqlx::PgPool,
}

#[async_trait]
impl MemoryStore for MyPostgresStore {
    async fn load_turns(
        &self,
        app_name: &str,
        user_id: &str,
        session_id: &str,
    ) -> Result<Vec<StoredMemoryTurn>, StoreError> {
        // query your DB, return turns in chronological order
        todo!()
    }

    async fn save_turn(&self, turn: &StoredMemoryTurn) -> Result<(), StoreError> {
        // upsert by turn.id (UUID v7, time-sortable)
        todo!()
    }

    async fn clear(
        &self,
        app_name: &str,
        user_id: &str,
        session_id: &str,
    ) -> Result<(), StoreError> {
        todo!()
    }

    async fn count(
        &self,
        app_name: &str,
        user_id: &str,
        session_id: &str,
    ) -> Result<usize, StoreError> {
        todo!()
    }
}
```

Pass it to the builder the same way as `SqliteMemoryStore`:

```rust
let store = Arc::new(MyPostgresStore { pool });
let agent = AgentBuilder::new()
    .with_memory_store("session_abc", store)
    .build()
    .await?;
```

---

## StoredMemoryTurn fields

`MemoryStore::save_turn` receives a `StoredMemoryTurn`. Implement upsert on `id` to maintain idempotency.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | UUID v7 (time-sortable, unique per turn) |
| `session_id` | `String` | From `with_memory_store("session_abc", ...)` |
| `app_name` | `String` | From `.app_name("my_app")` or `"default"` |
| `user_id` | `String` | From `.user_id("user_42")` or `"default"` |
| `invocation_id` | `String` | UUID v7 generated per `Agent` instance; groups turns within one agent run |
| `user` | `MessageNum` | User message |
| `assistant` | `MessageNum` | Assistant message |
| `event_data` | `Option<Value>` | Optional metadata (tool calls, token counts) |
| `created_at` | `DateTime<Utc>` | Insertion timestamp |

Sort by `(created_at, id)` to get chronological order.

---

## Important Caveats

**Sync push_turn drops persistence.** The `Memory` trait has a synchronous `push_turn` method. If code calls it on a `PersistentMemory` instance through a `Box<dyn Memory>` reference (not the concrete type), the turn will be cached in memory but not written to the backing store. The runtime uses the async path (`push_turn_async`) when it detects `PersistentMemory` via downcast. This warning is logged at `WARN` level if the sync path is hit accidentally.

**Hydration is per agent instance.** `PersistentMemory::hydrate()` runs at most once per `Agent` instance. If you create two `Agent` instances backed by the same session triple, both will load from the DB independently and maintain separate in-memory caches. Writes from one instance are not visible to the other until the next hydration.
