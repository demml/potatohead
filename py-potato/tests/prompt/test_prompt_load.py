from pathlib import Path

from potato_head import Prompt, Provider


def test_load_prompt_from_path():
    file_path = Path(__file__).parent / "assets" / "anthropic_prompt.json"
    prompt = Prompt.from_path(file_path)

    assert prompt.model == "claude-4.5-sonnet"
    assert prompt.provider == Provider.Anthropic

    # load yaml
    yaml_file_path = Path(__file__).parent / "assets" / "google_prompt.yaml"
    prompt_yaml = Prompt.from_path(yaml_file_path)
    assert prompt_yaml.model == "gemini-1.5-pro"
    assert prompt_yaml.provider == Provider.Google
    assert prompt_yaml.parameters == ["resume_text"]

    # test openai prompt
    openai_file_path = Path(__file__).parent / "assets" / "openai_prompt.json"
    prompt_openai = Prompt.from_path(openai_file_path)
    assert prompt_openai.model == "gpt-4o"
    assert prompt_openai.provider == Provider.OpenAI
