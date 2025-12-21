# type: ignore
from typing import List

from potato_head import Agent, Prompt, Provider
from pydantic import BaseModel


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

if __name__ == "__main__":
    result: StructuredTaskOutput = agent.execute_prompt(
        prompt=prompt,
        output_type=StructuredTaskOutput,
    ).result

    assert isinstance(result, StructuredTaskOutput)
    print("Tasks:", result.tasks)
