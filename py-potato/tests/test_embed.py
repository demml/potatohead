from potato_head import Embedder, Provider


def test_openai_embedding():
    with LLMTestServer():
        embedder = Embedder(Provider.OpenAI)
        response = embedder.embed("Test input")
        assert response is not None
