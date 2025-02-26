# type: ignore
from pydantic import BaseModel
from openai import OpenAI
import anthropic

# client = OpenAI()


# class CalendarEvent(BaseModel):
#    name: str
#    date: str
#    participants: list[str]
#
#
# print(str(CalendarEvent.model_json_schema()))
# print(str(CalendarEvent.__name__))
#
#
# client.beta.chat.completions.parse()
# client.chat.completions.create()


if __name__ == "__main__":
    client = OpenAI()
    completion = client.chat.completions.create(
        model="gpt-4o",
        messages=[
            {"role": "developer", "content": "You are a helpful assistant."},
            {"role": "user", "content": "Hello!", "name": None},
        ],
    )

    print(completion.choices[0].message)

# user query -> llm1 --> outputs --> llm2 --> outputs --> llm3 --> outputs
