use potato_type::anthropic::MessageParam;
use potato_type::anthropic::TextBlockParam;
use potato_type::google::v1::generate::{DataNum, GeminiContent, Part};
use potato_type::openai::v1::chat::request::{
    ChatMessage as OpenAIChatMessage, ContentPart, TextContentPart,
};
use potato_type::prompt::MessageNum;
use potato_type::prompt::{Prompt, Role};
use potato_type::traits::MessageFactory;
use serde_json::Value;
pub fn create_openai_prompt(response_format: Option<Value>) -> Prompt {
    let user_content = "Hello, how are you?".to_string();
    let system_content = "You are a helpful assistant.".to_string();

    let system_msg = OpenAIChatMessage {
        role: Role::Developer.to_string(),
        content: vec![ContentPart::Text(TextContentPart::new(system_content))],
        name: None,
    };

    let user_msg = OpenAIChatMessage {
        role: Role::User.to_string(),
        content: vec![ContentPart::Text(TextContentPart::new(user_content))],
        name: None,
    };
    Prompt::new_rs(
        vec![MessageNum::OpenAIMessageV1(user_msg)],
        "gpt-4o",
        potato_type::Provider::OpenAI,
        vec![MessageNum::OpenAIMessageV1(system_msg)],
        None,
        response_format,
        potato_type::prompt::ResponseType::Null,
    )
    .unwrap()
}

pub fn create_anthropic_prompt() -> Prompt {
    let user_content = "Hello, how are you?".to_string();
    let system_content = "You are a helpful assistant.".to_string();

    let anthropic_msg = MessageParam::from_text(user_content, &Role::User.to_string()).unwrap();
    let system_msg = TextBlockParam {
        text: system_content,
        r#type: "text".to_string(),
        citations: None,
        cache_control: None,
    };

    Prompt::new_rs(
        vec![MessageNum::AnthropicMessageV1(anthropic_msg)],
        "claude-sonnet-2024-11-06",
        potato_type::Provider::Anthropic,
        vec![MessageNum::AnthropicSystemMessageV1(system_msg)],
        None,
        None,
        potato_type::prompt::ResponseType::Null,
    )
    .unwrap()
}

pub fn create_google_prompt() -> Prompt {
    let user_content = "Hello, how are you?".to_string();
    let system_content = "You are a helpful assistant.".to_string();

    let user_msg = GeminiContent {
        role: Role::User.to_string(),
        parts: vec![Part {
            data: DataNum::Text(user_content),
            ..Default::default()
        }],
    };

    let system_msg = GeminiContent {
        role: Role::Assistant.to_string(),
        parts: vec![Part {
            data: DataNum::Text(system_content),
            ..Default::default()
        }],
    };

    Prompt::new_rs(
        vec![MessageNum::GeminiContentV1(user_msg)],
        "gemini-2.5-flash",
        potato_type::Provider::Gemini,
        vec![MessageNum::GeminiContentV1(system_msg)],
        None,
        None,
        potato_type::prompt::ResponseType::Null,
    )
    .unwrap()
}
