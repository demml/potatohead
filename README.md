
# Potato Head

Build an LLM Potato Head. Fun for all ages!

## Description

Potato Head currently supports OpenAI compatible apis (more coming soon!) and was originally built as a sister project to OpsML to help facilitate Prompt standardization and testing. 

In addition, Potato Head is built with rust and exposed via python bindings to help improve performance and reduce latency of LLM applications, so enjoy some Rusty Potatoes!

## Quick Start

### Chat Completion

```python
from potato_head.prompts import Message, ChatPrompt
from potato_head.openai import OpenAIConfig
from potato_head.parts import Mouth 

mouth = Mouth(OpenAIConfig())

prompt = ChatPrompt(
    model="gpt-4o",
    messages=[
        Message("user", "What is 4 + 1?"),
    ],
)

response = mouth.speak(prompt)
print(response)
```

### Chat Completion with Structured Response

```python
from potato_head.prompts import Message, ChatPrompt
from potato_head.openai import OpenAIConfig
from potato_head.parts import Mouth 

mouth = Mouth(OpenAIConfig())

class MathResult(BaseModel):
    result: int

prompt = ChatPrompt(
    model="gpt-4o",
    messages=[Message("user", "What is 4 + 1?")],
)

response = mouth.speak(prompt, MathResult)
print(response)
```

### Streaming Chat Completion

```python
from potato_head.prompts import Message, ChatPrompt
from potato_head.openai import OpenAIConfig
from potato_head.parts import Mouth 

mouth = Mouth(OpenAIConfig())

prompt = ChatPrompt(
    model="gpt-4o-mini",
    messages=[
        {
            "role": "user",
            "content": "Print the numbers from 1 to 10. One number at a time.",
        }
    ],
    temperature=0,
    stream=True,
)

response = mouth.speak_stream(prompt)
for message in response:
    for choice in message.choices:
        if choice.delta.content:
            print(choice.delta.content, end="", flush=True)
```

