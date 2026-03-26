# Completion Criteria

Completion criteria determine when the agentic loop stops after the model produces a text response. By default (no criteria configured), the loop stops on the first text response. Criteria let you continue the loop until a specific condition is met.

---

## How Criteria Work

After the model produces a text response (no tool calls), the runtime evaluates all registered criteria. If **any** criterion is satisfied, the loop terminates and `AgentRunOutcome::Complete` is returned. If none are satisfied, the assistant message is appended to the prompt and the loop continues.

Criteria are evaluated against `AgentRunContext`, which includes the accumulated text responses from all prior iterations. `ctx.last_response()` returns the most recent assistant message.

The loop also stops when `iteration >= max_iterations`, regardless of criteria.

---

## Stop on Keyword

Stops when the model's response contains a specific substring.

```rust
let agent = AgentBuilder::new()
    .provider(Provider::OpenAI)
    .model("gpt-4o")
    .system_prompt("When you have completed the task, end your response with DONE.")
    .stop_on_keyword("DONE")
    .build()
    .await?;
```

The check is case-sensitive and uses substring matching (`response.contains(keyword)`).

---

## Stop on Structured Output

Stops when the model produces a valid JSON response (optionally matching a schema).

### Stop on any valid JSON

```rust
.stop_on_structured_output(None)
```

### Stop on JSON matching a specific schema

```rust
.stop_on_structured_output(Some(serde_json::json!({
    "type": "object",
    "properties": {
        "answer": { "type": "string" },
        "confidence": { "type": "number" }
    },
    "required": ["answer", "confidence"]
})))
```

When a schema is provided, the response must be valid JSON **and** validate against the schema. Use this with a system prompt that instructs the model to return JSON. If the model returns plain text, the loop continues.

---

## Combining Criteria

Multiple criteria can be chained. The loop stops when **any** criterion is met.

```rust
let agent = AgentBuilder::new()
    .provider(Provider::OpenAI)
    .model("gpt-4o")
    .stop_on_keyword("COMPLETE")
    .stop_on_structured_output(Some(schema))
    .max_iterations(10)
    .build()
    .await?;
```

This agent stops when:
- The response contains `"COMPLETE"`, OR
- The response is valid JSON matching `schema`, OR
- 10 tool-call iterations are exhausted

The `completion_reason` field on `AgentRunResult` tells you which criterion fired.

---

## Custom Criteria

Implement the `CompletionCriteria` trait for custom stop logic.

```rust
use potato_agent::criteria::CompletionCriteria;
use potato_agent::run_context::AgentRunContext;

#[derive(Debug)]
struct MinLengthCriteria {
    min_chars: usize,
}

impl CompletionCriteria for MinLengthCriteria {
    fn is_complete(&self, ctx: &AgentRunContext) -> bool {
        ctx.last_response()
            .map(|r| r.len() >= self.min_chars)
            .unwrap_or(false)
    }

    fn completion_reason(&self, _ctx: &AgentRunContext) -> String {
        format!("response reached minimum length of {} chars", self.min_chars)
    }
}
```

Custom criteria are not yet exposed through `AgentBuilder`. Attach them directly to an `Agent` instance after building if needed (see the `criteria` field on `Agent`).

---

## Default Behavior (No Criteria)

When no criteria are registered, the loop stops on the first text response. This is the most common case for simple question-answer agents.

For agents that need to reason through multiple steps before producing a final answer, add a stopping condition so the loop knows when the model's answer is actually final rather than intermediate reasoning.

---

## MaxIterationsCriteria

`max_iterations` is not a `CompletionCriteria` instance — it is an upper bound that the loop checks before each model call. Exhausting all iterations on tool calls (never producing a text response) returns `Err(AgentError::MaxIterationsExceeded(n))`. Exhausting iterations after at least one text response returns `Complete` with `completion_reason = "reached max iterations (N)"`.

Increase `max_iterations` if you see `MaxIterationsExceeded` errors. The default is `10`.
