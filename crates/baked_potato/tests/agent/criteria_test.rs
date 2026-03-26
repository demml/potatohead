use potato_agent::{
    AgentRunContext, CompletionCriteria, KeywordCriteria, MaxIterationsCriteria,
    StructuredOutputCriteria,
};
use serde_json::json;

fn make_ctx(iteration: u32, max: u32, responses: Vec<&str>) -> AgentRunContext {
    let mut ctx = AgentRunContext::new("test-agent".into(), max);
    ctx.iteration = iteration;
    for r in responses {
        ctx.push_response(r.to_string());
    }
    ctx
}

// ── MaxIterationsCriteria ────────────────────────────────────────────────────

#[test]
fn max_iterations_below_threshold() {
    let criteria = MaxIterationsCriteria::new(3);
    let ctx = make_ctx(1, 10, vec![]);
    assert!(!criteria.is_complete(&ctx));
}

#[test]
fn max_iterations_at_threshold() {
    let criteria = MaxIterationsCriteria::new(3);
    let ctx = make_ctx(3, 10, vec![]);
    assert!(criteria.is_complete(&ctx));
}

#[test]
fn max_iterations_above_threshold() {
    let criteria = MaxIterationsCriteria::new(3);
    let ctx = make_ctx(5, 10, vec![]);
    assert!(criteria.is_complete(&ctx));
}

#[test]
fn max_iterations_reason() {
    let criteria = MaxIterationsCriteria::new(3);
    let ctx = make_ctx(3, 10, vec![]);
    assert!(criteria.completion_reason(&ctx).contains("3"));
}

// ── KeywordCriteria ──────────────────────────────────────────────────────────

#[test]
fn keyword_present() {
    let criteria = KeywordCriteria::new("DONE");
    let ctx = make_ctx(0, 10, vec!["The task is DONE now."]);
    assert!(criteria.is_complete(&ctx));
}

#[test]
fn keyword_absent() {
    let criteria = KeywordCriteria::new("DONE");
    let ctx = make_ctx(0, 10, vec!["Still working on it."]);
    assert!(!criteria.is_complete(&ctx));
}

#[test]
fn keyword_no_responses() {
    let criteria = KeywordCriteria::new("DONE");
    let ctx = make_ctx(0, 10, vec![]);
    assert!(!criteria.is_complete(&ctx));
}

#[test]
fn keyword_only_checks_last_response() {
    let criteria = KeywordCriteria::new("DONE");
    let ctx = make_ctx(0, 10, vec!["DONE", "not done"]);
    assert!(!criteria.is_complete(&ctx));
}

// ── StructuredOutputCriteria ─────────────────────────────────────────────────

#[test]
fn structured_output_valid_json_no_schema() {
    let criteria = StructuredOutputCriteria::new(None);
    let ctx = make_ctx(0, 10, vec![r#"{"key": "value"}"#]);
    assert!(criteria.is_complete(&ctx));
}

#[test]
fn structured_output_invalid_json() {
    let criteria = StructuredOutputCriteria::new(None);
    let ctx = make_ctx(0, 10, vec!["not json at all"]);
    assert!(!criteria.is_complete(&ctx));
}

#[test]
fn structured_output_with_schema_valid() {
    let schema = json!({
        "type": "object",
        "properties": {
            "score": {"type": "integer"},
            "reason": {"type": "string"}
        },
        "required": ["score", "reason"]
    });
    let criteria = StructuredOutputCriteria::new(Some(schema));
    let ctx = make_ctx(0, 10, vec![r#"{"score": 5, "reason": "good"}"#]);
    assert!(criteria.is_complete(&ctx));
}

#[test]
fn structured_output_with_schema_invalid() {
    let schema = json!({
        "type": "object",
        "properties": {
            "score": {"type": "integer"}
        },
        "required": ["score"]
    });
    let criteria = StructuredOutputCriteria::new(Some(schema));
    let ctx = make_ctx(0, 10, vec![r#"{"name": "test"}"#]);
    assert!(!criteria.is_complete(&ctx));
}

#[test]
fn structured_output_no_responses() {
    let criteria = StructuredOutputCriteria::new(None);
    let ctx = make_ctx(0, 10, vec![]);
    assert!(!criteria.is_complete(&ctx));
}
