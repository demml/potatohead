from typing import Optional

class OpenAIConfig:
    def __init__(
        self,
        api_key: Optional[str] = None,
        url: Optional[str] = None,
        organization: Optional[str] = None,
        project: Optional[str] = None,
    ) -> None:
        """OpenAIConfig for configuring the OpenAI API.
        api_key and url will be sourced from the environment if not provided.

        Args:
            api_key (Optional[str]):
                The API key to use for the OpenAI API.
            url (Optional[str]):
                The URL to use for the OpenAI API.
        """
