# type: ignore
from potato_head import Embedder, Provider
from potato_head.google import GeminiEmbeddingConfig
import numpy as np
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger
from google import genai
from google.genai.types import EmbedContentConfig


RustyLogger.setup_logging(LoggingConfig(log_level=LogLevel.Debug))

if __name__ == "__main__":
    embedder = Embedder(
        Provider.Gemini,
        config=GeminiEmbeddingConfig(
            model="gemini-embedding-001",
            output_dimensionality=512,
        ),
    )
    response = embedder.embed(input="Test input")

    nd_array = np.array(response.embedding.values)
    assert nd_array.shape == (512,)
