from potato_head import ChatPrompt, Mouth, OpenAIConfig
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger

RustyLogger.setup_logging(LoggingConfig(log_level=LogLevel.Debug))

mouth = Mouth(OpenAIConfig())

prompt = ChatPrompt(
    model="gpt-4o",
    messages=[
        {
            "role": "user",
            "content": "What is 4 + $1 + $2?",
        },
        {
            "role": "user",
            "content": "Give me a one sentence overview of $1. Also, what was the result of my first question?",
        },
    ],
)


if __name__ == "__main__":
    prompt.bind_context_at("5", 0)
    prompt.bind_context_at("10", 0)
    prompt.bind_context_at("Mr. Potato Head", 1)

    # verify the context is bound
    print(prompt.open_ai_spec())

    response = mouth.speak(prompt)
    print(response)
