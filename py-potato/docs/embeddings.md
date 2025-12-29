# Embeddings

`Potato Head` supports generating embeddings using various providers. Embeddings are numerical representations of text that capture semantic meaning, allowing for tasks such as similarity search, clustering, and classification.

## Creating an OpenAI Embedding

```python

from potato_head import Embedder, Provider
from potato_head.openai import OpenAIEmbeddingConfig
import numpy as np


embedder = Embedder( #(1)
    Provider.OpenAI,
    config=OpenAIEmbeddingConfig( #(2)
        model="text-embedding-3-small",
        dimensions=512,
    ),
)
response = embedder.embed(input="Test input")

nd_array = np.array(response.data[0].embedding)
assert nd_array.shape == (512,)
```

1. Create an `Embedder` instance with the desired provider.
2. Configure the embedder with the appropriate model and dimensions using the provider-specific configuration class.


## Creating an Gemini Embedding

```python
from potato_head import Embedder, Provider
from potato_head.google import GeminiEmbeddingConfig, GeminiEmbeddingResponse
import numpy as np
from typing import cast
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger

RustyLogger.setup_logging(LoggingConfig(log_level=LogLevel.Debug))

embedder = Embedder(
    Provider.Gemini,
    config=GeminiEmbeddingConfig(
        model="gemini-embedding-001",
        output_dimensionality=512,
    ),
)
response = cast(GeminiEmbeddingResponse, embedder.embed(input="Test input"))
nd_array = np.array(response.embedding.values)
assert nd_array.shape == (512,)
```

## References
- Embedder class: [Documentation](/potatohead/docs/api/stubs/#potato_head.stubs.Embedder)
- OpenAI Embedding Configuration: [Documentation](/potatohead/docs/api/stubs/#potato_head.stubs.OpenAIEmbeddingConfig)
- Gemini Embedding Configuration: [Documentation](/potatohead/docs/api/stubs/#potato_head.stubs.GeminiEmbeddingConfig)