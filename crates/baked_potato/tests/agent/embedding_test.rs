use baked_potato::LLMTestServer;
use potato_agent::Embedder;
use potato_type::Provider;

/// This test is performed in a sync context in order to maintain compatibility with python (LLMTestServer can be used in rust and python)
/// Because of this, we need to use a tokio runtime to run the async code within the test.
#[test]
fn test_openai_embedding() {
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

    let embeddings = runtime.block_on(async { embedder.create(inputs, settings).await.unwrap() });

    // get usage
    assert_eq!(embeddings.usage.prompt_tokens, 8);
    assert_eq!(embeddings.usage.total_tokens, 8);

    mock.stop_server().unwrap();
}

#[test]
fn test_gemini_embedding() {
    use potato_type::google::GeminiEmbeddingConfig;
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = LLMTestServer::new();

    mock.start_server().unwrap();

    let embedder = Embedder::new(Provider::Gemini).unwrap();

    let inputs = vec!["Test input 1".to_string()];
    let settings = GeminiEmbeddingConfig {
        model: Some("gemini-embedding-001".to_string()),
        ..Default::default()
    };

    let embeddings = runtime.block_on(async { embedder.create(inputs, config).await.unwrap() });

    // get usage
    assert_eq!(embeddings.usage.prompt_tokens, 8);
    assert_eq!(embeddings.usage.total_tokens, 8);

    mock.stop_server().unwrap();
}
