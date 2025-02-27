from potato_head import Message, ChatPrompt, PromptType
import json


def test_basic_message():
    # Create a Message object
    message = Message(
        role="system",
        content="You are a helpful assistant. Guide us through the solution step by step",
    )

    # Check the role property
    assert message.role == "system"

    # Check the content property
    assert (
        message.content
        == "You are a helpful assistant. Guide us through the solution step by step"
    )

    json_string = message.to_api_spec()

    loaded = json.loads(json_string)

    # check that a dictionary can be fed
    ChatPrompt(
        model="GPT-4o",
        messages=[loaded],
    )


def test_api_chat_ref_example():
    ChatPrompt(
        model="gpt-4o",
        messages=[
            {"role": "developer", "content": "You are a helpful assistant."},
            {"role": "user", "content": "Hello!"},
        ],
        logprobs=True,
        top_logprobs=2,
    )


def test_api_chat_image_ref_example():
    ChatPrompt(
        model="gpt-4o",
        messages=[
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "What's in this image?"},
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": "https://upload.wikimedia.org/wikipedia/commons/thumb/d/dd/Gfp-wisconsin-madison-the-nature-boardwalk.jpg/2560px-Gfp-wisconsin-madison-the-nature-boardwalk.jpg",
                        },
                    },
                ],
            }
        ],
        max_tokens=300,
    )


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

    # reset the ChatPrompt object
    copy.reset()

    assert (
        copy.messages[0].content
        == "You are a helpful assistant. Guide us through the solution step by step"
    )
    assert copy.messages[1].content == "What is 4 + $1?"
