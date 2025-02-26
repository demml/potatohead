from potato_head.parts import Mouth  # type: ignore
from potato_head.prompts import ChatPrompt, Message
from potato_head.openai import OpenAIConfig
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger
from pydantic import BaseModel


RustyLogger.setup_logging(LoggingConfig(log_level=LogLevel.Debug))


mouth = Mouth(OpenAIConfig())


class MathResult(BaseModel):
    result: int


prompt = ChatPrompt(
    model="gpt-4o",
    messages=[Message("user", "What is 4 + 1?")],
)


if __name__ == "__main__":
    response = mouth.speak(prompt, MathResult)
    print(response)
