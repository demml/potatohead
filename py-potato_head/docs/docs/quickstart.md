# Quick Start

### Chat Completion

```python
from potato_head import Message, ChatPrompt, OpenAIConfig, Mouth
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
from potato_head import Message, ChatPrompt, OpenAIConfig, Mouth

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
from potato_head import Message, ChatPrompt, OpenAIConfig, Mouth

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


### Using OpenAI SDK with ChatPrompt

Potato Head prompts are aimed to be compatible with the OpenAI SDK. You can use the `to_open_ai_request` method to convert a `ChatPrompt` to a dictionary that can be used with the OpenAI SDK.

```python
from potato_head import ChatPrompt
from openai import OpenAI

client = OpenAI()

prompt = ChatPrompt(
    model="gpt-4o",
    messages=[
        {"role": "developer", "content": "You are a helpful assistant."},
        {"role": "user", "content": "What is 4 + 1?"},
    ],
    n=1,
    temperature=0.7,
    max_completion_tokens=100,
)


if __name__ == "__main__":
    spec = prompt.to_open_ai_request()
    response = client.chat.completions.create(**spec)
    print(response)
```

### Binding Context

There are times when you may want to bind context to a prompt, like in RAG applications. You can use the `bind_context_at` method to bind context to a prompt for a specific message. `Potato Head` uses numbered params to bind context to a prompt (e.g. `$1`, `$2`, etc).

```python
from potato_head import ChatPrompt, Mouth, OpenAIConfig
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger

RustyLogger.setup_logging(LoggingConfig(log_level=LogLevel.Debug))

mouth = Mouth(OpenAIConfig())

prompt = ChatPrompt(
    model="gpt-4o",
    messages=[
        {
            "role": "user",
            "content": "What is 4 + $1 + $2?",
        },
        {
            "role": "user",
            "content": "Give me a one sentence overview of $1. Also, what was the result of my first question?",
        },
    ],
)


if __name__ == "__main__":
    prompt.bind_context_at("5", 0) # bind first param in first message
    prompt.bind_context_at("10", 0) # bind second param in first message
    prompt.bind_context_at("Mr. Potato Head", 1) # bind first param in second message

    # verify the context is bound
    print(prompt.open_ai_spec())

    response = mouth.speak(prompt)
    print(response)
```