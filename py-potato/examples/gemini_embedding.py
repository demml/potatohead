# type: ignore
from potato_head import Embedder, Provider
from potato_head.google import GeminiEmbeddingConfig
import numpy as np


if __name__ == "__main__":
    embedder = Embedder(Provider.Gemini)
    response = embedder.embed(
        input="Test input",
        config=GeminiEmbeddingConfig(
            model="gemini-embedding-001",
            output_dimensionality=512,
        ),
    )

    nd_array = np.array(response.embedding.values)
    assert nd_array.shape == (512,)
