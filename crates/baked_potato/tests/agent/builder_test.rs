use potato_agent::{
    AgentBuilder, AgentRunOutcome, AgentRunner, SessionState, SqliteAppStateStore,
    SqliteMemoryStore, SqliteSessionStore, SqliteUserStateStore,
};
use potato_type::Provider;
use std::sync::Arc;

#[test]
fn builder_missing_provider_error() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let result = runtime.block_on(async { AgentBuilder::new().model("gpt-4o").build().await });

    assert!(result.is_err());
    mock.stop_server().unwrap();
}

#[test]
fn builder_minimal_build() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let agent = runtime.block_on(async {
        AgentBuilder::new()
            .provider(Provider::OpenAI)
            .model("gpt-4o")
            .build()
            .await
            .unwrap()
    });

    assert!(!agent.id.is_empty());
    mock.stop_server().unwrap();
}

#[test]
fn builder_full_config() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let agent = runtime.block_on(async {
        let mem_store = Arc::new(SqliteMemoryStore::in_memory().await.unwrap());
        let sess_store = Arc::new(SqliteSessionStore::in_memory().await.unwrap());
        let user_store = Arc::new(SqliteUserStateStore::in_memory().await.unwrap());
        let app_store = Arc::new(SqliteAppStateStore::in_memory().await.unwrap());

        AgentBuilder::new()
            .provider(Provider::OpenAI)
            .model("gpt-4o")
            .system_prompt("You are a helpful assistant.")
            .max_iterations(5)
            .app_name("test-app")
            .user_id("user-1")
            .with_memory_store("sess-1", mem_store)
            .with_session_store("sess-1", sess_store)
            .with_user_state_store(user_store)
            .with_app_state_store(app_store)
            .stop_on_keyword("DONE")
            .stop_on_structured_output(None)
            .build()
            .await
            .unwrap()
    });

    assert!(agent.app_name.as_deref() == Some("test-app"));
    assert!(agent.user_id.as_deref() == Some("user-1"));
    assert!(agent.session_id.as_deref() == Some("sess-1"));
    assert!(agent.session_store.is_some());
    assert!(agent.user_state_store.is_some());
    assert!(agent.app_state_store.is_some());
    assert!(agent.memory.is_some());
    assert_eq!(agent.criteria.len(), 2);

    mock.stop_server().unwrap();
}

#[test]
fn builder_with_sub_agent() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let agent = runtime.block_on(async {
        let child = AgentBuilder::new()
            .provider(Provider::OpenAI)
            .model("gpt-4o")
            .build()
            .await
            .unwrap();

        AgentBuilder::new()
            .provider(Provider::OpenAI)
            .model("gpt-4o")
            .with_sub_agent("child", "a child agent", child as Arc<dyn AgentRunner>)
            .build()
            .await
            .unwrap()
    });

    // Sub-agent should be registered as an async tool
    let registry = agent.tools.read().unwrap();
    assert!(registry.get_async_tool("child").is_some());

    mock.stop_server().unwrap();
}

#[test]
fn builder_run_basic() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let agent = runtime.block_on(async {
        AgentBuilder::new()
            .provider(Provider::OpenAI)
            .model("gpt-4o")
            .max_iterations(1)
            .build()
            .await
            .unwrap()
    });

    let mut session = SessionState::new();
    let outcome = runtime.block_on(async { agent.run("hello", &mut session).await.unwrap() });

    match outcome {
        AgentRunOutcome::Complete(result) => {
            assert!(!result.final_response.response_text().is_empty());
        }
        _ => panic!("Expected Complete outcome"),
    }

    mock.stop_server().unwrap();
}
