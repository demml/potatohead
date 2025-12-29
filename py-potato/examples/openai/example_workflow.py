# type: ignore
from potato_head import Agent, Prompt, Provider, Task, Workflow
from pydantic import BaseModel


class TaskOutput(BaseModel):
    result: int


task_one_prompt = Prompt(
    messages="What is 1 + 1?",
    model="gpt-4o",
    provider="openai",
    output_type=TaskOutput,
)

task_two_prompt = Prompt(
    messages="Take the result of the previous task and multiply it by 2.",
    model="gpt-4o",
    provider="openai",
    output_type=TaskOutput,
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
    output = workflow.run().result
    print(output)  # Access the result of task_two
    assert isinstance(output, TaskOutput)
