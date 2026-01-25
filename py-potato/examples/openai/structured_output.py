# type: ignore
from typing import List

from potato_head import Agent, Prompt, Provider
from pydantic import BaseModel
from potato_head.openai import OpenAIChatSettings


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
    model="gpt-5.1",
    provider="openai",
    output_type=StructuredTaskOutput,
    model_settings=OpenAIChatSettings(
        max_completion_tokens=100,
        reasoning_effort="none",
    ),
)

agent = Agent(Provider.OpenAI)

if __name__ == "__main__":
    result = agent.execute_prompt(
        prompt=prompt,
        output_type=StructuredTaskOutput,
    ).structured_output

    assert isinstance(result, StructuredTaskOutput)
    print("Tasks:", result.tasks)
