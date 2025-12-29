# type: ignore

### The following example does the following:
# 1. Reformulates a user query to be more feature-rich and keyword-dense.
# 2. Evaluates the quality of the reformulated query.
# 3. Returns a score and reason for how well the reformulation improves the query.

from potato_head import Agent, Prompt, Provider, Score, Task, Workflow
from potato_head.google import GeminiSettings, GenerationConfig, GeminiThinkingConfig
from potato_head.logging import LoggingConfig, LogLevel, RustyLogger

RustyLogger.setup_logging(LoggingConfig(log_level=LogLevel.Debug))


def create_reformulation_evaluation_prompt():
    """
    Builds a prompt for evaluating the quality of a reformulated query.

    Returns:
        Prompt: A prompt that asks for a JSON evaluation of the reformulation.

    Example:
        >>> prompt = create_reformulation_evaluation_prompt()
    """
    return Prompt(
        messages=(
            "You are an expert evaluator of search query reformulations. "
            "Given the original user query and its reformulated version, your task is to assess how well the reformulation improves the query. "
            "Consider the following criteria:\n"
            "- Does the reformulation make the query more explicit and comprehensive?\n"
            "- Are relevant synonyms, related concepts, or specific features added?\n"
            "- Is the original intent preserved without changing the meaning?\n"
            "- Is the reformulation clear and unambiguous?\n\n"
            "Provide your evaluation as a JSON object with the following attributes:\n"
            "- score: An integer from 1 (poor) to 5 (excellent) indicating the overall quality of the reformulation.\n"
            "- reason: A brief explanation for your score.\n\n"
            "Format your response as:\n"
            "{\n"
            '  "score": <integer 1-5>,\n'
            '  "reason": "<your explanation>"\n'
            "}\n\n"
            "Original Query:\n"
            "${user_query}\n\n"
            "Reformulated Query:\n"
            "${reformulated_query}\n\n"
            "Evaluation:"
        ),
        model="gemini-2.5-flash",
        provider="gemini",
        model_settings=GeminiSettings(
            generation_config=GenerationConfig(
                thinking_config=GeminiThinkingConfig(thinking_budget=0),
            ),
        ),
    )


def create_query_reformulation_prompt():
    """Builds a prompt for query reformulation tasks."""
    return Prompt(
        messages=(
            "You are an expert at query reformulation. Your task is to take a user's original search query "
            "and rewrite it to be more feature-rich and keyword-dense, so it better aligns with the user's intent "
            "and improves search results.\n\n"
            "Guidelines:\n"
            "- Expand abbreviations and clarify ambiguous terms.\n"
            "- Add relevant synonyms, related concepts, and specific features.\n"
            "- Preserve the original intent, but make the query more explicit and comprehensive.\n"
            "- Do not change the meaning of the query.\n"
            "- Return only the reformulated query.\n\n"
            "User Query:\n"
            "${user_query}\n\n"
            "Reformulated Query:"
        ),
        model="gemini-2.5-flash",
        provider="gemini",
        model_settings=GeminiSettings(
            generation_config=GenerationConfig(
                thinking_config=GeminiThinkingConfig(thinking_budget=0),
            ),
        ),
    )


def create_workflow():
    """
    Creates a workflow that combines query reformulation and evaluation.

    Returns:
        Workflow: A workflow that reformulates a query and evaluates the reformulation.
    """
    eval_prompt = create_reformulation_evaluation_prompt()
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
    prompt = create_query_reformulation_prompt()

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
