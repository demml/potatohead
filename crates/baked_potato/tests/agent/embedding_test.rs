use baked_potato::LLMTestServer;
use potato_agent::{e, Agent, Embedder, Task};
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
    use potato_type::openai::embedding::OpenAIEmbeddingSettings;
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = LLMTestServer::new();

    mock.start_server().unwrap();

    let embedder = Embedder::new(Provider::OpenAI).unwrap();

    let inputs = vec!["Test input 1".to_string(), "Test input 2".to_string()];
    let settings = OpenAIEmbeddingSettings {
        model: "text-embedding-3-small".to_string(),
        ..Default::default()
    };

    runtime.block_on(async {
        embedder.create(inputs, settings).await.unwrap();
    });

    mock.stop_server().unwrap();
}
