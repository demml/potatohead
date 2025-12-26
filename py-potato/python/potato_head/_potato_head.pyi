# pylint: disable=redefined-builtin, invalid-name, dangerous-default-value

import datetime
from pathlib import Path
from typing import Any, Dict, List, Literal, Optional, Sequence, TypeVar

###### __potatohead__.anthropic module ######
class Metadata:
    """Metadata for Anthropic API requests.

    This class provides metadata configuration for Anthropic chat completions,
    including user identification for tracking and analytics.

    Examples:
        >>> metadata = Metadata(user_id="user_123")
        >>> metadata.user_id
        'user_123'
    """

    def __init__(
        self,
        user_id: Optional[str] = None,
    ) -> None:
        """Initialize Anthropic metadata.

        Args:
            user_id (Optional[str]):
                User identifier for request tracking and analytics
        """

    @property
    def user_id(self) -> Optional[str]:
        """The user identifier for the request."""

class CacheControl:
    """Cache control configuration for Anthropic prompt caching.

    This class provides configuration for controlling prompt caching behavior,
    including cache type and time-to-live (TTL) settings.

    Examples:
        >>> cache = CacheControl(cache_type="ephemeral", ttl="5m")
        >>> cache.cache_type
        'ephemeral'
    """

    def __init__(
        self,
        cache_type: str,
        ttl: Optional[str] = None,
    ) -> None:
        """Initialize cache control configuration.

        Args:
            cache_type (str):
                The type of cache to use (e.g., "ephemeral")
            ttl (Optional[str]):
                Time-to-live for cached content (e.g., "5m", "1h")
        """

    @property
    def cache_type(self) -> str:
        """The cache type."""

    @property
    def ttl(self) -> Optional[str]:
        """The time-to-live for cached content."""

class AnthropicTool:
    """Tool definition for Anthropic function calling.

    This class defines a tool that can be called by the model during generation,
    including its schema and optional cache control settings.

    Examples:
        >>> tool = Tool(
        ...     name="get_weather",
        ...     description="Get current weather",
        ...     input_schema={"type": "object", "properties": {"location": {"type": "string"}}}
        ... )
    """

    def __init__(
        self,
        name: str,
        description: Optional[str] = None,
        input_schema: Any = None,
        cache_control: Optional[CacheControl] = None,
    ) -> None:
        """Initialize a tool definition.

        Args:
            name (str):
                The name of the tool
            description (Optional[str]):
                Human-readable description of what the tool does
            input_schema (Any):
                JSON schema defining the tool's input parameters
            cache_control (Optional[CacheControl]):
                Cache control configuration for the tool definition
        """

    @property
    def name(self) -> str:
        """The name of the tool."""

    @property
    def description(self) -> Optional[str]:
        """The description of the tool."""

    @property
    def input_schema(self) -> Any:
        """The JSON schema for the tool's input."""

    @property
    def cache_control(self) -> Optional[CacheControl]:
        """The cache control configuration."""

class AnthropicThinkingConfig:
    """Configuration for Anthropic's extended thinking mode.

    This class provides configuration for enabling and controlling the model's
    reasoning capabilities, including budget allocation for thinking tokens.

    Examples:
        >>> config = ThinkingConfig(type="enabled", budget_tokens=1000)
        >>> config.budget_tokens
        1000
    """

    def __init__(
        self,
        type: str,
        budget_tokens: Optional[int] = None,
    ) -> None:
        """Initialize thinking configuration.

        Args:
            type (str):
                The type of thinking mode (e.g., "enabled", "disabled")
            budget_tokens (Optional[int]):
                Maximum number of tokens to allocate for thinking/reasoning
        """

    @property
    def type(self) -> str:
        """The thinking mode type."""

    @property
    def budget_tokens(self) -> Optional[int]:
        """The token budget for thinking."""

class AnthropicToolChoice:
    """Tool choice configuration for Anthropic function calling.

    This class controls how the model selects and uses tools during generation,
    including automatic selection, forced usage, or specific tool targeting.

    Examples:
        >>> # Auto tool selection
        >>> choice = ToolChoice(type="auto")
        >>>
        >>> # Force specific tool
        >>> choice = ToolChoice(type="tool", name="get_weather")
        >>>
        >>> # Disable parallel tool calls
        >>> choice = ToolChoice(type="any", disable_parallel_tool_use=True)
    """

    def __init__(
        self,
        type: str,
        disable_parallel_tool_use: Optional[bool] = None,
        name: Optional[str] = None,
    ) -> None:
        """Initialize tool choice configuration.

        Args:
            type (str):
                The tool choice mode: "auto", "any", "tool", or "none"
            disable_parallel_tool_use (Optional[bool]):
                Whether to disable parallel tool execution
            name (Optional[str]):
                Specific tool name to use (required when type is "tool")

        Raises:
            ValueError: If name is provided when type is not "tool"
            ValueError: If type is "tool" but name is not provided
        """

    @property
    def type(self) -> str:
        """The tool choice mode."""

    @property
    def disable_parallel_tool_use(self) -> Optional[bool]:
        """Whether parallel tool use is disabled."""

    @property
    def name(self) -> Optional[str]:
        """The specific tool name to use."""

class AnthropicSettings:
    """Anthropic chat completion settings configuration.

    This class provides comprehensive configuration options for Anthropic chat completions,
    including model parameters, tool usage, thinking modes, and request options.

    Examples:
        >>> settings = AnthropicSettings(
        ...     max_tokens=2048,
        ...     temperature=0.7,
        ...     top_p=0.9
        ... )
        >>>
        >>> # With tools
        >>> settings = AnthropicSettings(
        ...     max_tokens=4096,
        ...     tools=[tool1, tool2],
        ...     tool_choice=ToolChoice(type="auto")
        ... )
        >>>
        >>> # With thinking mode
        >>> settings = AnthropicSettings(
        ...     max_tokens=4096,
        ...     thinking=ThinkingConfig(type="enabled", budget_tokens=1000)
        ... )
    """

    def __init__(
        self,
        *,
        max_tokens: int = 4096,
        metadata: Optional[Metadata] = None,
        service_tier: Optional[str] = None,
        stop_sequences: Optional[List[str]] = None,
        stream: Optional[bool] = None,
        system: Optional[str] = None,
        temperature: Optional[float] = None,
        thinking: Optional[AnthropicThinkingConfig] = None,
        top_k: Optional[int] = None,
        top_p: Optional[float] = None,
        tools: Optional[List[AnthropicTool]] = None,
        tool_choice: Optional[AnthropicToolChoice] = None,
        extra_body: Optional[Any] = None,
    ) -> None:
        """Initialize Anthropic chat settings.

        Args:
            max_tokens (int):
                Maximum number of tokens to generate (default: 4096)
            metadata (Optional[Metadata]):
                Request metadata for tracking and analytics
            service_tier (Optional[str]):
                Service tier to use for the request
            stop_sequences (Optional[List[str]]):
                Sequences where generation should stop
            stream (Optional[bool]):
                Whether to stream the response
            system (Optional[str]):
                System prompt to guide model behavior
            temperature (Optional[float]):
                Sampling temperature (0.0 to 1.0)
            thinking (Optional[ThinkingConfig]):
                Configuration for extended thinking mode
            top_k (Optional[int]):
                Top-k sampling parameter
            top_p (Optional[float]):
                Nucleus sampling parameter (0.0 to 1.0)
            tools (Optional[List[Tool]]):
                Available tools for the model to use
            tool_choice (Optional[ToolChoice]):
                Tool selection configuration
            extra_body (Optional[Any]):
                Additional request body parameters
        """

    @property
    def max_tokens(self) -> int:
        """Maximum number of tokens to generate."""

    @property
    def metadata(self) -> Optional[Metadata]:
        """Request metadata."""

    @property
    def service_tier(self) -> Optional[str]:
        """Service tier."""

    @property
    def stop_sequences(self) -> Optional[List[str]]:
        """Stop sequences."""

    @property
    def stream(self) -> Optional[bool]:
        """Whether to stream the response."""

    @property
    def system(self) -> Optional[str]:
        """System prompt."""

    @property
    def temperature(self) -> Optional[float]:
        """Sampling temperature."""

    @property
    def thinking(self) -> Optional[AnthropicThinkingConfig]:
        """Thinking mode configuration."""

    @property
    def top_k(self) -> Optional[int]:
        """Top-k sampling parameter."""

    @property
    def top_p(self) -> Optional[float]:
        """Nucleus sampling parameter."""

    @property
    def tools(self) -> Optional[List[AnthropicTool]]:
        """Available tools."""

    @property
    def tool_choice(self) -> Optional[AnthropicToolChoice]:
        """Tool choice configuration."""

    @property
    def extra_body(self) -> Optional[Any]:
        """Additional request parameters."""

    def __str__(self) -> str:
        """Return string representation of the settings."""

    def model_dump(self) -> Dict[str, Any]:
        """Serialize the settings to a dictionary.

        Returns:
            Dict[str, Any]:
                Dictionary representation of the settings
        """

