from typing import List

from potato_head import Prompt, Provider, Role
from potato_head.anthropic import (
    AnthropicSettings,
    AnthropicThinkingConfig,
    MessageParam,
    TextBlockParam,
)
from pydantic import BaseModel


class CityLocation(BaseModel):
    city: str
    country: str
    zip_codes: List[int]


def test_settings_init():
    settings = AnthropicSettings()
    assert settings is not None


def test_prompt():
    # test string prompt
    prompt = Prompt(
        model="claude-4.5-sonnet",
        provider="anthropic",
        messages="My prompt",
        system_instructions="system_prompt",
    )

    assert prompt.anthropic_message.text() == "My prompt"
    assert prompt.system_instructions[0].text() == "system_prompt"

    # test string message
    prompt = Prompt(
        model="claude-4.5-sonnet",
        provider=Provider.Anthropic,
        messages=MessageParam(
            content="My prompt",
            role=Role.User.as_str(),
        ),
        system_instructions="system_prompt",
    )

    assert prompt.anthropic_message.content[0].text == "My prompt"

    prompt = Prompt(
        model="claude-4.5-sonnet",
        provider=Provider.Anthropic,
        messages=MessageParam(
            content=[
                TextBlockParam(text="My prompt"),
                TextBlockParam(text="My prompt 2"),
            ],
            role=Role.User.as_str(),
        ),
        system_instructions="system_prompt",
    )

    assert prompt.anthropic_message.content[0].text == "My prompt"
    assert prompt.anthropic_message.content[1].text == "My prompt 2"


def test_bind_prompt():
    prompt = Prompt(
        model="claude-4.5-sonnet",
        provider="anthropic",
        messages=[
            "Hello ${variable1}",
            "This is ${variable2}",
        ],
        system_instructions="system_prompt",
        model_settings=AnthropicSettings(
            thinking=AnthropicThinkingConfig(type="disabled")
        ),
    )

    bound_prompt = prompt.bind("variable1", "world").bind("variable2", "Foo")
    assert bound_prompt.messages[0].content[0].text == "Hello world"
    assert bound_prompt.messages[1].content[0].text == "This is Foo"


def test_prompt_structured_output():
    # test string prompt
    prompt = Prompt(
        model="claude-4.5-sonnet",
        provider="anthropic",
        messages="My prompt",
        system_instructions="system_prompt",
        output_type=CityLocation,
    )

    assert prompt.response_json_schema is not None
