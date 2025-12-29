from typing import List

import pytest
from potato_head import Prompt, Provider, Role, Task
from potato_head._potato_head import Score
from potato_head.anthropic import MessageParam
from potato_head.google import GeminiContent
from potato_head.openai import ChatMessage
from pydantic import BaseModel


class StructuredTaskOutput(BaseModel):
    tasks: List[str]
    status: str


@pytest.fixture(scope="module")
def prompt_step1():
    return Prompt(
        model="gpt-4o",
        provider="openai",
        messages="Prompt for task 1. Context: ${1}",
        system_instructions="You are a helpful assistant.",
    )


@pytest.fixture(scope="module")
def prompt_step2():
    return Prompt(
        model="gpt-4o",
        provider="openai",
        messages="Prompt for task 2. Context: ${1}",
    )


def openai_task(agent_id: str) -> Task:
    prompt = Prompt(
        model="gpt-4o",
        provider=Provider.OpenAI,
        messages=ChatMessage(content="Hello, how are you?", role=Role.User.as_str()),
        output_type=Score,
    )

    return Task(
        prompt=prompt,
        agent_id=agent_id,
        id="openai_task",
    )


def gemini_task(agent_id: str) -> Task:
    prompt = Prompt(
        model="gpt-4o",
        provider=Provider.Gemini,
        messages=GeminiContent(
            parts="You are a helpful assistant",
            role=Role.User.as_str(),
        ),
        output_type=Score,
    )

    return Task(
        prompt=prompt,
        agent_id=agent_id,
        id="gemini_task",
        dependencies=["openai_task"],
    )


def anthropic_task(agent_id: str) -> Task:
    prompt = Prompt(
        model="claude-4.5-sonnet",
        provider=Provider.Anthropic,
        messages=MessageParam(
            content="Give me a task list!",
            role=Role.User.as_str(),
        ),
        output_type=StructuredTaskOutput,
    )

    return Task(
        prompt=prompt,
        agent_id=agent_id,
        id="anthropic_task",
        dependencies=["gemini_task"],
    )
