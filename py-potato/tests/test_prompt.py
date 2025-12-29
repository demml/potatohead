import os
from typing import List, cast

from potato_head import ModelSettings, Prompt, Provider, Role
from potato_head.anthropic import MessageParam
from potato_head.google import GeminiContent, GeminiSettings, GenerationConfig
from potato_head.openai import ChatMessage, ImageContentPart, OpenAIChatSettings
from pydantic import BaseModel
from pydantic_ai.settings import ModelSettings as PydanticModelSettings
from potato_head.mock import LLMTestServer
from openai import OpenAI


class CityLocation(BaseModel):
    city: str
    country: str
    zip_codes: List[int]


class TaskReturn(BaseModel):
    tasks: List[str]
    status: str


def test_string_prompt():
    # test string prompt
    prompt = Prompt(
        model="gpt-4o",
        provider="openai",
        messages="My prompt",
        system_instructions="system_prompt",
    )

    assert prompt.openai_messages[0].content[0].text == "My prompt"
    assert prompt.openai_message.content[0].text == "My prompt"
    assert prompt.system_instructions[0].content[0].text == "system_prompt"

    # test string message
    prompt = Prompt(
        model="gpt-4o",
        provider=Provider.OpenAI,
        messages=ChatMessage(
            content="My prompt",
            role=Role.User.as_str(),
        ),
        system_instructions="system_prompt",
    )

    assert prompt.openai_messages[0].content[0].text == "My prompt"

    # test list of string messages
    prompt = Prompt(
        model="gpt-4o",
        provider=Provider.OpenAI,
        messages=[
            ChatMessage(content="Foo", role=Role.User.as_str()),
            ChatMessage(content="Bar", role=Role.User.as_str()),
        ],
        system_instructions="system_prompt",
    )

    messages = prompt.openai_messages

    assert messages[0].content[0].text == "Foo"
    assert messages[1].content[0].text == "Bar"

    # test list of strings
    prompt = Prompt(
        model="gpt-4o",
        provider=Provider.OpenAI,
        messages=[
            "Hello ${variable}",
            "Bar",
        ],
        system_instructions="system_prompt",
    )

    messages = cast(List[ChatMessage], prompt.messages)

    assert messages[0].content[0].text == "Hello ${variable}"
    assert messages[1].content[0].text == "Bar"

    bounded_message = prompt.bind("variable", "world").openai_messages[0]
    assert bounded_message.content[0].text == "Hello world"

    # test bind mut
    msg = prompt.openai_messages[0]
    msg.bind_mut("variable", "world")
    assert msg.content[0].text == "Hello world"


def test_bind_prompt():
    prompt = Prompt(
        model="gpt-4o",
        provider="openai",
        messages=[
            "Hello ${variable1}",
            "This is ${variable2}",
        ],
        system_instructions="system_prompt",
    )
    bound_prompt = prompt.bind("variable1", "world").bind("variable2", "Foo")
    assert bound_prompt.openai_messages[0].content[0].text == "Hello world"
    assert bound_prompt.openai_messages[1].content[0].text == "This is Foo"

    # testing binding with kwargs
    bound_prompt = prompt.bind(variable1="world")
    assert bound_prompt.openai_messages[0].content[0].text == "Hello world"

    bound_prompt = prompt.bind(variable1=10)
    assert bound_prompt.openai_messages[0].content[0].text == "Hello 10"

    bound_prompt = prompt.bind(variable1={"key": "value"})
    assert bound_prompt.openai_messages[0].content[0].text == 'Hello {"key":"value"}'

    # test bind mut
    prompt.openai_messages[0].content[0].text == "Hello ${variable1}"
    prompt.bind_mut("variable1", "world")
    assert prompt.openai_messages[0].content[0].text == "Hello world"


def test_image_prompt():
    prompt = Prompt(
        model="gpt-4o",
        provider=Provider.OpenAI,
        messages=[
            ChatMessage(
                content="What company is this logo from?",
                role=Role.User.as_str(),
            ),
            ChatMessage(
                content=ImageContentPart(url="https://iili.io/3Hs4FMg.png"),
                role=Role.User.as_str(),
            ),
        ],
    )

    messages = prompt.messages

    assert messages[0].content[0].text == "What company is this logo from?"

    # unwrap for image url will convert to expected pydantic dataclass
    assert isinstance(messages[1].content[0], ImageContentPart)


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
        messages=[
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
        messages=[
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
        messages=[
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
        messages="My prompt ${1} is ${2}",
        system_instructions="system_prompt",
        output_type=CityLocation,
    )

    assert prompt.response_json_schema is not None


def test_prompt_no_args():
    prompt = Prompt(
        model="gpt-4o",
        provider="openai",
        messages="My prompt",
        system_instructions="system_prompt",
    )

    assert prompt.openai_message.content[0].text == "My prompt"


def test_openai_model_dump():
    with LLMTestServer() as server:
        os.environ["OPENAI_API_KEY"] = "TEST"
        client = OpenAI(base_url=server.url)

        # test simple chat completion
        prompt = Prompt(
            model="gpt-4o",
            provider=Provider.OpenAI,
            messages=[
                ChatMessage(content="Foo", role=Role.User.as_str()),
                ChatMessage(content="Bar", role=Role.User.as_str()),
            ],
            system_instructions="system_prompt",
        )

        response = client.chat.completions.create(**prompt.model_dump())

        # test structured chat completion
        prompt = Prompt(
            model="gpt-4o",
            provider=Provider.OpenAI,
            messages=[
                ChatMessage(content="Foo", role=Role.User.as_str()),
                ChatMessage(content="Bar", role=Role.User.as_str()),
            ],
            system_instructions="system_prompt",
            output_type=CityLocation,
        )

        response = client.chat.completions.create(**prompt.model_dump())
        structured_output = TaskReturn.model_validate_json(
            response.choices[0].message.content
        )
        assert isinstance(structured_output, TaskReturn)
