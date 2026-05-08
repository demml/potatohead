from __future__ import annotations

from pathlib import Path

import pytest
from potato_head import MediaKind, MediaRef, Prompt, Provider


def make_prompt(provider: Provider, model: str, body: str) -> Prompt[None]:
    return Prompt(messages=body, provider=provider, model=model)


def test_media_parameters_extracted_separately():
    p = make_prompt(Provider.Anthropic, "claude-sonnet-4-5", "${greet} on ${media:chart} ${media:doc}")
    assert p.parameters == ["greet"]
    assert sorted(p.media_parameters) == ["chart", "doc"]


def test_no_media_yields_empty_list():
    p = make_prompt(Provider.Anthropic, "claude-sonnet-4-5", "${greet}")
    assert p.media_parameters == []


def test_media_kind_set_correctly():
    assert MediaRef.image_url("https://x/y.png").kind == MediaKind.Image
    assert MediaRef.document_bytes("application/pdf", b"%PDF").kind == MediaKind.Document


def test_image_path_infers_mime_from_extension(tmp_path: Path):
    f = tmp_path / "chart.png"
    f.write_bytes(b"FAKE")
    ref = MediaRef.image_path(str(f))
    assert ref.kind == MediaKind.Image


def test_image_path_unknown_extension_raises(tmp_path: Path):
    f = tmp_path / "chart.xyz"
    f.write_bytes(b"X")
    with pytest.raises(RuntimeError, match="unrecognized extension"):
        MediaRef.image_path(str(f))


def test_anthropic_image_bytes_serializes_base64_block():
    p = make_prompt(Provider.Anthropic, "claude-sonnet-4-5", "${media:chart}")
    bound = p.bind_media("chart", MediaRef.image_bytes("image/png", b"FAKEPNG"))
    blocks = bound.model_dump()["messages"][0]["content"]
    assert blocks[0]["type"] == "image"
    assert blocks[0]["source"]["type"] == "base64"
    assert blocks[0]["source"]["media_type"] == "image/png"


def test_anthropic_image_url_serializes_url_block():
    p = make_prompt(Provider.Anthropic, "claude-sonnet-4-5", "${media:chart}")
    bound = p.bind_media("chart", MediaRef.image_url("https://example.com/c.png"))
    blocks = bound.model_dump()["messages"][0]["content"]
    assert blocks[0]["source"]["type"] == "url"
    assert blocks[0]["source"]["url"] == "https://example.com/c.png"


def test_anthropic_document_bytes_serializes():
    p = make_prompt(Provider.Anthropic, "claude-sonnet-4-5", "${media:doc}")
    bound = p.bind_media("doc", MediaRef.document_bytes("application/pdf", b"%PDF"))
    blocks = bound.model_dump()["messages"][0]["content"]
    assert blocks[0]["type"] == "document"
    assert blocks[0]["source"]["type"] == "base64"


def test_openai_image_bytes_becomes_data_url():
    p = make_prompt(Provider.OpenAI, "gpt-4o", "${media:chart}")
    bound = p.bind_media("chart", MediaRef.image_bytes("image/png", b"X"))
    parts = bound.model_dump()["messages"][0]["content"]
    assert parts[0]["type"] == "image_url"
    assert parts[0]["image_url"]["url"].startswith("data:image/png;base64,")


def test_openai_image_url_passthrough():
    p = make_prompt(Provider.OpenAI, "gpt-4o", "${media:c}")
    bound = p.bind_media("c", MediaRef.image_url("https://x/y.png"))
    parts = bound.model_dump()["messages"][0]["content"]
    assert parts[0]["image_url"]["url"] == "https://x/y.png"


def test_openai_document_url_rejected():
    p = make_prompt(Provider.OpenAI, "gpt-4o", "${media:doc}")
    with pytest.raises(RuntimeError, match="does not support media"):
        p.bind_media("doc", MediaRef.document_url("https://x/y.pdf"))


def test_openai_document_bytes_serializes_file_content():
    p = make_prompt(Provider.OpenAI, "gpt-4o", "${media:doc}")
    bound = p.bind_media("doc", MediaRef.document_bytes("application/pdf", b"%PDF"))
    parts = bound.model_dump()["messages"][0]["content"]
    assert parts[0]["type"] == "file"
    assert parts[0]["file"]["file_data"].startswith("data:application/pdf;base64,")


def test_gemini_inline_data_from_bytes():
    p = make_prompt(Provider.Gemini, "gemini-2.0-flash", "${media:chart}")
    bound = p.bind_media("chart", MediaRef.image_bytes("image/png", b"X"))
    parts = bound.model_dump()["contents"][0]["parts"]
    assert "inlineData" in parts[0]
    assert parts[0]["inlineData"]["mime_type"] == "image/png"


