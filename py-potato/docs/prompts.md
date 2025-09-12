# Prompts


Prompts are standardized objects for interacting with language models in Potato Head. They encapsulate user messages, system instructions, and model settings, providing a consistent, provider-agnostic interface for LLMs.


## Prompt Class Inputs

| Parameter           | Type                                                                                                   | Description                                                                                                                                                                                                                                  | Default      |
|---------------------|--------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------|
| `message` | `str`<br>`Sequence[str | ImageUrl | AudioUrl | BinaryContent | DocumentUrl]`<br>`Message`<br>`List[Message]`<br>`List[Dict[str, Any]]` | The main prompt content. Can be a string, a sequence of strings or media URLs, a Message object, a list of Message objects, or a list of dictionaries representing messages. | **Required** |
| `model`             | `str`                                                                                        | The model to use for the prompt |**Required** |
| `provider`          | `str`                                                                                        | The provider to use for the prompt | **Required** |
| `system_instruction`| `Optional[str | List[str]]`                                                                           | System-level instructions to include in the prompt. Can be a string or a list of strings.                                                                                                             | `None`       |
| `model_settings`    | `Optional[ModelSettings]`                                                                              | Model settings for the prompt. If not provided, no additional model settings are used.                                                                         | `None`       |
| `response_format`   | `Optional[Any]`                                                                                        | Specifies the response format for structured outputs. Supports Pydantic `BaseModel` classes and the PotatoHead `Score` class. The format will be parsed into a JSON schema for the LLM API.           | `None`       |

## Binding Messages

One of the benefits of using the `Prompt` class is that is allows you to parameterize your messages and bind them at runtime. This is useful for creating dynamic prompts that can change based on user input or other factors.

Potato Head allows you to parameterize with named variables in the message strings. To create a parameterized message, you can use the `${variable_name}` syntax. When you execute the prompt, you can bind values to these variables.

Binding is done using the `bind` method of the `Prompt` class. This method takes a variable name and its value, replacing the `${variable_name}` in the message with the provided value and returning a new `Prompt` instance with the updated message.

If you wish to bind in-place, you can use the `bind_mut` method directly on the `Prompt` instance.

```python
from potato_head import Prompt, Agent, Provider

prompt = Prompt(
    model="gpt-4o",
    provider="openai",
    message="I'm looking for post-hardcore music recommendations. Can you suggest some bands similar to ${band}?",
    system_instruction="You are a music expert who specializes in various genres.",
)
agent = Agent(Provider.OpenAI)

agent.execute_prompt(
    prompt=prompt.bind("band", "Turnstile")
)

# or for in-place binding
agent.execute_prompt(
    prompt=prompt.bind_mut("band", "Turnstile")
)
```

## Structured Responses
If you'd like to associate a structured response with your prompt, you can use the `response_format` parameter and provide either a [pydantic](https://docs.pydantic.dev/latest/) `BaseModel` class or a `Score` class from Potato Head. `Score` classes are primarily used for evaluation purposes when building `Scouter` LLM monitoring workflows. This will allow the LLM to return a structured response that can be parsed into the specified format.

```python
from pydantic import BaseModel
from potato_head import Prompt

class StructuredTaskOutput(BaseModel):
    tasks: List[str]
    status: str

prompt = Prompt(
    message="""
    Please provide a list of tasks to complete and their status in JSON format.
    Example:
    {
        "tasks": ["task1", "task2"],
        "status": "in_progress"
    }

    Return the response in the same format.
    """,
    system_instruction="You are a helpful assistant.",
    model="gpt-4o",
    provider="openai",
    response_format=StructuredTaskOutput,
)

agent = Agent(Provider.OpenAI)

result: StructuredTaskOutput = agent.execute_prompt(
    prompt=prompt,
    output_type=StructuredTaskOutput,
).result

assert isinstance(result, StructuredTaskOutput)
print("Tasks:", result.tasks)
```

### Why do I need to specify `output_type`?

