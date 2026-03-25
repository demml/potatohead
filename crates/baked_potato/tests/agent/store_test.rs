use potato_agent::{
    validate_db_path, AppStateStore, MemoryStore, SessionSnapshot, SessionStore,
    SqliteAppStateStore, SqliteMemoryStore, SqliteSessionStore, SqliteUserStateStore,
    StoredMemoryTurn, UserStateStore,
};
use serde_json::json;
use std::collections::HashMap;

// ── Path validation ───────────────────────────────────────────────────────────

#[test]
fn validate_db_path_accepts_simple_path() {
    assert!(validate_db_path("my_db.sqlite").is_ok());
}

#[test]
fn validate_db_path_rejects_query_string() {
    assert!(validate_db_path("db?mode=delete").is_err());
}

#[test]
fn validate_db_path_rejects_fragment() {
    assert!(validate_db_path("db#section").is_err());
}

#[test]
fn validate_db_path_rejects_parent_dir() {
    assert!(validate_db_path("../etc/passwd").is_err());
}

fn dummy_msg() -> potato_type::prompt::MessageNum {
    potato_type::prompt::MessageNum::RawV1(json!({"role": "user", "content": "test"}))
}

// ── SqliteMemoryStore ────────────────────────────────────────────────────────

#[tokio::test]
async fn memory_store_save_and_load() {
    let store = SqliteMemoryStore::in_memory().await.unwrap();
    let turn = StoredMemoryTurn::new("sess1", "myapp", "user1", "inv1", dummy_msg(), dummy_msg());
    store.save_turn(&turn).await.unwrap();

    let loaded = store.load_turns("myapp", "user1", "sess1").await.unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, turn.id);
    assert_eq!(loaded[0].app_name, "myapp");
    assert_eq!(loaded[0].user_id, "user1");
    assert_eq!(loaded[0].session_id, "sess1");
    assert_eq!(loaded[0].invocation_id, "inv1");
}

