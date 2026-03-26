use potato_spec::spec::*;
use potato_spec::{SpecError, SpecLoader};

const SIMPLE_YAML: &str = include_str!("fixtures/simple.yaml");

// --- deserialization-only tests (no API keys needed) ---

#[test]
fn test_parse_simple_yaml() {
    let spec: PotatoSpec = serde_yaml::from_str(SIMPLE_YAML).unwrap();
    assert_eq!(spec.agents.len(), 1);
    assert_eq!(spec.agents[0].id, "summarizer");
    assert_eq!(spec.agents[0].provider, "anthropic");
    assert_eq!(spec.agents[0].model.as_deref(), Some("claude-haiku-4-5"));
    assert_eq!(spec.agents[0].max_iterations, Some(3));
    assert_eq!(spec.workflows.len(), 1);
}

#[test]
fn test_parse_memory_windowed() {
    let yaml = r#"
agents:
  - id: a
    provider: anthropic
    memory:
      type: windowed
      window_size: 10
workflows: []
"#;
    let spec: PotatoSpec = serde_yaml::from_str(yaml).unwrap();
    let mem = spec.agents[0].memory.as_ref().unwrap();
    assert!(matches!(mem, MemorySpec::Windowed { window_size: 10 }));
}

#[test]
fn test_parse_memory_in_memory() {
    let yaml = r#"
agents:
  - id: a
    provider: openai
    memory:
      type: in_memory
workflows: []
"#;
    let spec: PotatoSpec = serde_yaml::from_str(yaml).unwrap();
    let mem = spec.agents[0].memory.as_ref().unwrap();
    assert!(matches!(mem, MemorySpec::InMemory));
}

#[test]
fn test_parse_criteria_all_types() {
    let yaml = r#"
agents:
  - id: a
    provider: anthropic
    criteria:
      - type: max_iterations
        max: 5
      - type: keyword
        keyword: "STOP"
      - type: structured_output
workflows: []
"#;
    let spec: PotatoSpec = serde_yaml::from_str(yaml).unwrap();
    let criteria = &spec.agents[0].criteria;
    assert_eq!(criteria.len(), 3);
    assert!(matches!(
        criteria[0],
        CriteriaSpec::MaxIterations { max: 5 }
    ));
    assert!(matches!(&criteria[1], CriteriaSpec::Keyword { keyword } if keyword == "STOP"));
    assert!(matches!(criteria[2], CriteriaSpec::StructuredOutput { .. }));
}

#[test]
fn test_parse_callback_builtin() {
    let yaml = r#"
agents:
  - id: a
    provider: anthropic
    callbacks:
      - type: logging
workflows: []
"#;
    let spec: PotatoSpec = serde_yaml::from_str(yaml).unwrap();
    let cb = &spec.agents[0].callbacks[0];
    assert!(matches!(cb, CallbackSpec::BuiltIn { kind } if kind == "logging"));
}

#[test]
fn test_parse_callback_named() {
    let yaml = r#"
agents:
  - id: a
    provider: anthropic
    callbacks:
      - name: my_callback
workflows: []
"#;
    let spec: PotatoSpec = serde_yaml::from_str(yaml).unwrap();
    let cb = &spec.agents[0].callbacks[0];
    assert!(matches!(cb, CallbackSpec::Named { name } if name == "my_callback"));
}

#[test]
fn test_parse_sequential_workflow() {
    let spec: PotatoSpec = serde_yaml::from_str(SIMPLE_YAML).unwrap();
    let wf = &spec.workflows[0];
    assert!(matches!(wf, WorkflowSpec::Sequential { id, .. } if id == "simple_pipeline"));
    if let WorkflowSpec::Sequential {
        steps, pass_output, ..
    } = wf
    {
        assert_eq!(steps.len(), 2);
        assert_eq!(*pass_output, Some(true));
        assert!(matches!(&steps[0], StepSpec::Ref { agent_ref } if agent_ref == "summarizer"));
        assert!(matches!(&steps[1], StepSpec::Inline(_)));
    }
}

#[test]
fn test_parse_parallel_workflow() {
    let yaml = r#"
agents: []
workflows:
  - id: par
    type: parallel
    merge_strategy: collect_all
    steps:
      - ref: agent_a
      - ref: agent_b
"#;
    let spec: PotatoSpec = serde_yaml::from_str(yaml).unwrap();
    let wf = &spec.workflows[0];
    assert!(matches!(
        wf,
        WorkflowSpec::Parallel {
            merge_strategy: Some(MergeStrategySpec::CollectAll),
            ..
        }
    ));
}

