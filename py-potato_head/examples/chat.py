from potato_head import Message, ChatPrompt, Mouth
from potato_head.openai import OpenAIConfig


mouth = Mouth(OpenAIConfig())

message = Message("user", "What is 4 + 1?")
prompt = ChatPrompt(model="gpt-4o", messages=[message])


if __name__ == "__main__":
    response = mouth.speak(prompt)
    print(response)
