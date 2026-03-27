# From Spec

Agents and workflows can be defined declaratively in YAML and loaded at runtime using `SpecLoader`. This keeps agent configuration out of application code — models, memory settings, criteria, and callbacks are all expressed in a file rather than scattered across builder calls.

---

## YAML Format

A spec file has two top-level keys: `agents` and `workflows`.

```yaml
agents:
  - id: summarizer
    provider: anthropic
    model: claude-haiku-4-5
    system_prompt: |
      You are a summarization assistant.
    max_iterations: 3
    memory:
      type: windowed
      window_size: 5
    criteria:
      - type: max_iterations
        max: 3
      - type: keyword
        keyword: "DONE"
    callbacks:
      - type: logging
    tools: []

workflows:
  - id: pipeline
    type: sequential
    pass_output: true
    steps:
      - ref: summarizer
      - id: analyzer
        provider: anthropic
        model: claude-haiku-4-5
        system_prompt: Analyze the summary.
        criteria: []
        callbacks: []
        tools: []
```

### Agent fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique identifier — used to reference this agent in workflows and `LoadedSpec::agent()` |
| `provider` | string | yes | `openai`, `gemini`, `anthropic`, or `vertex` |
| `model` | string | no | Model name (e.g. `gpt-4o`, `gemini-2.5-flash`, `claude-haiku-4-5`) |
| `system_prompt` | string | no | System instruction text |
| `max_iterations` | int | no | Maximum iterations before stopping (overridden by a `max_iterations` criterion if both are set) |
| `memory` | object | no | See [Memory](#memory) |
| `criteria` | array | no | See [Criteria](#criteria) |
| `callbacks` | array | no | See [Callbacks](#callbacks) |
| `tools` | array | no | Named tool references — must be registered on the loader with `register_async_tool` |

### Memory

```yaml
memory:
  type: windowed
  window_size: 10
```

```yaml
memory:
  type: in_memory
```

| `type` | Description |
|--------|-------------|
| `in_memory` | Unbounded turn history |
| `windowed` | Sliding window over the last N turns; requires `window_size` |

### Criteria

```yaml
criteria:
  - type: max_iterations
    max: 5
  - type: keyword
    keyword: "DONE"
  - type: structured_output
    # schema: { ... }   # optional JSON Schema; omit to stop on any valid JSON
```

| `type` | Fields | Description |
|--------|--------|-------------|
| `max_iterations` | `max: int` | Stop after N iterations |
| `keyword` | `keyword: string` | Stop when the response contains the keyword (case-sensitive substring match) |
| `structured_output` | `schema: object` (optional) | Stop when the response is valid JSON. If `schema` is provided, the response must also validate against it. |

### Callbacks

```yaml
callbacks:
  - type: logging          # built-in
  - name: my_callback      # registered at load time
```

Built-in callbacks: `logging`. Custom callbacks are registered on the loader with `register_callback` before loading.

### Workflow types

**Sequential** — runs agents one after another:

```yaml
workflows:
  - id: pipeline
    type: sequential
    pass_output: true     # each agent receives the prior agent's output
    steps:
      - ref: summarizer   # reference an agent defined in the agents list
      - id: analyzer      # or define inline
        provider: openai
        model: gpt-4o
        criteria: []
        callbacks: []
        tools: []
```

**Parallel** — runs agents concurrently:

```yaml
workflows:
  - id: multi_analysis
    type: parallel
    merge_strategy: collect_all   # or: first
    steps:
      - ref: sentiment_agent
      - ref: summary_agent
```

**DAG (Workflow)** — tasks with explicit dependencies:

```yaml
workflows:
  - id: dag
    type: workflow
    tasks:
      - id: fetch
        agent: researcher
        prompt: "Gather background information on ${topic}"
        dependencies: []
      - id: synthesize
        agent: writer
        prompt: "Write a report based on the research"
        dependencies: [fetch]
```

`prompt` in a DAG task is a plain string sent as the user message for that task. It is not a full `Prompt` object — the loader constructs one internally using the agent's provider and model. For tasks with dependencies, upstream task results are available through parameter binding in the prompt string.

---

## Loading from a string

```rust
use potato_spec::SpecLoader;

let yaml = r#"
agents:
  - id: assistant
    provider: openai
    model: gpt-4o
    system_prompt: "You are a helpful assistant."
    max_iterations: 1
workflows: []
"#;

let loaded = SpecLoader::from_spec(yaml).await?;
let agent = loaded.agent("assistant").expect("agent not found");
```

## Loading from a file

```rust
use potato_spec::SpecLoader;

let loaded = SpecLoader::from_spec_path("agents/production.yaml").await?;
let agent = loaded.agent("summarizer").expect("agent not found");
```

## Registering tools and callbacks

Tools and named callbacks referenced in the YAML must be registered before loading. Use `SpecLoader::new()` and chain registrations:

```rust
use potato_spec::SpecLoader;
use std::sync::Arc;

let loaded = SpecLoader::new()
    .register_async_tool("web_search", Arc::new(WebSearchTool))
    .register_callback("my_callback", Arc::new(MyCallback))
    .load_file("agents/production.yaml")
    .await?;
```

Built-in callbacks (`type: logging`) are resolved automatically without registration.

---

## Accessing the result

`SpecLoader` returns a `LoadedSpec` with accessors for each object type:

| Method | Returns | Description |
|--------|---------|-------------|
| `agent(id)` | `Option<Arc<Agent>>` | Individual agent by ID |
| `sequential(id)` | `Option<Arc<SequentialAgent>>` | Sequential pipeline by ID |
| `parallel(id)` | `Option<Arc<ParallelAgent>>` | Parallel group by ID |
| `workflow(id)` | `Option<&Workflow>` | DAG workflow by ID |

---

## Complete example

**`agents/pipeline.yaml`**

```yaml
agents:
  - id: planner
    provider: openai
    model: gpt-4o
    system_prompt: "Outline the steps to complete the user's task."
    max_iterations: 1

  - id: executor
    provider: openai
    model: gpt-4o
    system_prompt: "Carry out the plan provided to you."
    max_iterations: 1

workflows:
  - id: plan_and_execute
    type: sequential
    pass_output: true
    steps:
      - ref: planner
      - ref: executor
```

**Rust**

```rust
use potato_spec::SpecLoader;
use potato_agent::{AgentRunOutcome, AgentRunner, SessionState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let loaded = SpecLoader::from_spec_path("agents/pipeline.yaml").await?;
    let pipeline = loaded.sequential("plan_and_execute").expect("workflow not found");

    let mut session = SessionState::new();

    match pipeline.run("Write a blog post about Rust", &mut session).await? {
        AgentRunOutcome::Complete(result) => {
            println!("{}", result.final_response.response_text());
        }
        AgentRunOutcome::NeedsInput { question, .. } => {
            println!("Needs input: {question}");
        }
    }

    Ok(())
}
```
