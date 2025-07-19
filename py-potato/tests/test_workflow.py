import os
from dataclasses import dataclass
from typing import List

from potato_head import (  # type: ignore
    Agent,
    Prompt,
    Provider,
    Score,
    Task,
    TaskStatus,
    Workflow,
)
from potato_head.mock import OpenAITestServer  # type: ignore
from pydantic import BaseModel
from pydantic_ai import Agent as PydanticAgent
from pydantic_ai import RunContext, models
from pydantic_ai.models.test import TestModel

models.ALLOW_MODEL_REQUESTS = False
os.environ["OPENAI_API_KEY"] = "mock_api_key"


class StructuredTaskOutput(BaseModel):
    tasks: List[str]
    status: str


@dataclass
class Prompts:
    prompt_step1: Prompt
    prompt_step2: Prompt


def test_simple_workflow(prompt_step1: Prompt):
    agent = PydanticAgent(
        prompt_step1.model_identifier,
        system_prompt=prompt_step1.system_message[0].unwrap(),
    )

    with agent.override(model=TestModel()):
        agent.run_sync(prompt_step1.user_message[0].unwrap())


def test_simple_dep_workflow(prompt_step1: Prompt, prompt_step2: Prompt):
    agent = PydanticAgent(
        prompt_step1.model_identifier,
        system_prompt=prompt_step1.system_message[0].unwrap(),
        deps_type=Prompts,
    )

    @agent.system_prompt
    def get_system_message(ctx: RunContext[Prompts]) -> str:
        return ctx.deps.prompt_step1.system_message[0].unwrap()

    with agent.override(model=TestModel()):
        agent.run_sync(
            "hello",
            deps=Prompts(
                prompt_step1=prompt_step1,
                prompt_step2=prompt_step2,
            ),
        )


def test_binding_workflow(prompt_step1: Prompt, prompt_step2: Prompt):
    agent = PydanticAgent(
        "openai:gpt-4o",
        system_prompt=prompt_step1.system_message[0].unwrap(),
        deps_type=Prompts,
    )

    @agent.tool
    def bind_context(ctx: RunContext[Prompts], search_query: str) -> str:
        bound = ctx.deps.prompt_step1.user_message[0].bind("1", search_query).unwrap()
        return bound

    with agent.override(model=TestModel()):
        result = agent.run_sync(
            "potatohead",
            deps=Prompts(
                prompt_step1=prompt_step1,
                prompt_step2=prompt_step2,
            ),
        )

        # make sure tool was called
        assert result.all_messages()[2].parts[0].tool_name == "bind_context"  # type: ignore


def test_potato_head_task_execution():
    with OpenAITestServer():
        prompt = Prompt(
            user_message="Hello, how are you?",
            system_message="You are a helpful assistant.",
            model="gpt-4o",
            provider="openai",
        )
        agent = Agent(Provider.OpenAI)
        agent.execute_task(Task(prompt=prompt, agent_id=agent.id, id="task1"))


def test_potato_head_workflow():
    with OpenAITestServer():
        prompt = Prompt(
            user_message="Hello, how are you?",
            system_message="You are a helpful assistant.",
            model="gpt-4o",
            provider="openai",
        )

        open_agent1 = Agent(Provider.OpenAI)
        open_agent2 = Agent(Provider.OpenAI)

        workflow = Workflow(
            name="test_workflow"
        )  # expand named argument to allow agents and tasks
        workflow.add_agent(open_agent1)  # allow adding list of agents
        workflow.add_agent(open_agent2)
        workflow.add_task(  # allow adding list of tasks
            Task(
                prompt=prompt,
                agent_id=open_agent1.id,
                id="task1",
            ),
        )

        workflow.add_task(
            Task(
                prompt=prompt,
                agent_id=open_agent1.id,
                id="task2",
            ),
        )
        workflow.add_task(
            Task(
                prompt=prompt,
                agent_id=open_agent2.id,  # maybe default this to first agent if none
                id="task3",
                dependencies=["task1", "task2"],
            ),
        )

        workflow.add_task(
            Task(
                prompt=prompt,
                agent_id=open_agent2.id,
                id="task4",
                dependencies=["task1"],
            ),
        )

        workflow.add_task(
            Task(
                prompt=prompt,
                agent_id=open_agent1.id,
                id="task5",
                dependencies=["task4", "task3"],
            ),
        )

        workflow.run()


def test_potato_head_structured_output():
    with OpenAITestServer():
        prompt = Prompt(
            user_message="Hello, how are you?",
            system_message="You are a helpful assistant.",
            model="gpt-4o",
            provider="openai",
            response_format=StructuredTaskOutput,
        )

        agent = Agent(Provider.OpenAI)
        result = agent.execute_task(
            Task(prompt=prompt, agent_id=agent.id, id="task1"),
            output_type=StructuredTaskOutput,
        )

        assert isinstance(result.result, StructuredTaskOutput)


