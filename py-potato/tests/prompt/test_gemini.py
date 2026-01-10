from potato_head import Prompt, Provider, Role
from potato_head.google import (
    GeminiContent,
    GeminiSettings,
    GeminiThinkingConfig,
    GenerationConfig,
    Part,
)
from pydantic import BaseModel
from typing import List


class CityLocation(BaseModel):
    city: str
    country: str
    zip_codes: List[int]


def test_gemini_settings_init():
    settings = GeminiSettings()
    assert settings is not None


def test_generation_config_init():
    config = GenerationConfig(
        top_k=5,
        top_p=0.9,
        temperature=0.8,
        thinking_config=GeminiThinkingConfig(
            include_thoughts=True,
            thinking_budget=1000,
        ),
    )
    assert config is not None


def test_prompt():
    # test string prompt
    prompt = Prompt(
        model="gemini-3.0-flash",
        provider="gemini",
        messages="My prompt",
        system_instructions="system_prompt",
    )

    assert prompt.gemini_message.parts[0].data == "My prompt"
    assert prompt.system_instructions[0].parts[0].data == "system_prompt"

    # test string message
    prompt = Prompt(
        model="gemini-3.0-flash",
        provider=Provider.Gemini,
        messages=GeminiContent(
            parts="My prompt",
            role=Role.User.as_str(),
        ),
        system_instructions="system_prompt",
    )

    assert prompt.gemini_message.parts[0].data == "My prompt"

    prompt = Prompt(
        model="gemini-3.0-flash",
        provider=Provider.Gemini,
        messages=GeminiContent(
            parts=[
                Part(data="My prompt"),
                Part(data="My prompt 2"),
            ],
            role=Role.User.as_str(),
        ),
        system_instructions="system_prompt",
    )

    assert prompt.gemini_message.parts[0].data == "My prompt"
    assert prompt.gemini_message.parts[1].data == "My prompt 2"


def test_bind_prompt():
    prompt = Prompt(
        model="gemini-3.0-flash",
        provider="gemini",
        messages=[
            "Hello ${variable1}",
            "This is ${variable2}",
        ],
        system_instructions="system_prompt",
        model_settings=GeminiSettings(
            generation_config=GenerationConfig(temperature=0.7)
        ),
    )

    bound_prompt = prompt.bind("variable1", "world").bind("variable2", "Foo")
    assert bound_prompt.messages[0].parts[0].data == "Hello world"
    assert bound_prompt.messages[1].parts[0].data == "This is Foo"


def test_prompt_structured_output():
    # test string prompt
    prompt = Prompt(
        model="gemini-3.0-flash",
        provider="gemini",
        messages="My prompt",
        system_instructions="system_prompt",
        output_type=CityLocation,
    )

    assert prompt.response_json_schema is not None

    print(prompt.response_json_schema)
    a
