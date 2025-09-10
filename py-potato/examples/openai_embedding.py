# type: ignore
from potato_head import Embedder, Provider
from potato_head.openai import OpenAIEmbeddingConfig
import numpy as np


if __name__ == "__main__":
    embedder = Embedder(
        Provider.OpenAI,
        config=OpenAIEmbeddingConfig(
            model="text-embedding-3-small",
            dimensions=512,
        ),
    )
    response = embedder.embed(input="Test input")

    nd_array = np.array(response.data[0].embedding)
    assert nd_array.shape == (512,)
