use baked_potato::OpenAITestServer;
use potato_agent::{Agent, Provider, Task};
use potato_prompt::prompt::{Message, Prompt, PromptContent};
use potato_type::StructuredOutput;
use potato_workflow::Workflow;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

#[allow(dead_code)]
#[derive(Debug, JsonSchema, Deserialize)]
struct Parameters {
    variable1: i32,
    variable2: i32,
}
impl StructuredOutput for Parameters {}

fn create_prompt(response_format: Option<Value>) -> Prompt {
    let user_content = PromptContent::Str("Hello, how are you?".to_string());
    let system_content = PromptContent::Str("You are a helpful assistant.".to_string());
    Prompt::new_rs(
        vec![Message::new_rs(user_content)],
        Some("gpt-4o"),
        Some("openai"),
        vec![Message::new_rs(system_content)],
        None,
        response_format,
    )
    .unwrap()
}

fn create_parameterized_prompt() -> Prompt {
    let user_content = PromptContent::Str("What is ${variable1} + ${variable2}?".to_string());
    let system_content = PromptContent::Str("You are a helpful assistant.".to_string());
    Prompt::new_rs(
        vec![Message::new_rs(user_content)],
        Some("gpt-4o"),
        Some("openai"),
        vec![Message::new_rs(system_content)],
        None,
        None,
    )
    .unwrap()
}

#[test]
fn test_workflow() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = OpenAITestServer::new();
    mock.start_server().unwrap();

    let prompt = create_prompt(None);
    let mut workflow = Workflow::new("My Workflow");

    let agent1 = Agent::new(Provider::OpenAI, None).unwrap();
    let agent2 = Agent::new(Provider::OpenAI, None).unwrap();

    workflow.add_agent(&agent1);
    workflow.add_agent(&agent2);

    // add a task to the workflow
    workflow
        .add_task(Task::new(&agent1.id, prompt.clone(), "task1", None, None))
        .unwrap();
    workflow
        .add_task(Task::new(&agent2.id, prompt.clone(), "task2", None, None))
        .unwrap();
    workflow
        .add_task(Task::new(
            &agent2.id,
            prompt.clone(),
            "task3",
            Some(vec!["task1".to_string(), "task2".to_string()]),
            None,
        ))
        .unwrap();
    workflow
        .add_task(Task::new(
            &agent1.id,
            prompt.clone(),
            "task4",
            Some(vec!["task3".to_string()]),
            None,
        ))
        .unwrap();

    // add final task
    workflow
        .add_task(Task::new(
            &agent1.id,
            prompt.clone(),
            "final_task",
            Some(vec!["task3".to_string(), "task4".to_string()]),
            None,
        ))
        .unwrap();

    assert_eq!(workflow.tasklist.len(), 5);
    assert!(!workflow.tasklist.is_empty());

    // print execution plan
    println!("Execution Plan: {:?}", workflow.execution_plan());

    // run the workflow
    runtime.block_on(async {
        workflow.run().await.unwrap();
    });

    mock.stop_server().unwrap();
}

#[test]
fn test_parameterized_workflow() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = OpenAITestServer::new();
    mock.start_server().unwrap();

    let prompt = create_prompt(Some(Parameters::get_structured_output_schema()));
    let parameterized_prompt = create_parameterized_prompt();

    // assert 2 variables are in the prompt
    assert_eq!(parameterized_prompt.parameters.len(), 2);

    let mut workflow = Workflow::new("My Workflow");
    let agent1 = Agent::new(Provider::OpenAI, None).unwrap();
    workflow.add_agent(&agent1);

    workflow
        .add_task(Task::new(&agent1.id, prompt.clone(), "task1", None, None))
        .unwrap();

    workflow
        .add_task(Task::new(
            &agent1.id,
            parameterized_prompt.clone(),
            "task2",
            Some(vec!["task1".to_string()]),
            None,
        ))
        .unwrap();

    let result = runtime.block_on(async { workflow.run().await.unwrap() });

    // assert result id is not the same as workflow id
    assert_ne!(result.read().unwrap().id, workflow.id);

    let task1_output = result
        .read()
        .unwrap()
        .tasklist
        .get_task("task1")
        .unwrap()
        .read()
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .content();

    let _ = Parameters::model_validate_json(&task1_output);

    // assert original workflow is unmodified
    assert_eq!(workflow.tasklist.len(), 2);

    // assert workflow event tracker is empty for the original workflow
    assert!(workflow.event_tracker.read().unwrap().is_empty());

    // assert the recent run workflow has events
    assert!(!result
        .read()
        .unwrap()
        .event_tracker
        .read()
        .unwrap()
        .is_empty());

    let binding = result.read().unwrap().tasklist.get_task("task2").unwrap();
    let task2_output = &binding.read().unwrap().prompt.user_message;

    // assert task2_output len is 2
    assert_eq!(task2_output.len(), 2);
}
