use baked_potato::OpenAITestServer;
use potato_agent::{Agent, Provider, Task};
use potato_prompt::prompt::{Message, Prompt, PromptContent};

/// This test is performed in a sync context in order to maintain compatibility with python (OpenAITestServer can be used in rust and python)
/// Because of this, we need to use a tokio runtime to run the async code within the test.
#[test]
fn test_agent() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = OpenAITestServer::new();
    mock.start_server().unwrap();

    let prompt_content = PromptContent::Str("Test prompt. ${param1} ${param2}".to_string());
    let prompt = Prompt::new_rs(
        vec![Message::new_rs(prompt_content)],
        Some("gpt-4o"),
        Some("openai"),
        vec![],
        None,
        None,
    )
    .unwrap();

    let agent = Agent::new(Provider::OpenAI, None).unwrap();
    let task = Task::new(&agent.id, prompt, "task1", None, None);

    runtime.block_on(async {
        agent.execute_async_task(&task).await.unwrap();
    });

    mock.stop_server().unwrap();
}
