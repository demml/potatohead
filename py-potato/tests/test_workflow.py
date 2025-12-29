import os
from dataclasses import dataclass

import pytest
from potato_head import Agent, Prompt, Provider, Score, Task, TaskStatus, Workflow
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger
from potato_head.mock import LLMTestServer
from potato_head.openai import TextContentPart
from pydantic_ai import Agent as PydanticAgent
from pydantic_ai import RunContext, models
from pydantic_ai.models.test import TestModel

from tests.conftest import (  # type: ignore
    StructuredTaskOutput,
    anthropic_task,
    gemini_task,
    openai_task,
)

# Sets up logging for tests
RustyLogger.setup_logging(LoggingConfig(log_level=LogLevel.Debug))


models.ALLOW_MODEL_REQUESTS = False
os.environ["OPENAI_API_KEY"] = "mock_api_key"


@dataclass
class Prompts:
    prompt_step1: Prompt
    prompt_step2: Prompt


def test_simple_workflow(prompt_step1: Prompt):
    agent = PydanticAgent(
        prompt_step1.model_identifier,
        system_prompt=prompt_step1.system_instructions[0].content[0].text,
    )

    with agent.override(model=TestModel()):
        agent.run_sync(prompt_step1.message.content[0].text)


def test_simple_dep_workflow(prompt_step1: Prompt, prompt_step2: Prompt):
    agent = PydanticAgent(
        prompt_step1.model_identifier,
        system_prompt=prompt_step1.system_instructions[0].content[0].text,
        deps_type=Prompts,
    )

    @agent.system_prompt
    def get_system_instruction(ctx: RunContext[Prompts]) -> str:
        return ctx.deps.prompt_step1.system_instructions[0].content[0].text

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
        system_prompt=prompt_step1.system_instructions[0].content[0].text,
        deps_type=Prompts,
    )

    @agent.tool
    def bind_context(ctx: RunContext[Prompts], search_query: str) -> str:
        msg = ctx.deps.prompt_step1.openai_message.bind("1", search_query)
        content = msg.content[0]
        if isinstance(content, TextContentPart):
            return content.text
        raise ValueError("Unexpected content type")

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
    with LLMTestServer():
        prompt = Prompt(
            messages="Hello, how are you?",
            system_instructions="You are a helpful assistant.",
            model="gpt-4o",
            provider="openai",
        )
        agent = Agent(Provider.OpenAI)
        agent.execute_task(Task(prompt=prompt, agent_id=agent.id, id="task1"))


def test_potato_agent_no_model():
    with LLMTestServer():
        prompt = Prompt(
            model="gpt-4o",
            provider="openai",
            messages="Hello, how are you?",
            system_instructions="You are a helpful assistant.",
        )
        agent = Agent(Provider.OpenAI)

        agent.execute_task(Task(prompt=prompt, agent_id=agent.id, id="task1"))
        agent.execute_prompt(prompt=prompt)


def test_potato_head_workflow():
    with LLMTestServer():
        prompt = Prompt(
            messages="Hello, how are you?",
            system_instructions="You are a helpful assistant.",
            model="gpt-4o",
            provider="openai",
        )

        open_agent1 = Agent(Provider.OpenAI)
        open_agent2 = Agent(Provider.OpenAI)

        workflow = Workflow(name="test_workflow")  # expand named argument to allow agents and tasks
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
    with LLMTestServer():
        prompt = Prompt(
            messages="Hello, how are you?",
            system_instructions="You are a helpful assistant.",
            model="gpt-4o",
            provider="openai",
            output_type=StructuredTaskOutput,
        )

        agent = Agent(Provider.OpenAI)
        result = agent.execute_task(
            Task(prompt=prompt, agent_id=agent.id, id="task1"),
            output_type=StructuredTaskOutput,
        )

        assert isinstance(result.structured_output, StructuredTaskOutput)


def test_potato_head_workflow_structured_output():
    with LLMTestServer():
        prompt = Prompt(
            messages="Hello, how are you?",
            system_instructions="You are a helpful assistant.",
            model="gpt-4o",
            provider="openai",
        )

        open_agent1 = Agent(Provider.OpenAI)
        open_agent2 = Agent(Provider.OpenAI)

        workflow = Workflow(name="test_workflow")  # expand named argument to allow agents and tasks
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


