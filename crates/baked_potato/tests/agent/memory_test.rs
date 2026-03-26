use potato_agent::{
    InMemoryMemory, Memory, MemoryStore, MemoryTurn, PersistentMemory, SqliteMemoryStore,
    WindowedMemory,
};
use serde_json::json;

fn user_msg() -> potato_type::prompt::MessageNum {
    potato_type::prompt::MessageNum::RawV1(json!({"role": "user", "content": "hello"}))
}

fn asst_msg() -> potato_type::prompt::MessageNum {
    potato_type::prompt::MessageNum::RawV1(json!({"role": "assistant", "content": "hi"}))
}

fn turn() -> MemoryTurn {
    MemoryTurn {
        user: user_msg(),
        assistant: asst_msg(),
    }
}

// ── InMemoryMemory ───────────────────────────────────────────────────────────

#[test]
fn in_memory_push_and_retrieve() {
    let mut mem = InMemoryMemory::new();
    assert_eq!(mem.len(), 0);

    mem.push_turn(turn());
    assert_eq!(mem.len(), 1);

    let msgs = mem.messages();
    assert_eq!(msgs.len(), 2); // user + assistant
}

#[test]
fn in_memory_clear() {
    let mut mem = InMemoryMemory::new();
    mem.push_turn(turn());
    mem.push_turn(turn());
    assert_eq!(mem.len(), 2);

    mem.clear();
    assert_eq!(mem.len(), 0);
    assert!(mem.messages().is_empty());
}

// ── WindowedMemory ───────────────────────────────────────────────────────────

#[test]
fn windowed_memory_within_capacity() {
    let mut mem = WindowedMemory::new(3);
    mem.push_turn(turn());
    mem.push_turn(turn());
    assert_eq!(mem.len(), 2);
}

#[test]
fn windowed_memory_eviction() {
    let mut mem = WindowedMemory::new(2);
    mem.push_turn(turn());
    mem.push_turn(turn());
    mem.push_turn(turn()); // evicts oldest

    assert_eq!(mem.len(), 2);
    assert_eq!(mem.messages().len(), 4); // 2 turns * 2 messages each
}

#[test]
fn windowed_memory_clear() {
    let mut mem = WindowedMemory::new(5);
    mem.push_turn(turn());
    mem.clear();
    assert_eq!(mem.len(), 0);
}

#[test]
fn windowed_memory_capacity_zero_stays_empty() {
    let mut mem = WindowedMemory::new(0);
    mem.push_turn(turn());
    mem.push_turn(turn());
    assert_eq!(mem.len(), 0, "capacity-0 WindowedMemory must stay empty");
    assert!(mem.messages().is_empty());
}

// ── PersistentMemory + SqliteMemoryStore ─────────────────────────────────────

#[tokio::test]
async fn persistent_memory_hydrate_and_push() {
    let store = std::sync::Arc::new(SqliteMemoryStore::in_memory().await.unwrap());

    let mut pm = PersistentMemory::new("sess1", "app", "user", store.clone());
    pm.hydrate().await.unwrap();
    assert_eq!(pm.len(), 0);

    pm.push_turn_async(turn()).await.unwrap();
    assert_eq!(pm.len(), 1);

    // Verify the store has the turn
    assert_eq!(store.count("app", "user", "sess1").await.unwrap(), 1);
}

#[tokio::test]
async fn persistent_memory_hydrate_loads_existing() {
    let store = std::sync::Arc::new(SqliteMemoryStore::in_memory().await.unwrap());

    // Write a turn directly to the store
    let stored =
        potato_agent::StoredMemoryTurn::new("sess1", "app", "user", "inv1", user_msg(), asst_msg());
    store.save_turn(&stored).await.unwrap();

    // New PersistentMemory should hydrate from the store
    let mut pm = PersistentMemory::new("sess1", "app", "user", store.clone());
    pm.hydrate().await.unwrap();
    assert_eq!(pm.len(), 1);
    assert_eq!(pm.messages().len(), 2);
}

#[tokio::test]
async fn persistent_memory_windowed_cache() {
    let store = std::sync::Arc::new(SqliteMemoryStore::in_memory().await.unwrap());

    let mut pm = PersistentMemory::windowed("sess1", "app", "user", store.clone(), 2);
    pm.hydrate().await.unwrap();

    pm.push_turn_async(turn()).await.unwrap();
    pm.push_turn_async(turn()).await.unwrap();
    pm.push_turn_async(turn()).await.unwrap();

    // Cache should only have 2 turns
    assert_eq!(pm.len(), 2);

    // But the store should have all 3
    assert_eq!(store.count("app", "user", "sess1").await.unwrap(), 3);
}

#[tokio::test]
async fn persistent_memory_clear_store() {
    let store = std::sync::Arc::new(SqliteMemoryStore::in_memory().await.unwrap());

    let mut pm = PersistentMemory::new("sess1", "app", "user", store.clone());
    pm.hydrate().await.unwrap();
    pm.push_turn_async(turn()).await.unwrap();

    pm.clear_store().await.unwrap();
    assert_eq!(pm.len(), 0);
    assert_eq!(store.count("app", "user", "sess1").await.unwrap(), 0);
}
