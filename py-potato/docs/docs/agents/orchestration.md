# Orchestration

Two orchestration patterns are available: sequential and parallel. Both implement the `AgentRunner` trait, so they compose — a sequential agent can contain a parallel step, and vice versa.

---

## The AgentRunner Trait

Every agent and orchestrator implements `AgentRunner`:

```rust
pub trait AgentRunner: Send + Sync + Debug {
    fn id(&self) -> &str;
    async fn run(&self, input: &str, session: &mut SessionState) -> Result<AgentRunOutcome, AgentError>;
    async fn resume(&self, user_answer: &str, ctx: ResumeContext, session: &mut SessionState) -> Result<AgentRunOutcome, AgentError>;
}
```

This means a `SequentialAgent` can be wrapped in a `ParallelAgent`, and a `ParallelAgent` can be used as a step in a `SequentialAgent`.

---

## SequentialAgent

A `SequentialAgent` runs a list of agents one after another, sharing the same `SessionState` throughout. Each agent can read and write to the session. Changes from one agent are visible to the next.

```rust
use potato_agent::{AgentBuilder, AgentRunner, SessionState};
use potato_agent::orchestration::{SequentialAgent, SequentialAgentBuilder};
use potato_type::Provider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let planner = AgentBuilder::new()
        .provider(Provider::OpenAI)
        .model("gpt-4o")
        .system_prompt("You are a planner. Outline the steps to complete the user's task.")
        .build()
        .await?;

    let executor = AgentBuilder::new()
        .provider(Provider::OpenAI)
        .model("gpt-4o")
        .system_prompt("You are an executor. Carry out the plan provided to you.")
        .build()
        .await?;

    let pipeline = SequentialAgentBuilder::new()
        .then(planner as Arc<dyn AgentRunner>)
        .then(executor as Arc<dyn AgentRunner>)
        .pass_output(true)   // executor receives planner's output as its input
        .build();

    let mut session = SessionState::new();

    match pipeline.run("Write a blog post about Rust", &mut session).await? {
        AgentRunOutcome::Complete(result) => {
            println!("{}", result.final_response.response_text());
        }
        AgentRunOutcome::NeedsInput { question, resume_context } => {
            println!("Pipeline needs input: {question}");
            // call pipeline.resume(answer, resume_context, &mut session)
        }
    }

    Ok(())
}
```

### pass_output

| `pass_output` value | What each agent receives as `input` |
|--------------------|--------------------------------------|
| `false` (default) | The original `input` string passed to `pipeline.run()` |
| `true` | The previous agent's `final_response.response_text()` |

When `pass_output(true)`, the pipeline acts as a text transformation pipeline. The first agent receives the original input; subsequent agents receive the prior agent's output.

### NeedsInput propagation

If any agent in the pipeline returns `NeedsInput`, the sequential runner propagates it upward immediately. The remaining agents in the pipeline are not executed. Call `pipeline.resume(answer, resume_context, session)` to continue from the agent that paused.

---

## ParallelAgent

A `ParallelAgent` runs all agents concurrently using Tokio tasks. Each agent receives a **snapshot clone** of the session at dispatch time. Their session writes are merged back when all agents complete (user-data keys only — `__`-prefixed system keys are not merged).

```rust
use potato_agent::{AgentBuilder, AgentRunner, SessionState};
use potato_agent::orchestration::{ParallelAgent, ParallelAgentBuilder, MergeStrategy};
use potato_type::Provider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sentiment_agent = AgentBuilder::new()
        .provider(Provider::OpenAI)
        .model("gpt-4o-mini")
        .system_prompt("Analyze the sentiment of the provided text. Return: positive, negative, or neutral.")
        .build()
        .await?;

    let summary_agent = AgentBuilder::new()
        .provider(Provider::OpenAI)
        .model("gpt-4o")
        .system_prompt("Summarize the provided text in one sentence.")
        .build()
        .await?;

    let parallel = ParallelAgentBuilder::new()
        .with_agent(sentiment_agent as Arc<dyn AgentRunner>)
        .with_agent(summary_agent as Arc<dyn AgentRunner>)
        .merge_strategy(MergeStrategy::CollectAll)
        .build();

    let mut session = SessionState::new();

    match parallel.run("The new Rust release adds...", &mut session).await? {
        AgentRunOutcome::Complete(result) => {
            // result.final_response is the last agent's raw response
            // result.combined_text is a JSON array of all responses
            if let Some(combined) = &result.combined_text {
                println!("All responses: {combined}");
            }
        }
        AgentRunOutcome::NeedsInput { question, .. } => {
            println!("Needs input: {question}");
        }
    }

    Ok(())
}
```