###### __potatohead__.openai module ######
class AudioParam:
    def __init__(self, format: str, voice: str) -> None: ...
    @property
    def format(self) -> str: ...
    @property
    def voice(self) -> str: ...

class ContentPart:
    def __init__(self, type: str, text: str) -> None: ...
    @property
    def type(self) -> str: ...
    @property
    def text(self) -> str: ...

class Content:
    def __init__(
        self,
        text: Optional[str] = None,
        parts: Optional[List[ContentPart]] = None,
    ) -> None: ...

class Prediction:
    def __init__(self, type: str, content: Content) -> None: ...
    @property
    def type(self) -> str: ...
    @property
    def content(self) -> Content: ...

class StreamOptions:
    def __init__(
        self,
        include_obfuscation: Optional[bool] = None,
        include_usage: Optional[bool] = None,
    ) -> None: ...
    @property
    def include_obfuscation(self) -> Optional[bool]: ...
    @property
    def include_usage(self) -> Optional[bool]: ...

class ToolChoiceMode:
    NA: "ToolChoiceMode"
    Auto: "ToolChoiceMode"
    Required: "ToolChoiceMode"

class FunctionChoice:
    def __init__(self, name: str) -> None: ...
    @property
    def name(self) -> str: ...

class FunctionToolChoice:
    def __init__(self, function: FunctionChoice) -> None: ...
    @property
    def function(self) -> FunctionChoice: ...
    @property
    def type(self) -> str: ...

class CustomChoice:
    def __init__(self, name: str) -> None: ...
    @property
    def name(self) -> str: ...

class CustomToolChoice:
    def __init__(self, custom: CustomChoice) -> None: ...
    @property
    def custom(self) -> CustomChoice: ...
    @property
    def type(self) -> str: ...

class ToolDefinition:
    def __init__(self, function_name: str) -> None: ...
    @property
    def function_name(self) -> str: ...
    @property
    def type(self) -> str: ...

class AllowedToolsMode:
    Auto: "AllowedToolsMode"
    Required: "AllowedToolsMode"

class InnerAllowedTools:
    @property
    def mode(self) -> AllowedToolsMode: ...
    @property
    def tools(self) -> List[ToolDefinition]: ...

class AllowedTools:
    def __init__(self, mode: AllowedToolsMode, tools: List[ToolDefinition]) -> None: ...
    @property
    def type(self) -> str: ...
    @property
    def allowed_tools(self) -> InnerAllowedTools: ...

class OpenAIToolChoice:
    Mode: "OpenAIToolChoice"
    Function: "OpenAIToolChoice"
    Custom: "OpenAIToolChoice"
    Allowed: "OpenAIToolChoice"
    @staticmethod
    def from_mode(mode: AllowedToolsMode) -> "OpenAIToolChoice": ...
    @staticmethod
    def from_function(function_name: str) -> "OpenAIToolChoice": ...
    @staticmethod
    def from_custom(custom_name: str) -> "OpenAIToolChoice": ...
    @staticmethod
    def from_allowed_tools(allowed_tools: AllowedTools) -> "OpenAIToolChoice": ...

class FunctionDefinition:
    def __init__(
        self,
        name: str,
        description: Optional[str] = None,
        parameters: Optional[dict] = None,
        strict: Optional[bool] = None,
    ) -> None: ...
    @property
    def name(self) -> str: ...
    @property
    def description(self) -> Optional[str]: ...
    @property
    def parameters(self) -> Optional[dict]: ...
    @property
    def strict(self) -> Optional[bool]: ...

class FunctionTool:
    def __init__(self, function: FunctionDefinition, type: str) -> None: ...
    @property
    def function(self) -> FunctionDefinition: ...
    @property
    def type(self) -> str: ...

class TextFormat:
    def __init__(self, type: str) -> None: ...
    @property
    def type(self) -> str: ...

class Grammar:
    def __init__(self, definition: str, syntax: str) -> None: ...
    @property
    def definition(self) -> str: ...
    @property
    def syntax(self) -> str: ...

class GrammarFormat:
    def __init__(self, grammar: Grammar, type: str) -> None: ...
    @property
    def type(self) -> str: ...
    @property
    def grammar(self) -> Grammar: ...

class CustomToolFormat:
    def __init__(
        self,
        type: Optional[str] = None,
        grammar: Optional[Grammar] = None,
    ) -> None: ...

class CustomDefinition:
    def __init__(
        self,
        name: str,
        description: Optional[str] = None,
        format: Optional[CustomToolFormat] = None,
    ) -> None: ...
    @property
    def name(self) -> str: ...
    @property
    def description(self) -> Optional[str]: ...
    @property
    def format(self) -> Optional[CustomToolFormat]: ...

class CustomTool:
    def __init__(self, custom: CustomDefinition, type: str) -> None: ...

class OpenAITool:
    def __init__(
        self,
        function: Optional[FunctionTool] = None,
        custom: Optional[CustomTool] = None,
    ) -> None: ...

