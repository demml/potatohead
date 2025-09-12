from potato_head import Embedder, Provider
from potato_head.google import PredictRequest, PredictResponse, GeminiEmbeddingConfig
import numpy as np
from typing import cast
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger


RustyLogger.setup_logging(LoggingConfig(log_level=LogLevel.Debug))

if __name__ == "__main__":
    request = PredictRequest(
        instances=[
            {
                "task_type": "RETRIEVAL_DOCUMENT",
                "title": "document title",
                "content": "I would like embeddings for this text!",
            }
        ],
    )
    embedder = Embedder(
        Provider.Vertex,
        GeminiEmbeddingConfig(model="gemini-embedding-001"),
    )
    # response = cast(GeminiEmbeddingResponse, embedder.embed(input="Test input"))
    # nd_array = np.array(response.embedding.values)
    # assert nd_array.shape == (512,)
