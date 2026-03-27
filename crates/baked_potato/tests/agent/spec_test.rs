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
fn spec_loader_invalid_provider_returns_error() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let yaml = r#"
agents:
  - id: bad
    provider: not_a_real_provider
workflows: []
"#;

    let result = runtime.block_on(async { SpecLoader::from_spec(yaml).await });

    assert!(
        matches!(result, Err(SpecError::InvalidProvider { ref value, .. }) if value == "not_a_real_provider"),
        "expected InvalidProvider error"
    );
}
