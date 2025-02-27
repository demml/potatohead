from potato_head.test import LLMTestServer
from potato_head.prompts import Message, ChatPrompt
from potato_head.openai import OpenAIConfig
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger
from potato_head.parts import Mouth  # type: ignore

RustyLogger.setup_logging(LoggingConfig(log_level=LogLevel.Debug))


def test_mouth():
    with LLMTestServer():
        mouth = Mouth(OpenAIConfig())
        prompt = ChatPrompt(
            model="gpt-4o",
            messages=[
                Message("user", "What is 4 + 1?"),
            ],
        )
        mouth.speak(prompt)

        prompt = ChatPrompt(
            model="gpt-4o",
            messages=[
                Message("user", "What is 4 + 1?"),
            ],
            stream=True,
        )
        mouth.speak_stream(prompt)
