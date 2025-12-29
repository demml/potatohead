from typing import List, cast

from potato_head import ModelSettings, Prompt, Provider
from potato_head.anthropic import MessageParam
from potato_head.google import GeminiContent, GeminiSettings, GenerationConfig
from potato_head.openai import ChatMessage, ImageContentPart, OpenAIChatSettings
from pydantic import BaseModel
from pydantic_ai.settings import ModelSettings as PydanticModelSettings


class CityLocation(BaseModel):
    city: str
    country: str
    zip_codes: List[int]


def test_string_prompt():
    # test string prompt
    prompt = Prompt(
        model="gpt-4o",
        provider="openai",
        message="My prompt",
        system_instruction="system_prompt",
    )

    assert prompt.messages[0].content == "My prompt"
    assert prompt.system_instructions[0].content == "system_prompt"

    # test string message
    prompt = Prompt(
        model="gpt-4o",
        provider=Provider.OpenAI,
        message=ChatMessage(content="My prompt"),
        system_instruction="system_prompt",
    )

    assert prompt.messages[0].content == "My prompt"

    # test list of string messages
    prompt = Prompt(
        model="gpt-4o",
        provider="openai",
        messages=[
            ChatMessage(content="Foo"),
            ChatMessage(content="Bar"),
        ],
        system_instructions="system_prompt",
    )

    messages = prompt.messages

    assert messages[0].content == "Foo"
    assert messages[1].content == "Bar"

    # test list of strings
    prompt = Prompt(
        model="gpt-4o",
        provider="openai",
        message=[
            "Hello ${variable}",
            "Bar",
        ],
        system_instruction="system_prompt",
    )

    messages = cast(List[ChatMessage], prompt.messages)

    assert messages[0].content == "Hello ${variable}"
    assert messages[1].content == "Bar"

    bounded_message = prompt.bind("variable", "world").messages[0]
    assert bounded_message == "Hello world"

    # test bind mut
    msg = cast(ChatMessage, prompt.messages[0])
    msg.bind_mut("variable", "world")
    assert msg.unwrap() == "Hello world"


def test_bind_prompt():
    prompt = Prompt(
        model="gpt-4o",
        provider="openai",
        message=[
            "Hello ${variable1}",
            "This is ${variable2}",
        ],
        system_instruction="system_prompt",
    )
    bound_prompt = prompt.bind("variable1", "world").bind("variable2", "Foo")
    assert bound_prompt.messages[0].content == "Hello world"
    assert bound_prompt.messages[1].content == "This is Foo"

    # testing binding with kwargs
    bound_prompt = prompt.bind(variable1="world")
    assert bound_prompt.messages[0].content == "Hello world"

    bound_prompt = prompt.bind(variable1=10)
    assert bound_prompt.messages[0].content == "Hello 10"

    bound_prompt = prompt.bind(variable1={"key": "value"})
    assert bound_prompt.messages[0].content == 'Hello {"key":"value"}'

    # test bind mut
    prompt.messages[0].unwrap() == "Hello ${variable1}"
    prompt.bind_mut("variable1", "world")
    assert prompt.messages[0].content == "Hello world"


def test_image_prompt():
    prompt = Prompt(
        model="gpt-4o",
        provider=Provider.OpenAI,
        messages=[
            ChatMessage(content="What company is this logo from?"),
            ChatMessage(
                content=ImageContentPart(url="https://iili.io/3Hs4FMg.png"),
            ),
        ],
        system_instructions="system_prompt",
    )

    messages = prompt.messages

    assert messages[0].content == "What company is this logo from?"

    # unwrap for image url will convert to expected pydantic dataclass
    assert isinstance(messages[1], ImageContentPart)


def test_model_settings_prompt():
    settings = ModelSettings(
        settings=OpenAIChatSettings(
            temperature=0.5,
            max_completion_tokens=500,
            top_p=0.9,
            frequency_penalty=0.0,
            presence_penalty=0.0,
            extra_body={"key": "value"},
        )
    )

    prompt = Prompt(
        model="gpt-4o",
        provider="openai",
        message=[
            "My prompt ${1} is ${2}",
            "My prompt ${3} is ${4}",
        ],
        model_settings=settings,
    )

    settings = PydanticModelSettings(**prompt.model_settings.model_dump())


def test_openai_settings_direct():
    # test openai settings directly
    Prompt(
        model="gpt-4o",
        provider="openai",
        message=[
            "My prompt ${1} is ${2}",
            "My prompt ${3} is ${4}",
        ],
        model_settings=OpenAIChatSettings(
            temperature=0.5,
            max_completion_tokens=500,
            top_p=0.9,
            frequency_penalty=0.0,
            presence_penalty=0.0,
            extra_body={"key": "value"},
        ),
    )


def test_gemini_settings_direct():
    # test gemini settings directly
    Prompt(
        model="gpt-4o",
        provider="gemini",
        message=[
            "My prompt ${1} is ${2}",
            "My prompt ${3} is ${4}",
        ],
        model_settings=GeminiSettings(
            generation_config=GenerationConfig(temperature=0.5)
        ),
    )


def test_prompt_response_format():
    prompt = Prompt(
        model="gpt-4o",
        provider="openai",
        message="My prompt ${1} is ${2}",
        system_instruction="system_prompt",
        response_format=CityLocation,
    )

    assert prompt.response_json_schema is not None


def test_prompt_no_args():
    prompt = Prompt(
        model="gpt-4o",
        provider="openai",
        message="My prompt",
        system_instruction="system_prompt",
    )

    assert prompt.messages[0].content == "My prompt"
