from wormtongue import ChatPrompt, PromptType, Message
from pydantic import BaseModel
from pydantic import TypeAdapter
from typing import List, TypedDict


def test_chat_prompt():
    # Create a ChatPrompt object
    chat_prompt = ChatPrompt(
        model="GPT-4o",
        messages=[
            Message(
                role="system",
                content="You are a helpful assistant. Guide us through the solution step by step",
            ),
            Message(role="user", content="What is 4 + $1?"),
        ],
    )

    # Bind a context at a specific index in the ChatPrompt object
    chat_prompt.bind_context_at(context="5", index=1)

    # Check the model property
    assert chat_prompt.model == "GPT-4o"

    # Check the messages property
    assert len(chat_prompt.messages) == 2

    # Check the prompt_type property
    assert chat_prompt.prompt_type == PromptType.Chat

    # Check the additional_data property
    assert chat_prompt.additional_data is None

    copy = chat_prompt.deep_copy()

    print(copy.open_ai_spec())

    # reset the ChatPrompt object
    copy.reset()

    assert (
        copy.messages[0].content
        == "You are a helpful assistant. Guide us through the solution step by step"
    )
    assert copy.messages[1].content == "What is 4 + $1?"


def test_chat_prompt_with_pydantic_model():
    class ChatResponse(BaseModel):
        result: int

    # Create a ChatPrompt object with response_format
    chat_prompt = ChatPrompt(
        model="GPT-4o",
        messages=[
            Message(
                role="system",
                content="You are a helpful assistant. Guide us through the solution step by step",
            ),
            Message(role="user", content="What is 4 + $1?"),
        ],
        response_format=ChatResponse,
    )

    # Check the model property
    assert chat_prompt.model == "GPT-4o"

    # Check the messages property
    assert len(chat_prompt.messages) == 2

    chat_prompt.open_ai_spec()


def test_chat_prompt_with_pydantic_type_adaptor():
    class User(BaseModel):
        name: str
        id: int

    user_list_adapter = TypeAdapter(List[User])

    isinstance(user_list_adapter, TypeAdapter)

    # Create a ChatPrompt object with response_format
    chat_prompt = ChatPrompt(
        model="GPT-4o",
        messages=[
            Message(
                role="system",
                content="You are a helpful assistant. Guide us through the solution step by step",
            ),
            Message(role="user", content="What is 4 + $1?"),
        ],
        response_format=user_list_adapter,
    )

    # Check the model property
    assert chat_prompt.model == "GPT-4o"

    # Check the messages property
    assert len(chat_prompt.messages) == 2

    chat_prompt.open_ai_spec()
