use potato_prompt::prompt::{Message, Prompt, PromptContent, Score};
use potato_type::StructuredOutput;
use serde_json::Value;

pub fn create_prompt(response_format: Option<Value>) -> Prompt {
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

pub fn create_parameterized_prompt() -> Prompt {
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

pub fn score_prompt() -> Prompt {
    let user_content = PromptContent::Str("What is the score?".to_string());
    let system_content = PromptContent::Str("You are a helpful assistant.".to_string());
    Prompt::new_rs(
        vec![Message::new_rs(user_content)],
        Some("gpt-4o"),
        Some("openai"),
        vec![Message::new_rs(system_content)],
        None,
        Some(Score::get_structured_output_schema()),
    )
    .unwrap()
}
