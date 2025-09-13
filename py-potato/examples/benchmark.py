# type: ignore
from openai import OpenAI
from potato_head.mock import LLMTestServer
from potato_head import Agent, Provider, Prompt
import time
import os
import numpy as np
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt
from typing import List

os.environ["OPENAI_API_KEY"] = "mock_api_key"


def benchmark_openai_sdk() -> List[float]:
    """Benchmark OpenAI SDK performance.

    Returns:
        List[float]: List of response times in milliseconds.
    """
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
            calls.append((end_time - start_time) * 1000)  # Convert to milliseconds

        avg_time = sum(calls) / len(calls)
        std_time = np.std(calls)
        print(f"OpenAI SDK - Average time per call: {avg_time:.4f} milliseconds")
        print(f"OpenAI SDK - Standard deviation: {std_time:.4f} milliseconds")
        print(f"OpenAI SDK - Total calls made: {len(calls)}")
        print(f"OpenAI SDK - Total time taken: {sum(calls):.4f} milliseconds")

        return calls


def benchmark_potatohead_sdk() -> List[float]:
    """Benchmark PotatoHead SDK performance.

    Returns:
        List[float]: List of response times in milliseconds.
    """
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
            calls.append((end_time - start_time) * 1000)  # Convert to milliseconds

        avg_time = sum(calls) / len(calls)
        std_time = np.std(calls)
        print(f"PotatoHead SDK - Average time per call: {avg_time:.4f} milliseconds")
        print(f"PotatoHead SDK - Standard deviation: {std_time:.4f} milliseconds")
        print(f"PotatoHead SDK - Total calls made: {len(calls)}")
        print(f"PotatoHead SDK - Total time taken: {sum(calls):.4f} milliseconds")

        return calls


def create_benchmark_chart(
    openai_times: List[float], potatohead_times: List[float]
) -> None:
    """Create a seaborn bar chart comparing OpenAI and PotatoHead performance.

    Args:
        openai_times: List of OpenAI response times in milliseconds.
        potatohead_times: List of PotatoHead response times in milliseconds.
    """

    # Create DataFrame for seaborn
    data = pd.DataFrame(
        {
            "SDK": ["OpenAI"] * len(openai_times)
            + ["PotatoHead"] * len(potatohead_times),
            "Response Time (ms)": openai_times + potatohead_times,
        }
    )

    # Set up the plot style
    plt.figure(figsize=(10, 6))
    sns.set_style("whitegrid")
    sns.set_palette("husl")

    # Create bar plot with error bars
    sns.barplot(
        data=data,
        x="SDK",
        y="Response Time (ms)",
        capsize=0.1,
        errwidth=2,
        alpha=0.8,
    )

    # Adjust layout and show
    plt.tight_layout()
    plt.show()

    # Print comparison summary
    openai_mean = np.mean(openai_times)
    openai_std = np.std(openai_times)
    potatohead_mean = np.mean(potatohead_times)
    potatohead_std = np.std(potatohead_times)

    print("\n" + "=" * 60)
    print("BENCHMARK COMPARISON SUMMARY")
    print("=" * 60)
    print(f"OpenAI SDK:     {openai_mean:.4f}ms ± {openai_std:.4f}ms")
    print(f"PotatoHead SDK: {potatohead_mean:.4f}ms ± {potatohead_std:.4f}ms")
    print(f"Samples:        {len(openai_times)} per SDK")

    if openai_mean < potatohead_mean:
        diff_percent = ((potatohead_mean - openai_mean) / openai_mean) * 100
        print(
            f"Result:         OpenAI SDK is {diff_percent:.2f}% faster than PotatoHead SDK"
        )
    else:
        diff_percent = ((openai_mean - potatohead_mean) / potatohead_mean) * 100
        print(
            f"Result:         PotatoHead SDK is {diff_percent:.2f}% faster than OpenAI SDK"
        )

    print("=" * 60)


if __name__ == "__main__":
    print("Running OpenAI SDK benchmark...")
    openai_results = benchmark_openai_sdk()

    print("\n" + "-" * 50)
    print("Running PotatoHead SDK benchmark...")
    potatohead_results = benchmark_potatohead_sdk()

    print("\n" + "-" * 50)
    print("Creating comparison chart...")
    create_benchmark_chart(openai_results, potatohead_results)