def test_gemini_https_url_rejected():
    p = make_prompt(Provider.Gemini, "gemini-2.0-flash", "${media:c}")
    with pytest.raises(RuntimeError, match="does not support media"):
        p.bind_media("c", MediaRef.image_url("https://x/y.png", "image/png"))


def test_gemini_gs_url_accepted():
    p = make_prompt(Provider.Gemini, "gemini-2.0-flash", "${media:c}")
    bound = p.bind_media("c", MediaRef.image_url("gs://bucket/c.png", "image/png"))
    parts = bound.model_dump()["contents"][0]["parts"]
    assert "fileData" in parts[0]
    assert parts[0]["fileData"]["file_uri"] == "gs://bucket/c.png"


def test_gemini_url_without_mime_rejected():
    p = make_prompt(Provider.Gemini, "gemini-2.0-flash", "${media:c}")
    with pytest.raises(RuntimeError, match="requires explicit mime_type"):
        p.bind_media("c", MediaRef.image_url("gs://b/c.png"))


def test_missing_placeholder_raises():
    p = make_prompt(Provider.Anthropic, "claude-sonnet-4-5", "${media:foo}")
    with pytest.raises(RuntimeError, match="not found"):
        p.bind_media("bar", MediaRef.image_bytes("image/png", b"X"))


@pytest.mark.parametrize(
    ("provider", "model"),
    [
        (Provider.Anthropic, "claude-sonnet-4-5"),
        (Provider.OpenAI, "gpt-4o"),
        (Provider.Gemini, "gemini-2.0-flash"),
    ],
)
def test_media_in_system_message_rejected(provider: Provider, model: str):
    with pytest.raises(RuntimeError, match="not allowed in system messages"):
        Prompt(
            messages="hi",
            provider=provider,
            model=model,
            system_instructions="system ${media:x}",
        )


def test_image_path_rejects_directory(tmp_path: Path):
    with pytest.raises(RuntimeError, match="not a regular file"):
        MediaRef.image_path(tmp_path)


def test_image_path_rejects_file_over_size_limit(tmp_path: Path):
    f = tmp_path / "large.png"
    with f.open("wb") as handle:
        handle.truncate((20 * 1024 * 1024) + 1)
    with pytest.raises(RuntimeError, match="too large"):
        MediaRef.image_path(f)


def test_bind_and_bind_media_coexist():
    p = make_prompt(Provider.Anthropic, "claude-sonnet-4-5", "${greet} ${media:chart}")
    p = p.bind(greet="Hello")
    p = p.bind_media("chart", MediaRef.image_bytes("image/png", b"X"))
    blocks = p.model_dump()["messages"][0]["content"]
    assert any(b.get("type") == "text" and "Hello" in b.get("text", "") for b in blocks)
    assert any(b.get("type") == "image" for b in blocks)


def test_bind_does_not_match_media_namespace():
    p = make_prompt(Provider.Anthropic, "claude-sonnet-4-5", "${name} and ${media:name}")
    p = p.bind("name", "Steven")
    blocks = p.model_dump()["messages"][0]["content"]
    text_blocks = [b for b in blocks if b.get("type") == "text"]
    assert any("Steven" in b["text"] for b in text_blocks)
    assert any(b["text"] == "${media:name}" for b in text_blocks)


def test_serialization_round_trip():
    p = make_prompt(Provider.Anthropic, "claude-sonnet-4-5", "${media:chart}")
    bound = p.bind_media("chart", MediaRef.image_bytes("image/png", b"X"))
    json_str = bound.model_dump_json()
    restored = Prompt.model_validate_json(json_str)
    assert bound.model_dump() == restored.model_dump()
    assert bound.provider == restored.provider
    assert bound.model == restored.model


def test_template_reused_across_bindings():
    template = make_prompt(Provider.Anthropic, "claude-sonnet-4-5", "Score this: ${media:img}")
    images = [b"IMG_A", b"IMG_B", b"IMG_C"]
    bound_prompts = [template.bind_media("img", MediaRef.image_bytes("image/png", img)) for img in images]
    assert len(bound_prompts) == 3
    assert all(p.model_dump()["messages"][0]["content"][1]["type"] == "image" for p in bound_prompts)
    template_blocks = template.model_dump()["messages"][0]["content"]
    assert template_blocks[1]["type"] == "text"
    assert template_blocks[1]["text"] == "${media:img}"
