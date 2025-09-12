from openai import OpenAI
from potato_head.mock import LLMTestServer
from potato_head import Agent, Provider, Prompt
import time
import os

os.environ["OPENAI_API_KEY"] = "mock_api_key"


def benchmark_openai_sdk():
    with LLMTestServer() as server:
        client = OpenAI(base_url=server.url)
        calls = []

        for _ in range(1000):
            start_time = time.time()
            response = client.chat.completions.create(
                model="gpt-4o",
                messages=[
                    {"role": "system", "content": "You are a helpful assistant."},
                ],
            )
            response.choices[0].message.content
            end_time = time.time()
            calls.append(end_time - start_time)

        avg_time = sum(calls) / len(calls)
        print(f"Average time per call: {avg_time * 1000:.4f} milliseconds")
        print(f"Total calls made: {len(calls)}")
        print(f"Total time taken: {sum(calls) * 1000:.4f} milliseconds")


def benchmark_potatohead_sdk():
    with LLMTestServer():
        prompt = Prompt(
            model="gpt-4o",
            provider="openai",
            message="Hello, how are you?",
            system_instruction="You are a helpful assistant.",
        )
        agent = Agent(Provider.OpenAI)

        calls = []

        for _ in range(1000):
            start_time = time.time()
            response = agent.execute_prompt(prompt=prompt, model="gpt-4o")
            response.result
            end_time = time.time()
            calls.append(end_time - start_time)

        avg_time = sum(calls) / len(calls)
        print(f"Average time per call: {avg_time * 1000:.4f} milliseconds")
        print(f"Total calls made: {len(calls)}")
        print(f"Total time taken: {sum(calls) * 1000:.4f} milliseconds")


if __name__ == "__main__":
    benchmark_openai_sdk()
    print("-----")
    benchmark_potatohead_sdk()
