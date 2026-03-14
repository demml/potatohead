use potato_agent::{SessionSnapshot, SessionState};
use serde_json::json;

#[test]
fn session_state_get_set_remove() {
    let state = SessionState::new();
    assert!(state.get("key").is_none());

    state.set("key", json!("value"));
    assert_eq!(state.get("key").unwrap(), json!("value"));

    let removed = state.remove("key");
    assert_eq!(removed.unwrap(), json!("value"));
    assert!(state.get("key").is_none());
}

#[test]
fn session_state_snapshot() {
    let state = SessionState::new();
    state.set("a", json!(1));
    state.set("b", json!(2));

    let snap = state.snapshot();
    assert_eq!(snap.len(), 2);
    assert_eq!(snap.get("a").unwrap(), &json!(1));
}

#[test]
fn session_state_merge() {
    let state = SessionState::new();
    state.set("a", json!(1));

    let mut other = std::collections::HashMap::new();
    other.insert("a".into(), json!(10));
    other.insert("b".into(), json!(2));
    state.merge(other);

    assert_eq!(state.get("a").unwrap(), json!(10)); // overwritten
    assert_eq!(state.get("b").unwrap(), json!(2));
}

#[test]
fn session_state_ancestor_tracking() {
    let state = SessionState::new();
    assert!(!state.is_ancestor("agent-1"));

    state.push_ancestor("agent-1");
    assert!(state.is_ancestor("agent-1"));
    assert!(!state.is_ancestor("agent-2"));

    state.push_ancestor("agent-2");
    assert!(state.is_ancestor("agent-2"));

    state.pop_ancestor();
    assert!(!state.is_ancestor("agent-2"));
    assert!(state.is_ancestor("agent-1"));
}

#[test]
fn session_snapshot_serde_roundtrip() {
    let state = SessionState::new();
    state.set("count", json!(42));
    state.set("name", json!("test"));

    let snapshot = SessionSnapshot::from(&state);
    let json = serde_json::to_string(&snapshot).unwrap();
    let restored: SessionSnapshot = serde_json::from_str(&json).unwrap();

    let restored_state = SessionState::from(restored);
    assert_eq!(restored_state.get("count").unwrap(), json!(42));
    assert_eq!(restored_state.get("name").unwrap(), json!("test"));
}

#[test]
fn session_snapshot_from_state_and_back() {
    let state = SessionState::new();
    state.set("x", json!("hello"));

    let snapshot = SessionSnapshot::from(&state);
    let state2 = SessionState::from(snapshot);

    assert_eq!(state2.get("x").unwrap(), json!("hello"));
}
