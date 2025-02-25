from typing import Any, List, Optional
from .openai import OpenAIConfig

class PromptType:
    Image: "PromptType"
    Chat: "PromptType"
    Vision: "PromptType"
    Voice: "PromptType"
    Batch: "PromptType"
    Embedding: "PromptType"

class Message:
    def __init__(self, role: str, content: str) -> None:
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
        """

    @property
    def role(self) -> str:
        """The role of the message."""

    @property
    def content(self) -> str:
        """The content of the message."""

class ChatPrompt:
    def __init__(
        self,
        model: str,
        messages: List[Message],
        additional_data: Optional[dict[str, Any]] = None,
        response_format: Optional[Any] = None,
    ) -> None:
        """ChatPrompt for interacting with an LLM Chat API.

        Args:
            model (str):
                The model to use for the chat prompt.
            messages (List[Message]):
                The messages to use in the chat prompt.
            additional_data (Optional[dict[str, Any]]):
                Additional data to pass to the API data field.
            response_format (Optional[Any]):
                The response format to use for the chat prompt. This
                is for structured responses and will be parsed accordingly.
                If provided, must be a subclass of pydantic `BaseModel`.
        """

    @property
    def model(self) -> str:
        """The model to use for the chat prompt."""

    @property
    def messages(self) -> List[Message]:
        """The messages to use in the chat prompt."""

    @property
    def prompt_type(self) -> PromptType:
        """The prompt type to use for the chat prompt."""

    @property
    def additional_data(self) -> Optional[str]:
        """Additional data as it will be passed to the API data field."""

    def add_message(self, message: Message) -> None:
        """Add a message to the chat prompt.

        Args:
            message (Message):
                The message to add to the chat prompt.
        """

    def bind_context_at(self, context: str, index: int = 0) -> None:
        """Bind a context at a specific index in the chat prompt.

        Example with ChatPrompt that contains two messages

        ```python
            chat_prompt = ChatPrompt("gpt-3.5-turbo", [
                Message("system", "Hello, $1"),
                Message("user", "World")
            ])
            chat_prompt.bind_context_at(0, "world") # we bind "world" to the first message
        ```

        Args:
            context (str):
                The context to bind.
             index (int):
                The index to bind the context at. Index refers
                to the index of the array in which the context will be bound.
                Defaults to 0.
        """

    def deep_copy(self) -> "ChatPrompt":
        """Return a copy of the chat prompt."""

    def reset(self) -> None:
        """Reset the chat prompt to its initial state."""

    def __str__(self) -> str:
        """Return a string representation of the chat prompt."""

    def open_ai_spec(self) -> str:
        """OpenAI spec for the chat prompt. How it will be sent to the API.
        This is intended for debugging purposes. There is a equivalent method in
        rust that will return the same spec when used with a `Tongue` for fast processing.
        """

class Mouth:
    def __init__(self, config: OpenAIConfig) -> None:
        """Mouth class to interact with API Clients

        Args:
            config (OpenAIConfig):
                The configuration to use for the mouth.
        """

    def speak(
        self,
        request: ChatPrompt,
        response_format: Optional[Any] = None,
    ) -> Any:
        """Speak to the API.

        Args:
            request (ChatPrompt):
                The request to send to the API.
            response_format (Optional[Any]):
                The response format to use for the chat prompt. This
                is for structured responses and will be parsed accordingly.
                If provided, must be a subclass of pydantic
                `BaseModel`.

        Returns:
            The response from the API.
        """
