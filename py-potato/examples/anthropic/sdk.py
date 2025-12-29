# type: ignore
from typing import List

from potato_head import Prompt
from pydantic import BaseModel
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger
from potato_head.anthropic import AnthropicSettings, AnthropicThinkingConfig
from anthropic import Anthropic

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

# come back to this later, looks like claude sdk docs mismatch the actual sdk
# this doesnt work because beta.messages.create does not recognize output_format even though its in the docs
if __name__ == "__main__":
    client = Anthropic()

    message = client.beta.messages.create(**prompt.model_dump())
    StructuredTaskOutput.model_validate_json(message.content[0].text)
