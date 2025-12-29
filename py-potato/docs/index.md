# Potato Head

<div style="margin-bottom: 20px; position: relative; z-index: 1;">
  <img align="right" width="200" src="docs/images/potatohead.svg">
</div>


Potato Head is considered a core/utility crate to both [opsml](https://github.com/demml/opsml) and [scouter](https://github.com/demml/scouter), providing essential rust and python components for building agentic workflows. At the current moment, only crates are published, which are then used in both opsml and scouter to provide user-friendly python interfaces. The documentation contained in this repository is meant to help you understand the core concepts of potatohead and how to use it within `Opsml` and `Scouter`. 

## Why?

While `Potato Head` provides functionality for the basics of interacting with LLMs (standard prompts, content generation, embeddings, Agents, simple Workflows), it is not a full fledged agent framework or drop-in replacement for more feature-rich libraries like `Pydantic AI` or `Google ADK`. Instead, `Potato Head` was originally built to enable developers to standardize some parts of their LLM workflows and cleanly translate python-defined LLM workflows into low-latency Rust implementations for on-line evaluation.

- Standardized prompt structure
- Standardized LLM workflows structure for LLM evaluations at scale
- A pure Rust implementation for low-latency LLM calls and workflow execution without the need for a Python runtime
- Full type-safe mapping of responses and requests for major LLM providers (OpenAI, Google Gemini, Anthropic)


## Creating a Talking Potato
To get your potato to talk, you first need to create a `Prompt`. It's gotta have something to say, right?

**A Note on all Examples**

All examples in this documentation use an import statement like this:

```python
from potato_head import Prompt

# for openai specific functionality
from potato_head.openai import ChatMessage

# for google gemini specific functionality
from potato_head.google import GeminiContent

#  for anthropic specific functionality
from potato_head.anthropic import MessageParam
```

In reality, when using `Opsml` or `Scouter`, you will import the `Prompt` class from the respective library, like so:

```python
from opsml.genai import Prompt
from scouter.genai import Prompt
```

### Create a Prompt

```python
from potato_head import Prompt

prompt = Prompt(
  model="gpt-5.2", # (1)
  provider="openai", # (2)
  messages="Tell me a joke about potatoes.", # (3)
  system_instructions="You are a helpful assistant.",
)

# access user only message
print(prompt.messages) # (4)

# access full message history including system instructions
print(prompt.openai_messages) #(5)
```

1. What model to use. This can be any model supported by the provider.
2. The potato provider to use. `Potato Head` currently supports the `OpenAI`, `Google Gemini`, and `Anthropic` specs, with more to come in the future
3. The message to send to the model. Check out the [Prompt Guide](/potatohead/docs/api/stubs/#potato_head.stubs.Prompt) for more details on how to structure your prompts.
4. Potatohead will automatically convert your message into the provider-specific format. Here, we access the user-only messages.
5. If you want better type hinting without casting, you can access the provider-specific message format directly. Here, we access the full OpenAI message format including system instructions.


### How do we make the potato talk?

#### Passing directly to a Provider SDK

```python
from openai import OpenAI

client = OpenAI()

response = client.chat.completions.create(
    **prompt.model_dump() # (1)
)
```

1. Calling model dump returns the provider-specific request object, which can be directly passed to the provider SDK.You can also access provider-specific fields directly on the `Prompt` object if needed.


#### Using Potato Head Agent (Simple Client)

```python
from potato_head import Agent, Provider

agent = Agent(Provider.OpenAI) # (1)
response = agent.execute_prompt(prompt=prompt) # (2)


print(response) # (3)
# Why did the potato win the talent show?
# Because it was outstanding in its field!
```

1. Create an agent with the provider you want to use
2. Execute the prompt with the agent. This will return a response object that contains the response from the model.
3. This will return the Provider-specific response object. [OpenAI Response](/potatohead/docs/api/stubs/#potato_head.stubs.OpenAIChatResponse), [Generate Response(Google)](/potatohead/docs/api/stubs/#potato_head.stubs.GenerateContentResponse), [Anthropic Response](/potatohead/docs/api/stubs/#potato_head.stubs.AnthropicMessageResponse)