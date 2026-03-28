use baked_potato::LLMTestServer;
use potato_agent::{AgentRunOutcome, AgentRunner, SessionState};
use potato_spec::{SpecError, SpecLoader};
use potato_type::openai::v1::chat::request::{
    ChatMessage as OpenAIChatMessage, ContentPart, TextContentPart,
};
use potato_type::prompt::{MessageNum, Prompt, ResponseType};
use potato_type::Provider;

const SINGLE_AGENT_YAML: &str = r#"
agents:
  - id: assistant
    provider: openai
    model: gpt-4o
    system_prompt: "You are a helpful assistant."
    max_iterations: 1
workflows: []
"#;

const SEQUENTIAL_YAML: &str = r#"
agents:
  - id: step_one
    provider: openai
    model: gpt-4o
    max_iterations: 1
  - id: step_two
    provider: openai
    model: gpt-4o
    max_iterations: 1
workflows:
  - id: pipeline
    type: sequential
    pass_output: true
    steps:
      - ref: step_one
      - ref: step_two
"#;

const PARALLEL_YAML: &str = r#"
agents:
  - id: agent_a
    provider: openai
    model: gpt-4o
    max_iterations: 1
  - id: agent_b
    provider: openai
    model: gpt-4o
    max_iterations: 1
workflows:
  - id: par
    type: parallel
    merge_strategy: collect_all
    steps:
      - ref: agent_a
      - ref: agent_b
"#;

const DAG_YAML: &str = r#"
agents:
  - id: worker
    provider: openai
    model: gpt-4o
    max_iterations: 1
workflows:
  - id: dag
    type: workflow
    tasks:
      - id: t1
        agent: worker
        prompt: "Hello"
        dependencies: []
      - id: t2
        agent: worker
        prompt: "Follow up"
        dependencies: [t1]
"#;

/// This test is performed in a sync context in order to maintain compatibility with python (LLMTestServer can be used in rust and python)
/// Because of this, we need to use a tokio runtime to run the async code within the test.
#[test]
fn spec_loader_builds_openai_agent_and_runs() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = LLMTestServer::new();
    mock.start_server().unwrap();

    let loaded = runtime
        .block_on(async { SpecLoader::from_spec(SINGLE_AGENT_YAML).await })
        .unwrap();

    let agent = loaded.agent("assistant").expect("agent 'assistant' not found");

    let prompt_msg = OpenAIChatMessage {
        role: "user".to_string(),
        content: vec![ContentPart::Text(TextContentPart::new("Hello!".to_string()))],
        name: None,
    };
    let prompt = Prompt::new_rs(
        vec![MessageNum::OpenAIMessageV1(prompt_msg)],
        "gpt-4o",
        Provider::OpenAI,
        vec![],
        None,
        None,
        ResponseType::Null,
    )
    .unwrap();

    let response = runtime
        .block_on(async { agent.execute_prompt(&prompt).await })
        .unwrap();

    assert!(!response.response_text().is_empty());

    mock.stop_server().unwrap();
}

#[test]
fn spec_loader_builds_sequential_workflow_and_runs() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = LLMTestServer::new();
    mock.start_server().unwrap();

    let loaded = runtime
        .block_on(async { SpecLoader::from_spec(SEQUENTIAL_YAML).await })
        .unwrap();

    let seq = loaded.sequential("pipeline").expect("sequential 'pipeline' not found");

    let mut session = SessionState::new();
    let outcome = runtime
        .block_on(async { seq.run("Start", &mut session).await })
        .unwrap();

    match outcome {
        AgentRunOutcome::Complete(result) => {
            assert!(!result.final_response.response_text().is_empty());
        }
        _ => panic!("expected Complete outcome"),
    }

    mock.stop_server().unwrap();
}

#[test]
fn spec_loader_builds_parallel_workflow_and_runs() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = LLMTestServer::new();
    mock.start_server().unwrap();

    let loaded = runtime
        .block_on(async { SpecLoader::from_spec(PARALLEL_YAML).await })
        .unwrap();

    let par = loaded.parallel("par").expect("parallel 'par' not found");

    let mut session = SessionState::new();
    let outcome = runtime
        .block_on(async { par.run("Start", &mut session).await })
        .unwrap();

    match outcome {
        AgentRunOutcome::Complete(result) => {
            assert!(!result.final_response.response_text().is_empty());
        }
        _ => panic!("expected Complete outcome"),
    }

    mock.stop_server().unwrap();
}

