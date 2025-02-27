from typing import Optional

class AnthropicConfig:
    def __init__(
        self,
        api_key: Optional[str] = None,
        url: Optional[str] = None,
        version: Optional[str] = None,
    ) -> None:
        """AnthropicConfig for configuring the OpenAI API.
        api_key and url will be sourced from the environment if not provided.

        Args:
            api_key (str):
                The API key to use for the OpenAI API.
            url (str):
                The URL to use for the OpenAI API.
            version (str):
                The version of the API to use.
        """
