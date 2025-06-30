use baked_potato::OpenAITestServer;
use potato_agents::{Agent, Provider, Task};
use potato_prompts::prompt::{Message, Prompt, PromptContent};
use potato_workflow::Workflow;

#[test]
fn test_workflow() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = OpenAITestServer::new();
    mock.start_server().unwrap();

    let user_content = PromptContent::Str("Hello, how are you?".to_string());
    let system_content = PromptContent::Str("You are a helpful assistant.".to_string());
    let prompt = Prompt::new_rs(
        vec![Message::new_rs(user_content)],
        Some("gpt-4o"),
        Some("openai"),
        vec![Message::new_rs(system_content)],
        None,
        None,
    )
    .unwrap();

    let mut workflow = Workflow::new("My Workflow");

    let agent1 = Agent::new(Provider::OpenAI, None).unwrap();
    let agent2 = Agent::new(Provider::OpenAI, None).unwrap();

    workflow.add_agent(&agent1);
    workflow.add_agent(&agent2);

    // add a task to the workflow
    workflow.add_task(Task::new(&agent1.id, prompt.clone(), "task1", None, None));
    workflow.add_task(Task::new(&agent2.id, prompt.clone(), "task2", None, None));
    workflow.add_task(Task::new(
        &agent2.id,
        prompt.clone(),
        "task3",
        Some(vec!["task1".to_string(), "task2".to_string()]),
        None,
    ));
    workflow.add_task(Task::new(
        &agent1.id,
        prompt.clone(),
        "task4",
        Some(vec!["task3".to_string()]),
        None,
    ));

    // add final task
    workflow.add_task(Task::new(
        &agent1.id,
        prompt.clone(),
        "final_task",
        Some(vec!["task3".to_string(), "task4".to_string()]),
        None,
    ));

    // print execution plan
    println!("Execution Plan: {:?}", workflow.execution_plan());

    // run the workflow
    runtime.block_on(async {
        workflow.run().await.unwrap();
    });

    // assert workflow tasks are completed
    assert!(workflow.is_complete());

    // assert pending count is 0
    assert_eq!(workflow.pending_count(), 0);

    mock.stop_server().unwrap();
}
