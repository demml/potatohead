from potato_head import Embedder, Provider
from potato_head.google import (
    PredictRequest,
    PredictResponse,
    GeminiEmbeddingConfig,
    EmbeddingTaskType,
)
import numpy as np
from typing import cast
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger


RustyLogger.setup_logging(LoggingConfig(log_level=LogLevel.Debug))

if __name__ == "__main__":
    request = PredictRequest(
        instances=[
            {
                "title": "document title",
                "content": "I would like embeddings for this text!",
            }
        ],
    )
    embedder = Embedder(
        Provider.Vertex,
        GeminiEmbeddingConfig(
            model="gemini-embedding-001",
            task_type=EmbeddingTaskType.RetrievalDocument,
            output_dimensionality=512,
        ),
    )
    response = cast(PredictResponse, embedder.embed(input=request))
    nd_array = np.array(response.predictions[0]["embeddings"]["values"])
    assert nd_array.shape == (512,)
