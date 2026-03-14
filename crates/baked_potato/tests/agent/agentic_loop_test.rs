use potato_agent::{
    AgentBuilder, AgentRunOutcome, AgentRunner, MemoryStore, SessionState, SessionStore,
    SqliteMemoryStore, SqliteSessionStore,
};
use potato_type::tools::Tool;
use potato_type::Provider;
use potato_type::TypeError;
use serde_json::{json, Value};
use std::sync::Arc;

/// A simple synchronous test tool that returns a fixed dice roll result.
#[derive(Debug)]
struct RollDiceTool;

impl Tool for RollDiceTool {
    fn name(&self) -> &str {
        "roll_dice"
    }
    fn description(&self) -> &str {
        "Roll a dice with N sides"
    }
    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "sides": {"type": "integer"}
            },
            "required": ["sides"]
        })
    }
    fn execute(&self, _args: Value) -> Result<Value, TypeError> {
        Ok(json!({"result": 17}))
    }
}

// ── OpenAI Agentic Loop ──────────────────────────────────────────────────────

#[test]
fn openai_tool_call_loop() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();
    mock.enable_tool_call_flow();

    let agent = runtime.block_on(async {
        AgentBuilder::new()
            .provider(Provider::OpenAI)
            .model("gpt-4o")
            .max_iterations(5)
            .with_tool(RollDiceTool)
            .build()
            .await
            .unwrap()
    });

    let mut session = SessionState::new();
    let outcome = runtime.block_on(async { agent.run("Roll a dice", &mut session).await.unwrap() });

    match outcome {
        AgentRunOutcome::Complete(result) => {
            assert!(
                result.iterations >= 1,
                "Expected at least 1 iteration for tool call loop"
            );
            assert!(!result.final_response.response_text().is_empty());
        }
        _ => panic!("Expected Complete outcome"),
    }

    mock.stop_server().unwrap();
}

#[test]
fn openai_no_tools_completes_immediately() {
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
            assert_eq!(result.iterations, 0);
        }
        _ => panic!("Expected Complete outcome"),
    }

    mock.stop_server().unwrap();
}

#[test]
fn openai_max_iterations_stops_loop() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let agent = runtime.block_on(async {
        AgentBuilder::new()
            .provider(Provider::OpenAI)
            .model("gpt-4o")
            .max_iterations(1)
            .with_tool(RollDiceTool)
            .build()
            .await
            .unwrap()
    });

    let mut session = SessionState::new();
    let outcome = runtime.block_on(async { agent.run("Roll a dice", &mut session).await.unwrap() });

    match outcome {
        AgentRunOutcome::Complete(result) => {
            assert!(result.completion_reason.contains("max iterations"));
        }
        _ => panic!("Expected Complete outcome"),
    }

    mock.stop_server().unwrap();
}

#[test]
fn openai_stop_on_keyword() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let agent = runtime.block_on(async {
        AgentBuilder::new()
            .provider(Provider::OpenAI)
            .model("gpt-4o")
            .max_iterations(5)
            .stop_on_keyword("Hello")
            .build()
            .await
            .unwrap()
    });

    let mut session = SessionState::new();
    let outcome = runtime.block_on(async { agent.run("Say hello", &mut session).await.unwrap() });

    match outcome {
        AgentRunOutcome::Complete(result) => {
            assert!(result.completion_reason.contains("keyword"));
        }
        _ => panic!("Expected Complete outcome"),
    }

    mock.stop_server().unwrap();
}

#[test]
fn openai_with_sqlite_memory_store() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let mem_store =
        runtime.block_on(async { Arc::new(SqliteMemoryStore::in_memory().await.unwrap()) });

    let agent = runtime.block_on(async {
        AgentBuilder::new()
            .provider(Provider::OpenAI)
            .model("gpt-4o")
            .max_iterations(1)
            .app_name("testapp")
            .user_id("user1")
            .with_memory_store("sess1", mem_store.clone())
            .build()
            .await
            .unwrap()
    });

    let mut session = SessionState::new();
    runtime.block_on(async {
        let _ = agent.run("hello", &mut session).await.unwrap();
    });

    // Verify turn was persisted
    let count =
        runtime.block_on(async { mem_store.count("testapp", "user1", "sess1").await.unwrap() });
    assert_eq!(count, 1);

    mock.stop_server().unwrap();
}

#[test]
fn openai_with_sqlite_session_store() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let sess_store =
        runtime.block_on(async { Arc::new(SqliteSessionStore::in_memory().await.unwrap()) });

    let agent = runtime.block_on(async {
        AgentBuilder::new()
            .provider(Provider::OpenAI)
            .model("gpt-4o")
            .max_iterations(1)
            .app_name("testapp")
            .user_id("user1")
            .with_session_store("sess1", sess_store.clone())
            .build()
            .await
            .unwrap()
    });

    let mut session = SessionState::new();
    session.set("test_key", json!("test_value"));
    runtime.block_on(async {
        let _ = agent.run("hello", &mut session).await.unwrap();
    });

    // Verify session snapshot was persisted
    let loaded =
        runtime.block_on(async { sess_store.load("testapp", "user1", "sess1").await.unwrap() });
    assert!(loaded.is_some());

    mock.stop_server().unwrap();
}

// ── Anthropic Agentic Loop ───────────────────────────────────────────────────

#[test]
fn anthropic_tool_call_loop() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();
    mock.enable_tool_call_flow();

    let agent = runtime.block_on(async {
        AgentBuilder::new()
            .provider(Provider::Anthropic)
            .model("claude-sonnet-4-5")
            .max_iterations(5)
            .with_tool(RollDiceTool)
            .build()
            .await
            .unwrap()
    });

    let mut session = SessionState::new();
    let outcome = runtime.block_on(async { agent.run("Roll a dice", &mut session).await.unwrap() });

    match outcome {
        AgentRunOutcome::Complete(result) => {
            assert!(result.iterations >= 1);
        }
        _ => panic!("Expected Complete outcome"),
    }

    mock.stop_server().unwrap();
}

// ── Gemini Agentic Loop ──────────────────────────────────────────────────────

#[test]
fn gemini_tool_call_loop() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();
    mock.enable_tool_call_flow();

    let agent = runtime.block_on(async {
        AgentBuilder::new()
            .provider(Provider::Gemini)
            .model("gemini-2.5-flash")
            .max_iterations(5)
            .with_tool(RollDiceTool)
            .build()
            .await
            .unwrap()
    });

    let mut session = SessionState::new();
    let outcome = runtime.block_on(async { agent.run("Roll a dice", &mut session).await.unwrap() });

    match outcome {
        AgentRunOutcome::Complete(result) => {
            assert!(result.iterations >= 1);
        }
        _ => panic!("Expected Complete outcome"),
    }

    mock.stop_server().unwrap();
}
