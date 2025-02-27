from potato_head import Message, ChatPrompt, Mouth, OpenAIConfig
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger

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
