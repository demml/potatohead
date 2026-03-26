# Callbacks

Callbacks hook into the agentic loop at four points. Each hook receives a read-only `AgentRunContext` and returns a `CallbackAction`: proceed normally, override the model response, or abort the loop.

---

## Hook Points

Hooks fire in this order during each loop iteration:

```
before_model_call()
    ↓
[LLM call]
    ↓
after_model_call()
    ↓
  (if tool calls present)
before_tool_call()      ← fires once per tool call
    ↓
[tool execution]
    ↓
after_tool_call()       ← fires once per tool call, with result
```

If no tools are called, `before_tool_call` and `after_tool_call` do not fire for that iteration.

---

## CallbackAction

Every hook returns a `CallbackAction`.

| Variant | Effect |
|---------|--------|
| `Continue` | Proceed normally |
| `OverrideResponse(String)` | Replace the model's response text with the provided string; terminates the loop as if the model had returned that text. Only valid from `after_model_call`. |
| `Abort(String)` | Stop the loop immediately; `run()` returns `Err(AgentError::CallbackAbort(msg))` |

When multiple callbacks are registered, they fire in registration order. The first `Abort` or `OverrideResponse` takes effect; remaining callbacks for that hook are still called but their return values are ignored once a non-`Continue` action has been triggered by an earlier callback.

---

## Implementing AgentCallback

```rust
use potato_agent::{AgentCallback, AgentRunOutcome, AgentRunner, SessionState};
use potato_agent::callbacks::CallbackAction;
use potato_agent::run_context::AgentRunContext;
use potato_agent::types::AgentResponse;
use potato_type::prompt::Prompt;
use potato_type::tools::ToolCall;
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug)]
struct LoggingCallback;

impl AgentCallback for LoggingCallback {
    fn before_model_call(&self, ctx: &AgentRunContext, _prompt: &Prompt) -> CallbackAction {
        println!("[{}] iteration {} — calling model", ctx.agent_id, ctx.iteration);
        CallbackAction::Continue
    }

    fn after_model_call(&self, ctx: &AgentRunContext, response: &AgentResponse) -> CallbackAction {
        println!("[{}] response: {}", ctx.agent_id, response.response_text());
        CallbackAction::Continue
    }

    fn before_tool_call(&self, ctx: &AgentRunContext, call: &ToolCall) -> CallbackAction {
        println!("[{}] calling tool: {}", ctx.agent_id, call.tool_name);
        CallbackAction::Continue
    }

    fn after_tool_call(&self, _ctx: &AgentRunContext, call: &ToolCall, result: &Value) -> CallbackAction {
        println!("tool '{}' result: {}", call.tool_name, result);
        CallbackAction::Continue
    }
}
```

Only override the hooks you need. All four methods have default implementations that return `Continue`.

### Registering a callback

```rust
use potato_agent::AgentBuilder;
use potato_type::Provider;
use std::sync::Arc;

let agent = AgentBuilder::new()
    .provider(Provider::OpenAI)
    .model("gpt-4o")
    .with_callback(Arc::new(LoggingCallback))
    .build()
    .await?;
```

---

## AgentRunContext

`AgentRunContext` is passed to every hook and provides read-only access to loop state.

| Field | Type | Description |
|-------|------|-------------|
| `agent_id` | `String` | The agent's unique ID |
| `iteration` | `u32` | Current tool-call iteration (0 on first model call) |
| `max_iterations` | `u32` | Configured limit |
| `responses` | `Vec<String>` | Accumulated text responses from prior iterations |

`ctx.responses` is empty during the first model call. It accumulates text responses if the loop continues past the first call without stopping.

---

## Usage Patterns

### Enforcing response length limits

```rust
#[derive(Debug)]
struct LengthGuard {
    max_chars: usize,
}

impl AgentCallback for LengthGuard {
    fn after_model_call(&self, _ctx: &AgentRunContext, response: &AgentResponse) -> CallbackAction {
        let text = response.response_text();
        if text.len() > self.max_chars {
            // Replace with truncated version
            let truncated = text.chars().take(self.max_chars).collect::<String>();
            CallbackAction::OverrideResponse(truncated)
        } else {
            CallbackAction::Continue
        }
    }
}
```

### Blocking specific tools

```rust
#[derive(Debug)]
struct ToolBlocklist {
    blocked: Vec<String>,
}

impl AgentCallback for ToolBlocklist {
    fn before_tool_call(&self, _ctx: &AgentRunContext, call: &ToolCall) -> CallbackAction {
        if self.blocked.contains(&call.tool_name) {
            CallbackAction::Abort(format!("tool '{}' is not allowed", call.tool_name))
        } else {
            CallbackAction::Continue
        }
    }
}
```

### Iteration budget enforcement

```rust
#[derive(Debug)]
struct BudgetGuard {
    max_tool_calls: u32,
}

impl AgentCallback for BudgetGuard {
    fn before_model_call(&self, ctx: &AgentRunContext, _prompt: &Prompt) -> CallbackAction {
        if ctx.iteration >= self.max_tool_calls {
            CallbackAction::Abort(format!(
                "exceeded tool call budget of {}",
                self.max_tool_calls
            ))
        } else {
            CallbackAction::Continue
        }
    }
}
```

---

## Callback Registration Order

Callbacks fire in the order they are registered. Register higher-priority callbacks (security, policy) before lower-priority callbacks (logging, metrics).

```rust
let agent = AgentBuilder::new()
    .with_callback(Arc::new(PolicyEnforcer))   // fires first
    .with_callback(Arc::new(MetricsCollector)) // fires second
    .with_callback(Arc::new(Logger))           // fires third
    .build()
    .await?;
```