def test_potato_head_structured_output_score_openai():
    with LLMTestServer():
        prompt = Prompt(
            messages="Hello, how are you?",
            system_instructions="You are a helpful assistant.",
            model="gpt-4o",
            provider="openai",
            output_type=Score,
        )

        agent = Agent(Provider.OpenAI)
        result = agent.execute_task(
            Task(prompt=prompt, agent_id=agent.id, id="task1"),
            output_type=Score,
        )

        assert isinstance(result.structured_output, Score)
        assert result.structured_output.score is not None
        assert result.structured_output.reason is not None


def test_potato_head_structured_output_score_gemini():
    with LLMTestServer():
        prompt = Prompt(
            messages="Hello, how are you?",
            system_instructions="You are a helpful assistant.",
            model="gemini-2.5-flash",
            provider="gemini",
            output_type=Score,
        )

        agent = Agent(Provider.Gemini)
        result = agent.execute_task(
            Task(prompt=prompt, agent_id=agent.id, id="task1"),
            output_type=Score,
        )

        assert isinstance(result.structured_output, Score)
        assert result.structured_output.score is not None
        assert result.structured_output.reason is not None


def test_potato_head_execute_prompt():
    with LLMTestServer():
        prompt = Prompt(
            messages="Hello, how are you?",
            system_instructions="You are a helpful assistant.",
            model="gpt-4o",
            provider="openai",
            output_type=Score,
        )

        agent = Agent(Provider.OpenAI)
        result = agent.execute_prompt(prompt=prompt, output_type=Score)

        assert isinstance(result.structured_output, Score)


def test_workflow_param_binding():
    with LLMTestServer():
        # this should return a Score object
        start_prompt = Prompt(
            messages="Hello, how are you?",
            system_instructions="You are a helpful assistant.",
            model="gpt-4o",
            provider="openai",
            output_type=Score,
        )

        # this should receive the score and reason from the previous step returned response
        # we assert this at the end of the test
        param_prompt = Prompt(
            messages="The score is ${score} and the reason is ${reason}.",
            model="gpt-4o",
            provider="openai",
        )

        agent = Agent(Provider.OpenAI)
        workflow = Workflow(name="test_workflow")  # expand named argument to allow agents and tasks
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
        assert len(result.tasks["param_task"].prompt.all_messages) == 2


def test_potato_head_workflow_serialization():
    with LLMTestServer():
        prompt = Prompt(
            messages="Hello, how are you?",
            system_instructions="You are a helpful assistant.",
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


def test_agent_env_var_failure():
    """
    Test that the agent fails when the environment variable is not set.
    """
    prompt = Prompt(
        messages="Hello, how are you?",
        system_instructions="You are a helpful assistant.",
        model="gpt-4o",
        provider="openai",
        output_type=Score,
    )

    agent = Agent(Provider.OpenAI)
    with pytest.raises(
        RuntimeError,
        match="Missing authentication information. Failed to find API_KEY or credentials in environment variables",
    ):
        # This should raise an error because the API key is not set
        agent.execute_prompt(prompt=prompt, output_type=Score)


def test_multi_provider_workflow():
    with LLMTestServer():
        openai_agent = Agent(Provider.OpenAI)
        gemini_agent = Agent(Provider.Gemini)
        anthropic_agent = Agent(Provider.Anthropic)

        workflow = Workflow(name="multi_provider_workflow")
        workflow.add_agents([openai_agent, gemini_agent, anthropic_agent])

        ## add tasks for each provider
        workflow.add_tasks(
            [
                openai_task(openai_agent.id),
                gemini_task(gemini_agent.id),
                anthropic_task(anthropic_agent.id),
            ]
        )

        ## add output types for structured outputs
        workflow.add_task_output_types(
            {
                "gemini_task": Score,
                "anthropic_task": StructuredTaskOutput,
            }
        )

        result = workflow.run()

        assert isinstance(result.tasks["gemini_task"].result, Score)
        assert isinstance(result.result, StructuredTaskOutput)
