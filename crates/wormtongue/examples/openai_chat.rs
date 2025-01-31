pub use wormtongue::error::TongueError;
pub use wormtongue::tongues::openai::{
    Message, MessageContent, MessageContentPart, OpenAIInterface, OpenAIPrompt,
};
fn main() {
    // Create a new OpenAI prompt (Will default to Chat prompt type)
    let mut prompt = OpenAIPrompt::default();

    prompt.add_message(Message::new(
        "User",
        MessageContent::Text("Hello, how are you?".to_string()),
    ));

    let interface = OpenAIInterface::new(prompt, None, None, None)
        .map_err(|e| TongueError::Error(e.to_string()))
        .unwrap();

    let response = interface
        .send()
        .map_err(|e| TongueError::Error(e.to_string()))
        .unwrap();
}
