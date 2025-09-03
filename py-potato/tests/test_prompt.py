from typing import List

import httpx
from potato_head import (  # type: ignore
    BinaryContent,
    DocumentUrl,
    ImageUrl,
    Message,
    ModelSettings,
    Prompt,
    Provider,
)
from potato_head.openai import OpenAIChatSettings  # type: ignore
from pydantic import BaseModel
from pydantic_ai import BinaryContent as PydanticBinaryContent
from pydantic_ai import DocumentUrl as PydanticDocumentUrl
from pydantic_ai import ImageUrl as PydanticImageUrl
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
    assert prompt.message[0].unwrap() == "My prompt"
    assert prompt.system_instruction[0].unwrap() == "system_prompt"

    # test string message
    prompt = Prompt(
        model="gpt-4o",
        provider=Provider.OPENAI,
        message=Message(content="My prompt"),
        system_instruction="system_prompt",
    )

    assert prompt.message[0].unwrap() == "My prompt"

    # test list of string messages
    prompt = Prompt(
        model="gpt-4o",
        provider="openai",
        message=[
            Message(content="Foo"),
            Message(content="Bar"),
        ],
        system_instruction="system_prompt",
    )

    assert prompt.message[0].unwrap() == "Foo"
    assert prompt.message[1].unwrap() == "Bar"

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

    assert prompt.message[0].unwrap() == "Hello ${variable}"
    assert prompt.message[1].unwrap() == "Bar"

    bounded_message = prompt.message[0].bind("variable", "world").unwrap()
    assert bounded_message == "Hello world"

    # test bind mut
    msg = prompt.message[0]
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
    assert bound_prompt.message[0].unwrap() == "Hello world"
    assert bound_prompt.message[1].unwrap() == "This is Foo"

    # testing binding with kwargs
    bound_prompt = prompt.bind(variable1="world")
    assert bound_prompt.message[0].unwrap() == "Hello world"

    bound_prompt = prompt.bind(variable1=10)
    assert bound_prompt.message[0].unwrap() == "Hello 10"

    bound_prompt = prompt.bind(variable1={"key": "value"})
    assert bound_prompt.message[0].unwrap() == 'Hello {"key":"value"}'

    # test bind mut
    prompt.message[0].unwrap() == "Hello ${variable1}"
    prompt.bind_mut("variable1", "world")
    assert prompt.message[0].unwrap() == "Hello world"


def test_image_prompt():
    prompt = Prompt(
        model="gpt-4o",
        provider="openai",
        message=[
            "What company is this logo from?",
            ImageUrl(url="https://iili.io/3Hs4FMg.png"),
        ],
        system_instruction="system_prompt",
    )

    assert prompt.message[0].unwrap() == "What company is this logo from?"

    # unwrap for image url will convert to expected pydantic dataclass
    assert isinstance(prompt.message[1].unwrap(), PydanticImageUrl)


def test_binary_prompt():
    image_response = httpx.get("https://iili.io/3Hs4FMg.png")

    prompt = Prompt(
        model="gpt-4o",
        provider="openai",
        message=[
            "What company is this logo from?",
            BinaryContent(data=image_response.content, media_type="image/png"),
        ],
        system_instruction="system_prompt",
    )

    assert prompt.message[0].unwrap() == "What company is this logo from?"
    assert isinstance(prompt.message[1].unwrap(), PydanticBinaryContent)


def test_document_prompt():
    prompt = Prompt(
        model="gpt-4o",
        provider="openai",
        message=[
            "What is the main content of this document?",
            DocumentUrl(
                url="https://storage.googleapis.com/cloud-samples-data/generative-ai/pdf/2403.05530.pdf"
            ),
        ],
        system_instruction="system_prompt",
    )

    assert prompt.message[0].unwrap() == "What is the main content of this document?"
    assert isinstance(prompt.message[1].unwrap(), PydanticDocumentUrl)


def test_model_settings_prompt():
    settings = ModelSettings(
        settings=OpenAIChatSettings(
            temperature=0.5,
            max_tokens=100,
            top_p=0.9,
            frequency_penalty=0.0,
            presence_penalty=0.0,
            extra_body={"key": "value"},
        )
    )

    prompt = Prompt(
        message=[
            "My prompt ${1} is ${2}",
            "My prompt ${3} is ${4}",
        ],
        model_settings=settings,
    )

    settings = PydanticModelSettings(**prompt.model_settings.model_dump())


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
        message="My prompt",
        system_instruction="system_prompt",
    )

    assert prompt.message[0].unwrap() == "My prompt"
