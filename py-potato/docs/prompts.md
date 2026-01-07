# Prompts


Prompts are standardized objects for interacting with language models in Potato Head. They encapsulate user messages, system instructions, and model settings, providing a consistent, provider-agnostic interface for LLMs.


## Prompt Class Inputs

| Parameter           | Type                                                                                                   | Description                                                                                                                                                                                                                                  | Default      |
|---------------------|--------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------|
| `messages` | `Union[str | ChatMessage | MessageParam | GeminiContent]`<br>`List[Union[str | ChatMessage | MessageParam | GeminiContent]]` | The main prompt content. Can be a string, a sequence of string or be provider-specific message formats. | **Required** |
| `model`             | `str`                                                                                        | The model to use for the prompt |**Required** |
| `provider`          | `str`                                                                                        | The provider to use for the prompt | **Required** |
| `system_instructions`| `Optional[str | List[str]]`                                                                           | System-level instructions to include in the prompt. Can be a string or a list of strings.                                                                                                             | `None`       |
| `model_settings`    | `Optional[ModelSettings | OpenAIChatSettings | GeminiSettings | AnthropicSettings]`                                                                              | Model settings for the prompt. If not provided, no additional model settings are used.                                                                         | `None`       |
| `output_type`   | `Optional[Any]`                                                                                        | Specifies the output type for structured outputs. Supports Pydantic `BaseModel` classes and the PotatoHead `Score` class. The format will be parsed into a JSON schema for the LLM API.           | `None`       |

## Binding Parameters

One of the benefits of using the `Prompt` class is that is allows you to parameterize your messages and bind them at runtime. This is useful for creating dynamic prompts that can change based on user input or other factors.

Potato Head allows you to parameterize with named variables in the message strings. To create a parameterized message, you can use the `${variable_name}` syntax. When you execute the prompt, you can bind values to these variables.

Binding is done using the `bind` method of the `Prompt` class. This method takes a variable name and its value, replacing the `${variable_name}` in the message with the provided value and returning a new `Prompt` instance with the updated message.

If you wish to bind in-place, you can use the `bind_mut` method directly on the `Prompt` instance.

```python
from potato_head import Prompt, Agent, Provider

prompt = Prompt(
    model="gpt-4o",
    provider="openai",
    messages="I'm looking for post-hardcore music recommendations. Can you suggest some bands similar to ${band}?",
    system_instructions="You are a music expert who specializes in various genres.",
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

### Binding Individual Messages

All messages within a prompt also implement `bind` and `bind_mut` methods, allowing you to bind variables to specific messages within a multi-message prompt.

```python
from openai import OpenAI

client = OpenAI()

prompt = Prompt(
    model="gpt-4o",
    provider="openai",
    messages="I'm looking for post-hardcore music recommendations. Can you suggest some bands similar to ${band}?",
    system_instructions="You are a music expert who specializes in various genres.",
)

# Bind only the user message
user_message = prompt.messages[0].bind("band", "Turnstile")

response = client.chat.completions.create(
    model=prompt.model,
    messages=[user_message.model_dump()]
)

# or access content directly
response = client.chat.completions.create(
    model=prompt.model,
    messages=[{"role": "user", "content": user_message.content[0].text}]
)
```

## Structured Responses
If you'd like to associate a structured response with your prompt, you can use the `output_type` parameter and provide either a [pydantic](https://docs.pydantic.dev/latest/) `BaseModel` class or a `Score` class from Potato Head. `Score` classes are primarily used for evaluation purposes when building `Scouter` LLM evaluation workflows. This will allow the LLM to return a structured response that can be parsed into the specified format.

```python
from pydantic import BaseModel
from potato_head import Prompt

class StructuredTaskOutput(BaseModel):
    tasks: List[str]
    status: str

prompt = Prompt(
    messages="""
    Please provide a list of tasks to complete and their status in JSON format.
    Example:
    {
        "tasks": ["task1", "task2"],
        "status": "in_progress"
    }

    Return the response in the same format.
    """,
    system_instructions="You are a helpful assistant.",
    model="gpt-4o",
    provider="openai",
    output_type=StructuredTaskOutput, # (1)
)

agent = Agent(Provider.OpenAI)

result: StructuredTaskOutput = agent.execute_prompt(
    prompt=prompt,
    output_type=StructuredTaskOutput,
).structured_output  # (2)

assert isinstance(result, StructuredTaskOutput)
print("Tasks:", result.tasks)
```

1. By specifying `output_type`, Potato Head will generate a JSON schema based on the `StructuredTaskOutput` model and include it in the prompt to guide the LLM's response formatting. When the response is received, it will be parsed into an instance of `StructuredTaskOutput` for easy access to its fields.
2. When executing the prompt, we specify the same `output_type` to ensure that the response is parsed correctly into the desired structured format.

### Why do I need to specify `output_type`?

When executing a prompt, you need to provide both the `output_type` (in the `Prompt`) and the `output_type` (when calling `execute_prompt`). The `output_type` defines the expected structure of the response and is used to generate a JSON schema for the LLM API. This schema is also saved with the prompt for future reference (decoupled from runtime execution). The `output_type` tells the system what Python type to parse the LLM's response into at runtime. Both should match to ensure type safety and that the response is parsed as expected.

In this simple example, it doesn't make sense to include both, and in fact, you could just specify the `output_type` when executing the prompt (it will overwrite the schema in the prompt). However, when running apis or agents, it's useful to have the schema saved with the prompt for documentation and validation purposes.


## Example Usage

### Simple Text Prompt

```python
from potato_head import Prompt