class OpenAIChatSettings:
    """OpenAI chat completion settings configuration.

    This class provides configuration options for OpenAI chat completions,
    including model parameters, tool usage, and request options.

    Examples:
        >>> settings = OpenAIChatSettings(
        ...     temperature=0.7,
        ...     max_completion_tokens=1000,
        ...     stream=True
        ... )
        >>> settings.temperature = 0.5
    """

    def __init__(
        self,
        *,
        max_completion_tokens: Optional[int] = None,
        temperature: Optional[float] = None,
        top_p: Optional[float] = None,
        top_k: Optional[int] = None,
        frequency_penalty: Optional[float] = None,
        timeout: Optional[float] = None,
        parallel_tool_calls: Optional[bool] = None,
        seed: Optional[int] = None,
        logit_bias: Optional[Dict[str, int]] = None,
        stop_sequences: Optional[List[str]] = None,
        logprobs: Optional[bool] = None,
        audio: Optional[AudioParam] = None,
        metadata: Optional[Dict[str, str]] = None,
        modalities: Optional[List[str]] = None,
        n: Optional[int] = None,
        prediction: Optional[Prediction] = None,
        presence_penalty: Optional[float] = None,
        prompt_cache_key: Optional[str] = None,
        reasoning_effort: Optional[str] = None,
        safety_identifier: Optional[str] = None,
        service_tier: Optional[str] = None,
        store: Optional[bool] = None,
        stream: Optional[bool] = None,
        stream_options: Optional[StreamOptions] = None,
        tool_choice: Optional[OpenAIToolChoice] = None,
        tools: Optional[List[OpenAITool]] = None,
        top_logprobs: Optional[int] = None,
        verbosity: Optional[str] = None,
        extra_body: Optional[Any] = None,
    ) -> None:
        """Initialize OpenAI chat settings.

        Args:
            max_completion_tokens (Optional[int]):
                Maximum number of tokens to generate
            temperature (Optional[float]):
                Sampling temperature (0.0 to 2.0)
            top_p (Optional[float]):
                Nucleus sampling parameter
            top_k (Optional[int]):
                Top-k sampling parameter
            frequency_penalty (Optional[float]):
                Frequency penalty (-2.0 to 2.0)
            timeout (Optional[float]):
                Request timeout in seconds
            parallel_tool_calls (Optional[bool]):
                Whether to enable parallel tool calls
            seed (Optional[int]):
                Random seed for deterministic outputs
            logit_bias (Optional[Dict[str, int]]):
                Token bias modifications
            stop_sequences (Optional[List[str]]):
                Sequences where generation should stop
            logprobs (Optional[bool]):
                Whether to return log probabilities
            audio (Optional[AudioParam]):
                Audio generation parameters
            metadata (Optional[Dict[str, str]]):
                Additional metadata for the request
            modalities (Optional[List[str]]):
                List of modalities to use
            n (Optional[int]):
                Number of completions to generate
            prediction (Optional[Prediction]):
                Prediction configuration
            presence_penalty (Optional[float]):
                Presence penalty (-2.0 to 2.0)
            prompt_cache_key (Optional[str]):
                Key for prompt caching
            reasoning_effort (Optional[str]):
                Reasoning effort level
            safety_identifier (Optional[str]):
                Safety configuration identifier
            service_tier (Optional[str]):
                Service tier to use
            store (Optional[bool]):
                Whether to store the conversation
            stream (Optional[bool]):
                Whether to stream the response
            stream_options (Optional[StreamOptions]):
                Streaming configuration options
            tool_choice (Optional[ToolChoice]):
                Tool choice configuration
            tools (Optional[List[Tool]]):
                Available tools for the model
            top_logprobs (Optional[int]):
                Number of top log probabilities to return
            verbosity (Optional[str]):
                Verbosity level for the response
            extra_body (Optional[Any]):
                Additional request body parameters
        """

    def __str__(self) -> str:
        """Return string representation of the settings."""

class OpenAIEmbeddingConfig:
    """OpenAI embedding configuration settings."""

    def __init__(
        self,
        model: str,
        dimensions: Optional[int] = None,
        encoding_format: Optional[str] = None,
        user: Optional[str] = None,
    ) -> None:
        """Initialize OpenAI embedding configuration.

        Args:
            model (str):
                The embedding model to use.
            dimensions (Optional[int]):
                The output dimensionality of the embeddings.
            encoding_format (Optional[str]):
                The encoding format to use for the embeddings.
                Can be either "float" or "base64".
            user (Optional[str]):
                The user ID for the embedding request.
        """

    @property
    def model(self) -> str: ...
    @property
    def dimensions(self) -> Optional[int]: ...
    @property
    def encoding_format(self) -> Optional[str]: ...
    @property
    def user(self) -> Optional[str]: ...

class EmbeddingObject:
    @property
    def object(self) -> str: ...
    @property
    def embedding(self) -> List[float]: ...
    @property
    def index(self) -> int: ...

class UsageObject:
    @property
    def prompt_tokens(self) -> int: ...
    @property
    def total_tokens(self) -> int: ...

class OpenAIEmbeddingResponse:
    @property
    def object(self) -> str: ...
    @property
    def data(self) -> List[EmbeddingObject]: ...
    @property
    def usage(self) -> UsageObject: ...

###### __potatohead__.gemini module ######

class Modality:
    """Represents different modalities for content generation."""

    ModalityUnspecified: "Modality"
    Text: "Modality"
    Image: "Modality"
    Audio: "Modality"

class GeminiThinkingConfig:
    """Configuration for thinking/reasoning capabilities."""

    def __init__(
        self,
        include_thoughts: Optional[bool] = None,
        thinking_budget: Optional[int] = None,
    ) -> None: ...

class MediaResolution:
    """Media resolution settings for content generation."""

    MediaResolutionUnspecified: "MediaResolution"
    MediaResolutionLow: "MediaResolution"
    MediaResolutionMedium: "MediaResolution"
    MediaResolutionHigh: "MediaResolution"

class SpeechConfig:
    """Configuration for speech generation."""

    def __init__(
        self,
        voice_config: Optional["VoiceConfig"] = None,
        language_code: Optional[str] = None,
    ) -> None: ...

class PrebuiltVoiceConfig:
    """Configuration for prebuilt voice models."""

    def __init__(
        self,
        voice_name: str,
    ) -> None: ...

class GenerationConfig:
    """Configuration for content generation with comprehensive parameter control.

    This class provides fine-grained control over the generation process including
    sampling parameters, output format, modalities, and various specialized features.

    Examples:
        Basic usage with temperature control:

        ```python
        GenerationConfig(temperature=0.7, max_output_tokens=1000)
        ```

        Multi-modal configuration:
        ```python
        config = GenerationConfig(
            response_modalities=[Modality.TEXT, Modality.AUDIO],
            speech_config=SpeechConfig(language_code="en-US")
        )
        ```

        Advanced sampling with penalties:
        ```python
        config = GenerationConfig(
            temperature=0.8,
            top_p=0.9,
            top_k=40,
            presence_penalty=0.1,
            frequency_penalty=0.2
        )
        ```
    """

    def __init__(
        self,
        stop_sequences: Optional[List[str]] = None,
        response_mime_type: Optional[str] = None,
        response_modalities: Optional[List[Modality]] = None,
        thinking_config: Optional[GeminiThinkingConfig] = None,
        temperature: Optional[float] = None,
        top_p: Optional[float] = None,
        top_k: Optional[int] = None,
        candidate_count: Optional[int] = None,
        max_output_tokens: Optional[int] = None,
        response_logprobs: Optional[bool] = None,
        logprobs: Optional[int] = None,
        presence_penalty: Optional[float] = None,
        frequency_penalty: Optional[float] = None,
        seed: Optional[int] = None,
        audio_timestamp: Optional[bool] = None,
        media_resolution: Optional[MediaResolution] = None,
        speech_config: Optional[SpeechConfig] = None,
        enable_affective_dialog: Optional[bool] = None,
    ) -> None:
        """Initialize GenerationConfig with optional parameters.

        Args:
            stop_sequences (Optional[List[str]]):
                List of strings that will stop generation when encountered
            response_mime_type (Optional[str]):
                MIME type for the response format
            response_modalities (Optional[List[Modality]]):
                List of modalities to include in the response
            thinking_config (Optional[ThinkingConfig]):
                Configuration for reasoning/thinking capabilities
            temperature (Optional[float]):
                Controls randomness in generation (0.0-1.0)
            top_p (Optional[float]):
                Nucleus sampling parameter (0.0-1.0)
            top_k (Optional[int]):
                Top-k sampling parameter
            candidate_count (Optional[int]):
                Number of response candidates to generate
            max_output_tokens (Optional[int]):
                Maximum number of tokens to generate
            response_logprobs (Optional[bool]):
                Whether to return log probabilities
            logprobs (Optional[int]):
                Number of log probabilities to return per token
            presence_penalty (Optional[float]):
                Penalty for token presence (-2.0 to 2.0)
            frequency_penalty (Optional[float]):
                Penalty for token frequency (-2.0 to 2.0)
            seed (Optional[int]):
                Random seed for deterministic generation
            audio_timestamp (Optional[bool]):
                Whether to include timestamps in audio responses
            media_resolution (Optional[MediaResolution]):
                Resolution setting for media content
            speech_config (Optional[SpeechConfig]):
                Configuration for speech synthesis
            enable_affective_dialog (Optional[bool]):
                Whether to enable emotional dialog features
        """

    def __str__(self) -> str: ...