When executing a prompt, you need to provide both the `response_format` (in the `Prompt`) and the `output_type` (when calling `execute_prompt`). The `response_format` defines the expected structure of the response and is used to generate a JSON schema for the LLM API. This schema is also saved with the prompt for future reference (decoupled from runtime execution). The `output_type` tells the system what Python type to parse the LLM's response into at runtime. Both should match to ensure type safety and that the response is parsed as expected.


## Example Usage

### Simple Text Prompt

```python
from potato_head import Prompt

prompt = Prompt(
    model="gpt-4o",
    provider="openai",
    message="My prompt",
    system_instruction="system_prompt",
)
```

### Prompt with Multiple Messages

```python
from potato_head import Prompt, Message

prompt = Prompt(
    model="gpt-4o",
    provider="openai",
    message=[
        Message(content="Foo"),
        Message(content="Bar"),
    ],
    system_instruction="system_prompt",
)

# or you can do
prompt = Prompt(
    model="gpt-4o",
    provider="openai",
    message=["Foo", "Bar"],
    system_instruction="system_prompt",
)
```

### ModelSettings

If you want to customize model settings like temperature, max tokens, or any provider-specific parameters, you can use the `ModelSettings` class, which accepts one of `OpenAIChatSettings` or `GeminiChatSettings`.
These settings will be auto-injected into the request based up the provider specification when the prompt is executed. In addition, the `Prompt` class supports any provider-specific model settings directly, which are then converted to the appropriate `ModelSettings` class based on the provider.

Settings:

- OpenAIChatSettings: [Documentation](./docs/api/openai.md#potato_head.openai._openai.OpenAIChatSettings)
- GeminiSettings: [Documentation](./docs/api/google.md#potato_head.google._google.GeminiSettings)

#### OpenAIChatSettings
```python
from potato_head import Prompt, ModelSettings
from potato_head.openai import OpenAIChatSettings

# use directly
prompt = Prompt(
    model="o4-mini",
    provider="openai",
    message="Tell me a joke about potatoes.",
    system_instruction="You are a helpful assistant.",
    model_settings=OpenAIChatSettings(
        max_completion_tokens=50,
        temperature=0.7,
    ),
)

# or use ModelSettings wrapper

prompt = Prompt(
    model="o4-mini",
    provider="openai",
    message="Tell me a joke about potatoes.",
    system_instruction="You are a helpful assistant.",
    model_settings=ModelSettings(
        settings=OpenAIChatSettings(
            max_completion_tokens=50,
            temperature=0.7,
        )
    ),
)
```

#### GeminiSettings

```python
from potato_head import Prompt, ModelSettings
from potato_head.google import GeminiSettings, GenerationConfig, ThinkingConfig


# Using GeminiSettings directly
prompt = Prompt(
    model="o4-mini",
    provider="google",
    message="Tell me a joke about potatoes.",
    system_instruction="You are a helpful assistant.",
    model_settings=GeminiSettings(
        generation_config=GenerationConfig(
            thinking_config=ThinkingConfig(thinking_budget=0),
        ),
    ),
)

# or use ModelSettings wrapper
prompt = Prompt(
    model="o4-mini",
    provider="google",
    message="Tell me a joke about potatoes.",
    system_instruction="You are a helpful assistant.",
    model_settings=ModelSettings(
        settings=GeminiSettings(
            generation_config=GenerationConfig(
                thinking_config=ThinkingConfig(thinking_budget=0),
            ),
        )
    ),
)
```

### DocumentUrl

```python
from potato_head import Prompt, DocumentUrl

prompt = Prompt(
    model="gpt-4o",
    provider="openai",
    message=[
        "What is the main content of this document?",
        DocumentUrl(url="https://storage.googleapis.com/cloud-samples-data/generative-ai/pdf/2403.05530.pdf"),
    ],
    system_instruction="system_prompt",
)
```

### Binary Content

```python
from potato_head import Prompt, BinaryContent

image_response = httpx.get("https://iili.io/3Hs4FMg.png")

prompt = Prompt(
    model="gpt-4o",
    provider="openai",
    message=[
        "What company is this logo from?",
        BinaryContent(data=image_response.content, media_type="image/png"),
    ],
    system_instruction="system_prompt",
)
```