#[test]
fn test_parse_dag_workflow() {
    let yaml = r#"
agents: []
workflows:
  - id: dag
    type: workflow
    tasks:
      - id: t1
        agent: a1
        prompt: "Do something"
        dependencies: []
      - id: t2
        agent: a2
        prompt: "Do something else"
        dependencies: [t1]
"#;
    let spec: PotatoSpec = serde_yaml::from_str(yaml).unwrap();
    let wf = &spec.workflows[0];
    if let WorkflowSpec::Workflow { tasks, .. } = wf {
        assert_eq!(tasks.len(), 2);
        assert!(tasks[1].dependencies.contains(&"t1".to_string()));
    } else {
        panic!("expected Workflow variant");
    }
}

#[test]
fn test_deserialize_dag_workflow_out_of_order() {
    // Verifies YAML deserialization preserves declaration order (t2 before t1).
    // Sort correctness is tested separately in loader unit tests.
    let yaml = r#"
agents: []
workflows:
  - id: dag_rev
    type: workflow
    tasks:
      - id: t2
        agent: a2
        prompt: "Second step"
        dependencies: [t1]
      - id: t1
        agent: a1
        prompt: "First step"
        dependencies: []
"#;
    let spec: PotatoSpec = serde_yaml::from_str(yaml).unwrap();
    if let WorkflowSpec::Workflow { tasks, .. } = &spec.workflows[0] {
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].id, "t2");
        assert_eq!(tasks[1].id, "t1");
        assert!(tasks[0].dependencies.contains(&"t1".to_string()));
    } else {
        panic!("expected Workflow variant");
    }
}

// --- error path tests (fail before provider initialization) ---

fn assert_spec_error<T>(result: Result<T, SpecError>, check: impl FnOnce(SpecError)) {
    match result {
        Ok(_) => panic!("expected error but got Ok"),
        Err(e) => check(e),
    }
}

#[tokio::test]
async fn test_unknown_tool_returns_error() {
    let yaml = r#"
agents:
  - id: a
    provider: anthropic
    tools:
      - name: nonexistent_tool
workflows: []
"#;
    let loader = SpecLoader::new();
    assert_spec_error(loader.load_str(yaml).await, |e| {
        assert!(matches!(e, SpecError::UnknownTool { name } if name == "nonexistent_tool"));
    });
}

#[tokio::test]
async fn test_unknown_named_callback_returns_error() {
    let yaml = r#"
agents:
  - id: a
    provider: anthropic
    callbacks:
      - name: not_registered
workflows: []
"#;
    let loader = SpecLoader::new();
    assert_spec_error(loader.load_str(yaml).await, |e| {
        assert!(matches!(e, SpecError::UnknownCallback { name } if name == "not_registered"));
    });
}

#[tokio::test]
async fn test_unknown_builtin_callback_returns_error() {
    let yaml = r#"
agents:
  - id: a
    provider: anthropic
    callbacks:
      - type: unknown_kind
workflows: []
"#;
    let loader = SpecLoader::new();
    assert_spec_error(loader.load_str(yaml).await, |e| {
        assert!(matches!(e, SpecError::UnknownCallback { name } if name == "unknown_kind"));
    });
}

#[tokio::test]
async fn test_invalid_provider_returns_error() {
    let yaml = r#"
agents:
  - id: a
    provider: not_a_real_provider
workflows: []
"#;
    let loader = SpecLoader::new();
    assert_spec_error(loader.load_str(yaml).await, |e| {
        assert!(
            matches!(e, SpecError::InvalidProvider { value, .. } if value == "not_a_real_provider")
        );
    });
}

#[tokio::test]
async fn test_unknown_agent_ref_in_sequential_returns_error() {
    let yaml = r#"
agents: []
workflows:
  - id: seq
    type: sequential
    steps:
      - ref: ghost_agent
"#;
    let loader = SpecLoader::new();
    assert_spec_error(loader.load_str(yaml).await, |e| {
        assert!(matches!(e, SpecError::UnknownAgentRef { id } if id == "ghost_agent"));
    });
}

#[tokio::test]
async fn test_unknown_agent_ref_in_dag_returns_error() {
    let yaml = r#"
agents: []
workflows:
  - id: dag
    type: workflow
    tasks:
      - id: t1
        agent: missing_agent
        prompt: "Do it"
        dependencies: []
"#;
    let loader = SpecLoader::new();
    assert_spec_error(loader.load_str(yaml).await, |e| {
        assert!(matches!(e, SpecError::UnknownAgentRef { id } if id == "missing_agent"));
    });
}
