# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with this repository.

@AGENTS.md

## Claude Code Notes

- After modifying any Rust type exposed to Python, run the `pyo3-checklist` skill to verify all 6 wiring layers are complete (Rust impl → re-export → PyO3 registration → `__init__.py` → `__all__` → `.pyi` stub).
- Use the `rust-python` skill when working with PyO3 bindings or the Rust↔Python boundary.
- Use the `agentic-architect` skill when modifying or extending agent orchestration, LLM client code, or tool-calling logic.
