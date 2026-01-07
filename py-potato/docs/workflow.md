# Workflows

`Potato Head` includes early support for agentic workflows. This feature was originally built for LLM monitoring in `Scouter`, allowing users to define and run workflows in Python, then serialize or deserialize them for execution in Rust—without needing a Python runtime. While current workflow capabilities are limited, the system is designed to be extensible for future enhancements.

You can think of the current workflow system as a DAG (Directed Acyclic Graph) of tasks, where each task consists of a prompt and an agent to execute the prompt.

## Example Workflow

```python
from potato_head import Prompt, Agent, Provider, Workflow, Task
from pydantic import BaseModel


class TaskOutput(BaseModel):
    result: int


task_one_prompt = Prompt(
    messages="What is 1 + 1?",
    model="gpt-4o",
    provider="openai",
    response_format=TaskOutput,
)

task_two_prompt = Prompt(
    messages="Take the result of the previous task and multiply it by 2.",
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
    TaskOutput, # (1)
)

workflow.add_task(
    Task(
        prompt=task_two_prompt,
        agent_id=agent.id,
        dependencies=["task_one"], # (2)
        id="task_two",
    ),
    TaskOutput,
)

if __name__ == "__main__":
    # This is just to ensure the script can be run directly
    print("Running potato workflow example...")
    result = workflow.run()
    print(result.tasks["task_two"].result.result)  # Access the result of task_two
```

1. When adding a task, you can specify an optional `output_type` that tells the system what Python type to parse the LLM's response into at runtime. This should match the `response_format` defined in the `Prompt`
2. You can specify dependencies for a task, which means that the task will only be executed after the specified tasks have been completed. In addition, multiple tasks can depend on the same task, allowing for complex workflows to be created and also allows for tasks to be executed in parallel if they do not depend on each other.


