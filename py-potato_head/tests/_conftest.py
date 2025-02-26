# type: ignore
from pydantic import BaseModel
from openai import OpenAI
import anthropic

client = OpenAI()


class CalendarEvent(BaseModel):
    name: str
    date: str
    participants: list[str]


print(str(CalendarEvent.model_json_schema()))
print(str(CalendarEvent.__name__))


client.beta.chat.completions.parse
client.chat.completions.create()


from potato_head import Voice, Tongue, OpenAIPrompt

# user query -> llm1 --> outputs --> llm2 --> outputs --> llm3 --> outputs
