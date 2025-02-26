from potato_head.prompts import Message, ChatPrompt
from potato_head.openai import OpenAIConfig
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger
from potato_head.parts import Mouth  # type: ignore


RustyLogger.setup_logging(LoggingConfig(log_level=LogLevel.Debug))


mouth = Mouth(OpenAIConfig())

prompt = ChatPrompt(
    model="gpt-4o-mini",
    messages=[{"role": "user", "content": "What's 1+1? Answer in one word."}],
    temperature=0,
    stream=True,
)


if __name__ == "__main__":
    response = mouth.stream_speak(prompt)

    for message in response:
        print(message)
