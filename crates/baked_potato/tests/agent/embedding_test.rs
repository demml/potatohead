use baked_potato::LLMTestServer;
use potato_agent::{Agent, Task};
use potato_prompt::{
    prompt::{Message, Prompt, PromptContent, ResponseType},
    Score,
};
use potato_type::Provider;
use potato_type::StructuredOutput;

/// This test is performed in a sync context in order to maintain compatibility with python (LLMTestServer can be used in rust and python)
/// Because of this, we need to use a tokio runtime to run the async code within the test.
#[test]
fn test_openai_agent() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = LLMTestServer::new();
    mock.start_server().unwrap();

    let prompt_content = PromptContent::Str("Test prompt. ${param1} ${param2}".to_string());
    let prompt = Prompt::new_rs(
        vec![Message::new_rs(prompt_content)],
        "gpt-4o",
        Provider::OpenAI,
        vec![],
        None,
        None,
        ResponseType::Null,
    )
    .unwrap();

    let agent = Agent::new(Provider::OpenAI, None).unwrap();
    let task = Task::new(&agent.id, prompt, "task1", None, None);

    runtime.block_on(async {
        agent.execute_task(&task).await.unwrap();
    });

    runtime.block_on(async {
        agent.execute_prompt(&task.prompt).await.unwrap();
    });

    mock.stop_server().unwrap();
}
