from potato_head.prompts import Message


def test_message():
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

    print(message.to_api_spec())
