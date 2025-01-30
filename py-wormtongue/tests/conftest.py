from pydantic import BaseModel


class CalendarEvent(BaseModel):
    name: str
    date: str
    participants: list[str]


print(str(CalendarEvent.model_json_schema()))
print(str(CalendarEvent.__name__))
