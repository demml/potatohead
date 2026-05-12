import pytest
from potato_head import Agent, Prompt, Provider
from potato_head.anthropic import MessageParam
from potato_head.openai import ChatMessage

ENV_VAR = "POTATO_HEAD_DEFAULT_PROVIDER"


def test_yaml_without_provider_uses_env_var(tmp_path, monkeypatch):
    monkeypatch.setenv(ENV_VAR, "openai")
    yaml = tmp_path / "no_provider.yaml"
    yaml.write_text("model: gpt-4\n" "messages:\n" '  - "Hello ${name}"\n')
    prompt = Prompt.from_path(yaml)
    assert prompt.provider == Provider.OpenAI
    assert prompt.model == "gpt-4"


def test_yaml_without_provider_no_env_var_raises(tmp_path, monkeypatch):
    monkeypatch.delenv(ENV_VAR, raising=False)
    yaml = tmp_path / "no_provider.yaml"
    yaml.write_text("model: gpt-4\n" "messages:\n" '  - "Hello"\n')
    with pytest.raises(RuntimeError, match=ENV_VAR):
        Prompt.from_path(yaml)


def test_explicit_provider_overrides_env_var(tmp_path, monkeypatch):
    monkeypatch.setenv(ENV_VAR, "anthropic")
    yaml = tmp_path / "with_provider.yaml"
    yaml.write_text("model: gpt-4\n" "provider: openai\n" "messages:\n" '  - "Hello"\n')
    prompt = Prompt.from_path(yaml)
    assert prompt.provider == Provider.OpenAI


def test_prompt_constructor_without_provider_uses_env_var(monkeypatch):
    monkeypatch.setenv(ENV_VAR, "anthropic")
    prompt = Prompt(messages="Hello", model="claude-3-5-sonnet")
    assert prompt.provider == Provider.Anthropic
    assert isinstance(prompt.anthropic_messages[0], MessageParam)


def test_prompt_constructor_no_provider_no_env_raises(monkeypatch):
    monkeypatch.delenv(ENV_VAR, raising=False)
    with pytest.raises(RuntimeError, match=ENV_VAR):
        Prompt(messages="Hello", model="gpt-4")


def test_invalid_env_var_value_raises(monkeypatch):
    monkeypatch.setenv(ENV_VAR, "not_a_provider")
    with pytest.raises(RuntimeError):
        Prompt(messages="Hello", model="gpt-4")


def test_empty_env_var_treated_as_unset(monkeypatch):
    monkeypatch.setenv(ENV_VAR, "   ")
    with pytest.raises(RuntimeError, match=ENV_VAR):
        Prompt(messages="Hello", model="gpt-4")


def test_litellm_gateway_scenario(monkeypatch):
    monkeypatch.setenv(ENV_VAR, "openai")
    prompt = Prompt(messages="Summarize", model="claude-3-5-sonnet")
    assert prompt.provider == Provider.OpenAI
    assert isinstance(prompt.openai_messages[0], ChatMessage)


def test_agent_without_provider_uses_env_var(monkeypatch):
    monkeypatch.setenv(ENV_VAR, "openai")
    agent = Agent()
    assert agent is not None


def test_agent_no_provider_no_env_raises(monkeypatch):
    monkeypatch.delenv(ENV_VAR, raising=False)
    with pytest.raises(RuntimeError, match=ENV_VAR):
        Agent()
