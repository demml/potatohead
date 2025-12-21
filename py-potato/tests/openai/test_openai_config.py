from potato_head.openai import OpenAIChatSettings


def test_openai_settings_init():
    _settings = OpenAIChatSettings()


def test_generation_config_init():
    config = OpenAIChatSettings(
        max_completion_tokens=1024,
        top_k=5,
        top_p=0.9,
        temperature=0.8,
    )
    assert config is not None
