use potato_agent::{
    AgentBuilder, AgentRunOutcome, AgentRunner, MergeStrategy, ParallelAgentBuilder,
    SequentialAgentBuilder, SessionState,
};
use potato_type::Provider;
use std::sync::Arc;

fn build_agent(runtime: &tokio::runtime::Runtime, provider: Provider) -> Arc<dyn AgentRunner> {
    runtime.block_on(async {
        AgentBuilder::new()
            .provider(provider)
            .model("gpt-4o")
            .max_iterations(1)
            .build()
            .await
            .unwrap() as Arc<dyn AgentRunner>
    })
}

// ── Sequential ───────────────────────────────────────────────────────────────

#[test]
fn sequential_a_then_b_pass_output_true() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let a = build_agent(&runtime, Provider::OpenAI);
    let b = build_agent(&runtime, Provider::OpenAI);

    let seq = SequentialAgentBuilder::new()
        .then(a)
        .then(b)
        .pass_output(true)
        .build();

    let mut session = SessionState::new();
    let outcome = runtime.block_on(async { seq.run("Start", &mut session).await.unwrap() });

    match outcome {
        AgentRunOutcome::Complete(result) => {
            assert!(!result.final_response.response_text().is_empty());
        }
        _ => panic!("Expected Complete outcome"),
    }

    mock.stop_server().unwrap();
}

#[test]
fn sequential_a_then_b_pass_output_false() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let a = build_agent(&runtime, Provider::OpenAI);
    let b = build_agent(&runtime, Provider::OpenAI);

    let seq = SequentialAgentBuilder::new()
        .then(a)
        .then(b)
        .pass_output(false)
        .build();

    let mut session = SessionState::new();
    let outcome = runtime.block_on(async { seq.run("Start", &mut session).await.unwrap() });

    match outcome {
        AgentRunOutcome::Complete(result) => {
            assert!(!result.final_response.response_text().is_empty());
        }
        _ => panic!("Expected Complete outcome"),
    }

    mock.stop_server().unwrap();
}

#[test]
fn sequential_empty_errors() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let seq = SequentialAgentBuilder::new().build();

    let mut session = SessionState::new();
    let result = runtime.block_on(async { seq.run("Start", &mut session).await });

    assert!(result.is_err());
}

// ── Parallel ─────────────────────────────────────────────────────────────────

#[test]
fn parallel_collect_all() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let a = build_agent(&runtime, Provider::OpenAI);
    let b = build_agent(&runtime, Provider::OpenAI);

    let par = ParallelAgentBuilder::new()
        .with_agent(a)
        .with_agent(b)
        .merge_strategy(MergeStrategy::CollectAll)
        .build();

    let mut session = SessionState::new();
    let outcome = runtime.block_on(async { par.run("Input", &mut session).await.unwrap() });

    match outcome {
        AgentRunOutcome::Complete(result) => {
            assert!(result.completion_reason.contains("parallel"));
            // __parallel_combined should be set in session
            assert!(session.get("__parallel_combined").is_some());
        }
        _ => panic!("Expected Complete outcome"),
    }

    mock.stop_server().unwrap();
}

#[test]
fn parallel_first_strategy() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let a = build_agent(&runtime, Provider::OpenAI);
    let b = build_agent(&runtime, Provider::OpenAI);

    let par = ParallelAgentBuilder::new()
        .with_agent(a)
        .with_agent(b)
        .merge_strategy(MergeStrategy::First)
        .build();

    let mut session = SessionState::new();
    let outcome = runtime.block_on(async { par.run("Input", &mut session).await.unwrap() });

    match outcome {
        AgentRunOutcome::Complete(result) => {
            assert!(!result.final_response.response_text().is_empty());
        }
        _ => panic!("Expected Complete outcome"),
    }

    mock.stop_server().unwrap();
}

// ── Mixed orchestration ──────────────────────────────────────────────────────

#[test]
fn sequential_with_parallel_step() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let a = build_agent(&runtime, Provider::OpenAI);
    let b = build_agent(&runtime, Provider::OpenAI);
    let c = build_agent(&runtime, Provider::OpenAI);

    // Parallel[B, C]
    let par = ParallelAgentBuilder::new()
        .with_agent(b)
        .with_agent(c)
        .merge_strategy(MergeStrategy::CollectAll)
        .build();

    // Sequential[A, Parallel[B, C]]
    let seq = SequentialAgentBuilder::new()
        .then(a)
        .then(par as Arc<dyn AgentRunner>)
        .pass_output(true)
        .build();

    let mut session = SessionState::new();
    let outcome = runtime.block_on(async { seq.run("Start", &mut session).await.unwrap() });

    match outcome {
        AgentRunOutcome::Complete(_) => {
            // B and C should have written combined output
            assert!(session.get("__parallel_combined").is_some());
        }
        _ => panic!("Expected Complete outcome"),
    }

    mock.stop_server().unwrap();
}

#[test]
fn parallel_with_sequential_branches() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let a = build_agent(&runtime, Provider::OpenAI);
    let b = build_agent(&runtime, Provider::OpenAI);
    let c = build_agent(&runtime, Provider::OpenAI);
    let d = build_agent(&runtime, Provider::OpenAI);

    // Sequential[A, B]
    let seq_ab = SequentialAgentBuilder::new()
        .then(a)
        .then(b)
        .pass_output(true)
        .build();

    // Sequential[C, D]
    let seq_cd = SequentialAgentBuilder::new()
        .then(c)
        .then(d)
        .pass_output(true)
        .build();

    // Parallel[Sequential[A,B], Sequential[C,D]]
    let par = ParallelAgentBuilder::new()
        .with_agent(seq_ab as Arc<dyn AgentRunner>)
        .with_agent(seq_cd as Arc<dyn AgentRunner>)
        .merge_strategy(MergeStrategy::CollectAll)
        .build();

    let mut session = SessionState::new();
    let outcome = runtime.block_on(async { par.run("Start", &mut session).await.unwrap() });

    match outcome {
        AgentRunOutcome::Complete(result) => {
            assert!(result.completion_reason.contains("parallel"));
        }
        _ => panic!("Expected Complete outcome"),
    }

    mock.stop_server().unwrap();
}

#[test]
fn session_state_shared_across_sequential() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let a = build_agent(&runtime, Provider::OpenAI);
    let b = build_agent(&runtime, Provider::OpenAI);

    let seq = SequentialAgentBuilder::new()
        .then(a)
        .then(b)
        .pass_output(false)
        .build();

    let mut session = SessionState::new();
    session.set("shared", serde_json::json!("before"));

    runtime.block_on(async { seq.run("Start", &mut session).await.unwrap() });

    // Session should still be accessible and not reset
    assert_eq!(session.get("shared").unwrap(), serde_json::json!("before"));

    mock.stop_server().unwrap();
}
