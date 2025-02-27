from potato_head import ChatPrompt
from openai import OpenAI

client = OpenAI()

prompt = ChatPrompt(
    model="gpt-4o",
    messages=[
        {"role": "developer", "content": "You are a helpful assistant."},
        {"role": "user", "content": "What is 4 + 1?"},
    ],
    n=1,
    temperature=0.7,
    max_completion_tokens=100,
)


if __name__ == "__main__":
    spec = prompt.to_open_ai_request()
    response = client.chat.completions.create(**spec)
    print(response)
