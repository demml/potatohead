from pathlib import Path

import pytest
from potato_head import PortableSpec


def _write_yaml(tmp_path: Path, content: str) -> Path:
    spec_path = tmp_path / "agent_spec.yaml"
    spec_path.write_text(content)
    return spec_path


def test_load_portable_spec_and_export_framework_configs(tmp_path: Path):
    spec_path = _write_yaml(
        tmp_path,
        """
version: "1.0"
prompts:
  - id: support_prompt
    version: "1.0.0"
    instructions:
      - role: system
        content: You classify support tickets.
      - role: user_template
        content: "Classify this ticket: ${ticket_text}"
    variables:
      - name: ticket_text
        required: true
        schema:
          type: string
    output_schema:
      type: object
      properties:
        category:
          type: string
        urgency:
          type: string
      required: [category, urgency]
agents:
  - id: support_agent
    version: "1.0.0"
    name: Support Triage Agent
    description: Routes support tickets to the right queue.
    primary_prompt: support_prompt
    tool_refs: [ticket_lookup]
    agent_refs: [escalation_agent]
    provider_hints: [gemini-2.5-pro]
    extensions:
      openai_agents:
        model: gpt-4o-mini
      crewai:
        verbose: true
      google_adk:
        temperature: 0.1
""",
    )

    spec = PortableSpec.from_path(spec_path)
    openai_export = spec.to_openai_agent_config("support_agent")
    crewai_export = spec.to_crewai_agent_config("support_agent")
    adk_export = spec.to_google_adk_agent_config("support_agent")

    assert openai_export["framework"] == "openai_agents"
    assert openai_export["config"]["name"] == "Support Triage Agent"
    assert openai_export["config"]["model"] == "gpt-4o-mini"
    assert openai_export["config"]["tools"][0]["name"] == "ticket_lookup"
    assert openai_export["config"]["handoffs"] == ["escalation_agent"]

    assert crewai_export["framework"] == "crewai"
    assert crewai_export["config"]["role"] == "Support Triage Agent"
    assert crewai_export["config"]["verbose"] is True

    assert adk_export["framework"] == "google_adk"
    assert adk_export["config"]["name"] == "Support Triage Agent"
    assert adk_export["config"]["model"] == "gemini-2.5-pro"
    assert adk_export["config"]["temperature"] == 0.1


def test_runtime_fields_are_rejected(tmp_path: Path):
    spec_path = _write_yaml(
        tmp_path,
        """
version: "1.0"
agents:
  - id: support_agent
    version: "1.0.0"
    max_iterations: 5
""",
    )

    with pytest.raises(RuntimeError, match="runtime-only field 'max_iterations'"):
        PortableSpec.from_path(spec_path)
