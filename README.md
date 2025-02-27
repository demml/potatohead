<div style="margin-bottom: 20px">
  <img align="right" width="200" src="images/potatohead.svg">
</div>

<br clear="right"/>

<div align="left">
  <h1>🥔 Potato Head</h1>
</div>

Build an LLM Potato Head. Add a **mouth** (chat), **eyes** (vision/image) and even **ears** (audio)! Fun for all ages!

**Note:** Potato Head currently supports OpenAI compatible apis (more coming soon!) and you can only use **Mouths**. We are actively working on adding more parts to the Potato Head!


<div align="left">
  <h2>📖 Description</h2>
</div>


Potato Head was originally built as a sister project to OpsML to help facilitate Prompt standardization and testing. The goal is to provide a simple and easy to use interface with low overhead. Thus, **Potato Head** is written entirely in **Rust** and is exposed via Python bindings to help improve performance and reduce latency of LLM applications when interacting with clients.

Potato Head Prompts are fully compatible with the OpenAI SDK if you prefer to use that instead. Check out the quickstart for more information!

Check out the quickstart for more information on how you can get started with Potato Head!

## MORE FUNCTIONALITY COMING SOON!


<div align="left">
  <h2>🚀 Quick Start</h2>
</div>

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

