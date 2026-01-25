# type: ignore
from typing import List

from potato_head import Agent, Prompt, Provider
from pydantic import BaseModel
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger
from potato_head.anthropic import AnthropicSettings, AnthropicThinkingConfig

RustyLogger.setup_logging(LoggingConfig(log_level=LogLevel.Debug))


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
    model="claude-sonnet-4-5",
    provider="anthropic",
    output_type=StructuredTaskOutput,
    model_settings=AnthropicSettings(
        thinking=AnthropicThinkingConfig(type="disabled"),
    ),
)


agent = Agent(Provider.Anthropic)

if __name__ == "__main__":
    result = agent.execute_prompt(
        prompt=prompt,
        output_type=StructuredTaskOutput,
    ).structured_output

    assert isinstance(result, StructuredTaskOutput)
    print("Tasks:", result.tasks)