class HarmCategory:
    HarmCategoryUnspecified: "HarmCategory"
    HarmCategoryHateSpeech: "HarmCategory"
    HarmCategoryDangerousContent: "HarmCategory"
    HarmCategoryHarassment: "HarmCategory"
    HarmCategorySexuallyExplicit: "HarmCategory"
    HarmCategoryImageHate: "HarmCategory"
    HarmCategoryImageDangerousContent: "HarmCategory"
    HarmCategoryImageHarassment: "HarmCategory"
    HarmCategoryImageSexuallyExplicit: "HarmCategory"

class HarmBlockThreshold:
    HarmBlockThresholdUnspecified: "HarmBlockThreshold"
    BlockLowAndAbove: "HarmBlockThreshold"
    BlockMediumAndAbove: "HarmBlockThreshold"
    BlockOnlyHigh: "HarmBlockThreshold"
    BlockNone: "HarmBlockThreshold"
    Off: "HarmBlockThreshold"

class HarmBlockMethod:
    HarmBlockMethodUnspecified: "HarmBlockMethod"
    Severity: "HarmBlockMethod"
    Probability: "HarmBlockMethod"

class ModelArmorConfig:
    def __init__(
        self,
        prompt_template_name: Optional[str],
        response_template_name: Optional[str],
    ) -> None:
        """
        Args:
            prompt_template_name (Optional[str]):
                The name of the prompt template to use.
            response_template_name (Optional[str]):
                The name of the response template to use.
        """

    @property
    def prompt_template_name(self) -> Optional[str]: ...
    @property
    def response_template_name(self) -> Optional[str]: ...

class SafetySetting:
    category: HarmCategory
    threshold: HarmBlockThreshold
    method: Optional[HarmBlockMethod]

    def __init__(
        self,
        category: HarmCategory,
        threshold: HarmBlockThreshold,
        method: Optional[HarmBlockMethod] = None,
    ) -> None:
        """Initialize SafetySetting with required and optional parameters.

        Args:
            category (HarmCategory):
                The category of harm to protect against.
            threshold (HarmBlockThreshold):
                The threshold for blocking content.
            method (Optional[HarmBlockMethod]):
                The method used for blocking (if any).
        """

class Mode:
    ModeUnspecified: "Mode"
    Any: "Mode"
    Auto: "Mode"
    None_Mode: "Mode"  # type: ignore

class FunctionCallingConfig:
    @property
    def mode(self) -> Optional[Mode]: ...
    @property
    def allowed_function_names(self) -> Optional[list[str]]: ...
    def __init__(self, mode: Optional[Mode], allowed_function_names: Optional[list[str]]) -> None: ...

class LatLng:
    @property
    def latitude(self) -> float: ...
    @property
    def longitude(self) -> float: ...
    def __init__(self, latitude: float, longitude: float) -> None:
        """Initialize LatLng with latitude and longitude.

        Args:
            latitude (float):
                The latitude value.
            longitude (float):
                The longitude value.
        """

class RetrievalConfig:
    @property
    def lat_lng(self) -> LatLng: ...
    @property
    def language_code(self) -> str: ...
    def __init__(self, lat_lng: LatLng, language_code: str) -> None:
        """Initialize RetrievalConfig with latitude/longitude and language code.

        Args:
            lat_lng (LatLng):
                The latitude and longitude configuration.
            language_code (str):
                The language code for the retrieval.
        """

class ToolConfig:
    @property
    def function_calling_config(self) -> Optional[FunctionCallingConfig]: ...
    @property
    def retrieval_config(self) -> Optional[RetrievalConfig]: ...
    def __init__(
        self,
        function_calling_config: Optional[FunctionCallingConfig],
        retrieval_config: Optional[RetrievalConfig],
    ) -> None: ...

class GeminiSettings:
    def __init__(
        self,
        labels: Optional[dict[str, str]] = None,
        tool_config: Optional[ToolConfig] = None,
        generation_config: Optional[GenerationConfig] = None,
        safety_settings: Optional[list[SafetySetting]] = None,
        model_armor_config: Optional[ModelArmorConfig] = None,
        extra_body: Optional[dict] = None,
    ) -> None:
        """Settings to pass to the Gemini API when creating a request

        Reference:
            https://cloud.google.com/vertex-ai/generative-ai/docs/reference/rest/v1beta1/projects.locations.endpoints/generateContent

        Args:
            labels (Optional[dict[str, str]]):
                An optional dictionary of labels for the settings.
            tool_config (Optional[ToolConfig]):
                Configuration for tools like function calling and retrieval.
            generation_config (Optional[GenerationConfig]):
                Configuration for content generation parameters.
            safety_settings (Optional[list[SafetySetting]]):
                List of safety settings to apply.
            model_armor_config (Optional[ModelArmorConfig]):
                Configuration for model armor templates.
            extra_body (Optional[dict]):
                Additional configuration as a dictionary.
        """

    @property
    def labels(self) -> Optional[dict[str, str]]: ...
    @property
    def tool_config(self) -> Optional[ToolConfig]: ...
    @property
    def generation_config(self) -> Optional[GenerationConfig]: ...
    @property
    def safety_settings(self) -> Optional[list[SafetySetting]]: ...
    @property
    def model_armor_config(self) -> Optional[ModelArmorConfig]: ...
    @property
    def extra_body(self) -> Optional[dict]: ...
    def __str__(self) -> str: ...

class EmbeddingTaskType:
    TaskTypeUnspecified = "EmbeddingTaskType"
    RetrievalQuery = "EmbeddingTaskType"
    RetrievalDocument = "EmbeddingTaskType"
    SemanticSimilarity = "EmbeddingTaskType"
    Classification = "EmbeddingTaskType"
    Clustering = "EmbeddingTaskType"
    QuestionAnswering = "EmbeddingTaskType"
    FactVerification = "EmbeddingTaskType"
    CodeRetrievalQuery = "EmbeddingTaskType"

class GeminiEmbeddingConfig:
    def __init__(
        self,
        model: Optional[str] = None,
        output_dimensionality: Optional[int] = None,
        task_type: Optional[EmbeddingTaskType | str] = None,
    ) -> None:
        """Configuration to pass to the Gemini Embedding API when creating a request


        Args:
            model (Optional[str]):
                The embedding model to use. If not specified, the default gemini model will be used.
            output_dimensionality (Optional[int]):
                The output dimensionality of the embeddings. If not specified, a default value will be used.
            task_type (Optional[EmbeddingTaskType]):
                The type of embedding task to perform. If not specified, the default gemini task type will be used.
        """

class ContentEmbedding:
    @property
    def values(self) -> List[float]: ...

class GeminiEmbeddingResponse:
    @property
    def embedding(self) -> ContentEmbedding: ...

