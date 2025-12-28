from potato_head.google import GeminiSettings, GenerationConfig, GeminiThinkingConfig


def test_gemini_settings_init():
    settings = GeminiSettings()
    assert settings is not None


def test_generation_config_init():
    config = GenerationConfig(
        top_k=5,
        top_p=0.9,
        temperature=0.8,
        thinking_config=GeminiThinkingConfig(
            include_thoughts=True,
            thinking_budget=1000,
        ),
    )
    assert config is not None
