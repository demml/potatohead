use potato_prompt::prompt::Prompt;
use potato_type::prompt::{Message, PromptContent, Score};
use potato_type::StructuredOutput;
use serde_json::Value;

pub fn create_prompt(response_format: Option<Value>) -> Prompt {
    let user_content = PromptContent::Str("Hello, how are you?".to_string());
    let system_content = PromptContent::Str("You are a helpful assistant.".to_string());
    Prompt::new_rs(
        vec![Message::new_rs(user_content)],
        "gpt-4o",
        potato_type::Provider::OpenAI,
        vec![Message::new_rs(system_content)],
        None,
        response_format,
        potato_type::prompt::ResponseType::Null,
    )
    .unwrap()
}

pub fn create_parameterized_prompt() -> Prompt {
    let user_content = PromptContent::Str("What is ${variable1} + ${variable2}?".to_string());
    let system_content = PromptContent::Str("You are a helpful assistant.".to_string());
    Prompt::new_rs(
        vec![Message::new_rs(user_content)],
        "gpt-4o",
        potato_type::Provider::OpenAI,
        vec![Message::new_rs(system_content)],
        None,
        None,
        potato_type::prompt::ResponseType::Null,
    )
    .unwrap()
}

#[allow(clippy::uninlined_format_args)]
pub fn create_score_prompt(params: Option<Vec<String>>) -> Prompt {
    let mut user_prompt = "What is the score?".to_string();

    // If parameters are provided, append them to the user prompt in format ${param}
    if let Some(params) = params {
        for param in params {
            user_prompt.push_str(&format!(" ${{{}}}", param));
        }
    }

    let user_content = PromptContent::Str(user_prompt);
    let system_content = PromptContent::Str("You are a helpful assistant.".to_string());
    Prompt::new_rs(
        vec![Message::new_rs(user_content)],
        "gpt-4o",
        potato_type::Provider::OpenAI,
        vec![Message::new_rs(system_content)],
        None,
        Some(Score::get_structured_output_schema()),
        potato_type::prompt::ResponseType::Score,
    )
    .unwrap()
}
