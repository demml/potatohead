use baked_potato::LLMTestServer;
use potato_provider::{Embedder, EmbeddingConfig};
use potato_type::google::GeminiEmbeddingConfig;
use potato_type::openai::embedding::OpenAIEmbeddingConfig;
use potato_type::Provider;

/// This test is performed in a sync context in order to maintain compatibility with python (LLMTestServer can be used in rust and python)
/// Because of this, we need to use a tokio runtime to run the async code within the test.
#[test]
fn test_openai_embedding() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = LLMTestServer::new();

    mock.start_server().unwrap();
    let config = EmbeddingConfig::OpenAI(OpenAIEmbeddingConfig {
        model: "text-embedding-3-small".to_string(),
        ..Default::default()
    });

    let embedder = runtime
        .block_on(async { Embedder::new(Provider::OpenAI, config).await })
        .unwrap();

    let inputs = vec!["Test input 1".to_string(), "Test input 2".to_string()];

    let embeddings = runtime.block_on(async { embedder.embed(inputs).await.unwrap() });

    let openai_response = embeddings.to_openai_response().unwrap();

    // get usage
    assert_eq!(openai_response.usage.prompt_tokens, 8);
    assert_eq!(openai_response.usage.total_tokens, 8);

    mock.stop_server().unwrap();
}

#[test]
fn test_gemini_embedding() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = LLMTestServer::new();

    mock.start_server().unwrap();

    let config = EmbeddingConfig::Gemini(GeminiEmbeddingConfig {
        model: Some("gemini-embedding-001".to_string()),
        ..Default::default()
    });
    let embedder = runtime
        .block_on(async { Embedder::new(Provider::Gemini, config).await })
        .unwrap();

    let inputs = vec!["Test input 1".to_string()];

    let _embeddings = runtime.block_on(async { embedder.embed(inputs).await.unwrap() });

    mock.stop_server().unwrap();
}