### MergeStrategy

| Strategy | Behavior |
|----------|---------|
| `CollectAll` (default) | Waits for all agents to complete. `result.combined_text` is a JSON array of all response texts: `["response1", "response2"]`. `result.final_response` is the last agent's response. |
| `First` | Returns the first agent to complete. `result.combined_text` is `None`. Remaining in-flight agents are not cancelled — they continue running but their results are discarded. |

When using `CollectAll`, read `result.combined_text` rather than `result.final_response` for the aggregated output. The `combined_text` field is a JSON array string — parse it with `serde_json::from_str::<Vec<String>>(&combined)`.

### Session state in parallel agents

Each parallel agent receives a clone of the session snapshot taken at the time `parallel.run()` is called. Session writes from one child agent are not visible to other child agents while they are running.

When all agents complete, their session snapshots are merged back into the original session using `merge_user_data`. If two agents write the same key, the last agent to complete wins.

`__`-prefixed keys (e.g., `__ancestor_ids`) are excluded from the merge. This prevents child agents from corrupting the parent's circular call tracking state.

### NeedsInput in parallel agents

On the first `NeedsInput` from any child agent, the parallel runner returns it immediately. Other in-flight agents continue running but their results are discarded. There is no mechanism to resume only the paused agent within a parallel context.

---

## Composing Orchestrators

Sequential and parallel agents both implement `AgentRunner`, so they can be nested.

### Parallel step inside a sequential pipeline

```rust
let research = ParallelAgentBuilder::new()
    .with_agent(web_search_agent)
    .with_agent(database_agent)
    .with_agent(knowledge_base_agent)
    .merge_strategy(MergeStrategy::CollectAll)
    .build();

let synthesizer = AgentBuilder::new()
    .provider(Provider::OpenAI)
    .model("gpt-4o")
    .system_prompt("Synthesize the research results into a coherent answer.")
    .build()
    .await?;

let pipeline = SequentialAgentBuilder::new()
    .then(research as Arc<dyn AgentRunner>)
    .then(synthesizer as Arc<dyn AgentRunner>)
    .pass_output(true)
    .build();
```

When `pass_output(true)` and the prior step was a parallel agent, the synthesizer receives `result.final_response.response_text()` — which is the last agent's response, not the `combined_text`. To pass all parallel results, write them to a session key in a callback or tool rather than relying on `pass_output`.

### Sequential steps inside a parallel agent

```rust
let pipeline_a = SequentialAgentBuilder::new()
    .then(step_a1)
    .then(step_a2)
    .pass_output(true)
    .build();

let pipeline_b = SequentialAgentBuilder::new()
    .then(step_b1)
    .then(step_b2)
    .pass_output(true)
    .build();

let parallel = ParallelAgentBuilder::new()
    .with_agent(pipeline_a)
    .with_agent(pipeline_b)
    .build();
```

Each pipeline runs its steps sequentially; the two pipelines run concurrently with each other.

---

## Sub-Agents via Tools (Dynamic Delegation)

For cases where the parent agent decides at runtime which sub-agent to call (rather than a fixed pipeline), use `with_sub_agent()` to register agents as tools. See the [Tools documentation](tools.md#sub-agents-as-tools) for details.

---

## SequentialAgentBuilder Reference

| Method | Description |
|--------|-------------|
| `new()` | Create builder |
| `then(Arc<dyn AgentRunner>)` | Append agent to pipeline |
| `pass_output(bool)` | If true, each agent receives prior agent's output as input |
| `build()` | Returns `Arc<SequentialAgent>` |

## ParallelAgentBuilder Reference

| Method | Description |
|--------|-------------|
| `new()` | Create builder |
| `with_agent(Arc<dyn AgentRunner>)` | Add agent to parallel group |
| `merge_strategy(MergeStrategy)` | Set merge strategy (default: `CollectAll`) |
| `build()` | Returns `Arc<ParallelAgent>` |