class PredictResponse:
    @property
    def predictions(self) -> List[dict]: ...
    @property
    def metadata(self) -> Any: ...
    @property
    def deployed_model_id(self) -> str: ...
    @property
    def model(self) -> str: ...
    @property
    def model_version_id(self) -> str: ...
    @property
    def model_display_name(self) -> str: ...
    def __str__(self): ...

class PredictRequest:
    def __init__(self, instances: List[dict], parameters: Optional[dict] = None) -> None:
        """Request to pass to the Vertex Predict API when creating a request

        Args:
            instances (List[dict]):
                A list of instances to be sent in the request.
            parameters (Optional[dict]):
                Optional parameters for the request.
        """

    @property
    def instances(self) -> List[dict]: ...
    @property
    def parameters(self) -> dict: ...
    def __str__(self): ...

###### __potatohead__.base module ######
class PromptTokenDetails:
    """Details about the prompt tokens used in a request."""

    @property
    def audio_tokens(self) -> int:
        """The number of audio tokens used in the request."""

    @property
    def cached_tokens(self) -> int:
        """The number of cached tokens used in the request."""

class CompletionTokenDetails:
    """Details about the completion tokens used in a model response."""

    @property
    def accepted_prediction_tokens(self) -> int:
        """The number of accepted prediction tokens used in the response."""

    @property
    def audio_tokens(self) -> int:
        """The number of audio tokens used in the response."""

    @property
    def reasoning_tokens(self) -> int:
        """The number of reasoning tokens used in the response."""

    @property
    def rejected_prediction_tokens(self) -> int:
        """The number of rejected prediction tokens used in the response."""

class Usage:
    """Usage statistics for a model response."""

    @property
    def completion_tokens(self) -> int:
        """The number of completion tokens used in the response."""

    @property
    def prompt_tokens(self) -> int:
        """The number of prompt tokens used in the request."""

    @property
    def total_tokens(self) -> int:
        """The total number of tokens used in the request and response."""

    @property
    def completion_tokens_details(self) -> CompletionTokenDetails:
        """Details about the completion tokens used in the response."""

    @property
    def prompt_tokens_details(self) -> "PromptTokenDetails":
        """Details about the prompt tokens used in the request."""

    @property
    def finish_reason(self) -> str:
        """The reason why the model stopped generating tokens"""

class ImageUrl:
    def __init__(
        self,
        url: str,
        kind: Literal["image-url"] = "image-url",
    ) -> None:
        """Create an ImageUrl object.

        Args:
            url (str):
                The URL of the image.
            kind (Literal["image-url"]):
                The kind of the content.
        """

    @property
    def url(self) -> str:
        """The URL of the image."""

    @property
    def kind(self) -> str:
        """The kind of the content."""

    @property
    def media_type(self) -> str:
        """The media type of the image URL."""

    @property
    def format(self) -> str:
        """The format of the image URL."""

class AudioUrl:
    def __init__(
        self,
        url: str,
        kind: Literal["audio-url"] = "audio-url",
    ) -> None:
        """Create an AudioUrl object.

        Args:
            url (str):
                The URL of the audio.
            kind (Literal["audio-url"]):
                The kind of the content.
        """

    @property
    def url(self) -> str:
        """The URL of the audio."""

    @property
    def kind(self) -> str:
        """The kind of the content."""

    @property
    def media_type(self) -> str:
        """The media type of the audio URL."""

    @property
    def format(self) -> str:
        """The format of the audio URL."""

class BinaryContent:
    def __init__(
        self,
        data: bytes,
        media_type: str,
        kind: str = "binary",
    ) -> None:
        """Create a BinaryContent object.

        Args:
            data (bytes):
                The binary data.
            media_type (str):
                The media type of the binary data.
            kind (str):
                The kind of the content
        """

    @property
    def media_type(self) -> str:
        """The media type of the binary content."""

    @property
    def format(self) -> str:
        """The format of the binary content."""

    @property
    def data(self) -> bytes:
        """The binary data."""

    @property
    def kind(self) -> str:
        """The kind of the content."""

class DocumentUrl:
    def __init__(
        self,
        url: str,
        kind: Literal["document-url"] = "document-url",
    ) -> None:
        """Create a DocumentUrl object.

        Args:
            url (str):
                The URL of the document.
            kind (Literal["document-url"]):
                The kind of the content.
        """

    @property
    def url(self) -> str:
        """The URL of the document."""

    @property
    def kind(self) -> str:
        """The kind of the content."""

    @property
    def media_type(self) -> str:
        """The media type of the document URL."""

    @property
    def format(self) -> str:
        """The format of the document URL."""

class Message:
    def __init__(self, content: str | ImageUrl | AudioUrl | BinaryContent | DocumentUrl) -> None:
        """Create a Message object.

        Args:
            content (str | ImageUrl | AudioUrl | BinaryContent | DocumentUrl):
                The content of the message.
        """

    @property
    def content(self) -> str | ImageUrl | AudioUrl | BinaryContent | DocumentUrl:
        """The content of the message"""

    def bind(self, name: str, value: str) -> "Message":
        """Bind context to a specific variable in the prompt. This is an immutable operation meaning that it
        will return a new Message object with the context bound.

            Example with Prompt that contains two messages

            ```python
                prompt = Prompt(
                    model="openai:gpt-4o",
                    message=[
                        "My prompt variable is ${variable}",
                        "This is another message",
                    ],
                    system_instruction="system_prompt",
                )
                bounded_prompt = prompt.message[0].bind("variable", "hello world").unwrap() # we bind "hello world" to "variable"
            ```

        Args:
            name (str):
                The name of the variable to bind.
            value (str):
                The value to bind the variable to.

        Returns:
            Message:
                The message with the context bound.
        """

    def bind_mut(self, name: str, value: str) -> "Message":
        """Bind context to a specific variable in the prompt. This is a mutable operation meaning that it
        will modify the current Message object.

            Example with Prompt that contains two messages

            ```python
                prompt = Prompt(
                    model="openai:gpt-4o",
                    message=[
                        "My prompt variable is ${variable}",
                        "This is another message",
                    ],
                    system_instruction="system_prompt",
                )
                prompt.message[0].bind_mut("variable", "hello world") # we bind "hello world" to "variable"
            ```

        Args:
            name (str):
                The name of the variable to bind.
            value (str):
                The value to bind the variable to.

        Returns:
            Message:
                The message with the context bound.
        """

    def unwrap(self) -> Any:
        """Unwrap the message content.

        Returns:
            A serializable representation of the message content, which can be a string, list, or dict.
        """

    def model_dump(self) -> Dict[str, Any]:
        """Unwrap the message content and serialize it to a dictionary.

        Returns:
            Dict[str, Any]:
                The message dictionary with keys "content" and "role".
        """

class ModelSettings:
    def __init__(self, settings: OpenAIChatSettings | GeminiSettings) -> None:
        """ModelSettings for configuring the model.

        Args:
            settings (OpenAIChatSettings | GeminiSettings):
                The settings to use for the model. Currently supports OpenAI and Gemini settings.
        """

    @property
    def settings(self) -> OpenAIChatSettings | GeminiSettings:
        """The settings to use for the model."""

    def model_dump_json(self) -> str:
        """The JSON representation of the model settings."""

