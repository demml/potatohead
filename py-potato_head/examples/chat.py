from potato_head.prompts import Message, ChatPrompt
from potato_head.openai import OpenAIConfig
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger
from potato_head.parts import Mouth  # type: ignore


RustyLogger.setup_logging(LoggingConfig(log_level=LogLevel.Debug))


mouth = Mouth(OpenAIConfig())

prompt = ChatPrompt(
    model="gpt-4o",
    messages=[
        Message("user", "What is 4 + 1?"),
    ],
)


if __name__ == "__main__":
    response = mouth.speak(prompt)
    print(response)
