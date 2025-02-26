from potato_head.prompts import Message, ChatPrompt
from potato_head.openai import OpenAIConfig
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger
from potato_head.parts import Mouth  # type: ignore


RustyLogger.setup_logging(LoggingConfig(log_level=LogLevel.Info))


mouth = Mouth(OpenAIConfig())

prompt = ChatPrompt(
    model="gpt-4o-mini",
    messages=[
        {
            "role": "user",
            "content": "Print the numbers from 1 to 10. One number at a time.",
        }
    ],
    temperature=0,
    stream=True,
)


if __name__ == "__main__":
    response = mouth.stream_speak(prompt)

    for message in response:
        for choice in message.choices:
            if choice.delta.content:  # Check if content exists
                print(choice.delta.content, end="", flush=True)
    print()
