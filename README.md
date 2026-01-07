# Potato Head

<div style="margin-bottom: 20px; position: relative; z-index: 1;">
  <img align="right" width="200" src="images/potatohead.svg">
</div>


Potato Head is a core/utility crate to both [opsml](https://github.com/demml/opsml) and [scouter](https://github.com/demml/scouter), providing essential rust and python components for building agentic workflows. At the current moment, only crates are published, which are then used in both opsml and scouter to provide user-friendly interfaces. The documentation contained in this repository is meant to help you understand the core concepts of potatohead and how to use it within `Opsml` and `Scouter`. 


## Creating a Talking Potato
To get your potato to talk, you first need to create a `Prompt`. It's gotta have something to say, right?

```python
from potato_head import Prompt

prompt = Prompt(
  model="gpt-4o",
  provider="openai",
  messages="Tell me a joke about potatoes.",
  system_instructions="You are a helpful assistant.",
)
```

### How do we make the potato talk?

```python
from potato_head import Agent, Provider

agent = Agent(Provider.OpenAI)
response = agent.execute_prompt(prompt=prompt)


print(response.result)
# Why did the potato win the talent show?
# Because it was outstanding in its field!
```

## Documentation

For more information, please refer to the [documentation](https://docs.demml.io/potatohead).