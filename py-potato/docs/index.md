# Potato Head

<div style="margin-bottom: 20px; position: relative; z-index: 1;">
  <img align="right" width="200" src="docs/images/potatohead.svg">
</div>


Potato Head is considered a core/utility crate to both [opsml](https://github.com/demml/opsml) and [scouter](https://github.com/demml/scouter), providing essential rust and python components for building agentic workflows. At the current moment, only crates are published, which are then used in both opsml and scouter to provide user-friendly python interfaces. The documentation contained in this repository is meant to help you understand the core concepts of potatohead and how to use it within `Opsml` and `Scouter`. 

## Why?

While `Potato Head` provides functionality for the basics of interacting with LLMs (standard prompts, content generation, embeddings, Agents, simple Workflows), it is not meant to be a full fledged agent framework or drop-in replacement for more feature-rich libraries like `Pydantic AI` or `Google ADK`. Instead, `Potato Head` was originally built to enable developers to standardize some parts of their LLM workflows:

- Standardized prompt structure
- Standardized LLM workflows structure for LLM evaluations at scale
- A pure Rust implementation for low-latency LLM calls and workflow execution without the need for a Python runtime


## Creating a Talking Potato
To get your potato to talk, you first need to create a `Prompt`. It's gotta have something to say, right?

**A Note on all Examples**

All examples in this documentation use an import statement like this:

```python
from potato_head import Prompt
```

In reality, when using `Opsml` or `Scouter`, you will import the `Prompt` class from the respective library, like so:

```python
from opsml.llm import Prompt
from scouter.llm import Prompt
```

### Create a Prompt

```python
from potato_head import Prompt

prompt = Prompt(
  model="gpt-4o", # (1)
  provider="openai", # (2)
  message="Tell me a joke about potatoes.", # (3)
  system_instruction="You are a helpful assistant.",
)
```

1. What model to use. This can be any model supported by the provider.
2. The potato provider to use. `Potato Head` currently supports the `OpenAI` spec and gemini spec, with more to come in the future
3. The message to send to the model. Check out the [Prompt Guide](./docs/api/potato_head.md#potato_head._potato_head.Prompt.__init__) for more details on how to structure your prompts.

### How do we make the potato talk?

```python
from potato_head import Agent, Provider

agent = Agent(Provider.OpenAI) # (1)
response = agent.execute_prompt(prompt=prompt) # (2)


print(response.result)
# Why did the potato win the talent show?
# Because it was outstanding in its field!
```

1. Create an agent with the provider you want to use
2. Execute the prompt with the agent. This will return a response object that contains the response from the model.