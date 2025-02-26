from potato_head.prompts import Message, ChatPrompt
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