#[tokio::test]
async fn memory_store_idempotent_upsert() {
    let store = SqliteMemoryStore::in_memory().await.unwrap();
    let turn = StoredMemoryTurn::new("sess1", "myapp", "user1", "inv1", dummy_msg(), dummy_msg());
    store.save_turn(&turn).await.unwrap();
    store.save_turn(&turn).await.unwrap();

    let count = store.count("myapp", "user1", "sess1").await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn memory_store_clear() {
    let store = SqliteMemoryStore::in_memory().await.unwrap();
    let turn = StoredMemoryTurn::new("sess1", "myapp", "user1", "inv1", dummy_msg(), dummy_msg());
    store.save_turn(&turn).await.unwrap();
    assert_eq!(store.count("myapp", "user1", "sess1").await.unwrap(), 1);

    store.clear("myapp", "user1", "sess1").await.unwrap();
    assert_eq!(store.count("myapp", "user1", "sess1").await.unwrap(), 0);
}

#[tokio::test]
async fn memory_store_session_isolation() {
    let store = SqliteMemoryStore::in_memory().await.unwrap();

    let t1 = StoredMemoryTurn::new("sess1", "myapp", "user1", "inv1", dummy_msg(), dummy_msg());
    let t2 = StoredMemoryTurn::new("sess2", "myapp", "user1", "inv1", dummy_msg(), dummy_msg());
    store.save_turn(&t1).await.unwrap();
    store.save_turn(&t2).await.unwrap();

    assert_eq!(store.count("myapp", "user1", "sess1").await.unwrap(), 1);
    assert_eq!(store.count("myapp", "user1", "sess2").await.unwrap(), 1);

    store.clear("myapp", "user1", "sess1").await.unwrap();
    assert_eq!(store.count("myapp", "user1", "sess1").await.unwrap(), 0);
    assert_eq!(store.count("myapp", "user1", "sess2").await.unwrap(), 1);
}

#[tokio::test]
async fn memory_store_app_user_scoping() {
    let store = SqliteMemoryStore::in_memory().await.unwrap();

    let t1 = StoredMemoryTurn::new("sess1", "app1", "user1", "inv1", dummy_msg(), dummy_msg());
    let t2 = StoredMemoryTurn::new("sess1", "app2", "user1", "inv1", dummy_msg(), dummy_msg());
    store.save_turn(&t1).await.unwrap();
    store.save_turn(&t2).await.unwrap();

    assert_eq!(store.count("app1", "user1", "sess1").await.unwrap(), 1);
    assert_eq!(store.count("app2", "user1", "sess1").await.unwrap(), 1);
    assert_eq!(
        store
            .load_turns("app1", "user1", "sess1")
            .await
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn memory_store_event_data() {
    let store = SqliteMemoryStore::in_memory().await.unwrap();
    let turn = StoredMemoryTurn::new("sess1", "myapp", "user1", "inv1", dummy_msg(), dummy_msg())
        .with_event_data(json!({"tool": "roll_dice", "tokens": 42}));

    store.save_turn(&turn).await.unwrap();
    let loaded = store.load_turns("myapp", "user1", "sess1").await.unwrap();
    assert_eq!(
        loaded[0].event_data,
        Some(json!({"tool": "roll_dice", "tokens": 42}))
    );
}

// ── SqliteSessionStore ───────────────────────────────────────────────────────

#[tokio::test]
async fn session_store_save_and_load() {
    let store = SqliteSessionStore::in_memory().await.unwrap();
    let mut map = HashMap::new();
    map.insert("key".into(), json!("value"));
    let snapshot = SessionSnapshot(map);

    store.save("app", "user", "sess1", &snapshot).await.unwrap();
    let loaded = store.load("app", "user", "sess1").await.unwrap().unwrap();
    assert_eq!(loaded.0.get("key").unwrap(), &json!("value"));
}

#[tokio::test]
async fn session_store_overwrite() {
    let store = SqliteSessionStore::in_memory().await.unwrap();
    let snap1 = SessionSnapshot({
        let mut m = HashMap::new();
        m.insert("k".into(), json!(1));
        m
    });
    let snap2 = SessionSnapshot({
        let mut m = HashMap::new();
        m.insert("k".into(), json!(2));
        m
    });

    store.save("app", "user", "sess1", &snap1).await.unwrap();
    store.save("app", "user", "sess1", &snap2).await.unwrap();

    let loaded = store.load("app", "user", "sess1").await.unwrap().unwrap();
    assert_eq!(loaded.0.get("k").unwrap(), &json!(2));
}

#[tokio::test]
async fn session_store_delete() {
    let store = SqliteSessionStore::in_memory().await.unwrap();
    let snap = SessionSnapshot(HashMap::new());
    store.save("app", "user", "sess1", &snap).await.unwrap();

    store.delete("app", "user", "sess1").await.unwrap();
    assert!(store.load("app", "user", "sess1").await.unwrap().is_none());
}

#[tokio::test]
async fn session_store_exists() {
    let store = SqliteSessionStore::in_memory().await.unwrap();
    assert!(!store.exists("app", "user", "sess1").await.unwrap());

    let snap = SessionSnapshot(HashMap::new());
    store.save("app", "user", "sess1", &snap).await.unwrap();
    assert!(store.exists("app", "user", "sess1").await.unwrap());
}

#[tokio::test]
async fn session_store_scoping() {
    let store = SqliteSessionStore::in_memory().await.unwrap();
    let snap = SessionSnapshot({
        let mut m = HashMap::new();
        m.insert("v".into(), json!("a"));
        m
    });
    store.save("app1", "user1", "sess1", &snap).await.unwrap();

    assert!(store
        .load("app1", "user1", "sess1")
        .await
        .unwrap()
        .is_some());
    assert!(store
        .load("app2", "user1", "sess1")
        .await
        .unwrap()
        .is_none());
    assert!(store
        .load("app1", "user2", "sess1")
        .await
        .unwrap()
        .is_none());
    assert!(store
        .load("app1", "user1", "sess2")
        .await
        .unwrap()
        .is_none());
}

// ── SqliteUserStateStore ─────────────────────────────────────────────────────

#[tokio::test]
async fn user_state_store_save_and_load() {
    let store = SqliteUserStateStore::in_memory().await.unwrap();
    let snap = SessionSnapshot({
        let mut m = HashMap::new();
        m.insert("pref".into(), json!("dark_mode"));
        m
    });
    store.save("app", "user1", &snap).await.unwrap();

    let loaded = store.load("app", "user1").await.unwrap().unwrap();
    assert_eq!(loaded.0.get("pref").unwrap(), &json!("dark_mode"));
}

#[tokio::test]
async fn user_state_store_overwrite() {
    let store = SqliteUserStateStore::in_memory().await.unwrap();
    let snap1 = SessionSnapshot({
        let mut m = HashMap::new();
        m.insert("v".into(), json!(1));
        m
    });
    let snap2 = SessionSnapshot({
        let mut m = HashMap::new();
        m.insert("v".into(), json!(2));
        m
    });
    store.save("app", "user1", &snap1).await.unwrap();
    store.save("app", "user1", &snap2).await.unwrap();

    let loaded = store.load("app", "user1").await.unwrap().unwrap();
    assert_eq!(loaded.0.get("v").unwrap(), &json!(2));
}

#[tokio::test]
async fn user_state_store_delete() {
    let store = SqliteUserStateStore::in_memory().await.unwrap();
    let snap = SessionSnapshot(HashMap::new());
    store.save("app", "user1", &snap).await.unwrap();
    store.delete("app", "user1").await.unwrap();
    assert!(store.load("app", "user1").await.unwrap().is_none());
}

#[tokio::test]
async fn user_state_store_scoping() {
    let store = SqliteUserStateStore::in_memory().await.unwrap();
    let snap = SessionSnapshot(HashMap::new());
    store.save("app1", "user1", &snap).await.unwrap();

    assert!(store.load("app1", "user1").await.unwrap().is_some());
    assert!(store.load("app2", "user1").await.unwrap().is_none());
    assert!(store.load("app1", "user2").await.unwrap().is_none());
}

// ── SqliteAppStateStore ──────────────────────────────────────────────────────

#[tokio::test]
async fn app_state_store_save_and_load() {
    let store = SqliteAppStateStore::in_memory().await.unwrap();
    let snap = SessionSnapshot({
        let mut m = HashMap::new();
        m.insert("rate_limit".into(), json!(100));
        m
    });
    store.save("myapp", &snap).await.unwrap();

    let loaded = store.load("myapp").await.unwrap().unwrap();
    assert_eq!(loaded.0.get("rate_limit").unwrap(), &json!(100));
}

#[tokio::test]
async fn app_state_store_overwrite() {
    let store = SqliteAppStateStore::in_memory().await.unwrap();
    let snap1 = SessionSnapshot({
        let mut m = HashMap::new();
        m.insert("v".into(), json!(1));
        m
    });
    let snap2 = SessionSnapshot({
        let mut m = HashMap::new();
        m.insert("v".into(), json!(2));
        m
    });
    store.save("myapp", &snap1).await.unwrap();
    store.save("myapp", &snap2).await.unwrap();

    let loaded = store.load("myapp").await.unwrap().unwrap();
    assert_eq!(loaded.0.get("v").unwrap(), &json!(2));
}

#[tokio::test]
async fn app_state_store_delete() {
    let store = SqliteAppStateStore::in_memory().await.unwrap();
    let snap = SessionSnapshot(HashMap::new());
    store.save("myapp", &snap).await.unwrap();
    store.delete("myapp").await.unwrap();
    assert!(store.load("myapp").await.unwrap().is_none());
}

#[tokio::test]
async fn app_state_store_scoping() {
    let store = SqliteAppStateStore::in_memory().await.unwrap();
    let snap = SessionSnapshot(HashMap::new());
    store.save("app1", &snap).await.unwrap();

    assert!(store.load("app1").await.unwrap().is_some());
    assert!(store.load("app2").await.unwrap().is_none());
}
