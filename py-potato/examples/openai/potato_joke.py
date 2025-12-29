# type: ignore
from potato_head import Agent, Prompt, Provider
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger
from potato_head.openai import OpenAIChatSettings

RustyLogger.setup_logging(LoggingConfig(log_level=LogLevel.Debug))


prompt = Prompt(
    model="gpt-5.1",
    provider="openai",
    messages="Tell me a joke about potatoes.",
    system_instructions="You are a helpful assistant.",
    model_settings=OpenAIChatSettings(
        max_completion_tokens=100,
        reasoning_effort="none",
    ),
)

if __name__ == "__main__":
    # This is just to ensure the script can be run directly
    print("Running potato joke example...")
    agent = Agent(Provider.OpenAI)
    response = agent.execute_prompt(prompt=prompt)
    print(response.response.choices[0].message.content)
    # Why did the potato win the talent show?
    # Because it was outstanding in its field!
