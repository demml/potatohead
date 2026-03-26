use potato_agent::{
    AgentBuilder, AgentCallback, AgentRunContext, AgentRunner, CallbackAction, SessionState,
};
use potato_type::prompt::Prompt;
use potato_type::tools::ToolCall;
use potato_type::Provider;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

/// Callback that counts model calls.
#[derive(Debug)]
struct CountingCallback {
    model_calls: AtomicU32,
    tool_calls: AtomicU32,
}

impl CountingCallback {
    fn new() -> Self {
        Self {
            model_calls: AtomicU32::new(0),
            tool_calls: AtomicU32::new(0),
        }
    }

    fn model_count(&self) -> u32 {
        self.model_calls.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    fn tool_count(&self) -> u32 {
        self.tool_calls.load(Ordering::Relaxed)
    }
}

impl AgentCallback for CountingCallback {
    fn before_model_call(&self, _ctx: &AgentRunContext, _prompt: &Prompt) -> CallbackAction {
        self.model_calls.fetch_add(1, Ordering::Relaxed);
        CallbackAction::Continue
    }

    fn before_tool_call(&self, _ctx: &AgentRunContext, _call: &ToolCall) -> CallbackAction {
        self.tool_calls.fetch_add(1, Ordering::Relaxed);
        CallbackAction::Continue
    }
}

/// Callback that aborts after N model calls.
#[derive(Debug)]
struct AbortCallback {
    abort_after: u32,
    calls: AtomicU32,
}

impl AbortCallback {
    fn new(abort_after: u32) -> Self {
        Self {
            abort_after,
            calls: AtomicU32::new(0),
        }
    }
}

impl AgentCallback for AbortCallback {
    fn before_model_call(&self, _ctx: &AgentRunContext, _prompt: &Prompt) -> CallbackAction {
        let n = self.calls.fetch_add(1, Ordering::Relaxed) + 1;
        if n > self.abort_after {
            CallbackAction::Abort(format!("aborted after {} calls", self.abort_after))
        } else {
            CallbackAction::Continue
        }
    }
}

#[test]
fn counting_callback_tracks_model_calls() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let cb = Arc::new(CountingCallback::new());

    let agent = runtime.block_on(async {
        AgentBuilder::new()
            .provider(Provider::OpenAI)
            .model("gpt-4o")
            .max_iterations(1)
            .with_callback(cb.clone())
            .build()
            .await
            .unwrap()
    });

    let mut session = SessionState::new();
    runtime.block_on(async {
        let _ = agent.run("hello", &mut session).await;
    });

    assert!(cb.model_count() >= 1);
    mock.stop_server().unwrap();
}

#[test]
fn abort_callback_stops_run() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = baked_potato::LLMTestServer::new();
    mock.start_server().unwrap();

    let cb = Arc::new(AbortCallback::new(0)); // abort immediately

    let agent = runtime.block_on(async {
        AgentBuilder::new()
            .provider(Provider::OpenAI)
            .model("gpt-4o")
            .max_iterations(5)
            .with_callback(cb)
            .build()
            .await
            .unwrap()
    });

    let mut session = SessionState::new();
    let result = runtime.block_on(async { agent.run("hello", &mut session).await });

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("aborted"));
    mock.stop_server().unwrap();
}