class Prompt:
    def __init__(
        self,
        message: (
            str
            | Sequence[str | ImageUrl | AudioUrl | BinaryContent | DocumentUrl]
            | Message
            | List[Message]
            | List[Dict[str, Any]]
        ),
        model: str,
        provider: Provider | str,
        system_instruction: Optional[str | List[str]] = None,
        model_settings: Optional[ModelSettings | OpenAIChatSettings | GeminiSettings] = None,
        response_format: Optional[Any] = None,
    ) -> None:
        """Prompt for interacting with an LLM API.

        Args:
            message (str | Sequence[str | ImageUrl | AudioUrl | BinaryContent | DocumentUrl] | Message | List[Message]):
                The prompt to use.
            model (str):
                The model to use for the prompt
            provider (Provider | str):
                The provider to use for the prompt.
            system_instruction (Optional[str | List[str]]):
                The system prompt to use in the prompt.
            model_settings (None):
                The model settings to use for the prompt.
                Defaults to None which means no model settings will be used
            response_format (Optional[BaseModel | Score]):
                The response format to use for the prompt. This is used for Structured Outputs
                (https://platform.openai.com/docs/guides/structured-outputs?api-mode=chat).
                Currently, response_format only support Pydantic BaseModel classes and the PotatoHead Score class.
                The provided response_format will be parsed into a JSON schema.

        """

    @property
    def model(self) -> str:
        """The model to use for the prompt."""

    @property
    def provider(self) -> str:
        """The provider to use for the prompt."""

    @property
    def model_identifier(self) -> Any:
        """Concatenation of provider and model, used for identifying the model in the prompt. This
        is commonly used with pydantic_ai to identify the model to use for the agent.

        Example:
            ```python
                prompt = Prompt(
                    model="gpt-4o",
                    message="My prompt variable is ${variable}",
                    system_instruction="system_instruction",
                    provider="openai",
                )
                agent = Agent(
                    prompt.model_identifier, # "openai:gpt-4o"
                    system_instructions=prompt.system_instruction[0].unwrap(),
                )
            ```
        """

    @property
    def model_settings(self) -> ModelSettings:
        """The model settings to use for the prompt."""

    @property
    def message(
        self,
    ) -> List[Message]:
        """The user message to use in the prompt."""

    @property
    def system_instruction(self) -> List[Message]:
        """The system message to use in the prompt."""

    def save_prompt(self, path: Optional[Path] = None) -> None:
        """Save the prompt to a file.

        Args:
            path (Optional[Path]):
                The path to save the prompt to. If None, the prompt will be saved to
                the current working directory.
        """

    @staticmethod
    def from_path(path: Path) -> "Prompt":
        """Load a prompt from a file.

        Args:
            path (Path):
                The path to the prompt file.

        Returns:
            Prompt:
                The loaded prompt.
        """

    @staticmethod
    def model_validate_json(json_string: str) -> "Prompt":
        """Validate the model JSON.

        Args:
            json_string (str):
                The JSON string to validate.
        Returns:
            Prompt:
                The prompt object.
        """

    def model_dump_json(self) -> str:
        """Dump the model to a JSON string.

        Returns:
            str:
                The JSON string.
        """

    def bind(
        self,
        name: Optional[str] = None,
        value: Optional[str | int | float | bool | list] = None,
        **kwargs: Any,
    ) -> "Prompt":
        """Bind context to a specific variable in the prompt. This is an immutable operation meaning that it
        will return a new Prompt object with the context bound. This will iterate over all user messages.

        Args:
            name (str):
                The name of the variable to bind.
            value (str | int | float | bool | list):
                The value to bind the variable to. Must be a JSON serializable type.
            **kwargs (Any):
                Additional keyword arguments to bind to the prompt. This can be used to bind multiple variables at once.

        Returns:
            Prompt:
                The prompt with the context bound.
        """

    def bind_mut(
        self,
        name: Optional[str] = None,
        value: Optional[str | int | float | bool | list] = None,
        **kwargs: Any,
    ) -> "Prompt":
        """Bind context to a specific variable in the prompt. This is a mutable operation meaning that it
        will modify the current Prompt object. This will iterate over all user messages.

        Args:
            name (str):
                The name of the variable to bind.
            value (str | int | float | bool | list):
                The value to bind the variable to. Must be a JSON serializable type.
            **kwargs (Any):
                Additional keyword arguments to bind to the prompt. This can be used to bind multiple variables at once.

        Returns:
            Prompt:
                The prompt with the context bound.
        """

    @property
    def response_json_schema(self) -> Optional[str]:
        """The JSON schema for the response if provided."""

    def __str__(self): ...

class Provider:
    OpenAI: "Provider"
    Gemini: "Provider"
    Vertex: "Provider"
    Google: "Provider"
    Anthropic: "Provider"

class TaskStatus:
    Pending: "TaskStatus"
    Running: "TaskStatus"
    Completed: "TaskStatus"
    Failed: "TaskStatus"

class ResponseLogProbs:
    @property
    def token(self) -> str:
        """The token for which the log probabilities are calculated."""

    @property
    def logprob(self) -> float:
        """The log probability of the token."""

class LogProbs:
    @property
    def tokens(self) -> List[ResponseLogProbs]:
        """The log probabilities of the tokens in the response.
        This is primarily used for debugging and analysis purposes.
        """

    def __str__(self) -> str:
        """String representation of the log probabilities."""

class AgentResponse:
    @property
    def id(self) -> str:
        """The ID of the agent response."""

    @property
    def result(self) -> Any:
        """The result of the agent response. This can be a Pydantic BaseModel class or a supported
        potato_head response type such as `Score`. If neither is provided, the response json
        will be returned as a dictionary.
        """

    @property
    def token_usage(self) -> Usage:
        """Returns the token usage of the agent response if supported"""

    @property
    def log_probs(self) -> List["ResponseLogProbs"]:
        """Returns the log probabilities of the agent response if supported.
        This is primarily used for debugging and analysis purposes.
        """

class Task:
    def __init__(
        self,
        agent_id: str,
        prompt: Prompt,
        dependencies: List[str] = [],
        id: Optional[str] = None,
    ) -> None:
        """Create a Task object.

        Args:
            agent_id (str):
                The ID of the agent that will execute the task.
            prompt (Prompt):
                The prompt to use for the task.
            dependencies (List[str]):
                The dependencies of the task.
            id (Optional[str]):
                The ID of the task. If None, a random uuid7 will be generated.
        """

    @property
    def prompt(self) -> Prompt:
        """The prompt to use for the task."""

    @property
    def dependencies(self) -> List[str]:
        """The dependencies of the task."""

    @property
    def id(self) -> str:
        """The ID of the task."""

    @property
    def status(self) -> TaskStatus:
        """The status of the task."""

class TaskList:
    def __init__(self) -> None:
        """Create a TaskList object."""

class Agent:
    def __init__(
        self,
        provider: Provider | str,
        system_instruction: Optional[str | List[str] | Message | List[Message]] = None,
    ) -> None:
        """Create an Agent object.

        Args:
            provider (Provider | str):
                The provider to use for the agent. This can be a Provider enum or a string
                representing the provider.
            system_instruction (Optional[str | List[str] | Message | List[Message]]):
                The system message to use for the agent. This can be a string, a list of strings,
                a Message object, or a list of Message objects. If None, no system message will be used.
                This is added to all tasks that the agent executes. If a given task contains it's own
                system message, the agent's system message will be prepended to the task's system message.

        Example:
        ```python
            agent = Agent(
                provider=Provider.OpenAI,
                system_instruction="You are a helpful assistant.",
            )
        ```
        """

    @property
    def system_instruction(self) -> List[Message]:
        """The system message to use for the agent. This is a list of Message objects."""

    def execute_task(
        self,
        task: Task,
        output_type: Optional[Any] = None,
        model: Optional[str] = None,
    ) -> AgentResponse:
        """Execute a task.

        Args:
            task (Task):
                The task to execute.
            output_type (Optional[Any]):
                The output type to use for the task. This can either be a Pydantic `BaseModel` class
                or a supported PotatoHead response type such as `Score`.
            model (Optional[str]):
                The model to use for the task. If not provided, defaults to the `model` provided within
                the Task's prompt. If the Task's prompt does not have a model, an error will be raised.

        Returns:
            AgentResponse:
                The response from the agent after executing the task.
        """

    def execute_prompt(
        self,
        prompt: Prompt,
        output_type: Optional[Any] = None,
        model: Optional[str] = None,
    ) -> AgentResponse:
        """Execute a prompt.

        Args:
            prompt (Prompt):`
                The prompt to execute.
            output_type (Optional[Any]):
                The output type to use for the task. This can either be a Pydantic `BaseModel` class
                or a supported potato_head response type such as `Score`.
            model (Optional[str]):
                The model to use for the task. If not provided, defaults to the `model` provided within
                the Prompt. If the Prompt does not have a model, an error will be raised.

        Returns:
            AgentResponse:
                The response from the agent after executing the task.
        """

    @property
    def id(self) -> str:
        """The ID of the agent. This is a random uuid7 that is generated when the agent is created."""

