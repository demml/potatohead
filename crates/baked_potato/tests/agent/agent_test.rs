use std::vec;

use baked_potato::LLMTestServer;
use potato_agent::{Agent, Task};
use potato_type::google::v1::generate::request::{DataNum, GeminiContent, Part};
use potato_type::openai::v1::chat::request::{
    ChatMessage as OpenAIChatMessage, ContentPart, TextContentPart,
};
use potato_type::prompt::Prompt;
use potato_type::prompt::{MessageNum, ResponseType, Score};
use potato_type::Provider;
use potato_type::StructuredOutput;

/// This test is performed in a sync context in order to maintain compatibility with python (LLMTestServer can be used in rust and python)
/// Because of this, we need to use a tokio runtime to run the async code within the test.
#[test]
fn test_openai_agent() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = LLMTestServer::new();
    mock.start_server().unwrap();

    let prompt_content = "Test prompt. ${param1} ${param2}".to_string();
    let prompt_msg = OpenAIChatMessage {
        role: "user".to_string(),
        content: vec![ContentPart::Text(TextContentPart::new(
            prompt_content.clone(),
        ))],
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

    let agent = runtime
        .block_on(async { Agent::new(Provider::OpenAI, None).await })
        .unwrap();
    let task = Task::new(&agent.id, prompt, "task1", None, None);

    runtime.block_on(async {
        agent.execute_task(&task).await.unwrap();
    });

    runtime.block_on(async {
        agent.execute_prompt(&task.prompt).await.unwrap();
    });

    mock.stop_server().unwrap();
}

#[test]
fn test_gemini_agent() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = LLMTestServer::new();
    mock.start_server().unwrap();

    let prompt_content = "You are a helpful assistant".to_string();
    let gemini_msg = MessageNum::GeminiContentV1(GeminiContent {
        role: "user".to_string(),
        parts: vec![Part {
            data: DataNum::Text(prompt_content.clone()),
            ..Default::default()
        }],
    });
    let prompt = Prompt::new_rs(
        vec![gemini_msg],
        "gemini-2.5-flash",
        Provider::Gemini,
        vec![],
        None,
        None,
        ResponseType::Null,
    )
    .unwrap();

    let agent = runtime
        .block_on(async { Agent::new(Provider::Gemini, None).await })
        .unwrap();
    let task = Task::new(&agent.id, prompt, "task1", None, None);

    runtime.block_on(async {
        agent.execute_task(&task).await.unwrap();
    });

    let response = runtime.block_on(async { agent.execute_prompt(&task.prompt).await.unwrap() });

    let content = response.response_text().unwrap();
    assert!(content.contains("AI learns from data to make predictions or decisions"));

    mock.stop_server().unwrap();
}

#[test]
fn test_gemini_score_agent() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = LLMTestServer::new();
    mock.start_server().unwrap();

    let prompt_content =
        "Return a json object with the attributes score and explanation".to_string();

    let gemini_msg = MessageNum::GeminiContentV1(GeminiContent {
        role: "user".to_string(),
        parts: vec![Part {
            data: DataNum::Text(prompt_content.clone()),
            ..Default::default()
        }],
    });
    let prompt = Prompt::new_rs(
        vec![gemini_msg],
        "gemini-2.5-flash",
        Provider::Gemini,
        vec![],
        None,
        Some(Score::get_structured_output_schema()),
        ResponseType::Score,
    )
    .unwrap();

    let agent = runtime
        .block_on(async { Agent::new(Provider::Gemini, None).await })
        .unwrap();
    let task = Task::new(&agent.id, prompt, "task1", None, None);

    runtime.block_on(async {
        agent.execute_task(&task).await.unwrap();
    });

    let response = runtime.block_on(async { agent.execute_prompt(&task.prompt).await.unwrap() });

    let content = response.response_text().unwrap();
    let _score: Score = Score::model_validate_json_str(&content).unwrap();

    mock.stop_server().unwrap();
}

#[test]
fn test_gemini_score_agent_integration() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = LLMTestServer::new();
    mock.start_server().unwrap();

    let prompt_content =
        "Return a json object with the attributes score and explanation".to_string();
    let gemini_msg = MessageNum::GeminiContentV1(GeminiContent {
        role: "user".to_string(),
        parts: vec![Part {
            data: DataNum::Text(prompt_content.clone()),
            ..Default::default()
        }],
    });
    let prompt = Prompt::new_rs(
        vec![gemini_msg],
        "gemini-2.5-flash",
        Provider::Gemini,
        vec![],
        None,
        Some(Score::get_structured_output_schema()),
        ResponseType::Score,
    )
    .unwrap();

    let agent = runtime
        .block_on(async { Agent::new(Provider::Gemini, None).await })
        .unwrap();
    let task = Task::new(&agent.id, prompt, "task1", None, None);

    runtime.block_on(async {
        agent.execute_task(&task).await.unwrap();
    });

    let response = runtime.block_on(async { agent.execute_prompt(&task.prompt).await.unwrap() });

    let content = response.response_text().unwrap();
    let _score: Score = Score::model_validate_json_str(&content).unwrap();

    mock.stop_server().unwrap();
}
