### The following example does the following:
# 1. Reformulates a user query to be more feature-rich and keyword-dense.
# 2. Evaluates the quality of the reformulated query.
# 3. Returns a score and reason for how well the reformulation improves the query.

from pathlib import Path
from potato_head import Agent, Prompt, Provider, Score, Task, Workflow
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger

RustyLogger.setup_logging(LoggingConfig(log_level=LogLevel.Debug))

_PARENT_PATH = Path(__file__).parent


def create_workflow():
    """
    Creates a workflow that combines query reformulation and evaluation.

    Returns:
        Workflow: A workflow that reformulates a query and evaluates the reformulation.
    """
    eval_prompt = Prompt.from_path(_PARENT_PATH / "evaluation.yaml")
    agent = Agent(Provider.Gemini)

    workflow = Workflow("Query Reformulation and Evaluation Workflow")
    workflow.add_agent(agent)
    workflow.add_task(
        task=Task(
            agent_id=agent.id,
            id="Reformulate Query",
            prompt=eval_prompt,
        ),
        output_type=Score,
    )

    return workflow


if __name__ == "__main__":
    # Example usage
    prompt = Prompt.from_path(_PARENT_PATH / "reformulation.yaml")
    agent = Agent(Provider.Gemini)
    user_query = "How do I find good post-hardcore bands?"
    response = agent.execute_prompt(prompt=prompt.bind(user_query=user_query))
    workflow = create_workflow()
    result = workflow.run(
        global_context={
            "user_query": user_query,
            "reformulated_query": response.structured_output,
        }
    )

    print(result.tasks["Reformulate Query"].result)