ConfigT = TypeVar("ConfigT", OpenAIEmbeddingConfig, GeminiEmbeddingConfig, None)

class Embedder:
    """Class for creating embeddings."""

    def __init__(  # type: ignore
        self,
        provider: Provider | str,
        config: Optional[OpenAIEmbeddingConfig | GeminiEmbeddingConfig] = None,
    ) -> None:
        """Create an Embedder object.

        Args:
            provider (Provider | str):
                The provider to use for the embedder. This can be a Provider enum or a string
                representing the provider.
            config (Optional[OpenAIEmbeddingConfig | GeminiEmbeddingConfig]):
                The configuration to use for the embedder. This can be a Pydantic BaseModel class
                representing the configuration for the provider. If no config is provided,
                defaults to OpenAI provider configuration.
        """

    def embed(
        self,
        input: str | List[str] | PredictRequest,
    ) -> OpenAIEmbeddingResponse | GeminiEmbeddingResponse | PredictResponse:
        """Create embeddings for input.

        Args:
            input: The input to embed. Type depends on provider:
                - OpenAI/Gemini: str | List[str]
                - Vertex: PredictRequest

        Returns:
            Provider-specific response type.
            OpenAIEmbeddingResponse for OpenAI,
            GeminiEmbeddingResponse for Gemini,
            PredictResponse for Vertex.

        Examples:
            ```python
            ## OpenAI
            embedder = Embedder(Provider.OpenAI)
            response = embedder.embed(input="Test input")

            ## Gemini
            embedder = Embedder(Provider.Gemini, config=GeminiEmbeddingConfig(model="gemini-embedding-001"))
            response = embedder.embed(input="Test input")

            ## Vertex
            from potato_head.google import PredictRequest
            embedder = Embedder(Provider.Vertex)
            response = embedder.embed(input=PredictRequest(text="Test input"))
            ```
        """

class Workflow:
    def __init__(self, name: str) -> None:
        """Create a Workflow object.

        Args:
            name (str):
                The name of the workflow.
        """

    @property
    def name(self) -> str:
        """The name of the workflow."""

    @property
    def task_list(self) -> TaskList:
        """The tasks in the workflow."""

    @property
    def agents(self) -> Dict[str, Agent]:
        """The agents in the workflow."""

    @property
    def is_workflow(self) -> bool:
        """Returns True if the workflow is a valid workflow, otherwise False.
        This is used to determine if the workflow can be executed.
        """

    def __workflow__(self) -> str:
        """Returns a string representation of the workflow."""

    def add_task_output_types(self, task_output_types: Dict[str, Any]) -> None:
        """Add output types for tasks in the workflow. This is primarily used for
        when loading a workflow as python objects are not serializable.

        Args:
            task_output_types (Dict[str, Any]):
                A dictionary mapping task IDs to their output types.
                This can either be a Pydantic `BaseModel` class or a supported potato_head response type such as `Score`.
        """

    def add_task(self, task: Task, output_type: Optional[Any]) -> None:
        """Add a task to the workflow.

        Args:
            task (Task):
                The task to add to the workflow.
            output_type (Optional[Any]):
                The output type to use for the task. This can either be a Pydantic `BaseModel` class
                or a supported potato_head response type such as `Score`.
        """

    def add_tasks(self, tasks: List[Task]) -> None:
        """Add multiple tasks to the workflow.

        Args:
            tasks (List[Task]):
                The tasks to add to the workflow.
        """

    def add_agent(self, agent: Agent) -> None:
        """Add an agent to the workflow.

        Args:
            agent (Agent):
                The agent to add to the workflow.
        """

    def is_complete(self) -> bool:
        """Check if the workflow is complete.

        Returns:
            bool:
                True if the workflow is complete, False otherwise.
        """

    def pending_count(self) -> int:
        """Get the number of pending tasks in the workflow.

        Returns:
            int:
                The number of pending tasks in the workflow.
        """

    def execution_plan(self) -> Dict[str, List[str]]:
        """Get the execution plan for the workflow.

        Returns:
            Dict[str, List[str]]:
                A dictionary where the keys are task IDs and the values are lists of task IDs
                that the task depends on.
        """

    def run(
        self,
        global_context: Optional[Dict[str, Any]] = None,
    ) -> "WorkflowResult":
        """Run the workflow. This will execute all tasks in the workflow and return when all tasks are complete.

        Args:
            global_context (Optional[Dict[str, Any]]):
                A dictionary of global context to bind to the workflow.
                All tasks in the workflow will have this context bound to them.
        """

    def model_dump_json(self) -> str:
        """Dump the workflow to a JSON string.

        Returns:
            str:
                The JSON string.
        """

    @staticmethod
    def model_validate_json(json_string: str, output_types: Optional[Dict[str, Any]]) -> "Workflow":
        """Load a workflow from a JSON string.

        Args:
            json_string (str):
                The JSON string to validate.
            output_types (Optional[Dict[str, Any]]):
                A dictionary mapping task IDs to their output types.
                This can either be a Pydantic `BaseModel` class or a supported potato_head response type such as `Score`.

        Returns:
            Workflow:
                The workflow object.
        """

class PyTask:
    """Python-specific task interface for Task objects and results"""

    @property
    def prompt(self) -> Prompt:
        """The prompt to use for the task."""

    @property
    def dependencies(self) -> List[str]:
        """The dependencies of the task."""

    @property
    def id(self) -> str:
        """The ID of the task."""

    @property
    def agent_id(self) -> str:
        """The ID of the agent that will execute the task."""

    @property
    def status(self) -> TaskStatus:
        """The status of the task."""

    @property
    def result(self) -> Optional[AgentResponse]:
        """The result of the task if it has been executed, otherwise None."""

    def __str__(self) -> str: ...

class ChatResponse:
    def to_py(self) -> Any:
        """Convert the ChatResponse to it's Python representation."""

    def __str__(self) -> str:
        """Return a string representation of the ChatResponse."""

class EventDetails:
    @property
    def prompt(self) -> Optional[Prompt]:
        """The prompt used for the task."""

    @property
    def response(self) -> Optional[ChatResponse]:
        """The response from the agent after executing the task."""

    @property
    def duration(self) -> Optional[datetime.timedelta]:
        """The duration of the task execution."""

    @property
    def start_time(self) -> Optional[datetime.datetime]:
        """The start time of the task execution."""

    @property
    def end_time(self) -> Optional[datetime.datetime]:
        """The end time of the task execution."""

    @property
    def error(self) -> Optional[str]:
        """The error message if the task failed, otherwise None."""

