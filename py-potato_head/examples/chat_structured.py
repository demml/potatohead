from potato_head import Message, ChatPrompt, Mouth
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
    response_format=MathResult,
)


if __name__ == "__main__":
    print(prompt.open_ai_spec())
    response = mouth.speak(prompt, MathResult)
    print(response)
