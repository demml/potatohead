# type: ignore
from potato_head import Agent, Prompt, Provider, Task, Workflow
from pydantic import BaseModel


class TaskOutput(BaseModel):
    result: int


task_one_prompt = Prompt(
    message="What is 1 + 1?",
    model="gpt-4o",
    provider="openai",
    response_format=TaskOutput,
)

task_two_prompt = Prompt(
    message="Take the result of the previous task and multiply it by 2.",
    model="gpt-4o",
    provider="openai",
    response_format=TaskOutput,
)

agent = Agent(Provider.OpenAI)

workflow = Workflow(name="potato_workflow_example")

# add your agent to the workflow
workflow.add_agent(agent)

# create tasks and add them to the workflow
workflow.add_task(
    Task(
        prompt=task_one_prompt,
        agent_id=agent.id,
        id="task_one",
    ),
    TaskOutput,
)

workflow.add_task(
    Task(
        prompt=task_two_prompt,
        agent_id=agent.id,
        dependencies=["task_one"],
        id="task_two",
    ),
    TaskOutput,
)

if __name__ == "__main__":
    # This is just to ensure the script can be run directly
    print("Running potato workflow example...")
    result = workflow.run()
    print(result.tasks["task_two"].result.result)  # Access the result of task_two
