from typing import Any, List, Optional

class PromptType:
    Image: "PromptType"
    Chat: "PromptType"
    Vision: "PromptType"
    Voice: "PromptType"
    Batch: "PromptType"
    Embedding: "PromptType"

class ChatPartText:
    text: str
    type: str

    def __init__(self, text: str, type: str = "text") -> None:
        """Text content for a chat prompt.

        Args:
            text (str):
                The text content.
            type (str):
                The type of the content
        """

class ImageUrl:
    url: str
    detail: str

    def __init__(self, url: str, detail: str = "auto") -> None:
        """Either a URL of the image or the base64 encoded image data.

        Args:
            url (str):
                The URL of the image.
            detail (str):
                Specifies the detail level of the image.
        """

class ChatPartImage:
    image_url: ImageUrl
    type: str

    def __init__(self, image_url: ImageUrl, type: str = "image_url") -> None:
        """Image content for a chat prompt.

        Args:
            image_url (ImageUrl):
                The image URL.
            type (str):
                The type of the content.
        """

class Message:
    def __init__(self, role: str, content: str, name: Optional[str]) -> None:
        """Message class to represent a message in a chat prompt.
        Messages can be parameterized with numbered arguments in the form of
        $1, $2, $3, etc. These arguments will be replaced with the corresponding context
        when bound.

        Example:
        ```python
            message = Message("system", "Params: $1, $2")
            message.bind("world")
            message.bind("hello")
        ```

        Args:
            role (str)
                The role to assign the message. Refer to the
                specific model's documentation for possible roles.
            content (str):
                The content of the message.
            name (Optional[str]):
                An optional name for the participant.
        """

    @property
    def role(self) -> str:
        """The role of the message."""

    @property
    def content(self) -> str:
        """The content of the message."""
