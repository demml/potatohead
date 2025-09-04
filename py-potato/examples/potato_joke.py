# type: ignore
from potato_head import Agent, Prompt, Provider
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger

RustyLogger.setup_logging(LoggingConfig(log_level=LogLevel.Debug))


prompt = Prompt(
    model="o4-mini",
    provider="openai",
    message="Tell me a joke about potatoes.",
    system_instruction="You are a helpful assistant.",
)

if __name__ == "__main__":
    # This is just to ensure the script can be run directly
    print("Running potato joke example...")
    agent = Agent(Provider.OpenAI)
    response = agent.execute_prompt(prompt=prompt)

    print(response.result)
    # Why did the potato win the talent show?
    # Because it was outstanding in its field!