prompt = Prompt(
    model="gpt-4o",
    provider="openai",
    messages="My prompt",
    system_instructions="system_prompt",
)
```

### Prompt with Multiple Messages (OpenAI Format)

```python
from potato_head import Prompt, ChatMessage

prompt = Prompt(
    model="gpt-4o",
    provider="openai",
    messages=[
        ChatMessage(content="Foo", role="user"),
        ChatMessage(content="Bar", role="user"),
    ],
    system_instructions="system_prompt",
)

# or you can do
prompt = Prompt(
    model="gpt-4o",
    provider="openai",
    messages=["Foo", "Bar"],
    system_instructions="system_prompt",
)

```

### Prompt with Multiple Messages (Google Gemini Format)

```python
from potato_head import Prompt, Provider, Role
from potato_head.google import GeminiContent

prompt = Prompt(
    model="gemini-1.5-pro",
    provider=Provider.Gemini,
    messages=GeminiContent(
        parts="My prompt",
        role=Role.User.as_str(),
    )
)

prompt.messages[0].parts[0].data # "Foo"
prompt.gemini_messages[0].parts[0].data # "Foo"
```


### Anthropic Message Format

```python
from potato_head import Prompt, Provider
from potato_head.anthropic import MessageParam, TextBlockParam


prompt = Prompt(
    model="claude-4.5-sonnet",
    provider=Provider.Anthropic,
    messages=MessageParam(
        content="My prompt",
        role=Role.User.as_str(),
    ),
    system_instructions="system_prompt",
)

assert prompt.anthropic_message.content[0].text == "My prompt"

prompt = Prompt(
    model="claude-4.5-sonnet",
    provider=Provider.Anthropic,
    messages=MessageParam(
        content=[
            TextBlockParam(text="My prompt"),
            TextBlockParam(text="My prompt 2"),
        ],
        role=Role.User.as_str(),
    ),
    system_instructions="system_prompt",
)

assert prompt.anthropic_message.content[0].text == "My prompt"
assert prompt.anthropic_message.content[1].text == "My prompt 2"

```

### ModelSettings

If you want to customize model settings like temperature, max tokens, or any provider-specific parameters, you can use the `ModelSettings` class, which accepts one of `OpenAIChatSettings` ,`GeminiSettings` , or `AnthropicSettings`.
These settings will be auto-injected into the request based up the provider specification when the prompt is executed. In addition, the `Prompt` class supports any provider-specific model settings directly, which are then converted to the appropriate `ModelSettings` class based on the provider.

Settings:

- OpenAIChatSettings: [Documentation](/potatohead/docs/api/stubs/#potato_head.stubs.OpenAIChatSettings)
- GeminiSettings: [Documentation](/potatohead/docs/api/stubs/#potato_head.stubs.GeminiSettings)
- AnthropicSettings: [Documentation](/potatohead/docs/api/stubs/#potato_head.stubs.AnthropicSettings)

#### OpenAIChatSettings
```python
from potato_head import Prompt, ModelSettings
from potato_head.openai import OpenAIChatSettings

# use directly
prompt = Prompt(
    model="o4-mini",
    provider="openai",
    messages="Tell me a joke about potatoes.",
    system_instructions="You are a helpful assistant.",
    model_settings=OpenAIChatSettings(
        max_completion_tokens=50,
        temperature=0.7,
    ),
)

# or use ModelSettings wrapper

prompt = Prompt(
    model="o4-mini",
    provider="openai",
    messages="Tell me a joke about potatoes.",
    system_instructions="You are a helpful assistant.",
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
    messages="Tell me a joke about potatoes.",
    system_instructions="You are a helpful assistant.",
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
    messages="Tell me a joke about potatoes.",
    system_instructions="You are a helpful assistant.",
    model_settings=ModelSettings(
        settings=GeminiSettings(
            generation_config=GenerationConfig(
                thinking_config=ThinkingConfig(thinking_budget=0),
            ),
        )
    ),
)
```
#### AnthropicSettings

```python
from potato_head import Prompt
from potato_head.anthropic import AnthropicSettings

prompt = Prompt(
    model="claude-4.5-sonnet",
    provider="anthropic",
    messages=[
        "Hello ${variable1}",
        "This is ${variable2}",
    ],
    system_instructions="system_prompt",
    model_settings=AnthropicSettings(thinking=AnthropicThinkingConfig(type="enabled")),
)
```