import pytest
from potato_head import Prompt


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
