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


def test_load_simple_gemini_prompt_from_path():
    yaml_file_path = Path(__file__).parent / "assets" / "simple_gemini.yaml"
    prompt = Prompt.from_path(yaml_file_path)

    assert prompt.model == "gemini-2.5-flash"
    assert prompt.provider == Provider.Google

    messages = prompt.gemini_messages
    assert messages[0].parts[0].data == "Hello ${variable1}"


def test_load_simple_openai_prompt_from_path():
    yaml_file_path = Path(__file__).parent / "assets" / "simple_openai.yaml"
    prompt = Prompt.from_path(yaml_file_path)

    assert prompt.model == "gpt-4"
    assert prompt.provider == Provider.OpenAI

    messages = prompt.openai_messages
    assert messages[0].content[0].text == "Summarize this: ${text}"


def test_load_simple_anthropic_prompt_from_path():
    yaml_file_path = Path(__file__).parent / "assets" / "simple_anthropic.yaml"
    prompt = Prompt.from_path(yaml_file_path)

    assert prompt.model == "claude-3-5-sonnet-20241022"
    assert prompt.provider == Provider.Anthropic

    messages = prompt.anthropic_messages
    assert messages[0].text == "Analyze this: ${content}"

    system_instructions = prompt.system_instructions

    assert system_instructions[0].text == "You are an expert analyst"
