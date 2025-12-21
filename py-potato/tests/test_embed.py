import numpy as np
from potato_head import Embedder, Provider
from potato_head.google import GeminiEmbeddingConfig
from potato_head.mock import LLMTestServer


def test_openai_embedding():
    with LLMTestServer():
        embedder = Embedder(Provider.OpenAI)
        response = embedder.embed("Test input")
        assert response is not None

        # assert data is > 0
        assert len(response.data) > 0
        assert response.data[0].embedding is not None

        nd_array = np.array(response.data[0].embedding)
        assert nd_array.shape == (512,)


def test_gemini_embedding():
    with LLMTestServer():
        embedder = Embedder(
            Provider.Gemini,
            GeminiEmbeddingConfig(model="gemini-embedding-001"),
        )
        response = embedder.embed("Test input")

        assert len(response.embedding.values) > 0

        nd_array = np.array(response.embedding.values)
        assert nd_array.shape == (512,)
