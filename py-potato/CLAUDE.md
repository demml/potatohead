# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

PotatoHead is a Rust-Python agentic framework for building AI-powered workflows. The `py-potato/` directory contains the Python bindings (via PyO3/maturin) and is the primary working directory for Python development. The Rust workspace lives one level up in the repo root.

## Build & Development Commands

### Python (run from `py-potato/`)

| Command | Purpose |
|---------|---------|
| `make setup.project` | Install deps + build Rust → Python bindings (`uv sync` + `maturin develop`) |
| `make format` | Run isort, ruff, black |
| `make lints` | Run ruff + mypy |
| `make test.unit` | Run pytest (ignores integration tests) |
| `make build.docs` | Generate stubs + build MkDocs |

Run a single test:
```bash
uv run pytest tests/test_workflow.py::test_name -v
```

### Rust (run from repo root `/`)

| Command | Purpose |
|---------|---------|
| `make format` | `cargo fmt --all` |
| `make lints` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| `make test` | `cargo test -- --nocapture --test-threads=1` |
| `make test.baked` | Test the `baked-potato` mock/testing crate only |

After any Rust change, rebuild bindings before running Python tests:
```bash
uv run maturin develop
```

## Architecture

### Rust Workspace (repo root)

Nine crates under `crates/`:

- **potato_head** — Public API aggregator; re-exports from other crates
- **potato_type** — Type definitions for all providers (OpenAI, Gemini, Anthropic). PyO3 `#[pyclass]` annotations live here.
- **potato_provider** — LLM provider HTTP clients
- **potato_agent** — Agent execution engine
- **potato_workflow** — Workflow/task orchestration
- **potato_macro** — Procedural macros (derive macros for provider types)
- **potato_state** — Global state management
- **potato_util** — Shared utilities
- **baked_potato** — Mock server and testing utilities

### PyO3 Binding Layer (`py-potato/src/`)

`lib.rs` registers the `_potato_head` PyO3 module. Provider-specific bindings are split into submodules: `openai.rs`, `google.rs`, `anthropic.rs`, `mock.rs`, `logging.rs`. Each adds a Python submodule (e.g., `potato_head.openai`).

### Python Package (`py-potato/python/potato_head/`)

Thin wrappers and re-exports from the compiled `_potato_head` native module. Provider types are organized into subpackages (`openai/`, `google/`, `anthropic/`). The `.pyi` stub file is auto-generated via `scripts/create_stubs.py`.

### Adding a New Type (end-to-end)

1. Define the Rust struct with `#[pyclass]` in the appropriate `potato_type` submodule
2. Register it in the PyO3 module (`py-potato/src/<provider>.rs`)
3. Export from the Python `__init__.py` for that provider
4. Regenerate stubs: `uv run python scripts/create_stubs.py`

## Code Style

- **Python**: black (line-length 120), isort (black profile), ruff, mypy
- **Rust**: cargo fmt, clippy with `-D warnings`
- Python requires 3.10+; CI tests 3.11, 3.12, 3.13

## Testing

- Unit tests: `py-potato/tests/` — uses `LLMTestServer` mock for provider responses
- Integration tests: `py-potato/examples/` — require real API keys (OPENAI_API_KEY, GEMINI_API_KEY, ANTHROPIC_API_KEY)
- Rust tests: `cargo test` from repo root
- Conftest fixtures in `tests/conftest.py` provide pre-built Prompt, Task, and Score objects
