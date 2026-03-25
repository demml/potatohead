# Tools

Tools let an agent call Rust functions during its agentic loop. When the model decides to use a tool, the runtime executes the registered function, appends the result to the conversation, and continues the loop. The model calls tools by name; the runtime dispatches to the matching implementation.

---

## The Tool Trait

Sync tools implement four methods.

```rust
use potato_type::tools::Tool;
use potato_type::TypeError;
use serde_json::{json, Value};

#[derive(Debug)]
struct WeatherTool;

impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "get_weather"
    }

    fn description(&self) -> &str {
        "Get the current temperature for a city in Celsius"
    }

    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "The city name"
                }
            },
            "required": ["city"]
        })
    }

    fn execute(&self, args: Value) -> Result<Value, TypeError> {
        let city = args["city"]
            .as_str()
            .ok_or_else(|| TypeError::Error("missing city".into()))?;

        Ok(json!({ "city": city, "temperature_celsius": 22 }))
    }
}
```

`parameter_schema` is sent verbatim to the LLM as a JSON Schema. The model generates arguments matching that schema; `execute` receives those arguments as a `serde_json::Value`.

Return a JSON object with descriptive keys so the model can parse the output. Return `Err(TypeError::Error(...))` to surface a tool failure as `AgentError` — this stops the loop.

### Registering a sync tool

```rust
let agent = AgentBuilder::new()
    .provider(Provider::OpenAI)
    .model("gpt-4o")
    .with_tool(WeatherTool)         // accepts any type implementing Tool + 'static
    .build()
    .await?;
```

---

## The AsyncTool Trait

For tools that call external services or do async I/O, implement `AsyncTool`.

```rust
use async_trait::async_trait;
use potato_type::tools::AsyncTool;
use potato_type::TypeError;
use serde_json::{json, Value};

#[derive(Debug)]
struct DatabaseLookupTool;

#[async_trait]
impl AsyncTool for DatabaseLookupTool {
    fn name(&self) -> &str {
        "lookup_record"
    }

    fn description(&self) -> &str {
        "Look up a record by ID in the application database"
    }

    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Record ID" }
            },
            "required": ["id"]
        })
    }

    async fn execute(&self, args: Value) -> Result<Value, TypeError> {
        let id = args["id"].as_str().unwrap_or_default();
        // async database call
        Ok(json!({ "id": id, "name": "example record", "status": "active" }))
    }
}
```

### Registering an async tool

```rust
use std::sync::Arc;

let agent = AgentBuilder::new()
    .provider(Provider::OpenAI)
    .model("gpt-4o")
    .with_async_tool(Arc::new(DatabaseLookupTool))
    .build()
    .await?;
```

### Dispatch order

The runtime checks async tools first, then sync tools. If you register the same tool name as both async and sync, the async variant takes precedence.

---

## Sub-Agents as Tools

An `AgentRunner` can be wrapped as a tool, allowing one agent to call another as part of its tool-use loop. This is the primary mechanism for composing specialized agents.

```rust
use potato_agent::{AgentBuilder, AgentRunner, SessionState};
use potato_type::Provider;
use std::sync::Arc;

// Build a specialized sub-agent
let research_agent = AgentBuilder::new()
    .provider(Provider::OpenAI)
    .model("gpt-4o")
    .system_prompt("You are a research assistant. Return factual summaries.")
    .max_iterations(3)
    .build()
    .await?;

// Register it as a tool on a parent agent
let orchestrator = AgentBuilder::new()
    .provider(Provider::OpenAI)
    .model("gpt-4o")
    .system_prompt("You are an orchestrator. Delegate research to the research tool.")
    .with_sub_agent(
        "research",
        "Research a topic and return a factual summary",
        research_agent as Arc<dyn AgentRunner>,
    )
    .build()
    .await?;
```

The sub-agent tool has a fixed parameter schema: `{"type": "object", "properties": {"input": {"type": "string"}}, "required": ["input"]}`. The model passes its query as the `input` field. The tool runs the sub-agent's full agentic loop and returns the final text response as the tool result.

### Sub-agent tool policy

You can restrict what sub-agents are allowed to call using `AgentToolPolicy`.

```rust
use potato_agent::tool_ext::AgentToolPolicy;
use std::collections::HashSet;

let policy = AgentToolPolicy {
    disallow_sub_agent_calls: true,   // sub-agent cannot call any AgentTools
    disallowed_agent_ids: HashSet::new(),
};

let orchestrator = AgentBuilder::new()
    .provider(Provider::OpenAI)
    .model("gpt-4o")
    .with_sub_agent_policy(
        "research",
        "Research a topic",
        research_agent as Arc<dyn AgentRunner>,
        policy,
    )
    .build()
    .await?;
```

| Policy field | Effect |
|-------------|--------|
| `disallow_sub_agent_calls: true` | Sub-agent cannot invoke any tools that are `AgentTool` instances |
| `disallowed_agent_ids` | Sub-agent cannot call any agent in the blocklist by agent ID |

---

## Circular Call Prevention

The runtime tracks ancestor agent IDs in `SessionState` under the reserved key `__ancestor_ids`. Before dispatching a sub-agent tool, it checks whether the target agent's ID is already in the ancestor chain. If it is, `AgentError::CircularAgentCall` is returned.

This applies regardless of nesting depth. If A calls B calls C calls A, the third dispatch is rejected.

The `__ancestor_ids` key is system-reserved and is not merged back to the parent session when a child agent completes.

---

## Tool Trait Reference

### `Tool` (sync)

| Method | Signature | Required |
|--------|-----------|----------|
| `name` | `fn name(&self) -> &str` | Yes |
| `description` | `fn description(&self) -> &str` | Yes |
| `parameter_schema` | `fn parameter_schema(&self) -> Value` | Yes |
| `execute` | `fn execute(&self, args: Value) -> Result<Value, TypeError>` | Yes |

Implement `Send + Sync + Debug` in addition to the trait methods.

### `AsyncTool`

Same as `Tool` but `execute` is `async`:

```rust
async fn execute(&self, args: Value) -> Result<Value, TypeError>
```

Register with `Arc::new(my_tool)` and `.with_async_tool(...)`.