#[test]
fn spec_loader_builds_dag_workflow_and_runs() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = LLMTestServer::new();
    mock.start_server().unwrap();

    let loaded = runtime
        .block_on(async { SpecLoader::from_spec(DAG_YAML).await })
        .unwrap();

    let wf = loaded.workflow("dag").expect("workflow 'dag' not found");
    let result = runtime.block_on(async { wf.run(None).await });
    assert!(result.is_ok());

    mock.stop_server().unwrap();
}

#[test]
fn spec_loader_dag_task_with_prompt_file_runs() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = LLMTestServer::new();
    mock.start_server().unwrap();

    let prompt_path = format!(
        "{}/tests/agent/fixtures/prompt.yaml",
        env!("CARGO_MANIFEST_DIR")
    );

    let yaml = format!(
        r#"
agents:
  - id: worker
    provider: openai
    model: gpt-4o
    max_iterations: 1
workflows:
  - id: dag
    type: workflow
    tasks:
      - id: t1
        agent: worker
        prompt:
          path: "{}"
        dependencies: []
"#,
        prompt_path
    );

    let loaded = runtime
        .block_on(async { SpecLoader::from_spec(&yaml).await })
        .unwrap();

    let prompt = Prompt::from_path(std::path::PathBuf::from(&prompt_path)).unwrap();
    assert_eq!(prompt.model, "gpt-4o");
    assert_eq!(prompt.provider, Provider::OpenAI);
    assert!(!prompt.openai_messages().unwrap().messages.is_empty());

    let wf = loaded.workflow("dag").expect("workflow 'dag' not found");
    let result = runtime.block_on(async { wf.run(None).await });
    assert!(result.is_ok());

    mock.stop_server().unwrap();
}

#[test]
fn spec_loader_dag_task_prompt_file_not_found_returns_error() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let yaml = r#"
agents:
  - id: worker
    provider: openai
    model: gpt-4o
    max_iterations: 1
workflows:
  - id: dag
    type: workflow
    tasks:
      - id: t1
        agent: worker
        prompt:
          path: "/nonexistent/path/prompt.yaml"
        dependencies: []
"#;

    let result = runtime.block_on(async { SpecLoader::from_spec(yaml).await });
    assert!(
        matches!(result, Err(SpecError::PromptLoad { .. })),
        "expected PromptLoad error for nonexistent prompt file"
    );
}

#[test]
fn spec_loader_dag_agent_missing_model_returns_error() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let yaml = r#"
agents:
  - id: no_model_agent
    provider: openai
workflows:
  - id: dag_bad
    type: workflow
    tasks:
      - id: t1
        agent: no_model_agent
        prompt: "Hello"
        dependencies: []
"#;

    let result = runtime.block_on(async { SpecLoader::from_spec(yaml).await });
    assert!(
        matches!(result, Err(SpecError::WorkflowBuild { .. })),
        "expected WorkflowBuild error for dag agent with no model"
    );
}

#[test]
fn spec_loader_loads_agent_from_file() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let path = format!(
        "{}/tests/agent/fixtures/spec.yaml",
        env!("CARGO_MANIFEST_DIR")
    );
    let loaded = runtime
        .block_on(async { SpecLoader::from_spec_path(&path).await })
        .unwrap();

    assert!(loaded.agent("file_agent").is_some());
}

#[test]
fn spec_loader_file_not_found_returns_io_error() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let result = runtime.block_on(async {
        SpecLoader::from_spec_path("/nonexistent/path/spec.yaml").await
    });

    assert!(
        matches!(result, Err(SpecError::Io(_))),
        "expected Io error for nonexistent path"
    );
}

#[test]
fn spec_loader_accessor_returns_none_for_unknown_id() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let loaded = runtime
        .block_on(async { SpecLoader::from_spec(SINGLE_AGENT_YAML).await })
        .unwrap();

    assert!(loaded.agent("nonexistent").is_none());
    assert!(loaded.sequential("nonexistent").is_none());
    assert!(loaded.parallel("nonexistent").is_none());
    assert!(loaded.workflow("nonexistent").is_none());
}
