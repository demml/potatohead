# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Project Is

**Potato Head** is a Rust-based agentic workflow framework with Python bindings (via PyO3/Maturin). It provides a unified multi-provider LLM client (OpenAI, Gemini, Vertex AI, Anthropic), agent orchestration, and workflow management. It serves as a core utility for the [opsml](https://github.com/demml/opsml) and [scouter](https://github.com/demml/scouter) projects.

## Commands

### Rust (root)
```bash
make format       # cargo fmt --all
make lints        # cargo clippy --workspace --all-targets --all-features -- -D warnings
make test         # cargo test -- --nocapture --test-threads=1
make test.baked   # cargo test -p baked-potato -- --nocapture --test-threads=1
```

Run a single Rust test:
```bash
cargo test -p <crate-name> test_name -- --nocapture
```

### Python (py-potato/)
```bash
make setup.project              # uv sync + maturin develop (full dev setup)
make format                     # isort + ruff + black
make lints                      # ruff + mypy
make test.unit                  # pytest with coverage
make test.integration.openai    # requires OPENAI_API_KEY
make test.integration.gemini    # requires GEMINI_API_KEY
make test.integration.anthropic # requires ANTHROPIC_API_KEY
make test.integration.vertex    # requires GOOGLE_ACCOUNT_JSON_BASE64
```

## Architecture

The project is a Cargo workspace. Crates in `crates/*` are pure Rust; `py-potato` is the Python extension built with Maturin.

### Rust Crate Dependency Flow

```
potato-type         ← shared types (Provider, Model enums, Prompt, tool defs)
    ↓
potato-provider     ← GenAiClient enum wrapping per-provider HTTP clients
    ↓
potato-agent        ← Agent struct + Task/TaskStatus + AgentResponse
    ↓
potato-workflow     ← Workflow orchestration over multiple agents/tasks
    ↓
potato-head         ← facade crate re-exporting everything (published to crates.io)
```

Supporting crates:
- **potato-state** — `PotatoState` singleton holding a dedicated Tokio runtime; bridges async Rust with Python's sync model via `block_on()`
- **potato-util** — UUID v7, weighted scoring, JSON helpers, `PyHelperFuncs`
- **potatohead-macro** — proc macros `try_extract_py_object!` and `extract_and_push!` for PyO3 enum extraction
- **baked-potato** — integration tests and mocks; not published

### Python Binding Layer (`py-potato/`)

- `py-potato/src/lib.rs` — PyO3 module root exporting all Rust types to Python
- `py-potato/python/potato_head/__init__.py` — Python package re-exporting the Rust extension
- `py-potato/python/potato_head/_potato_head.pyi` — type stubs for IDEs
- Provider submodules: `openai/`, `google/`, `anthropic/` under `potato_head/`
- Extension module name: `_potato_head` (ABI3, supports Python ≥ 3.10)

### Provider Abstraction

`GenAiClient` (in `potato-provider`) is a Rust enum with variants `OpenAI`, `Gemini`, `Vertex`, `Anthropic`. Each variant wraps a provider-specific client that handles auth, HTTP, streaming, and structured output. Adding a new provider means adding a new variant plus implementation in `crates/potato_provider/src/providers/`.

### Async/Sync Bridge

Python calls are synchronous. `PotatoState` (in `potato-state`) holds a `tokio::runtime::Runtime` in a `OnceLock`, and `block_on()` is used throughout PyO3 `#[pymethods]` to execute async Rust functions. Do not create ad-hoc Tokio runtimes — always use `PotatoState::global().block_on(...)`.

## Publishing

All crates except `baked-potato` are published to crates.io. Releases are automated via `release-plz` (configured in `release-plz.toml`). The Python wheel is published separately via the `release.yml` workflow using Maturin.

## Integration Tests

Integration tests in `baked-potato` and `py-potato/tests/integration/` hit real LLM APIs and require environment variables (`OPENAI_API_KEY`, `GEMINI_API_KEY`, `ANTHROPIC_API_KEY`, `GOOGLE_ACCOUNT_JSON_BASE64`). Unit tests do not require any credentials.