def test_potato_head_workflow_structured_output():
    with OpenAITestServer():
        prompt = Prompt(
            user_message="Hello, how are you?",
            system_message="You are a helpful assistant.",
            model="gpt-4o",
            provider="openai",
        )

        open_agent1 = Agent(Provider.OpenAI)
        open_agent2 = Agent(Provider.OpenAI)

        workflow = Workflow(
            name="test_workflow"
        )  # expand named argument to allow agents and tasks
        workflow.add_agent(open_agent1)  # allow adding list of agents
        workflow.add_agent(open_agent2)
        workflow.add_task(  # allow adding list of tasks
            Task(
                prompt=prompt,
                agent_id=open_agent1.id,
                id="task1",
            ),
            output_type=StructuredTaskOutput,  # specify output type for task
        )

        workflow.add_task(
            Task(
                prompt=prompt,
                agent_id=open_agent1.id,
                id="task2",
            ),
        )
        workflow.add_task(
            Task(
                prompt=prompt,
                agent_id=open_agent2.id,  # maybe default this to first agent if none
                id="task3",
                dependencies=["task1", "task2"],
            ),
        )

        workflow.add_task(
            Task(
                prompt=prompt,
                agent_id=open_agent2.id,
                id="task4",
                dependencies=["task1"],
            ),
        )

        workflow.add_task(
            Task(
                prompt=prompt,
                agent_id=open_agent1.id,
                id="task5",
                dependencies=["task4", "task3"],
            ),
        )

        result = workflow.run()

        assert isinstance(result.tasks["task1"].result, StructuredTaskOutput)
        assert result.tasks["task1"].status == TaskStatus.Completed
        assert result.tasks["task2"].status == TaskStatus.Completed
        assert result.tasks["task3"].status == TaskStatus.Completed
        assert result.tasks["task4"].status == TaskStatus.Completed
        assert len(result.events) > 0


def test_potato_head_structured_output_score():
    with OpenAITestServer():
        prompt = Prompt(
            user_message="Hello, how are you?",
            system_message="You are a helpful assistant.",
            model="gpt-4o",
            provider="openai",
            response_format=Score,
        )

        agent = Agent(Provider.OpenAI)
        result = agent.execute_task(
            Task(prompt=prompt, agent_id=agent.id, id="task1"),
            output_type=Score,
        )

        assert isinstance(result.result, Score)


def test_potato_head_execute_prompt():
    with OpenAITestServer():
        prompt = Prompt(
            user_message="Hello, how are you?",
            system_message="You are a helpful assistant.",
            model="gpt-4o",
            provider="openai",
            response_format=Score,
        )

        agent = Agent(Provider.OpenAI)
        result = agent.execute_prompt(prompt=prompt, output_type=Score)

        assert isinstance(result.result, Score)


def test_workflow_param_binding():
    with OpenAITestServer():
        # this should return a Score object
        start_prompt = Prompt(
            user_message="Hello, how are you?",
            system_message="You are a helpful assistant.",
            model="gpt-4o",
            provider="openai",
            response_format=Score,
        )

        # this should receive the score and reason from the previous step returned response
        # we assert this at the end of the test
        param_prompt = Prompt(
            user_message="The score is ${score} and the reason is ${reason}.",
            model="gpt-4o",
            provider="openai",
        )

        agent = Agent(Provider.OpenAI)
        workflow = Workflow(
            name="test_workflow"
        )  # expand named argument to allow agents and tasks
        workflow.add_agent(agent)
        workflow.add_task(
            Task(
                prompt=start_prompt,
                agent_id=agent.id,
                id="start_task",
            ),
            output_type=Score,  # specify output type for task
        )
        workflow.add_task(
            Task(
                prompt=param_prompt,
                agent_id=agent.id,
                id="param_task",
                dependencies=["start_task"],
            ),
        )
        result = workflow.run()

        # param task should contain first task output as context
        assert len(result.tasks["param_task"].prompt.user_message) == 2


def test_potato_head_workflow_serialization():
    with OpenAITestServer():
        prompt = Prompt(
            user_message="Hello, how are you?",
            system_message="You are a helpful assistant.",
            model="gpt-4o",
            provider="openai",
        )

        open_agent1 = Agent(Provider.OpenAI)

        workflow = Workflow(name="test_workflow")
        workflow.add_agent(open_agent1)  # allow adding list of agents
        workflow.add_task(  # allow adding list of tasks
            Task(
                prompt=prompt,
                agent_id=open_agent1.id,
                id="task1",
            ),
        )

        dumped = workflow.model_dump_json()
        loaded = Workflow.model_validate_json(dumped)

        assert isinstance(loaded, Workflow)
        assert loaded.name == "test_workflow"
        assert len(loaded.agents) == 1