class TaskEvent:
    @property
    def id(self) -> str:
        """The ID of the event"""

    @property
    def workflow_id(self) -> str:
        """The ID of the workflow that the task is part of."""

    @property
    def task_id(self) -> str:
        """The ID of the task that the event is associated with."""

    @property
    def status(self) -> TaskStatus:
        """The status of the task at the time of the event."""

    @property
    def timestamp(self) -> datetime.datetime:
        """The timestamp of the event. This is the time when the event occurred."""

    @property
    def updated_at(self) -> datetime.datetime:
        """The timestamp of when the event was last updated. This is useful for tracking changes to the event."""

    @property
    def details(self) -> EventDetails:
        """Additional details about the event. This can include information such as error messages or other relevant data."""

class WorkflowResult:
    @property
    def tasks(self) -> Dict[str, PyTask]:
        """The tasks in the workflow result."""

    @property
    def events(self) -> List[TaskEvent]:
        """The events that occurred during the workflow execution. This is a list of dictionaries
        where each dictionary contains information about the event such as the task ID, status, and timestamp.
        """

class Score:
    """A class representing a score with a score value and a reason. This is typically used
    as a response type for tasks/prompts that require scoring or evaluation of results.

    Example:
    ```python
        Prompt(
            model="openai:gpt-4o",
            message="What is the score of this response?",
            system_instruction="system_prompt",
            response_format=Score,
        )
    ```
    """

    @property
    def score(self) -> int:
        """The score value."""

    @property
    def reason(self) -> str:
        """The reason for the score."""

    @staticmethod
    def model_validate_json(json_string: str) -> "Score":
        """Validate the score JSON.

        Args:
            json_string (str):
                The JSON string to validate.

        Returns:
            Score:
                The score object.
        """

    def __str__(self): ...

###### __potatohead__.mock module ######

class LLMTestServer:
    """
    Mock server for OpenAI API.
    This class is used to simulate the OpenAI API for testing purposes.
    """

    def __init__(self):
        """Initialize the mock server."""

    def __enter__(self):
        """
        Start the mock server.
        """

    def __exit__(self, exc_type, exc_value, traceback):
        """
        Stop the mock server.
        """

###### __potatohead__.logging module ######

class LogLevel:
    Debug: "LogLevel"
    Info: "LogLevel"
    Warn: "LogLevel"
    Error: "LogLevel"
    Trace: "LogLevel"

class WriteLevel:
    Stdout: "WriteLevel"
    Stderror: "WriteLevel"

class LoggingConfig:
    show_threads: bool
    log_level: LogLevel
    write_level: WriteLevel
    use_json: bool

    def __init__(
        self,
        show_threads: bool = True,
        log_level: LogLevel = LogLevel.Info,
        write_level: WriteLevel = WriteLevel.Stdout,
        use_json: bool = False,
    ) -> None:
        """
        Logging configuration options.

        Args:
            show_threads:
                Whether to include thread information in log messages.
                Default is True.

            log_level:
                Log level for the logger.
                Default is LogLevel.Info.

            write_level:
                Write level for the logger.
                Default is WriteLevel.Stdout.

            use_json:
                Whether to write log messages in JSON format.
                Default is False.
        """

    @staticmethod
    def json_default() -> "LoggingConfig":
        """Gets a default JSON configuration.

        show_threads: True
        log_level: Env or LogLevel.Info
        write_level: WriteLevel.Stdout
        use_json: True

        Returns:
            LoggingConfig:
                The default JSON configuration.
        """

    @staticmethod
    def default() -> "LoggingConfig":
        """Gets a default configuration.

        show_threads: True
        log_level: Env or LogLevel.Info
        write_level: WriteLevel.Stdout
        use_json: False

        Returns:
            LoggingConfig:
                The default JSON configuration.
        """

class RustyLogger:
    """The Rusty Logger class to use with your python and rust-backed projects."""

    @staticmethod
    def setup_logging(config: Optional[LoggingConfig] = None) -> None:
        """Sets up the logger with the given configuration.

        Args:
            config (LoggingConfig):
                The configuration to use for the logger.
        """

    @staticmethod
    def get_logger(config: Optional[LoggingConfig] = None) -> "RustyLogger":
        """Gets the logger instance.

        Args:
            config (LoggingConfig):
                The configuration to use for the logger.

        Returns:
            RustyLogger:
                The logger instance.
        """

    def debug(self, message: str, *args) -> None:
        """Logs a debug message.

        Args:
            message (str):
                The message to log.

            *args:
                Additional arguments to log.
        """

    def info(self, message: str, *args) -> None:
        """Logs an info message.

        Args:
            message (str):
                The message to log.

            *args:
                Additional arguments to log.
        """

    def warn(self, message: str, *args) -> None:
        """Logs a warning message.

        Args:
            message (str):
                The message to log.

            *args:
                Additional arguments to log.
        """

    def error(self, message: str, *args) -> None:
        """Logs an error message.

        Args:
            message (str):
                The message to log.

            *args:
                Additional arguments to log.
        """

    def trace(self, message: str, *args) -> None:
        """Logs a trace message.

        Args:
            message (str):
                The message to log.

            *args:
                Additional arguments to log.
        """

__all__ = [
    "ImageUrl",
    "AudioUrl",
    "BinaryContent",
    "DocumentUrl",
    "Message",
    "ModelSettings",
    "Prompt",
    "Provider",
    "TaskStatus",
    "AgentResponse",
    "Task",
    "TaskList",
    "Agent",
    "Workflow",
    "PyTask",
    "ChatResponse",
    "EventDetails",
    "TaskEvent",
    "WorkflowResult",
    "Score",
    "Embedder",
    "LLMTestServer",
    ### google
    "Modality",
    "GeminiThinkingConfig",
    "MediaResolution",
    "SpeechConfig",
    "PrebuiltVoiceConfig",
    "VoiceConfig",
    "GenerationConfig",
    "ToolConfig",
    "FunctionCallingConfig",
    "RetrievalConfig",
    "LatLng",
    "ModelArmorConfig",
    "Mode",
    "GeminiSettings",
    "HarmCategory",
    "HarmBlockThreshold",
    "HarmBlockMethod",
    "SafetySetting",
    "GeminiEmbeddingConfig",
    "GeminiEmbeddingResponse",
    "PredictRequest",
    "PredictResponse",
    "EmbeddingTaskType",
    ### openai
    "PromptTokenDetails",
    "CompletionTokenDetails",
    "Usage",
    "AudioParam",
    "ContentPart",
    "Content",
    "Prediction",
    "StreamOptions",
    "ToolChoiceMode",
    "FunctionChoice",
    "FunctionToolChoice",
    "CustomChoice",
    "CustomToolChoice",
    "ToolDefinition",
    "AllowedToolsMode",
    "AllowedTools",
    "OpenAIToolChoice",
    "FunctionDefinition",
    "FunctionTool",
    "TextFormat",
    "Grammar",
    "GrammarFormat",
    "CustomToolFormat",
    "CustomDefinition",
    "CustomTool",
    "OpenAITool",
    "OpenAIChatSettings",
    "OpenAIEmbeddingConfig",
    "OpenAIEmbeddingResponse",
    ### Anthropic
    "AnthropicSettings",
    "Metadata",
    "CacheControl",
    "AnthropicTool",
    "AnthropicThinkingConfig",
    "AnthropicToolChoice",
    ### logging
    "LogLevel",
    "RustyLogger",
    "LoggingConfig",
    "WriteLevel",
]
