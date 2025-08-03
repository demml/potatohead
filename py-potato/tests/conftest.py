import pytest
from potato_head import Prompt  # type: ignore


@pytest.fixture(scope="module")
def prompt_step1():
    return Prompt(
        model="gpt-4o",
        provider="openai",
        message="Prompt for task 1. Context: ${1}",
        system_instruction="You are a helpful assistant.",
    )


@pytest.fixture(scope="module")
def prompt_step2():
    return Prompt(
        model="gpt-4o",
        provider="openai",
        message="Prompt for task 2. Context: ${1}",
    )
