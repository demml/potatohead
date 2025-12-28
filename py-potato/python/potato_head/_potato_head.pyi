# pylint: disable=redefined-builtin, invalid-name, dangerous-default-value

import datetime
from pathlib import Path
from typing import Any, Dict, List, Literal, Optional, Sequence, TypeVar, Union

###### __potatohead__.openai module ######

# ============================================================================
# Settings Types
# ============================================================================

class AudioParam:
    """Audio output configuration for OpenAI chat completions.

    This class provides configuration for audio output in chat completions,
    including format and voice selection for text-to-speech capabilities.

    Examples:
        >>> audio = AudioParam(format="mp3", voice="alloy")
        >>> audio.format
        'mp3'
        >>> audio.voice
        'alloy'
    """

    def __init__(
        self,
        format: str,
        voice: str,
    ) -> None:
        """Initialize audio output parameters.

        Args:
            format (str):
                Audio output format (e.g., "mp3", "opus", "aac", "flac", "wav", "pcm")
            voice (str):
                Voice to use for text-to-speech (e.g., "alloy", "echo", "fable",
                "onyx", "nova", "shimmer")
        """

    @property
    def format(self) -> str:
        """The audio output format."""

    @property
    def voice(self) -> str:
        """The voice to use for text-to-speech."""

    def model_dump(self) -> Dict[str, Any]:
        """Convert audio parameters to a dictionary.

        Returns:
            Dict[str, Any]: Dictionary representation of audio parameters
        """

class PredictionContentPart:
    """Content part for predicted outputs in OpenAI requests.

    This class represents a single content part within a predicted output,
    used to improve response times when large parts of the response are known.

    Examples:
        >>> part = PredictionContentPart(type="text", text="Hello, world!")
        >>> part.type
        'text'
        >>> part.text
        'Hello, world!'
    """

    def __init__(
        self,
        type: str,
        text: str,
    ) -> None:
        """Initialize prediction content part.

        Args:
            type (str):
                Type of content (typically "text")
            text (str):
                The predicted text content
        """

    @property
    def type(self) -> str:
        """The content type."""

    @property
    def text(self) -> str:
        """The predicted text content."""

    def model_dump(self) -> Dict[str, Any]:
        """Convert content part to a dictionary.

        Returns:
            Dict[str, Any]: Dictionary representation of content part
        """

class Content:
    """Content for predicted outputs, supporting text or structured parts.

    This class represents the content of a predicted output, which can be
    either a simple text string or an array of structured content parts.

    Examples:
        >>> # Text content
        >>> content = Content(text="Predicted response")
        >>>
        >>> # Structured content
        >>> parts = [PredictionContentPart(type="text", text="Part 1")]
        >>> content = Content(parts=parts)
    """

    def __init__(
        self,
        text: Optional[str] = None,
        parts: Optional[List[PredictionContentPart]] = None,
    ) -> None:
        """Initialize content for predictions.

        Args:
            text (Optional[str]):
                Simple text content (mutually exclusive with parts)
            parts (Optional[List[PredictionContentPart]]):
                Structured content parts (mutually exclusive with text)

        Raises:
            TypeError: If both text and parts are provided or neither is provided
        """

class Prediction:
    """Configuration for predicted outputs in OpenAI requests.

    This class provides configuration for predicted outputs, which can greatly
    improve response times when large parts of the model response are known ahead
    of time.

    Examples:
        >>> content = Content(text="Expected response")
        >>> prediction = Prediction(type="content", content=content)
        >>> prediction.type
        'content'
    """

    def __init__(
        self,
        type: str,
        content: Content,
    ) -> None:
        """Initialize prediction configuration.

        Args:
            type (str):
                Type of prediction (typically "content")
            content (Content):
                The predicted content
        """

    @property
    def type(self) -> str:
        """The prediction type."""

    @property
    def content(self) -> Content:
        """The predicted content."""

class StreamOptions:
    """Options for streaming chat completion responses.

    This class provides configuration for streaming behavior, including
    usage information and obfuscation settings.

    Examples:
        >>> options = StreamOptions(include_usage=True)
        >>> options.include_usage
        True
    """

    def __init__(
        self,
        include_obfuscation: Optional[bool] = None,
        include_usage: Optional[bool] = None,
    ) -> None:
        """Initialize stream options.

        Args:
            include_obfuscation (Optional[bool]):
                Whether to include obfuscation in the stream
            include_usage (Optional[bool]):
                Whether to include usage information in the stream
        """

    @property
    def include_obfuscation(self) -> Optional[bool]:
        """Whether obfuscation is included."""

    @property
    def include_usage(self) -> Optional[bool]:
        """Whether usage information is included."""

class ToolChoiceMode:
    """Mode for tool choice behavior in chat completions.

    This enum defines how the model should handle tool calls during generation.

    Examples:
        >>> mode = ToolChoiceMode.Auto
        >>> mode.value
        'auto'
    """

    NA = "ToolChoiceMode"
    """Model will not call any tools"""

    Auto = "ToolChoiceMode"
    """Model can choose to call tools or generate a message"""

    Required = "ToolChoiceMode"
    """Model must call one or more tools"""

class FunctionChoice:
    """Specification for a specific function to call.

    This class identifies a specific function by name for tool calling.

    Examples:
        >>> function = FunctionChoice(name="get_weather")
        >>> function.name
        'get_weather'
    """

    def __init__(self, name: str) -> None:
        """Initialize function choice.

        Args:
            name (str):
                Name of the function to call
        """

    @property
    def name(self) -> str:
        """The function name."""

class FunctionToolChoice:
    """Tool choice configuration for a specific function.

    This class specifies that the model should call a specific function tool.

    Examples:
        >>> function = FunctionChoice(name="get_weather")
        >>> tool_choice = FunctionToolChoice(function=function)
        >>> tool_choice.type
        'function'
    """

    def __init__(self, function: FunctionChoice) -> None:
        """Initialize function tool choice.

        Args:
            function (FunctionChoice):
                The function to call
        """

    @property
    def type(self) -> str:
        """The tool type (always 'function')."""

    @property
    def function(self) -> FunctionChoice:
        """The function specification."""

class CustomChoice:
    """Specification for a custom tool to call.

    This class identifies a custom tool by name for tool calling.

    Examples:
        >>> custom = CustomChoice(name="custom_tool")
        >>> custom.name
        'custom_tool'
    """

    def __init__(self, name: str) -> None:
        """Initialize custom choice.

        Args:
            name (str):
                Name of the custom tool to call
        """

    @property
    def name(self) -> str:
        """The custom tool name."""

class CustomToolChoice:
    """Tool choice configuration for a custom tool.

    This class specifies that the model should call a specific custom tool.

    Examples:
        >>> custom = CustomChoice(name="custom_tool")
        >>> tool_choice = CustomToolChoice(custom=custom)
        >>> tool_choice.type
        'custom'
    """

    def __init__(self, custom: CustomChoice) -> None:
        """Initialize custom tool choice.

        Args:
            custom (CustomChoice):
                The custom tool to call
        """

    @property
    def type(self) -> str:
        """The tool type (always 'custom')."""

    @property
    def custom(self) -> CustomChoice:
        """The custom tool specification."""

class ToolDefinition:
    """Definition of a tool for allowed tools configuration.

    This class defines a tool that can be included in an allowed tools list.

    Examples:
        >>> tool = ToolDefinition(function_name="get_weather")
        >>> tool.type
        'function'
    """

    def __init__(self, function_name: str) -> None:
        """Initialize tool definition.

        Args:
            function_name (str):
                Name of the function this tool wraps
        """

    @property
    def type(self) -> str:
        """The tool type (always 'function')."""

    @property
    def function(self) -> FunctionChoice:
        """The function specification."""

class AllowedToolsMode:
    """Mode for allowed tools constraint behavior.

    This enum defines how the model should behave when constrained to
    specific tools.

    Examples:
        >>> mode = AllowedToolsMode.Auto
        >>> mode.value
        'auto'
    """

    Auto = "AllowedToolsMode"
    """Model can pick from allowed tools or generate a message"""

    Required = "AllowedToolsMode"
    """Model must call one or more of the allowed tools"""

class InnerAllowedTools:
    """Inner configuration for allowed tools.

    This class contains the actual list of allowed tools and the mode.

    Examples:
        >>> tools = [ToolDefinition("get_weather")]
        >>> inner = InnerAllowedTools(mode=AllowedToolsMode.Auto, tools=tools)
    """

    @property
    def mode(self) -> AllowedToolsMode:
        """The mode for allowed tools."""

    @property
    def tools(self) -> List[ToolDefinition]:
        """The list of allowed tools."""

class AllowedTools:
    """Configuration for constraining model to specific tools.

    This class specifies a list of tools the model is allowed to use,
    along with the behavior mode.

    Examples:
        >>> tools = [ToolDefinition("get_weather")]
        >>> allowed = AllowedTools(mode=AllowedToolsMode.Auto, tools=tools)
        >>>
        >>> # Or from function names
        >>> allowed = AllowedTools.from_function_names(
        ...     mode=AllowedToolsMode.Required,
        ...     function_names=["get_weather", "get_time"]
        ... )
    """

    def __init__(
        self,
        mode: AllowedToolsMode,
        tools: List[ToolDefinition],
    ) -> None:
        """Initialize allowed tools configuration.

        Args:
            mode (AllowedToolsMode):
                The mode for tool usage behavior
            tools (List[ToolDefinition]):
                List of allowed tools
        """

    @staticmethod
    def from_function_names(
        mode: AllowedToolsMode,
        function_names: List[str],
    ) -> "AllowedTools":
        """Create AllowedTools from function names.

        Args:
            mode (AllowedToolsMode):
                The mode for tool usage behavior
            function_names (List[str]):
                List of function names to allow

        Returns:
            AllowedTools: Configured allowed tools instance
        """

    @property
    def type(self) -> str:
        """The configuration type (always 'allowed_tools')."""

    @property
    def allowed_tools(self) -> InnerAllowedTools:
        """The inner allowed tools configuration."""

    def __str__(self) -> str:
        """String representation of allowed tools.

        Returns:
            str: String representation
        """

class ToolChoice:
    """Tool choice configuration for chat completions.

    This class configures how the model should handle tool calling, supporting
    multiple modes including simple mode selection, specific tool choice, and
    allowed tools constraints.

    Examples:
        >>> # Simple mode
        >>> choice = ToolChoice.from_mode(ToolChoiceMode.Auto)
        >>>
        >>> # Specific function
        >>> choice = ToolChoice.from_function("get_weather")
        >>>
        >>> # Custom tool
        >>> choice = ToolChoice.from_custom("custom_tool")
        >>>
        >>> # Allowed tools
        >>> allowed = AllowedTools.from_function_names(
        ...     AllowedToolsMode.Auto,
        ...     ["get_weather"]
        ... )
        >>> choice = ToolChoice.from_allowed_tools(allowed)
    """

    @staticmethod
    def from_mode(mode: ToolChoiceMode) -> "ToolChoice":
        """Create tool choice from mode.

        Args:
            mode (ToolChoiceMode):
                The tool choice mode

        Returns:
            ToolChoice: Tool choice configured with mode
        """

    @staticmethod
    def from_function(function_name: str) -> "ToolChoice":
        """Create tool choice for specific function.

        Args:
            function_name (str):
                Name of the function to call

        Returns:
            ToolChoice: Tool choice configured for function
        """

    @staticmethod
    def from_custom(custom_name: str) -> "ToolChoice":
        """Create tool choice for custom tool.

        Args:
            custom_name (str):
                Name of the custom tool to call

        Returns:
            ToolChoice: Tool choice configured for custom tool
        """

    @staticmethod
    def from_allowed_tools(allowed_tools: AllowedTools) -> "ToolChoice":
        """Create tool choice from allowed tools.

        Args:
            allowed_tools (AllowedTools):
                Allowed tools configuration

        Returns:
            ToolChoice: Tool choice configured with allowed tools
        """

    def __str__(self) -> str:
        """String representation of tool choice.

        Returns:
            str: String representation
        """

class FunctionDefinition:
    """Definition of a function tool for OpenAI chat completions.

    This class defines a function that can be called by the model, including
    its name, description, parameters schema, and strict mode setting.

    Examples:
        >>> # Simple function
        >>> func = FunctionDefinition(
        ...     name="get_weather",
        ...     description="Get weather for a location"
        ... )
        >>>
        >>> # With parameters
        >>> params = {
        ...     "type": "object",
        ...     "properties": {
        ...         "location": {"type": "string"}
        ...     },
        ...     "required": ["location"]
        ... }
        >>> func = FunctionDefinition(
        ...     name="get_weather",
        ...     description="Get weather",
        ...     parameters=params,
        ...     strict=True
        ... )
    """

    def __init__(
        self,
        name: str,
        description: Optional[str] = None,
        parameters: Optional[Any] = None,
        strict: Optional[bool] = None,
    ) -> None:
        """Initialize function definition.

        Args:
            name (str):
                Name of the function
            description (Optional[str]):
                Description of what the function does
            parameters (Optional[Any]):
                JSON schema for function parameters
            strict (Optional[bool]):
                Whether to use strict schema validation
        """

    @property
    def name(self) -> str:
        """The function name."""

    @property
    def description(self) -> Optional[str]:
        """The function description."""

    @property
    def strict(self) -> Optional[bool]:
        """Whether strict schema validation is enabled."""

class FunctionTool:
    """Function tool for OpenAI chat completions.

    This class wraps a function definition to create a callable tool for
    the model.

    Examples:
        >>> func = FunctionDefinition(name="get_weather")
        >>> tool = FunctionTool(function=func, type="function")
        >>> tool.type
        'function'
    """

    def __init__(
        self,
        function: FunctionDefinition,
        type: str,
    ) -> None:
        """Initialize function tool.

        Args:
            function (FunctionDefinition):
                The function definition
            type (str):
                Tool type (typically "function")
        """

    @property
    def function(self) -> FunctionDefinition:
        """The function definition."""

    @property
    def type(self) -> str:
        """The tool type."""

class TextFormat:
    """Text format for custom tool outputs.

    This class defines unconstrained free-form text output format for
    custom tools.

    Examples:
        >>> format = TextFormat(type="text")
        >>> format.type
        'text'
    """

    def __init__(self, type: str) -> None:
        """Initialize text format.

        Args:
            type (str):
                Format type (typically "text")
        """

    @property
    def type(self) -> str:
        """The format type."""

class Grammar:
    """Grammar definition for structured custom tool outputs.

    This class defines a grammar that constrains custom tool outputs to
    follow specific syntax rules.

    Examples:
        >>> grammar = Grammar(
        ...     definition="number: /[0-9]+/",
        ...     syntax="lark"
        ... )
        >>> grammar.syntax
        'lark'
    """

    def __init__(
        self,
        definition: str,
        syntax: str,
    ) -> None:
        """Initialize grammar definition.

        Args:
            definition (str):
                The grammar definition
            syntax (str):
                Grammar syntax type ("lark" or "regex")
        """

    @property
    def definition(self) -> str:
        """The grammar definition."""

    @property
    def syntax(self) -> str:
        """The grammar syntax type."""

class GrammarFormat:
    """Grammar-based format for custom tool outputs.

    This class wraps a grammar definition to create a structured output
    format for custom tools.

    Examples:
        >>> grammar = Grammar(definition="...", syntax="lark")
        >>> format = GrammarFormat(grammar=grammar, type="grammar")
    """

    def __init__(
        self,
        grammar: Grammar,
        type: str,
    ) -> None:
        """Initialize grammar format.

        Args:
            grammar (Grammar):
                The grammar definition
            type (str):
                Format type (typically "grammar")
        """

    @property
    def grammar(self) -> Grammar:
        """The grammar definition."""

    @property
    def type(self) -> str:
        """The format type."""

class CustomToolFormat:
    """Format specification for custom tool outputs.

    This class supports either free-form text or grammar-constrained output
    formats for custom tools.

    Examples:
        >>> # Text format
        >>> format = CustomToolFormat(type="text")
        >>>
        >>> # Grammar format
        >>> grammar = Grammar(definition="...", syntax="lark")
        >>> format = CustomToolFormat(grammar=grammar)
    """

    def __init__(
        self,
        type: Optional[str] = None,
        grammar: Optional[Grammar] = None,
    ) -> None:
        """Initialize custom tool format.

        Args:
            type (Optional[str]):
                Format type for text output
            grammar (Optional[Grammar]):
                Grammar definition for structured output
        """

class CustomDefinition:
    """Definition of a custom tool for OpenAI chat completions.

    This class defines a custom tool with optional format constraints.

    Examples:
        >>> # Simple custom tool
        >>> custom = CustomDefinition(
        ...     name="analyzer",
        ...     description="Analyze data"
        ... )
        >>>
        >>> # With format constraints
        >>> format = CustomToolFormat(type="text")
        >>> custom = CustomDefinition(
        ...     name="analyzer",
        ...     description="Analyze data",
        ...     format=format
        ... )
    """

    def __init__(
        self,
        name: str,
        description: Optional[str] = None,
        format: Optional[CustomToolFormat] = None,
    ) -> None:
        """Initialize custom tool definition.

        Args:
            name (str):
                Name of the custom tool
            description (Optional[str]):
                Description of what the tool does
            format (Optional[CustomToolFormat]):
                Output format constraints
        """

    @property
    def name(self) -> str:
        """The tool name."""

    @property
    def description(self) -> Optional[str]:
        """The tool description."""

    @property
    def format(self) -> Optional[CustomToolFormat]:
        """The output format constraints."""

class CustomTool:
    """Custom tool for OpenAI chat completions.

    This class wraps a custom tool definition to create a callable tool
    for the model.

    Examples:
        >>> custom = CustomDefinition(name="analyzer")
        >>> tool = CustomTool(custom=custom, type="custom")
        >>> tool.type
        'custom'
    """

    def __init__(
        self,
        custom: CustomDefinition,
        type: str,
    ) -> None:
        """Initialize custom tool.

        Args:
            custom (CustomDefinition):
                The custom tool definition
            type (str):
                Tool type (typically "custom")
        """

    @property
    def custom(self) -> CustomDefinition:
        """The custom tool definition."""

    @property
    def type(self) -> str:
        """The tool type."""

class OpenAITool:
    """Tool for OpenAI chat completions.

    This class represents either a function tool or custom tool that can
    be called by the model.

    Examples:
        >>> # Function tool
        >>> func = FunctionDefinition(name="get_weather")
        >>> func_tool = FunctionTool(function=func, type="function")
        >>> tool = Tool(function=func_tool)
        >>>
        >>> # Custom tool
        >>> custom = CustomDefinition(name="analyzer")
        >>> custom_tool = CustomTool(custom=custom, type="custom")
        >>> tool = Tool(custom=custom_tool)
    """

    def __init__(
        self,
        function: Optional[FunctionTool] = None,
        custom: Optional[CustomTool] = None,
    ) -> None:
        """Initialize tool.

        Args:
            function (Optional[FunctionTool]):
                Function tool definition
            custom (Optional[CustomTool]):
                Custom tool definition

        Raises:
            TypeError: If both or neither tool types are provided
        """

class OpenAIChatSettings:
    """Settings for OpenAI chat completion requests.

    This class provides comprehensive configuration options for OpenAI chat
    completions, including sampling parameters, tool usage, audio output,
    caching, and more.

    Examples:
        >>> # Basic settings
        >>> settings = OpenAIChatSettings(
        ...     max_completion_tokens=1000,
        ...     temperature=0.7,
        ...     top_p=0.9
        ... )
        >>>
        >>> # With tools
        >>> func = FunctionDefinition(name="get_weather")
        >>> tool = Tool(function=FunctionTool(function=func, type="function"))
        >>> settings = OpenAIChatSettings(
        ...     tools=[tool],
        ...     tool_choice=ToolChoice.from_mode(ToolChoiceMode.Auto)
        ... )
        >>>
        >>> # With audio output
        >>> audio = AudioParam(format="mp3", voice="alloy")
        >>> settings = OpenAIChatSettings(
        ...     audio=audio,
        ...     modalities=["text", "audio"]
        ... )
    """

    def __init__(
        self,
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
        tool_choice: Optional[ToolChoice] = None,
        tools: Optional[List[OpenAITool]] = None,
        top_logprobs: Optional[int] = None,
        verbosity: Optional[str] = None,
        extra_body: Optional[Any] = None,
    ) -> None:
        """Initialize OpenAI chat settings.

        Args:
            max_completion_tokens (Optional[int]):
                Maximum tokens for completion (including reasoning tokens)
            temperature (Optional[float]):
                Sampling temperature (0.0 to 2.0)
            top_p (Optional[float]):
                Nucleus sampling parameter (0.0 to 1.0)
            top_k (Optional[int]):
                Top-k sampling parameter
            frequency_penalty (Optional[float]):
                Frequency penalty (-2.0 to 2.0)
            timeout (Optional[float]):
                Request timeout in seconds
            parallel_tool_calls (Optional[bool]):
                Enable parallel function calling
            seed (Optional[int]):
                Random seed for deterministic sampling
            logit_bias (Optional[Dict[str, int]]):
                Token bias map (-100 to 100)
            stop_sequences (Optional[List[str]]):
                Stop sequences (max 4)
            logprobs (Optional[bool]):
                Return log probabilities
            audio (Optional[AudioParam]):
                Audio output configuration
            metadata (Optional[Dict[str, str]]):
                Request metadata (max 16 key-value pairs)
            modalities (Optional[List[str]]):
                Output modalities (e.g., ["text", "audio"])
            n (Optional[int]):
                Number of completions to generate
            prediction (Optional[Prediction]):
                Predicted output configuration
            presence_penalty (Optional[float]):
                Presence penalty (-2.0 to 2.0)
            prompt_cache_key (Optional[str]):
                Cache key for prompt caching
            reasoning_effort (Optional[str]):
                Reasoning effort level (e.g., "low", "medium", "high")
            safety_identifier (Optional[str]):
                User identifier for safety checks
            service_tier (Optional[str]):
                Service tier ("auto", "default", "flex", "priority")
            store (Optional[bool]):
                Store completion for later retrieval
            stream (Optional[bool]):
                Stream response with SSE
            stream_options (Optional[StreamOptions]):
                Streaming configuration
            tool_choice (Optional[ToolChoice]):
                Tool choice configuration
            tools (Optional[List[Tool]]):
                Available tools
            top_logprobs (Optional[int]):
                Number of top log probs to return (0-20)
            verbosity (Optional[str]):
                Response verbosity ("low", "medium", "high")
            extra_body (Optional[Any]):
                Additional request parameters
        """

    @property
    def max_completion_tokens(self) -> Optional[int]:
        """Maximum completion tokens."""

    @property
    def temperature(self) -> Optional[float]:
        """Sampling temperature."""

    @property
    def top_p(self) -> Optional[float]:
        """Nucleus sampling parameter."""

    @property
    def top_k(self) -> Optional[int]:
        """Top-k sampling parameter."""

    @property
    def frequency_penalty(self) -> Optional[float]:
        """Frequency penalty."""

    @property
    def timeout(self) -> Optional[float]:
        """Request timeout."""

    @property
    def parallel_tool_calls(self) -> Optional[bool]:
        """Whether parallel tool calls are enabled."""

    @property
    def seed(self) -> Optional[int]:
        """Random seed."""

    @property
    def logit_bias(self) -> Optional[Dict[str, int]]:
        """Token bias map."""

    @property
    def stop_sequences(self) -> Optional[List[str]]:
        """Stop sequences."""

    @property
    def logprobs(self) -> Optional[bool]:
        """Whether to return log probabilities."""

    @property
    def audio(self) -> Optional[AudioParam]:
        """Audio output configuration."""

    @property
    def metadata(self) -> Optional[Dict[str, str]]:
        """Request metadata."""

    @property
    def modalities(self) -> Optional[List[str]]:
        """Output modalities."""

    @property
    def n(self) -> Optional[int]:
        """Number of completions."""

    @property
    def prediction(self) -> Optional[Prediction]:
        """Predicted output configuration."""

    @property
    def presence_penalty(self) -> Optional[float]:
        """Presence penalty."""

    @property
    def prompt_cache_key(self) -> Optional[str]:
        """Prompt cache key."""

    @property
    def reasoning_effort(self) -> Optional[str]:
        """Reasoning effort level."""

    @property
    def safety_identifier(self) -> Optional[str]:
        """Safety identifier."""

    @property
    def service_tier(self) -> Optional[str]:
        """Service tier."""

    @property
    def store(self) -> Optional[bool]:
        """Whether to store completion."""

    @property
    def stream(self) -> Optional[bool]:
        """Whether to stream response."""

    @property
    def stream_options(self) -> Optional[StreamOptions]:
        """Stream options."""

    @property
    def tool_choice(self) -> Optional[ToolChoice]:
        """Tool choice configuration."""

    @property
    def tools(self) -> Optional[List[OpenAITool]]:
        """Available tools."""

    @property
    def top_logprobs(self) -> Optional[int]:
        """Number of top log probabilities."""

    @property
    def verbosity(self) -> Optional[str]:
        """Response verbosity."""

    @property
    def extra_body(self) -> Optional[Dict[str, Any]]:
        """Additional request parameters."""

    def model_dump(self) -> Dict[str, Any]:
        """Convert settings to a dictionary.

        Returns:
            Dict[str, Any]: Dictionary representation of settings
        """

    def settings_type(self) -> str:
        """Get the settings type identifier.

        Returns:
            str: Settings type ("OpenAIChat")
        """

    def __str__(self) -> str:
        """String representation of settings.

        Returns:
            str: String representation
        """

# ============================================================================
# Request Types
# ============================================================================

class File:
    """File reference for OpenAI chat completion messages.

    This class represents a file that can be included in a message, either
    by providing file data directly or referencing a file by ID.

    Examples:
        >>> # File by ID
        >>> file = File(file_id="file-abc123", filename="document.pdf")
        >>>
        >>> # File with data
        >>> file = File(
        ...     file_data="base64_encoded_data",
        ...     filename="document.pdf"
        ... )
    """

    def __init__(
        self,
        file_data: Optional[str] = None,
        file_id: Optional[str] = None,
        filename: Optional[str] = None,
    ) -> None:
        """Initialize file reference.

        Args:
            file_data (Optional[str]):
                Base64 encoded file data
            file_id (Optional[str]):
                OpenAI file ID
            filename (Optional[str]):
                File name
        """

    @property
    def file_data(self) -> Optional[str]:
        """The base64 encoded file data."""

    @property
    def file_id(self) -> Optional[str]:
        """The OpenAI file ID."""

    @property
    def filename(self) -> Optional[str]:
        """The file name."""

class FileContentPart:
    """File content part for OpenAI chat messages.

    This class represents a file as part of a message's content.

    Examples:
        >>> file_part = FileContentPart(
        ...     file_id="file-abc123",
        ...     filename="document.pdf"
        ... )
        >>> file_part.type
        'file'
    """

    def __init__(
        self,
        file_data: Optional[str] = None,
        file_id: Optional[str] = None,
        filename: Optional[str] = None,
    ) -> None:
        """Initialize file content part.

        Args:
            file_data (Optional[str]):
                Base64 encoded file data
            file_id (Optional[str]):
                OpenAI file ID
            filename (Optional[str]):
                File name
        """

    @property
    def file(self) -> File:
        """The file reference."""

    @property
    def type(self) -> str:
        """The content part type (always 'file')."""

class InputAudioData:
    """Audio data for input in OpenAI chat messages.

    This class represents audio input data with format specification.

    Examples:
        >>> audio_data = InputAudioData(
        ...     data="base64_encoded_audio",
        ...     format="wav"
        ... )
        >>> audio_data.format
        'wav'
    """

    def __init__(
        self,
        data: str,
        format: str,
    ) -> None:
        """Initialize input audio data.

        Args:
            data (str):
                Base64 encoded audio data
            format (str):
                Audio format (e.g., "wav", "mp3")
        """

    @property
    def data(self) -> str:
        """The base64 encoded audio data."""

    @property
    def format(self) -> str:
        """The audio format."""

class InputAudioContentPart:
    """Audio content part for OpenAI chat messages.

    This class represents audio input as part of a message's content.

    Examples:
        >>> audio_part = InputAudioContentPart(
        ...     data="base64_encoded_audio",
        ...     format="wav"
        ... )
        >>> audio_part.type
        'input_audio'
    """

    def __init__(
        self,
        data: str,
        format: str,
    ) -> None:
        """Initialize audio content part.

        Args:
            data (str):
                Base64 encoded audio data
            format (str):
                Audio format (e.g., "wav", "mp3")
        """

    @property
    def input_audio(self) -> InputAudioData:
        """The audio data."""

    @property
    def type(self) -> str:
        """The content part type (always 'input_audio')."""

class ImageUrl:
    """Image URL reference for OpenAI chat messages.

    This class represents an image by URL with optional detail level.

    Examples:
        >>> # Standard detail
        >>> image = ImageUrl(url="https://example.com/image.jpg")
        >>>
        >>> # High detail
        >>> image = ImageUrl(
        ...     url="https://example.com/image.jpg",
        ...     detail="high"
        ... )
    """

    def __init__(
        self,
        url: str,
        detail: Optional[str] = None,
    ) -> None:
        """Initialize image URL.

        Args:
            url (str):
                Image URL (can be HTTP URL or data URL)
            detail (Optional[str]):
                Detail level ("low", "high", or "auto")
        """

    @property
    def url(self) -> str:
        """The image URL."""

    @property
    def detail(self) -> Optional[str]:
        """The detail level."""

class ImageContentPart:
    """Image content part for OpenAI chat messages.

    This class represents an image as part of a message's content.

    Examples:
        >>> image_part = ImageContentPart(
        ...     url="https://example.com/image.jpg",
        ...     detail="high"
        ... )
        >>> image_part.type
        'image_url'
    """

    def __init__(
        self,
        url: str,
        detail: Optional[str] = None,
    ) -> None:
        """Initialize image content part.

        Args:
            url (str):
                Image URL (can be HTTP URL or data URL)
            detail (Optional[str]):
                Detail level ("low", "high", or "auto")
        """

    @property
    def image_url(self) -> ImageUrl:
        """The image URL reference."""

    @property
    def type(self) -> str:
        """The content part type (always 'image_url')."""

class TextContentPart:
    """Text content part for OpenAI chat messages.

    This class represents text as part of a message's content.

    Examples:
        >>> text_part = TextContentPart(text="Hello, world!")
        >>> text_part.text
        'Hello, world!'
        >>> text_part.type
        'text'
    """

    def __init__(self, text: str) -> None:
        """Initialize text content part.

        Args:
            text (str):
                Text content
        """

    @property
    def text(self) -> str:
        """The text content."""

    @property
    def type(self) -> str:
        """The content part type (always 'text')."""

class ChatMessage:
    """Message for OpenAI chat completions.

    This class represents a single message in a chat completion conversation,
    supporting multiple content types including text, images, audio, and files.

    Examples:
        >>> # Simple text message
        >>> msg = ChatMessage(role="user", content="Hello!")
        >>>
        >>> # Message with image
        >>> image = ImageContentPart(url="https://example.com/image.jpg")
        >>> msg = ChatMessage(role="user", content=[image])
        >>>
        >>> # Mixed content message
        >>> msg = ChatMessage(
        ...     role="user",
        ...     content=["Describe this image:", image]
        ... )
        >>>
        >>> # System message with name
        >>> msg = ChatMessage(
        ...     role="system",
        ...     content="You are a helpful assistant.",
        ...     name="assistant_v1"
        ... )
    """

    def __init__(
        self,
        role: str,
        content: Union[
            str,
            List[
                Union[
                    str,
                    TextContentPart,
                    ImageContentPart,
                    InputAudioContentPart,
                    FileContentPart,
                ]
            ],
        ],
        name: Optional[str] = None,
    ) -> None:
        """Initialize chat message.

        Args:
            role (str):
                Message role ("system", "user", "assistant", "tool", "developer")
            content (Union[str, List[...]]):
                Message content - can be:
                - String: Simple text message
                - List: Mixed content with strings and content parts
                - ContentPart: Single structured content part
            name (Optional[str]):
                Optional name for the message

        Raises:
            TypeError: If content format is invalid
        """

    @property
    def role(self) -> str:
        """The message role."""

    @property
    def content(
        self,
    ) -> List[
        Union[TextContentPart, ImageContentPart, InputAudioContentPart, FileContentPart]
    ]:
        """The message content parts."""

    @property
    def name(self) -> Optional[str]:
        """The message name."""

# ============================================================================
# Response Types
# ============================================================================

class Function:
    """Function call information from OpenAI tool calls.

    This class represents a function call made by the model, including
    the function name and JSON-formatted arguments.

    Examples:
        >>> func = Function(
        ...     name="get_weather",
        ...     arguments='{"location": "San Francisco"}'
        ... )
        >>> func.name
        'get_weather'
    """

    @property
    def arguments(self) -> str:
        """The JSON-formatted function arguments."""

    @property
    def name(self) -> str:
        """The function name."""

class ToolCall:
    """Tool call information from OpenAI responses.

    This class represents a single tool call made by the model during
    generation.

    Examples:
        >>> # Accessing tool call from response
        >>> choice = response.choices[0]
        >>> if choice.message.tool_calls:
        ...     tool_call = choice.message.tool_calls[0]
        ...     print(tool_call.function.name)
        ...     print(tool_call.function.arguments)
    """

    @property
    def id(self) -> str:
        """The tool call ID."""

    @property
    def type(self) -> str:
        """The tool call type."""

    @property
    def function(self) -> Function:
        """The function call information."""

class UrlCitation:
    """URL citation from OpenAI web search.

    This class represents a citation to a web source used by the model
    when web search is enabled.

    Examples:
        >>> # Accessing citations from response
        >>> choice = response.choices[0]
        >>> for annotation in choice.message.annotations:
        ...     for citation in annotation.url_citations:
        ...         print(f"{citation.title}: {citation.url}")
    """

    @property
    def end_index(self) -> int:
        """The end index in the message content."""

    @property
    def start_index(self) -> int:
        """The start index in the message content."""

    @property
    def title(self) -> str:
        """The page title."""

    @property
    def url(self) -> str:
        """The URL."""

class Annotations:
    """Annotations attached to OpenAI message content.

    This class contains metadata and citations for message content,
    such as URL citations from web search.

    Examples:
        >>> # Checking for citations
        >>> choice = response.choices[0]
        >>> for annotation in choice.message.annotations:
        ...     print(f"Type: {annotation.type}")
        ...     for citation in annotation.url_citations:
        ...         print(f"  {citation.title}")
    """

    @property
    def type(self) -> str:
        """The annotation type."""

    @property
    def url_citations(self) -> List[UrlCitation]:
        """URL citations."""

class Audio:
    """Audio output from OpenAI chat completions.

    This class contains audio data generated by the model when audio
    output is requested.

    Examples:
        >>> # Accessing audio from response
        >>> choice = response.choices[0]
        >>> if choice.message.audio:
        ...     audio = choice.message.audio
        ...     print(f"Audio ID: {audio.id}")
        ...     print(f"Transcript: {audio.transcript}")
        ...     # audio.data contains base64 encoded audio
    """

    @property
    def data(self) -> str:
        """Base64 encoded audio data."""

    @property
    def expires_at(self) -> int:
        """Unix timestamp when audio expires."""

    @property
    def id(self) -> str:
        """Audio ID."""

    @property
    def transcript(self) -> str:
        """Audio transcript."""

class ChatCompletionMessage:
    """Message from OpenAI chat completion response.

    This class represents the model's response message, including text
    content, tool calls, audio, and annotations.

    Examples:
        >>> # Accessing message from response
        >>> choice = response.choices[0]
        >>> message = choice.message
        >>> print(f"Role: {message.role}")
        >>> print(f"Content: {message.content}")
        >>>
        >>> # Checking for tool calls
        >>> if message.tool_calls:
        ...     for call in message.tool_calls:
        ...         print(f"Function: {call.function.name}")
    """

    @property
    def content(self) -> Optional[str]:
        """The message content."""

    @property
    def refusal(self) -> Optional[str]:
        """Refusal reason if model refused request."""

    @property
    def role(self) -> str:
        """The message role."""

    @property
    def annotations(self) -> List[Annotations]:
        """Message annotations."""

    @property
    def tool_calls(self) -> List[ToolCall]:
        """Tool calls made by the model."""

    @property
    def audio(self) -> Optional[Audio]:
        """Audio output if requested."""

class TopLogProbs:
    """Top log probability information for a token.

    This class represents one of the top alternative tokens considered
    by the model, with its log probability.

    Examples:
        >>> # Accessing top log probs
        >>> choice = response.choices[0]
        >>> if choice.logprobs and choice.logprobs.content:
        ...     for log_content in choice.logprobs.content:
        ...         if log_content.top_logprobs:
        ...             for top in log_content.top_logprobs:
        ...                 print(f"{top.token}: {top.logprob}")
    """

    @property
    def bytes(self) -> Optional[List[int]]:
        """UTF-8 bytes of the token."""

    @property
    def logprob(self) -> float:
        """Log probability of the token."""

    @property
    def token(self) -> str:
        """The token."""

class LogContent:
    """Log probability content for a single token.

    This class contains detailed probability information for a token
    generated by the model.

    Examples:
        >>> # Analyzing token probabilities
        >>> choice = response.choices[0]
        >>> if choice.logprobs and choice.logprobs.content:
        ...     for log_content in choice.logprobs.content:
        ...         print(f"Token: {log_content.token}")
        ...         print(f"Log prob: {log_content.logprob}")
    """

    @property
    def bytes(self) -> Optional[List[int]]:
        """UTF-8 bytes of the token."""

    @property
    def logprob(self) -> float:
        """Log probability of the token."""

    @property
    def token(self) -> str:
        """The token."""

    @property
    def top_logprobs(self) -> Optional[List[TopLogProbs]]:
        """Top alternative tokens."""

class LogProbs:
    """Log probability information for OpenAI responses.

    This class contains log probability data for both generated content
    and refusals.

    Examples:
        >>> # Checking log probabilities
        >>> choice = response.choices[0]
        >>> if choice.logprobs:
        ...     if choice.logprobs.content:
        ...         print(f"Content tokens: {len(choice.logprobs.content)}")
        ...     if choice.logprobs.refusal:
        ...         print("Refusal log probs available")
    """

    @property
    def content(self) -> Optional[List[LogContent]]:
        """Log probabilities for content tokens."""

    @property
    def refusal(self) -> Optional[List[LogContent]]:
        """Log probabilities for refusal tokens."""

class Choice:
    """Choice from OpenAI chat completion response.

    This class represents one possible completion from the model, including
    the message, finish reason, and optional log probabilities.

    Examples:
        >>> # Accessing choice from response
        >>> choice = response.choices[0]
        >>> print(f"Message: {choice.message.content}")
        >>> print(f"Finish reason: {choice.finish_reason}")
        >>>
        >>> # Multiple choices (when n > 1)
        >>> for i, choice in enumerate(response.choices):
        ...     print(f"Choice {i}: {choice.message.content}")
    """

    @property
    def message(self) -> ChatCompletionMessage:
        """The completion message."""

    @property
    def finish_reason(self) -> str:
        """Reason for completion finishing."""

    @property
    def logprobs(self) -> Optional[LogProbs]:
        """Log probability information."""

class CompletionTokenDetails:
    """Detailed token usage for completion output.

    This class provides granular information about tokens used in the
    completion, including reasoning tokens and audio tokens.

    Examples:
        >>> # Accessing token details
        >>> usage = response.usage
        >>> details = usage.completion_tokens_details
        >>> print(f"Reasoning tokens: {details.reasoning_tokens}")
        >>> print(f"Audio tokens: {details.audio_tokens}")
    """

    @property
    def accepted_prediction_tokens(self) -> int:
        """Number of accepted prediction tokens."""

    @property
    def audio_tokens(self) -> int:
        """Number of audio tokens."""

    @property
    def reasoning_tokens(self) -> int:
        """Number of reasoning tokens."""

    @property
    def rejected_prediction_tokens(self) -> int:
        """Number of rejected prediction tokens."""

class PromptTokenDetails:
    """Detailed token usage for input prompt.

    This class provides information about tokens used in the prompt,
    including cached tokens and audio tokens.

    Examples:
        >>> # Accessing prompt token details
        >>> usage = response.usage
        >>> details = usage.prompt_tokens_details
        >>> print(f"Cached tokens: {details.cached_tokens}")
        >>> print(f"Audio tokens: {details.audio_tokens}")
    """

    @property
    def audio_tokens(self) -> int:
        """Number of audio tokens."""

    @property
    def cached_tokens(self) -> int:
        """Number of cached tokens."""

class Usage:
    """Token usage statistics for OpenAI chat completions.

    This class provides comprehensive token usage information, including
    detailed breakdowns for both prompt and completion tokens.

    Examples:
        >>> # Accessing usage information
        >>> usage = response.usage
        >>> print(f"Total tokens: {usage.total_tokens}")
        >>> print(f"Prompt tokens: {usage.prompt_tokens}")
        >>> print(f"Completion tokens: {usage.completion_tokens}")
        >>>
        >>> # Detailed breakdown
        >>> print(f"Cached tokens: {usage.prompt_tokens_details.cached_tokens}")
        >>> print(f"Reasoning tokens: {usage.completion_tokens_details.reasoning_tokens}")
    """

    @property
    def completion_tokens(self) -> int:
        """Total completion tokens."""

    @property
    def prompt_tokens(self) -> int:
        """Total prompt tokens."""

    @property
    def total_tokens(self) -> int:
        """Total tokens (prompt + completion)."""

    @property
    def completion_tokens_details(self) -> CompletionTokenDetails:
        """Detailed completion token breakdown."""

    @property
    def prompt_tokens_details(self) -> PromptTokenDetails:
        """Detailed prompt token breakdown."""

    @property
    def finish_reason(self) -> Optional[str]:
        """Finish reason if applicable."""

class OpenAIChatResponse:
    """Response from OpenAI chat completion API.

    This class represents a complete response from the chat completion API,
    including all choices, usage statistics, and metadata.

    Examples:
        >>> # Basic usage
        >>> response = OpenAIChatResponse(...)
        >>> print(response.choices[0].message.content)
        >>>
        >>> # Accessing metadata
        >>> print(f"Model: {response.model}")
        >>> print(f"ID: {response.id}")
        >>> print(f"Created: {response.created}")
        >>>
        >>> # Usage statistics
        >>> print(f"Total tokens: {response.usage.total_tokens}")
    """

    @property
    def id(self) -> str:
        """Unique completion ID."""

    @property
    def object(self) -> str:
        """Object type (always 'chat.completion')."""

    @property
    def created(self) -> int:
        """Unix timestamp of creation."""

    @property
    def model(self) -> str:
        """Model used for completion."""

    @property
    def choices(self) -> List[Choice]:
        """List of completion choices."""

    @property
    def usage(self) -> Usage:
        """Token usage statistics."""

    @property
    def service_tier(self) -> Optional[str]:
        """Service tier used."""

    @property
    def system_fingerprint(self) -> Optional[str]:
        """System fingerprint for backend configuration."""

    def __str__(self) -> str:
        """String representation of response."""

# ============================================================================
# Embedding Types
# ============================================================================

class OpenAIEmbeddingConfig:
    """Configuration for OpenAI embedding requests.

    This class provides settings for embedding generation, including
    model selection, dimensions, and encoding format.

    Examples:
        >>> # Standard configuration
        >>> config = OpenAIEmbeddingConfig(
        ...     model="text-embedding-3-small"
        ... )
        >>>
        >>> # Custom dimensions
        >>> config = OpenAIEmbeddingConfig(
        ...     model="text-embedding-3-large",
        ...     dimensions=512
        ... )
    """

    def __init__(
        self,
        model: str,
        dimensions: Optional[int] = None,
        encoding_format: Optional[str] = None,
        user: Optional[str] = None,
    ) -> None:
        """Initialize embedding configuration.

        Args:
            model (str):
                Model ID for embeddings
            dimensions (Optional[int]):
                Number of dimensions for output embeddings
            encoding_format (Optional[str]):
                Format for embeddings ("float" or "base64")
            user (Optional[str]):
                User identifier for tracking
        """

    @property
    def model(self) -> str:
        """The embedding model ID."""

    @property
    def dimensions(self) -> Optional[int]:
        """Number of embedding dimensions."""

    @property
    def encoding_format(self) -> Optional[str]:
        """Encoding format for embeddings."""

    @property
    def user(self) -> Optional[str]:
        """User identifier."""

class EmbeddingObject:
    """Single embedding from OpenAI embedding response.

    This class represents one embedding vector from the response.

    Examples:
        >>> # Accessing embeddings
        >>> for embedding in response.data:
        ...     print(f"Index: {embedding.index}")
        ...     print(f"Dimensions: {len(embedding.embedding)}")
    """

    @property
    def embedding(self) -> List[float]:
        """The embedding vector."""

    @property
    def index(self) -> int:
        """Index in the input list."""

    @property
    def object(self) -> str:
        """Object type (always 'embedding')."""

class UsageObject:
    """Token usage for embedding request.

    This class provides token usage statistics for embedding requests.

    Examples:
        >>> usage = response.usage
        >>> print(f"Prompt tokens: {usage.prompt_tokens}")
        >>> print(f"Total tokens: {usage.total_tokens}")
    """

    @property
    def prompt_tokens(self) -> int:
        """Tokens in input prompts."""

    @property
    def total_tokens(self) -> int:
        """Total tokens processed."""

class OpenAIEmbeddingResponse:
    """Response from OpenAI embedding API.

    This class represents a complete response from the embedding API,
    including all generated embeddings and usage statistics.

    Examples:
        >>> # Accessing embeddings
        >>> response = OpenAIEmbeddingResponse(...)
        >>> for embedding in response.data:
        ...     vector = embedding.embedding
        ...     # Use embedding vector
        >>>
        >>> # Usage information
        >>> print(f"Tokens used: {response.usage.total_tokens}")
    """

    @property
    def data(self) -> List[EmbeddingObject]:
        """List of embedding objects."""

    @property
    def model(self) -> str:
        """Model used for embeddings."""

    @property
    def object(self) -> str:
        """Object type (always 'list')."""

    @property
    def usage(self) -> UsageObject:
        """Token usage statistics."""

    def __str__(self) -> str:
        """String representation of response."""

###### __potatohead__.google module ######'

class SchemaType:
    """Schema type definitions for Google/Gemini API.

    Defines the available data types that can be used in schema definitions
    for structured outputs and function parameters.

    Examples:
        >>> schema_type = SchemaType.String
        >>> schema_type.value
        'STRING'
    """

    TypeUnspecified = "SchemaType"
    """Unspecified type"""

    String = "SchemaType"
    """String data type"""

    Number = "SchemaType"
    """Numeric data type (floating point)"""

    Integer = "SchemaType"
    """Integer data type"""

    Boolean = "SchemaType"
    """Boolean data type"""

    Array = "SchemaType"
    """Array/list data type"""

    Object = "SchemaType"
    """Object/dictionary data type"""

    Null = "SchemaType"
    """Null data type"""

class HarmCategory:
    """Harm categories for safety filtering in Google/Gemini API.

    Defines categories of potentially harmful content that can be detected
    and filtered by the model's safety systems.

    Examples:
        >>> category = HarmCategory.HarmCategoryHateSpeech
        >>> category.value
        'HARM_CATEGORY_HATE_SPEECH'
    """

    HarmCategoryUnspecified = "HarmCategory"
    """Unspecified harm category"""

    HarmCategoryDerogatory = "HarmCategory"
    """Derogatory content"""

    HarmCategoryToxicity = "HarmCategory"
    """Toxic content"""

    HarmCategoryViolence = "HarmCategory"
    """Violent content"""

    HarmCategorySexual = "HarmCategory"
    """Sexual content"""

    HarmCategoryMedical = "HarmCategory"
    """Medical misinformation"""

    HarmCategoryDangerous = "HarmCategory"
    """Dangerous content"""

    HarmCategoryHarassment = "HarmCategory"
    """Harassment content"""

    HarmCategoryHateSpeech = "HarmCategory"
    """Hate speech content"""

    HarmCategorySexuallyExplicit = "HarmCategory"
    """Sexually explicit content"""

    HarmCategoryDangerousContent = "HarmCategory"
    """Dangerous content (alternative)"""

class HarmBlockThreshold:
    """Thresholds for blocking harmful content.

    Defines sensitivity levels for blocking content based on harm probability.

    Examples:
        >>> threshold = HarmBlockThreshold.BlockMediumAndAbove
        >>> threshold.value
        'BLOCK_MEDIUM_AND_ABOVE'
    """

    HarmBlockThresholdUnspecified = "HarmBlockThreshold"
    """Unspecified threshold"""

    BlockLowAndAbove = "HarmBlockThreshold"
    """Block content with low or higher harm probability"""

    BlockMediumAndAbove = "HarmBlockThreshold"
    """Block content with medium or higher harm probability"""

    BlockOnlyHigh = "HarmBlockThreshold"
    """Block only high harm probability content"""

    BlockNone = "HarmBlockThreshold"
    """Do not block any content"""

    Off = "HarmBlockThreshold"
    """Turn off safety filtering entirely"""

class HarmBlockMethod:
    """Method for blocking harmful content.

    Specifies whether blocking decisions use probability or severity scores.

    Examples:
        >>> method = HarmBlockMethod.Probability
        >>> method.value
        'PROBABILITY'
    """

    HarmBlockMethodUnspecified = "HarmBlockMethod"
    """Unspecified blocking method"""

    Severity = "HarmBlockMethod"
    """Use severity scores for blocking decisions"""

    Probability = "HarmBlockMethod"
    """Use probability scores for blocking decisions"""

class Modality:
    """Content modality types supported by the model.

    Defines the types of content (text, image, audio, etc.) that can be
    included in requests and responses.

    Examples:
        >>> modality = Modality.Text
        >>> modality.value
        'TEXT'
    """

    ModalityUnspecified = "Modality"
    """Unspecified modality"""

    Text = "Modality"
    """Text content"""

    Image = "Modality"
    """Image content"""

    Audio = "Modality"
    """Audio content"""

    Video = "Modality"
    """Video content"""

    Document = "Modality"
    """Document content"""

class MediaResolution:
    """Media resolution levels for input processing.

    Controls the token resolution at which media content is sampled,
    affecting quality and token usage.

    Examples:
        >>> resolution = MediaResolution.MediaResolutionHigh
        >>> resolution.value
        'MEDIA_RESOLUTION_HIGH'
    """

    MediaResolutionUnspecified = "MediaResolution"
    """Unspecified resolution"""

    MediaResolutionLow = "MediaResolution"
    """Low resolution (64 tokens)"""

    MediaResolutionMedium = "MediaResolution"
    """Medium resolution (256 tokens)"""

    MediaResolutionHigh = "MediaResolution"
    """High resolution with zoomed reframing (256 tokens)"""

class ModelRoutingPreference:
    """Preference for automatic model routing.

    Controls how models are selected when using automatic routing,
    balancing quality, cost, and performance.

    Examples:
        >>> preference = ModelRoutingPreference.Balanced
        >>> preference.value
        'BALANCED'
    """

    Unknown = "ModelRoutingPreference"
    """Unknown preference"""

    PrioritizeQuality = "ModelRoutingPreference"
    """Prioritize response quality"""

    Balanced = "ModelRoutingPreference"
    """Balance quality and cost"""

    PrioritizeCost = "ModelRoutingPreference"
    """Prioritize lower cost"""

class ThinkingLevel:
    """Level of model thinking/reasoning to apply.

    Controls the depth of reasoning the model performs before generating
    its final response.

    Examples:
        >>> level = ThinkingLevel.High
        >>> level.value
        'HIGH'
    """

    ThinkingLevelUnspecified = "ThinkingLevel"
    """Unspecified thinking level"""

    Low = "ThinkingLevel"
    """Low level of thinking"""

    High = "ThinkingLevel"
    """High level of thinking"""

class Mode:
    """Function calling mode for tool usage.

    Controls how the model handles function/tool calls during generation.

    Examples:
        >>> mode = Mode.Auto
        >>> mode.value
        'AUTO'
    """

    ModeUnspecified = "Mode"
    """Unspecified mode"""

    Validated = "Mode"
    """Model may call functions or respond naturally, validated"""

    Any = "Mode"
    """Model must call a function"""

    Auto = "Mode"
    """Model decides whether to call functions or respond naturally"""

    None_Mode = "Mode"
    """Model will not call any functions"""

class Behavior:
    """Function execution behavior.

    Specifies whether function calls are blocking or non-blocking.

    Examples:
        >>> behavior = Behavior.Blocking
        >>> behavior.value
        'BLOCKING'
    """

    Unspecified = "Behavior"
    """Unspecified behavior"""

    Blocking = "Behavior"
    """Function execution blocks until complete"""

    NonBlocking = "Behavior"
    """Function execution does not block"""

class Language:
    """Programming language for executable code.

    Specifies the language used when the model generates executable code.

    Examples:
        >>> lang = Language.Python
        >>> lang.value
        'PYTHON'
    """

    LanguageUnspecified = "Language"
    """Unspecified language"""

    Python = "Language"
    """Python programming language"""

class Outcome:
    """Code execution outcome status.

    Indicates the result of executing generated code.

    Examples:
        >>> outcome = Outcome.OutcomeOk
        >>> outcome.value
        'OUTCOME_OK'
    """

    OutcomeUnspecified = "Outcome"
    """Unspecified outcome"""

    OutcomeOk = "Outcome"
    """Execution completed successfully"""

    OutcomeFailed = "Outcome"
    """Execution failed"""

    OutcomeDeadlineExceeded = "Outcome"
    """Execution exceeded time limit"""

class ApiSpecType:
    """API specification type for external retrieval.

    Defines the type of external API used for grounding/retrieval.

    Examples:
        >>> spec = ApiSpecType.ElasticSearch
        >>> spec.value
        'ELASTIC_SEARCH'
    """

    ApiSpecUnspecified = "ApiSpecType"
    """Unspecified API spec"""

    SimpleSearch = "ApiSpecType"
    """Simple search API"""

    ElasticSearch = "ApiSpecType"
    """Elasticsearch API"""

class AuthType:
    """Authentication type for external APIs.

    Specifies the authentication method used to access external APIs.

    Examples:
        >>> auth = AuthType.ApiKeyAuth
        >>> auth.value
        'API_KEY_AUTH'
    """

    AuthTypeUnspecified = "AuthType"
    """Unspecified auth type"""

    NoAuth = "AuthType"
    """No authentication"""

    ApiKeyAuth = "AuthType"
    """API key authentication"""

    HttpBasicAuth = "AuthType"
    """HTTP basic authentication"""

    GoogleServiceAccountAuth = "AuthType"
    """Google service account authentication"""

    Oauth = "AuthType"
    """OAuth authentication"""

    OidcAuth = "AuthType"
    """OIDC authentication"""

class HttpElementLocation:
    """Location of HTTP authentication element.

    Specifies where authentication information appears in HTTP requests.

    Examples:
        >>> location = HttpElementLocation.HttpInHeader
        >>> location.value
        'HTTP_IN_HEADER'
    """

    HttpInUnspecified = "HttpElementLocation"
    """Unspecified location"""

    HttpInQuery = "HttpElementLocation"
    """In query parameters"""

    HttpInHeader = "HttpElementLocation"
    """In HTTP headers"""

    HttpInPath = "HttpElementLocation"
    """In URL path"""

    HttpInBody = "HttpElementLocation"
    """In request body"""

    HttpInCookie = "HttpElementLocation"
    """In cookies"""

class PhishBlockThreshold:
    """Phishing/malicious URL blocking threshold.

    Controls the confidence level required to block potentially malicious URLs.

    Examples:
        >>> threshold = PhishBlockThreshold.BlockMediumAndAbove
        >>> threshold.value
        'BLOCK_MEDIUM_AND_ABOVE'
    """

    PhishBlockThresholdUnspecified = "PhishBlockThreshold"
    """Unspecified threshold"""

    BlockLowAndAbove = "PhishBlockThreshold"
    """Block low confidence and above"""

    BlockMediumAndAbove = "PhishBlockThreshold"
    """Block medium confidence and above"""

    BlockHighAndAbove = "PhishBlockThreshold"
    """Block high confidence and above"""

    BlockHigherAndAbove = "PhishBlockThreshold"
    """Block higher confidence and above"""

    BlockVeryHighAndAbove = "PhishBlockThreshold"
    """Block very high confidence and above"""

    BlockOnlyExtremelyHigh = "PhishBlockThreshold"
    """Block only extremely high confidence"""

class DynamicRetrievalMode:
    """Mode for dynamic retrieval behavior.

    Controls when the model triggers retrieval operations.

    Examples:
        >>> mode = DynamicRetrievalMode.ModeDynamic
        >>> mode.value
        'MODE_DYNAMIC'
    """

    ModeUnspecified = "DynamicRetrievalMode"
    """Unspecified mode (always trigger)"""

    ModeDynamic = "DynamicRetrievalMode"
    """Trigger retrieval only when necessary"""

class ComputerUseEnvironment:
    """Environment for computer use capabilities.

    Specifies the environment in which the model operates when using
    computer control features.

    Examples:
        >>> env = ComputerUseEnvironment.EnvironmentBrowser
        >>> env.value
        'ENVIRONMENT_BROWSER'
    """

    EnvironmentUnspecified = "ComputerUseEnvironment"
    """Unspecified environment"""

    EnvironmentBrowser = "ComputerUseEnvironment"
    """Web browser environment"""

class TrafficType:
    """Type of API traffic for billing purposes.

    Indicates whether the request uses pay-as-you-go or provisioned quota.

    Examples:
        >>> traffic = TrafficType.OnDemand
        >>> traffic.value
        'ON_DEMAND'
    """

    TrafficTypeUnspecified = "TrafficType"
    """Unspecified traffic type"""

    OnDemand = "TrafficType"
    """Pay-as-you-go quota"""

    ProvisionedThroughput = "TrafficType"
    """Provisioned throughput quota"""

class BlockedReason:
    """Reason why content was blocked.

    Indicates why a prompt or response was blocked by content filters.

    Examples:
        >>> reason = BlockedReason.Safety
        >>> reason.value
        'SAFETY'
    """

    BlockedReasonUnspecified = "BlockedReason"
    """Unspecified reason"""

    Safety = "BlockedReason"
    """Blocked for safety reasons"""

    Other = "BlockedReason"
    """Blocked for other reasons"""

    Blocklist = "BlockedReason"
    """Blocked due to blocklist match"""

    ModelArmor = "BlockedReason"
    """Blocked by Model Armor"""

    ProhibitedContent = "BlockedReason"
    """Contains prohibited content"""

    ImageSafety = "BlockedReason"
    """Blocked for image safety"""

    Jailbreak = "BlockedReason"
    """Blocked as jailbreak attempt"""

class UrlRetrievalStatus:
    """Status of URL retrieval operation.

    Indicates whether a URL was successfully retrieved by the tool.

    Examples:
        >>> status = UrlRetrievalStatus.UrlRetrievalStatusSuccess
        >>> status.value
        'URL_RETRIEVAL_STATUS_SUCCESS'
    """

    UrlRetrievalStatusUnspecified = "UrlRetrievalStatus"
    """Unspecified status"""

    UrlRetrievalStatusSuccess = "UrlRetrievalStatus"
    """URL retrieved successfully"""

    UrlRetrievalStatusError = "UrlRetrievalStatus"
    """URL retrieval failed"""

class HarmProbability:
    """Probability level of harmful content.

    Indicates the likelihood that content contains harmful material.

    Examples:
        >>> prob = HarmProbability.Medium
        >>> prob.value
        'MEDIUM'
    """

    HarmProbabilityUnspecified = "HarmProbability"
    """Unspecified probability"""

    Negligible = "HarmProbability"
    """Negligible harm probability"""

    Low = "HarmProbability"
    """Low harm probability"""

    Medium = "HarmProbability"
    """Medium harm probability"""

    High = "HarmProbability"
    """High harm probability"""

class HarmSeverity:
    """Severity level of harmful content.

    Indicates the severity of potentially harmful content.

    Examples:
        >>> severity = HarmSeverity.HarmSeverityMedium
        >>> severity.value
        'HARM_SEVERITY_MEDIUM'
    """

    HarmSeverityUnspecified = "HarmSeverity"
    """Unspecified severity"""

    HarmSeverityNegligible = "HarmSeverity"
    """Negligible severity"""

    HarmSeverityLow = "HarmSeverity"
    """Low severity"""

    HarmSeverityMedium = "HarmSeverity"
    """Medium severity"""

    HarmSeverityHigh = "HarmSeverity"
    """High severity"""

class FinishReason:
    """Reason why generation stopped.

    Indicates why the model stopped generating tokens.

    Examples:
        >>> reason = FinishReason.Stop
        >>> reason.value
        'STOP'
    """

    FinishReasonUnspecified = "FinishReason"
    """Unspecified reason"""

    Stop = "FinishReason"
    """Natural stopping point or stop sequence reached"""

    MaxTokens = "FinishReason"
    """Maximum token limit reached"""

    Safety = "FinishReason"
    """Stopped due to safety concerns"""

    Recitation = "FinishReason"
    """Stopped due to potential recitation"""

    Other = "FinishReason"
    """Stopped for other reasons"""

    Blocklist = "FinishReason"
    """Stopped due to blocklist match"""

    ProhibitedContent = "FinishReason"
    """Stopped due to prohibited content"""

    Spii = "FinishReason"
    """Stopped due to sensitive personally identifiable information"""

    MalformedFunctionCall = "FinishReason"
    """Stopped due to malformed function call"""

    ModelArmor = "FinishReason"
    """Stopped by Model Armor"""

    ImageSafety = "FinishReason"
    """Generated image violates safety policies"""

    ImageProhibitedContent = "FinishReason"
    """Generated image contains prohibited content"""

    ImageRecitation = "FinishReason"
    """Generated image may be recitation"""

    ImageOther = "FinishReason"
    """Image generation stopped for other reasons"""

    UnexpectedToolCall = "FinishReason"
    """Unexpected tool call generated"""

    NoImage = "FinishReason"
    """Expected image but none generated"""

class EmbeddingTaskType:
    """Task type for embedding generation.

    Specifies the intended use case for embeddings, which may affect
    how they are computed.

    Examples:
        >>> task = EmbeddingTaskType.RetrievalDocument
        >>> task.value
        'RETRIEVAL_DOCUMENT'
    """

    TaskTypeUnspecified = "EmbeddingTaskType"
    """Unspecified task type"""

    RetrievalQuery = "EmbeddingTaskType"
    """Query for retrieval tasks"""

    RetrievalDocument = "EmbeddingTaskType"
    """Document for retrieval tasks"""

    SemanticSimilarity = "EmbeddingTaskType"
    """Semantic similarity comparison"""

    Classification = "EmbeddingTaskType"
    """Classification tasks"""

    Clustering = "EmbeddingTaskType"
    """Clustering tasks"""

class Schema:
    """JSON Schema definition for structured outputs and parameters.

    Defines the structure, types, and constraints for JSON data used in
    function parameters and structured outputs. Based on OpenAPI 3.0 schema.

    Examples:
        >>> # Simple string schema
        >>> schema = Schema(
        ...     type=SchemaType.String,
        ...     description="User's name",
        ...     min_length="1",
        ...     max_length="100"
        ... )

        >>> # Object schema with properties
        >>> schema = Schema(
        ...     type=SchemaType.Object,
        ...     properties={
        ...         "name": Schema(type=SchemaType.String),
        ...         "age": Schema(type=SchemaType.Integer, minimum=0.0)
        ...     },
        ...     required=["name"]
        ... )
    """

    def __init__(
        self,
        type: Optional[SchemaType] = None,
        format: Optional[str] = None,
        title: Optional[str] = None,
        description: Optional[str] = None,
        nullable: Optional[bool] = None,
        enum_: Optional[List[str]] = None,
        max_items: Optional[str] = None,
        min_items: Optional[str] = None,
        properties: Optional[Dict[str, "Schema"]] = None,
        required: Optional[List[str]] = None,
        min_properties: Optional[str] = None,
        max_properties: Optional[str] = None,
        min_length: Optional[str] = None,
        max_length: Optional[str] = None,
        pattern: Optional[str] = None,
        example: Optional[Any] = None,
        any_of: Optional[List["Schema"]] = None,
        property_ordering: Optional[List[str]] = None,
        default: Optional[Any] = None,
        items: Optional["Schema"] = None,
        minimum: Optional[float] = None,
        maximum: Optional[float] = None,
    ) -> None:
        """Initialize a schema definition.

        Args:
            type (Optional[SchemaType]):
                The data type (string, number, object, etc.)
            format (Optional[str]):
                Format hint for the type (e.g., "date-time")
            title (Optional[str]):
                Human-readable title
            description (Optional[str]):
                Description of the schema
            nullable (Optional[bool]):
                Whether null values are allowed
            enum_ (Optional[List[str]]):
                List of allowed values
            max_items (Optional[str]):
                Maximum array length (for arrays)
            min_items (Optional[str]):
                Minimum array length (for arrays)
            properties (Optional[Dict[str, "Schema"]]):
                Object properties (for objects)
            required (Optional[List[str]]):
                Required property names (for objects)
            min_properties (Optional[str]):
                Minimum number of properties (for objects)
            max_properties (Optional[str]):
                Maximum number of properties (for objects)
            min_length (Optional[str]):
                Minimum string length (for strings)
            max_length (Optional[str]):
                Maximum string length (for strings)
            pattern (Optional[str]):
                Regular expression pattern (for strings)
            example (Optional[Any]):
                Example value
            any_of (Optional[List["Schema"]]):
                List of alternative schemas
            property_ordering (Optional[List[str]]):
                Order of properties
            default (Optional[Any]):
                Default value
            items (Optional["Schema"]):
                Schema for array items (for arrays)
            minimum (Optional[float]):
                Minimum numeric value (for numbers)
            maximum (Optional[float]):
                Maximum numeric value (for numbers)
        """

class SafetySetting:
    """Safety filtering configuration for harmful content.

    Controls how the model handles potentially harmful content in specific
    harm categories. Each setting applies to one harm category.

    Examples:
        >>> # Block hate speech with medium threshold
        >>> setting = SafetySetting(
        ...     category=HarmCategory.HarmCategoryHateSpeech,
        ...     threshold=HarmBlockThreshold.BlockMediumAndAbove
        ... )

        >>> # Disable blocking for harassment
        >>> setting = SafetySetting(
        ...     category=HarmCategory.HarmCategoryHarassment,
        ...     threshold=HarmBlockThreshold.BlockNone
        ... )
    """

    def __init__(
        self,
        category: HarmCategory,
        threshold: HarmBlockThreshold,
    ) -> None:
        """Initialize a safety setting.

        Args:
            category (HarmCategory):
                The harm category to configure
            threshold (HarmBlockThreshold):
                The blocking threshold to apply
        """

    @property
    def category(self) -> HarmCategory:
        """The harm category."""

    @property
    def threshold(self) -> HarmBlockThreshold:
        """The blocking threshold."""

class GeminiThinkingConfig:
    """Configuration for model thinking/reasoning features.

    Controls the model's internal reasoning process, including whether to
    include thoughts in the response and the computational budget.

    Examples:
        >>> # Enable high-level thinking with thoughts included
        >>> config = ThinkingConfig(
        ...     include_thoughts=True,
        ...     thinking_level=ThinkingLevel.High
        ... )

        >>> # Limit thinking budget
        >>> config = ThinkingConfig(
        ...     include_thoughts=False,
        ...     thinking_budget=1000
        ... )
    """

    def __init__(
        self,
        include_thoughts: Optional[bool] = None,
        thinking_budget: Optional[int] = None,
        thinking_level: Optional[ThinkingLevel] = None,
    ) -> None:
        """Initialize thinking configuration.

        Args:
            include_thoughts (Optional[bool]):
                Whether to include reasoning steps in response
            thinking_budget (Optional[int]):
                Token budget for thinking process
            thinking_level (Optional[ThinkingLevel]):
                Depth of reasoning to apply
        """

    @property
    def include_thoughts(self) -> Optional[bool]:
        """Whether to include thoughts in response."""

    @property
    def thinking_budget(self) -> Optional[int]:
        """Token budget for thinking."""

    @property
    def thinking_level(self) -> Optional[ThinkingLevel]:
        """Level of thinking/reasoning."""

class ImageConfig:
    """Configuration for image generation features.

    Controls aspect ratio and size for generated images.

    Examples:
        >>> # Generate widescreen 4K image
        >>> config = ImageConfig(
        ...     aspect_ratio="16:9",
        ...     image_size="4K"
        ... )

        >>> # Generate square 1K image
        >>> config = ImageConfig(
        ...     aspect_ratio="1:1",
        ...     image_size="1K"
        ... )
    """

    def __init__(
        self,
        aspect_ratio: Optional[str] = None,
        image_size: Optional[str] = None,
    ) -> None:
        """Initialize image configuration.

        Args:
            aspect_ratio (Optional[str]):
                Desired aspect ratio (e.g., "16:9", "1:1")
            image_size (Optional[str]):
                Image size ("1K", "2K", "4K")
        """

    @property
    def aspect_ratio(self) -> Optional[str]:
        """The image aspect ratio."""

    @property
    def image_size(self) -> Optional[str]:
        """The image size."""

class AutoRoutingMode:
    """Configuration for automatic model routing.

    Controls model selection based on routing preferences when using
    automatic routing features.

    Examples:
        >>> # Prioritize quality over cost
        >>> mode = AutoRoutingMode(
        ...     model_routing_preference=ModelRoutingPreference.PrioritizeQuality
        ... )

        >>> # Balance quality and cost
        >>> mode = AutoRoutingMode(
        ...     model_routing_preference=ModelRoutingPreference.Balanced
        ... )
    """

    def __init__(
        self,
        model_routing_preference: Optional[ModelRoutingPreference] = None,
    ) -> None:
        """Initialize automatic routing configuration.

        Args:
            model_routing_preference (Optional[ModelRoutingPreference]):
                Preference for model selection
        """

    @property
    def model_routing_preference(self) -> Optional[ModelRoutingPreference]:
        """The routing preference."""

class ManualRoutingMode:
    """Configuration for manual model routing.

    Explicitly specifies which model to use instead of automatic selection.

    Examples:
        >>> mode = ManualRoutingMode(model_name="gemini-2.0-flash-exp")
    """

    def __init__(
        self,
        model_name: str,
    ) -> None:
        """Initialize manual routing configuration.

        Args:
            model_name (str):
                Name of the model to use
        """

    @property
    def model_name(self) -> str:
        """The model name."""

class RoutingConfigMode:
    """Union type for routing configuration modes.

    Represents either automatic or manual routing configuration.

    Examples:
        >>> # Automatic routing
        >>> mode = RoutingConfigMode(
        ...     auto_mode=AutoRoutingMode(
        ...         model_routing_preference=ModelRoutingPreference.Balanced
        ...     )
        ... )

        >>> # Manual routing
        >>> mode = RoutingConfigMode(
        ...     manual_mode=ManualRoutingMode(model_name="gemini-2.0-flash-exp")
        ... )
    """

    def __init__(
        self,
        auto_mode: Optional[AutoRoutingMode] = None,
        manual_mode: Optional[ManualRoutingMode] = None,
    ) -> None:
        """Initialize routing mode.

        Exactly one of auto_mode or manual_mode must be provided.

        Args:
            auto_mode (Optional[AutoRoutingMode]):
                Automatic routing configuration
            manual_mode (Optional[ManualRoutingMode]):
                Manual routing configuration

        Raises:
            TypeError: If both or neither modes are provided
        """

class RoutingConfig:
    """Model routing configuration wrapper.

    Wraps the routing mode configuration.

    Examples:
        >>> config = RoutingConfig(
        ...     routing_config=RoutingConfigMode(
        ...         auto_mode=AutoRoutingMode(
        ...             model_routing_preference=ModelRoutingPreference.Balanced
        ...         )
        ...     )
        ... )
    """

    def __init__(
        self,
        routing_config: RoutingConfigMode,
    ) -> None:
        """Initialize routing configuration.

        Args:
            routing_config (RoutingConfigMode):
                The routing mode configuration
        """

    @property
    def routing_config(self) -> RoutingConfigMode:
        """The routing configuration mode."""

class PrebuiltVoiceConfig:
    """Configuration for prebuilt voice selection.

    Selects a prebuilt voice for text-to-speech generation.

    Examples:
        >>> config = PrebuiltVoiceConfig(voice_name="Puck")
    """

    def __init__(
        self,
        voice_name: str,
    ) -> None:
        """Initialize prebuilt voice configuration.

        Args:
            voice_name (str):
                Name of the prebuilt voice
        """

    @property
    def voice_name(self) -> str:
        """The voice name."""

class VoiceConfig:
    """Voice configuration for speech generation.

    Configures the voice to use for text-to-speech.

    Examples:
        >>> config = VoiceConfig(
        ...     prebuilt_voice_config=PrebuiltVoiceConfig(voice_name="Puck")
        ... )
    """

    def __init__(
        self,
        prebuilt_voice_config: PrebuiltVoiceConfig,
    ) -> None:
        """Initialize voice configuration.

        Args:
            prebuilt_voice_config (PrebuiltVoiceConfig):
                Prebuilt voice to use
        """

    @property
    def prebuilt_voice_config(self) -> PrebuiltVoiceConfig:
        """The prebuilt voice configuration."""

class SpeakerVoiceConfig:
    """Voice configuration for a specific speaker.

    Maps a speaker identifier to a voice configuration for multi-speaker
    text-to-speech.

    Examples:
        >>> config = SpeakerVoiceConfig(
        ...     speaker="Alice",
        ...     voice_config=VoiceConfig(
        ...         prebuilt_voice_config=PrebuiltVoiceConfig(voice_name="Puck")
        ...     )
        ... )
    """

    def __init__(
        self,
        speaker: str,
        voice_config: VoiceConfig,
    ) -> None:
        """Initialize speaker voice configuration.

        Args:
            speaker (str):
                Speaker identifier/name
            voice_config (VoiceConfig):
                Voice configuration for this speaker
        """

    @property
    def speaker(self) -> str:
        """The speaker identifier."""

    @property
    def voice_config(self) -> VoiceConfig:
        """The voice configuration."""

class MultiSpeakerVoiceConfig:
    """Configuration for multi-speaker text-to-speech.

    Configures voices for multiple speakers in a conversation or dialogue.

    Examples:
        >>> config = MultiSpeakerVoiceConfig(
        ...     speaker_voice_configs=[
        ...         SpeakerVoiceConfig(
        ...             speaker="Alice",
        ...             voice_config=VoiceConfig(
        ...                 prebuilt_voice_config=PrebuiltVoiceConfig(voice_name="Puck")
        ...             )
        ...         ),
        ...         SpeakerVoiceConfig(
        ...             speaker="Bob",
        ...             voice_config=VoiceConfig(
        ...                 prebuilt_voice_config=PrebuiltVoiceConfig(voice_name="Charon")
        ...             )
        ...         )
        ...     ]
        ... )
    """

    def __init__(
        self,
        speaker_voice_configs: List[SpeakerVoiceConfig],
    ) -> None:
        """Initialize multi-speaker configuration.

        Args:
            speaker_voice_configs (List[SpeakerVoiceConfig]):
                List of speaker voice configurations
        """

    @property
    def speaker_voice_configs(self) -> List[SpeakerVoiceConfig]:
        """The speaker voice configurations."""

class SpeechConfig:
    """Configuration for speech synthesis.

    Controls text-to-speech generation including voice selection and language.

    Examples:
        >>> # Single speaker
        >>> config = SpeechConfig(
        ...     voice_config=VoiceConfig(
        ...         prebuilt_voice_config=PrebuiltVoiceConfig(voice_name="Puck")
        ...     ),
        ...     language_code="en-US"
        ... )

        >>> # Multiple speakers
        >>> config = SpeechConfig(
        ...     multi_speaker_voice_config=MultiSpeakerVoiceConfig(...),
        ...     language_code="en-US"
        ... )
    """

    def __init__(
        self,
        voice_config: Optional[VoiceConfig] = None,
        multi_speaker_voice_config: Optional[MultiSpeakerVoiceConfig] = None,
        language_code: Optional[str] = None,
    ) -> None:
        """Initialize speech configuration.

        Args:
            voice_config (Optional[VoiceConfig]):
                Single voice configuration
            multi_speaker_voice_config (Optional[MultiSpeakerVoiceConfig]):
                Multi-speaker configuration
            language_code (Optional[str]):
                ISO 639-1 language code
        """

    @property
    def voice_config(self) -> Optional[VoiceConfig]:
        """The voice configuration."""

    @property
    def multi_speaker_voice_config(self) -> Optional[MultiSpeakerVoiceConfig]:
        """The multi-speaker configuration."""

    @property
    def language_code(self) -> Optional[str]:
        """The language code."""

class GenerationConfig:
    """Configuration for content generation behavior.

    Controls all aspects of how the model generates responses including
    sampling parameters, output format, modalities, and more.

    Examples:
        >>> # Basic text generation
        >>> config = GenerationConfig(
        ...     temperature=0.7,
        ...     max_output_tokens=1024,
        ...     top_p=0.95
        ... )

        >>> # Structured JSON output
        >>> config = GenerationConfig(
        ...     response_mime_type="application/json",
        ...     response_json_schema={"type": "object", ...},
        ...     temperature=0.3
        ... )

        >>> # Multi-modal with thinking
        >>> config = GenerationConfig(
        ...     response_modalities=[Modality.Text, Modality.Image],
        ...     thinking_config=ThinkingConfig(
        ...         include_thoughts=True,
        ...         thinking_level=ThinkingLevel.High
        ...     ),
        ...     temperature=0.5
        ... )
    """

    def __init__(
        self,
        stop_sequences: Optional[List[str]] = None,
        response_mime_type: Optional[str] = None,
        response_json_schema: Optional[Any] = None,
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
        enable_enhanced_civic_answers: Optional[bool] = None,
        image_config: Optional[ImageConfig] = None,
    ) -> None:
        """Initialize generation configuration.

        Args:
            stop_sequences (Optional[List[str]]):
                Sequences that stop generation
            response_mime_type (Optional[str]):
                MIME type for response (e.g., "application/json")
            response_json_schema (Optional[Any]):
                JSON schema for structured output
            response_modalities (Optional[List[Modality]]):
                Output modalities to include
            thinking_config (Optional[GeminiThinkingConfig]):
                Configuration for thinking/reasoning
            temperature (Optional[float]):
                Sampling temperature (0.0-2.0)
            top_p (Optional[float]):
                Nucleus sampling threshold
            top_k (Optional[int]):
                Top-k sampling threshold
            candidate_count (Optional[int]):
                Number of candidates to generate
            max_output_tokens (Optional[int]):
                Maximum tokens to generate
            response_logprobs (Optional[bool]):
                Whether to return log probabilities
            logprobs (Optional[int]):
                Number of top log probabilities to return
            presence_penalty (Optional[float]):
                Penalty for token presence (-2.0 to 2.0)
            frequency_penalty (Optional[float]):
                Penalty for token frequency (-2.0 to 2.0)
            seed (Optional[int]):
                Random seed for deterministic generation
            audio_timestamp (Optional[bool]):
                Whether to include audio timestamps
            media_resolution (Optional[MediaResolution]):
                Resolution for media processing
            speech_config (Optional[SpeechConfig]):
                Configuration for speech synthesis
            enable_affective_dialog (Optional[bool]):
                Enable emotion detection/adaptation
            enable_enhanced_civic_answers (Optional[bool]):
                Enable enhanced civic answers
            image_config (Optional[ImageConfig]):
                Configuration for image generation
        """

    @property
    def stop_sequences(self) -> Optional[List[str]]:
        """Stop sequences that halt generation."""

    @property
    def response_mime_type(self) -> Optional[str]:
        """The response MIME type."""

    @property
    def response_json_schema(self) -> Optional[Any]:
        """JSON schema for structured output."""

    @property
    def response_modalities(self) -> Optional[List[Modality]]:
        """Output modalities."""

    @property
    def thinking_config(self) -> Optional[GeminiThinkingConfig]:
        """Thinking configuration."""

    @property
    def temperature(self) -> Optional[float]:
        """Sampling temperature."""

    @property
    def top_p(self) -> Optional[float]:
        """Nucleus sampling threshold."""

    @property
    def top_k(self) -> Optional[int]:
        """Top-k sampling threshold."""

    @property
    def candidate_count(self) -> Optional[int]:
        """Number of candidates to generate."""

    @property
    def max_output_tokens(self) -> Optional[int]:
        """Maximum output tokens."""

    @property
    def response_logprobs(self) -> Optional[bool]:
        """Whether to return log probabilities."""

    @property
    def logprobs(self) -> Optional[int]:
        """Number of top log probabilities."""

    @property
    def presence_penalty(self) -> Optional[float]:
        """Presence penalty."""

    @property
    def frequency_penalty(self) -> Optional[float]:
        """Frequency penalty."""

    @property
    def seed(self) -> Optional[int]:
        """Random seed."""

    @property
    def audio_timestamp(self) -> Optional[bool]:
        """Whether to include audio timestamps."""

    @property
    def media_resolution(self) -> Optional[MediaResolution]:
        """Media resolution."""

    @property
    def speech_config(self) -> Optional[SpeechConfig]:
        """Speech configuration."""

    @property
    def enable_affective_dialog(self) -> Optional[bool]:
        """Whether affective dialog is enabled."""

    @property
    def image_config(self) -> Optional[ImageConfig]:
        """Image configuration."""

class ModelArmorConfig:
    """Configuration for Model Armor security filtering.

    Model Armor provides safety and security filtering for prompts and
    responses using customized templates.

    Examples:
        >>> config = ModelArmorConfig(
        ...     prompt_template_name="projects/my-project/locations/us/templates/strict",
        ...     response_template_name="projects/my-project/locations/us/templates/moderate"
        ... )
    """

    def __init__(
        self,
        prompt_template_name: Optional[str] = None,
        response_template_name: Optional[str] = None,
    ) -> None:
        """Initialize Model Armor configuration.

        Args:
            prompt_template_name (Optional[str]):
                Template for prompt screening
            response_template_name (Optional[str]):
                Template for response screening
        """

    @property
    def prompt_template_name(self) -> Optional[str]:
        """The prompt template name."""

    @property
    def response_template_name(self) -> Optional[str]:
        """The response template name."""

class FunctionCallingConfig:
    """Configuration for function calling behavior.

    Controls how the model handles function calls, including whether
    functions are required and which functions are allowed.

    Examples:
        >>> # Auto mode - model decides
        >>> config = FunctionCallingConfig(mode=Mode.Auto)

        >>> # Require specific functions
        >>> config = FunctionCallingConfig(
        ...     mode=Mode.Any,
        ...     allowed_function_names=["get_weather", "search_web"]
        ... )

        >>> # Disable function calling
        >>> config = FunctionCallingConfig(mode=Mode.None_Mode)
    """

    def __init__(
        self,
        mode: Optional[Mode] = None,
        allowed_function_names: Optional[List[str]] = None,
    ) -> None:
        """Initialize function calling configuration.

        Args:
            mode (Optional[Mode]):
                Function calling mode
            allowed_function_names (Optional[List[str]]):
                List of allowed function names (for ANY mode)
        """

    @property
    def mode(self) -> Optional[Mode]:
        """The function calling mode."""

    @property
    def allowed_function_names(self) -> Optional[List[str]]:
        """Allowed function names."""

class LatLng:
    """Geographic coordinates.

    Represents a latitude/longitude pair for location-based features.

    Examples:
        >>> # New York City coordinates
        >>> coords = LatLng(latitude=40.7128, longitude=-74.0060)
    """

    def __init__(
        self,
        latitude: float,
        longitude: float,
    ) -> None:
        """Initialize coordinates.

        Args:
            latitude (float):
                Latitude in degrees
            longitude (float):
                Longitude in degrees
        """

    @property
    def latitude(self) -> float:
        """The latitude."""

    @property
    def longitude(self) -> float:
        """The longitude."""

class RetrievalConfig:
    """Configuration for retrieval operations.

    Provides location and language context for retrieval tools.

    Examples:
        >>> config = RetrievalConfig(
        ...     lat_lng=LatLng(latitude=37.7749, longitude=-122.4194),
        ...     language_code="en-US"
        ... )
    """

    def __init__(
        self,
        lat_lng: LatLng,
        language_code: str,
    ) -> None:
        """Initialize retrieval configuration.

        Args:
            lat_lng (LatLng):
                Geographic coordinates
            language_code (str):
                Language code
        """

    @property
    def lat_lng(self) -> LatLng:
        """The geographic coordinates."""

    @property
    def language_code(self) -> str:
        """The language code."""

class ToolConfig:
    """Configuration for tool usage.

    Controls function calling and retrieval behavior across all tools.

    Examples:
        >>> config = ToolConfig(
        ...     function_calling_config=FunctionCallingConfig(mode=Mode.Auto),
        ...     retrieval_config=RetrievalConfig(
        ...         lat_lng=LatLng(latitude=37.7749, longitude=-122.4194),
        ...         language_code="en-US"
        ...     )
        ... )
    """

    def __init__(
        self,
        function_calling_config: Optional[FunctionCallingConfig] = None,
        retrieval_config: Optional[RetrievalConfig] = None,
    ) -> None:
        """Initialize tool configuration.

        Args:
            function_calling_config (Optional[FunctionCallingConfig]):
                Function calling configuration
            retrieval_config (Optional[RetrievalConfig]):
                Retrieval configuration
        """

    @property
    def function_calling_config(self) -> Optional[FunctionCallingConfig]:
        """The function calling configuration."""

    @property
    def retrieval_config(self) -> Optional[RetrievalConfig]:
        """The retrieval configuration."""

class GeminiSettings:
    """Settings for Gemini/Google API requests.

    Comprehensive configuration for all aspects of model behavior including
    generation, safety, tools, and more.

    Examples:
        >>> settings = GeminiSettings(
        ...     generation_config=GenerationConfig(
        ...         temperature=0.7,
        ...         max_output_tokens=1024
        ...     ),
        ...     safety_settings=[
        ...         SafetySetting(
        ...             category=HarmCategory.HarmCategoryHateSpeech,
        ...             threshold=HarmBlockThreshold.BlockMediumAndAbove
        ...         )
        ...     ],
        ...     tool_config=ToolConfig(
        ...         function_calling_config=FunctionCallingConfig(mode=Mode.Auto)
        ...     )
        ... )
    """

    def __init__(
        self,
        labels: Optional[Dict[str, str]] = None,
        tool_config: Optional[ToolConfig] = None,
        generation_config: Optional[GenerationConfig] = None,
        safety_settings: Optional[List[SafetySetting]] = None,
        model_armor_config: Optional[ModelArmorConfig] = None,
        extra_body: Optional[Any] = None,
        cached_content: Optional[str] = None,
        tools: Optional[List[GeminiTool]] = None,
    ) -> None:
        """Initialize Gemini settings.

        Args:
            labels (Optional[Dict[str, str]]):
                Metadata labels
            tool_config (Optional[ToolConfig]):
                Tool configuration
            generation_config (Optional[GenerationConfig]):
                Generation configuration
            safety_settings (Optional[List[SafetySetting]]):
                Safety filter settings
            model_armor_config (Optional[ModelArmorConfig]):
                Model Armor configuration
            extra_body (Optional[Any]):
                Additional request parameters
            cached_content (Optional[str]):
                Cached content resource name
            tools (Optional[List["Tool"]]):
                Tools available to the model
        """

    @property
    def labels(self) -> Optional[Dict[str, str]]:
        """Metadata labels."""

    @property
    def tool_config(self) -> Optional[ToolConfig]:
        """Tool configuration."""

    @property
    def generation_config(self) -> Optional[GenerationConfig]:
        """Generation configuration."""

    @property
    def safety_settings(self) -> Optional[List[SafetySetting]]:
        """Safety settings."""

    @property
    def model_armor_config(self) -> Optional[ModelArmorConfig]:
        """Model Armor configuration."""

    @property
    def extra_body(self) -> Optional[Dict[str, Any]]:
        """Additional request parameters."""

    @property
    def cached_content(self) -> Optional[str]:
        """Cached content resource name."""

    @property
    def tools(self) -> Optional[List[GeminiTool]]:
        """Available tools."""

    def __str__(self) -> str:
        """String representation."""

    def model_dump(self) -> Dict[str, Any]:
        """Convert settings to dictionary."""

    def settings_type(self) -> str:
        """Get settings type identifier."""

class FileData:
    """URI-based media data reference.

    References media stored in Google Cloud Storage or other URIs.

    Examples:
        >>> file_data = FileData(
        ...     mime_type="image/png",
        ...     file_uri="gs://my-bucket/image.png",
        ...     display_name="Example Image"
        ... )
    """

    def __init__(
        self,
        mime_type: str,
        file_uri: str,
        display_name: Optional[str] = None,
    ) -> None:
        """Initialize file data reference.

        Args:
            mime_type (str):
                IANA MIME type
            file_uri (str):
                URI to the file (e.g., gs:// URL)
            display_name (Optional[str]):
                Optional display name
        """

    @property
    def mime_type(self) -> str:
        """The MIME type."""

    @property
    def file_uri(self) -> str:
        """The file URI."""

    @property
    def display_name(self) -> Optional[str]:
        """The display name."""

class Blob:
    """Inline binary data.

    Contains raw binary data encoded in base64.

    Examples:
        >>> import base64
        >>> image_data = base64.b64encode(image_bytes).decode('utf-8')
        >>> blob = Blob(
        ...     mime_type="image/png",
        ...     data=image_data,
        ...     display_name="Example Image"
        ... )
    """

    def __init__(
        self,
        mime_type: str,
        data: str,
        display_name: Optional[str] = None,
    ) -> None:
        """Initialize binary data blob.

        Args:
            mime_type (str):
                IANA MIME type
            data (str):
                Base64-encoded binary data
            display_name (Optional[str]):
                Optional display name
        """

    @property
    def mime_type(self) -> str:
        """The MIME type."""

    @property
    def data(self) -> str:
        """The base64-encoded data."""

    @property
    def display_name(self) -> Optional[str]:
        """The display name."""

class PartialArgs:
    """Partial function call arguments for streaming.

    Represents incrementally streamed function call arguments.

    Examples:
        >>> args = PartialArgs(
        ...     json_path="$.location",
        ...     string_value="New York",
        ...     will_continue=True
        ... )
    """

    def __init__(
        self,
        json_path: str,
        will_continue: Optional[bool] = None,
        null_value: Optional[bool] = None,
        number_value: Optional[float] = None,
        string_value: Optional[str] = None,
        bool_value: Optional[bool] = None,
    ) -> None:
        """Initialize partial arguments.

        Args:
            json_path (str):
                JSON Path (RFC 9535) to the argument
            will_continue (Optional[bool]):
                Whether more parts follow for this path
            null_value (Optional[bool]):
                Null value
            number_value (Optional[float]):
                Numeric value
            string_value (Optional[str]):
                String value
            bool_value (Optional[bool]):
                Boolean value
        """

    @property
    def json_path(self) -> str:
        """The JSON path."""

    @property
    def will_continue(self) -> Optional[bool]:
        """Whether more parts follow."""

    @property
    def null_value(self) -> Optional[bool]:
        """Null value indicator."""

    @property
    def number_value(self) -> Optional[float]:
        """Numeric value."""

    @property
    def string_value(self) -> Optional[str]:
        """String value."""

    @property
    def bool_value(self) -> Optional[bool]:
        """Boolean value."""

class FunctionCall:
    """Function call request from the model.

    Represents a function that the model wants to call, including
    the function name and arguments.

    Examples:
        >>> call = FunctionCall(
        ...     name="get_weather",
        ...     args={"location": "San Francisco", "units": "celsius"},
        ...     id="call_123"
        ... )
    """

    def __init__(
        self,
        name: str,
        id: Optional[str] = None,
        args: Optional[Dict[str, Any]] = None,
        will_continue: Optional[bool] = None,
        partial_args: Optional[List[PartialArgs]] = None,
    ) -> None:
        """Initialize function call.

        Args:
            name: Function name to call
            id (Optional[str]):
                Unique call identifier
            args (Optional[Dict[str, Any]]):
                Function arguments as dictionary
            will_continue (Optional[bool]):
                Whether this is the final part of the call
            partial_args (Optional[List[PartialArgs]]):
                Incrementally streamed arguments
        """

    @property
    def name(self) -> str:
        """The function name."""

    @property
    def id(self) -> Optional[str]:
        """The call identifier."""

    @property
    def args(self) -> Optional[Dict[str, Any]]:
        """The function arguments."""

    @property
    def will_continue(self) -> Optional[bool]:
        """Whether more parts follow."""

    @property
    def partial_args(self) -> Optional[List[PartialArgs]]:
        """Partial arguments."""

class FunctionResponse:
    """Function execution result.

    Contains the result of executing a function call.
    """

    @property
    def name(self) -> str:
        """The function name."""

    @property
    def response(self) -> Dict[str, Any]:
        """The function response."""

class ExecutableCode:
    """Executable code generated by the model.

    Contains code that can be executed to perform computations.
    """

    @property
    def language(self) -> Language:
        """The programming language."""

    @property
    def code(self) -> str:
        """The code."""

class CodeExecutionResult:
    """Result of code execution.

    Contains the outcome and output from executing code.

    Examples:
        >>> result = CodeExecutionResult(
        ...     outcome=Outcome.OutcomeOk,
        ...     output="4\\n"
        ... )

        >>> # Error result
        >>> result = CodeExecutionResult(
        ...     outcome=Outcome.OutcomeFailed,
        ...     output="NameError: name 'x' is not defined"
        ... )
    """
    @property
    def outcome(self) -> Outcome:
        """The execution outcome."""

    @property
    def output(self) -> Optional[str]:
        """The output."""

class VideoMetadata:
    """Metadata for video content.

    Specifies time ranges and frame rates for video processing.
    """

    @property
    def start_offset(self) -> Optional[str]:
        """The start offset."""

    @property
    def end_offset(self) -> Optional[str]:
        """The end offset."""

class PartMetadata:
    """Custom metadata for content parts.

    Allows arbitrary structured metadata to be attached to parts.

    Examples:
        >>> metadata = PartMetadata(
        ...     struct_={"custom_field": "value", "priority": 1}
        ... )
    """

    def __init__(
        self,
        struct_: Optional[Dict[str, Any]] = None,
    ) -> None:
        """Initialize part metadata.

        Args:
            struct_: Arbitrary metadata dictionary
        """

class Part:
    """A part of a multi-part message.

    Represents a single piece of content which can be text, media, function
    calls, or other data types.

    Examples:
        >>> # Text part
        >>> part = Part(data="Hello, world!")

        >>> # Image part
        >>> part = Part(
        ...     data=Blob(
        ...         mime_type="image/png",
        ...         data=base64_encoded_data
        ...     )
        ... )

        >>> # Function call part
        >>> part = Part(
        ...     data=FunctionCall(
        ...         name="get_weather",
        ...         args={"location": "NYC"}
        ...     )
        ... )

        >>> # Part with metadata
        >>> part = Part(
        ...     data="Analyze this carefully",
        ...     thought=True,
        ...     media_resolution=MediaResolution.MediaResolutionHigh
        ... )
    """

    def __init__(
        self,
        data: Union[
            str,
            Blob,
            FileData,
            FunctionCall,
            FunctionResponse,
            ExecutableCode,
            CodeExecutionResult,
        ],
        thought: Optional[bool] = None,
        thought_signature: Optional[str] = None,
        part_metadata: Optional[PartMetadata] = None,
        media_resolution: Optional[MediaResolution] = None,
        video_metadata: Optional[VideoMetadata] = None,
    ) -> None:
        """Initialize a content part.

        Args:
            data (Union[str, Blob, FileData, FunctionCall, FunctionResponse, ExecutableCode, CodeExecutionResult]):
                The content data (text, blob, function call, etc.)
            thought (Optional[bool]):
                Whether this is part of the model's reasoning
            thought_signature (Optional[str]):
                Signature for reusing thoughts
            part_metadata (Optional[PartMetadata]):
                Custom metadata
            media_resolution (Optional[MediaResolution]):
                Media resolution level
            video_metadata (Optional[VideoMetadata]):
                Video-specific metadata
        """

    @property
    def thought(self) -> Optional[bool]:
        """Whether this is a thought/reasoning part."""

    @property
    def thought_signature(self) -> Optional[str]:
        """The thought signature."""

    @property
    def part_metadata(self) -> Optional[PartMetadata]:
        """Custom metadata."""

    @property
    def media_resolution(self) -> Optional[MediaResolution]:
        """Media resolution."""

    @property
    def data(
        self,
    ) -> Union[
        str,
        Blob,
        FileData,
        FunctionCall,
        FunctionResponse,
        ExecutableCode,
        CodeExecutionResult,
    ]:
        """The content data."""

    @property
    def video_metadata(self) -> Optional[VideoMetadata]:
        """Video metadata."""

class GeminiContent:
    """Multi-part message content.

    Represents a complete message from a user or model, consisting of one
    or more parts. This is the fundamental message structure for Gemini API.

    Examples:
        >>> # Simple text message
        >>> content = GeminiContent(
        ...     role="user",
        ...     parts="What's the weather in San Francisco?"
        ... )

        >>> # Multi-part message with image
        >>> content = GeminiContent(
        ...     role="user",
        ...     parts=[
        ...         "What's in this image?",
        ...         Blob(mime_type="image/png", data=image_data)
        ...     ]
        ... )

        >>> # Function call response
        >>> content = GeminiContent(
        ...     role="model",
        ...     parts=[
        ...         FunctionCall(
        ...             name="get_weather",
        ...             args={"location": "San Francisco"}
        ...         )
        ...     ]
        ... )

        >>> # Function result
        >>> content = GeminiContent(
        ...     role="function",
        ...     parts=[
        ...         FunctionResponse(
        ...             name="get_weather",
        ...             response={"output": {"temperature": 72}}
        ...         )
        ...     ]
        ... )
    """

    def __init__(
        self,
        parts: Union[
            str,
            Part,
            List[
                Union[
                    str,
                    Part,
                    Blob,
                    FileData,
                    FunctionCall,
                    FunctionResponse,
                    ExecutableCode,
                    CodeExecutionResult,
                ]
            ],
        ],
        role: Optional[str] = None,
    ) -> None:
        """Initialize message content.

        Args:
            parts (Union[str, Part, List[Union[str, Part, Blob, FileData, FunctionCall, FunctionResponse, ExecutableCode, CodeExecutionResult]]]):
                Content from typing import Any, Dict, List, Optional, Union from the message
            role (Optional[str]):
                Role of the message sender (e.g., "user", "model", "function")
        """

    @property
    def role(self) -> Optional[str]:
        """The role of the message sender."""

    @property
    def parts(self) -> List[Part]:
        """The message parts."""

class FunctionDeclaration:
    """Function declaration for tool use.

    Defines a function that the model can call, including its name,
    description, parameters, and return type.

    Examples:
        >>> func = FunctionDeclaration(
        ...     name="get_weather",
        ...     description="Get current weather for a location",
        ...     parameters=Schema(
        ...         type=SchemaType.Object,
        ...         properties={
        ...             "location": Schema(type=SchemaType.String),
        ...             "units": Schema(
        ...                 type=SchemaType.String,
        ...                 enum_=["celsius", "fahrenheit"]
        ...             )
        ...         },
        ...         required=["location"]
        ...     )
        ... )
    """

    @property
    def name(self) -> str:
        """The function name."""

    @property
    def description(self) -> str:
        """The function description."""

    @property
    def behavior(self) -> Optional[Behavior]:
        """Execution behavior (blocking/non-blocking)."""

    @property
    def parameters(self) -> Optional[Schema]:
        """Parameter schema."""

    @property
    def parameters_json_schema(self) -> Optional[Any]:
        """Parameters as raw JSON schema."""

    @property
    def response(self) -> Optional[Schema]:
        """Response schema."""

    @property
    def response_json_schema(self) -> Optional[Any]:
        """Response as raw JSON schema."""

class DataStoreSpec:
    """Specification for a Vertex AI Search datastore.

    Defines a datastore to search with optional filtering.

    Examples:
        >>> spec = DataStoreSpec(
        ...     data_store="projects/my-project/locations/us/collections/default/dataStores/my-store",
        ...     filter="category:electronics"
        ... )
    """

    def __init__(
        self,
        data_store: str,
        filter: Optional[str] = None,
    ) -> None:
        """Initialize datastore specification.

        Args:
            data_store (str):
                Full resource name of the datastore
            filter (Optional[str]):
                Optional filter expression
        """

    @property
    def data_store(self) -> str:
        """The datastore resource name."""

    @property
    def filter(self) -> Optional[str]:
        """The filter expression."""

class VertexAISearch:
    """Vertex AI Search retrieval configuration.

    Configures retrieval from Vertex AI Search datastores or engines.

    Examples:
        >>> # Using a datastore
        >>> search = VertexAISearch(
        ...     datastore="projects/my-project/locations/us/collections/default/dataStores/my-store",
        ...     max_results=5
        ... )

        >>> # Using an engine with multiple datastores
        >>> search = VertexAISearch(
        ...     engine="projects/my-project/locations/us/collections/default/engines/my-engine",
        ...     data_store_specs=[
        ...         DataStoreSpec(data_store="store1", filter="category:a"),
        ...         DataStoreSpec(data_store="store2", filter="category:b")
        ...     ]
        ... )
    """

    def __init__(
        self,
        datastore: Optional[str] = None,
        engine: Optional[str] = None,
        max_results: Optional[int] = None,
        filter: Optional[str] = None,
        data_store_specs: Optional[List[DataStoreSpec]] = None,
    ) -> None:
        """Initialize Vertex AI Search configuration.

        Args:
            datastore (Optional[str]):
                Datastore resource name
            engine (Optional[str]):
                Engine resource name
            max_results (Optional[int]):
                Maximum number of results (default 10, max 10)
            filter (Optional[str]):
                Filter expression
            data_store_specs (Optional[List[DataStoreSpec]]):
                Datastore specifications (for engines)
        """

    @property
    def datastore(self) -> Optional[str]:
        """The datastore resource name."""

    @property
    def engine(self) -> Optional[str]:
        """The engine resource name."""

    @property
    def max_results(self) -> Optional[int]:
        """Maximum results to return."""

    @property
    def filter(self) -> Optional[str]:
        """The filter expression."""

    @property
    def data_store_specs(self) -> Optional[List[DataStoreSpec]]:
        """Datastore specifications."""

class RagResource:
    """RAG corpus and file specification.

    Specifies which RAG corpus and optionally which files to use.

    Examples:
        >>> # Use entire corpus
        >>> resource = RagResource(
        ...     rag_corpus="projects/my-project/locations/us/ragCorpora/my-corpus"
        ... )

        >>> # Use specific files from corpus
        >>> resource = RagResource(
        ...     rag_corpus="projects/my-project/locations/us/ragCorpora/my-corpus",
        ...     rag_file_ids=["file1", "file2"]
        ... )
    """

    def __init__(
        self,
        rag_corpus: Optional[str] = None,
        rag_file_ids: Optional[List[str]] = None,
    ) -> None:
        """Initialize RAG resource.

        Args:
            rag_corpus (Optional[str]):
                RAG corpus resource name
            rag_file_ids (Optional[List[str]]):
                List of file IDs within the corpus
        """

    @property
    def rag_corpus(self) -> Optional[str]:
        """The RAG corpus resource name."""

    @property
    def rag_file_ids(self) -> Optional[List[str]]:
        """The file IDs."""

class Filter:
    """Filtering configuration for RAG retrieval.

    Configures metadata and vector-based filtering.

    Examples:
        >>> # Metadata filtering
        >>> filter = Filter(
        ...     metadata_filter="category = 'technical'",
        ...     vector_similarity_threshold=0.7
        ... )
    """

    def __init__(
        self,
        metadata_filter: Optional[str] = None,
        vector_distance_threshold: Optional[float] = None,
        vector_similarity_threshold: Optional[float] = None,
    ) -> None:
        """Initialize filter configuration.

        Args:
            metadata_filter (Optional[str]):
                Metadata filter expression
            vector_distance_threshold (Optional[float]):
                Maximum vector distance
            vector_similarity_threshold (Optional[float]):
                Minimum vector similarity
        """

    @property
    def metadata_filter(self) -> Optional[str]:
        """The metadata filter expression."""

    @property
    def vector_distance_threshold(self) -> Optional[float]:
        """Maximum vector distance threshold."""

    @property
    def vector_similarity_threshold(self) -> Optional[float]:
        """Minimum vector similarity threshold."""

class RankService:
    """Rank service configuration.

    Configures the ranking service for RAG results.

    Examples:
        >>> service = RankService(model_name="semantic-ranker-512@latest")
    """

    def __init__(
        self,
        model_name: Optional[str] = None,
    ) -> None:
        """Initialize rank service.

        Args:
            model_name (Optional[str]):
                Model name for ranking
        """

    @property
    def model_name(self) -> Optional[str]:
        """The ranking model name."""

class LlmRanker:
    """LLM-based ranker configuration.

    Uses an LLM to rank RAG results.

    Examples:
        >>> ranker = LlmRanker(model_name="gemini-1.5-flash")
    """

    def __init__(
        self,
        model_name: Optional[str] = None,
    ) -> None:
        """Initialize LLM ranker.

        Args:
            model_name (Optional[str]):
                Model name for ranking
        """

    @property
    def model_name(self) -> Optional[str]:
        """The ranking model name."""

class RankingConfig:
    """Union type for ranking configuration.

    Represents either rank service or LLM ranker configuration.

    Examples:
        >>> # Use rank service
        >>> config = RankingConfig.RankService(
        ...     RankService(model_name="semantic-ranker-512@latest")
        ... )

        >>> # Use LLM ranker
        >>> config = RankingConfig.LlmRanker(
        ...     LlmRanker(model_name="gemini-1.5-flash")
        ... )
    """

class Ranking:
    """Ranking and reranking configuration.

    Configures how RAG results are ranked.

    Examples:
        >>> # Using rank service
        >>> ranking = Ranking(
        ...     rank_service=RankService(model_name="semantic-ranker-512@latest")
        ... )

        >>> # Using LLM ranker
        >>> ranking = Ranking(
        ...     llm_ranker=LlmRanker(model_name="gemini-1.5-flash")
        ... )
    """

    def __init__(
        self,
        rank_service: Optional[RankService] = None,
        llm_ranker: Optional[LlmRanker] = None,
    ) -> None:
        """Initialize ranking configuration.

        Exactly one of rank_service or llm_ranker must be provided.

        Args:
            rank_service (Optional[RankService]):
                Rank service configuration
            llm_ranker (Optional[LlmRanker]):
                LLM ranker configuration

        Raises:
            TypeError: If both or neither are provided
        """

    @property
    def ranking_config(self) -> RankingConfig:
        """The ranking configuration."""

class RagRetrievalConfig:
    """Configuration for RAG retrieval behavior.

    Controls filtering, ranking, and other retrieval parameters.

    Examples:
        >>> config = RagRetrievalConfig(
        ...     top_k=5,
        ...     filter=Filter(metadata_filter="category='technical'"),
        ...     ranking=Ranking(
        ...         rank_service=RankService(model_name="semantic-ranker-512@latest")
        ...     )
        ... )
    """

    def __init__(
        self,
        top_k: Optional[int] = None,
        filter: Optional[Filter] = None,
        ranking: Optional[Ranking] = None,
    ) -> None:
        """Initialize RAG retrieval configuration.

        Args:
            top_k (Optional[int]):
                Number of top results to retrieve
            filter (Optional[Filter]):
                Filtering configuration
            ranking (Optional[Ranking]):
                Ranking configuration
        """

    @property
    def top_k(self) -> Optional[int]:
        """Number of top results."""

    @property
    def filter(self) -> Optional[Filter]:
        """Filter configuration."""

    @property
    def ranking(self) -> Optional[Ranking]:
        """Ranking configuration."""

class VertexRagStore:
    """Vertex RAG Store retrieval configuration.

    Configures retrieval from Vertex RAG Store.

    Examples:
        >>> store = VertexRagStore(
        ...     rag_resources=[
        ...         RagResource(
        ...             rag_corpus="projects/my-project/locations/us/ragCorpora/my-corpus"
        ...         )
        ...     ],
        ...     rag_retrieval_config=RagRetrievalConfig(top_k=5),
        ...     similarity_top_k=10
        ... )
    """

    @property
    def rag_resources(self) -> Optional[List[RagResource]]:
        """RAG resources to use."""

    @property
    def rag_retrieval_config(self) -> Optional[RagRetrievalConfig]:
        """Retrieval configuration."""

    @property
    def similarity_top_k(self) -> Optional[int]:
        """Number of similar results."""

    @property
    def vector_distance_threshold(self) -> Optional[float]:
        """Vector distance threshold."""

class SimpleSearchParams:
    """Parameters for simple search API.

    This type has no configuration fields.

    Examples:
        >>> params = SimpleSearchParams()
    """

    def __init__(self) -> None:
        """Initialize simple search parameters."""

class ElasticSearchParams:
    """Parameters for Elasticsearch API.

    Configures Elasticsearch index and search template.

    Examples:
        >>> params = ElasticSearchParams(
        ...     index="my-index",
        ...     search_template="my-template",
        ...     num_hits=10
        ... )
    """

    def __init__(
        self,
        index: str,
        search_template: str,
        num_hits: Optional[int] = None,
    ) -> None:
        """Initialize Elasticsearch parameters.

        Args:
            index (str):
                Elasticsearch index name
            search_template (str):
                Search template name
            num_hits (Optional[int]):
                Number of hits to request
        """

    @property
    def index(self) -> str:
        """The Elasticsearch index."""

    @property
    def search_template(self) -> str:
        """The search template."""

    @property
    def num_hits(self) -> Optional[int]:
        """Number of hits."""

class ApiKeyConfig:
    """API key authentication configuration.

    Configures API key authentication for external APIs.

    Examples:
        >>> config = ApiKeyConfig(
        ...     name="X-API-Key",
        ...     api_key_secret="projects/my-project/secrets/api-key",
        ...     http_element_location=HttpElementLocation.HttpInHeader
        ... )
    """

    def __init__(
        self,
        name: Optional[str] = None,
        api_key_secret: Optional[str] = None,
        api_key_string: Optional[str] = None,
        http_element_location: Optional[HttpElementLocation] = None,
    ) -> None:
        """Initialize API key configuration.

        Args:
            name (Optional[str]):
                Name of the API key parameter
            api_key_secret (Optional[str]):
                Secret manager resource name
            api_key_string (Optional[str]):
                Direct API key string
            http_element_location (Optional[HttpElementLocation]):
                Where to place the API key
        """

    @property
    def name(self) -> Optional[str]:
        """The API key parameter name."""

    @property
    def api_key_secret(self) -> Optional[str]:
        """The secret resource name."""

    @property
    def api_key_string(self) -> Optional[str]:
        """The direct API key string."""

    @property
    def http_element_location(self) -> Optional[HttpElementLocation]:
        """Where to place the API key."""

class HttpBasicAuthConfig:
    """HTTP Basic authentication configuration.

    Configures HTTP Basic authentication for external APIs.

    Examples:
        >>> config = HttpBasicAuthConfig(
        ...     credential_secret="projects/my-project/secrets/credentials"
        ... )
    """

    def __init__(
        self,
        credential_secret: str,
    ) -> None:
        """Initialize HTTP Basic auth configuration.

        Args:
            credential_secret (str):
                Secret manager resource name for credentials
        """

    @property
    def credential_secret(self) -> str:
        """The credential secret resource name."""

class GoogleServiceAccountConfig:
    """Google Service Account authentication configuration.

    Configures service account authentication.

    Examples:
        >>> config = GoogleServiceAccountConfig(
        ...     service_account="my-service-account@my-project.iam.gserviceaccount.com"
        ... )
    """

    def __init__(
        self,
        service_account: Optional[str] = None,
    ) -> None:
        """Initialize service account configuration.

        Args:
            service_account (Optional[str]):
                Service account email
        """

    @property
    def service_account(self) -> Optional[str]:
        """The service account email."""

class OauthConfigValue:
    """Union type for OAuth configuration.

    Represents either an access token or service account OAuth configuration.

    Examples:
        >>> # Using access token
        >>> config = OauthConfigValue(access_token="ya29....")

        >>> # Using service account
        >>> config = OauthConfigValue(
        ...     service_account="my-sa@project.iam.gserviceaccount.com"
        ... )
    """

    def __init__(
        self,
        access_token: Optional[str] = None,
        service_account: Optional[str] = None,
    ) -> None:
        """Initialize OAuth configuration value.

        Exactly one of access_token or service_account must be provided.

        Args:
            access_token (Optional[str]):
                OAuth access token
            service_account (Optional[str]):
                Service account email

        Raises:
            TypeError: If both or neither are provided
        """

class OauthConfig:
    """OAuth authentication configuration.

    Configures OAuth authentication for external APIs.

    Examples:
        >>> config = OauthConfig(access_token="ya29....")
    """

    def __init__(
        self,
        access_token: Optional[str] = None,
        service_account: Optional[str] = None,
    ) -> None:
        """Initialize OAuth configuration.

        Args:
            access_token (Optional[str]):
                OAuth access token
            service_account (Optional[str]):
                Service account email

        Raises:
            TypeError: If configuration is invalid
        """

    @property
    def oauth_config(self) -> OauthConfigValue:
        """The OAuth configuration value."""

class OidcConfig:
    """OIDC authentication configuration.

    Configures OIDC authentication for external APIs.

    Examples:
        >>> config = OidcConfig(id_token="eyJhbGc...")
    """

    def __init__(
        self,
        id_token: Optional[str] = None,
        service_account: Optional[str] = None,
    ) -> None:
        """Initialize OIDC configuration.

        Args:
            id_token (Optional[str]):
                OIDC ID token
            service_account (Optional[str]):
                Service account email

        Raises:
            TypeError: If configuration is invalid
        """

    @property
    def oidc_config(self) -> Any:
        """The OIDC configuration value."""

class AuthConfigValue:
    """Union type for authentication configuration.

    Represents one of several authentication methods.

    Examples:
        >>> # API key auth
        >>> config = AuthConfigValue(
        ...     api_key_config=ApiKeyConfig(...)
        ... )

        >>> # OAuth
        >>> config = AuthConfigValue(
        ...     oauth_config=OauthConfig(...)
        ... )
    """

    def __init__(
        self,
        api_key_config: Optional[ApiKeyConfig] = None,
        http_basic_auth_config: Optional[HttpBasicAuthConfig] = None,
        google_service_account_config: Optional[GoogleServiceAccountConfig] = None,
        oauth_config: Optional[OauthConfig] = None,
        oidc_config: Optional[OidcConfig] = None,
    ) -> None:
        """Initialize auth configuration value.

        Exactly one configuration type must be provided.

        Args:
            api_key_config (Optional[ApiKeyConfig]):
                API key authentication
            http_basic_auth_config (Optional[HttpBasicAuthConfig]):
                HTTP Basic authentication
            google_service_account_config (Optional[GoogleServiceAccountConfig]):
                Service account authentication
            oauth_config (Optional[OauthConfig]):
                OAuth authentication
            oidc_config (Optional[OidcConfig]):
                OIDC authentication

        Raises:
            TypeError: If configuration is invalid
        """

class AuthConfig:
    """Authentication configuration wrapper.

    Wraps authentication type and configuration.

    Examples:
        >>> config = AuthConfig(
        ...     auth_type=AuthType.ApiKeyAuth,
        ...     auth_config=AuthConfigValue(
        ...         api_key_config=ApiKeyConfig(...)
        ...     )
        ... )
    """

    @property
    def auth_type(self) -> AuthType:
        """The authentication type."""

    @property
    def auth_config(self) -> AuthConfigValue:
        """The authentication configuration."""

class ExternalApi:
    """External API retrieval configuration.

    Configures retrieval from external APIs.

    Examples:
        >>> api = ExternalApi(
        ...     api_spec=ApiSpecType.ElasticSearch,
        ...     endpoint="https://my-es-cluster.com",
        ...     auth_config=AuthConfig(...),
        ...     elastic_search_params=ElasticSearchParams(...)
        ... )
    """

    def __init__(
        self,
        api_spec: ApiSpecType,
        endpoint: str,
        auth_config: Optional[AuthConfig] = None,
        simple_search_params: Optional[SimpleSearchParams] = None,
        elastic_search_params: Optional[ElasticSearchParams] = None,
    ) -> None:
        """Initialize external API configuration.

        Args:
            api_spec (ApiSpecType):
                API specification type
            endpoint (str):
                API endpoint URL
            auth_config (Optional[AuthConfig]):
                Authentication configuration
            simple_search_params (Optional[SimpleSearchParams]):
                Simple search parameters
            elastic_search_params (Optional[ElasticSearchParams]):
                Elasticsearch parameters
        """

    @property
    def api_spec(self) -> ApiSpecType:
        """The API specification type."""

    @property
    def endpoint(self) -> str:
        """The API endpoint."""

    @property
    def auth_config(self) -> Optional[AuthConfig]:
        """The authentication configuration."""

    @property
    def params(self) -> Optional[Union[SimpleSearchParams, ElasticSearchParams]]:
        """The API parameters."""

class RetrievalSource:
    """Union type for retrieval sources.

    Represents one of several retrieval source types.

    Examples:
        >>> # Vertex AI Search
        >>> source = RetrievalSource(
        ...     vertex_ai_search=VertexAISearch(...)
        ... )

        >>> # RAG Store
        >>> source = RetrievalSource(
        ...     vertex_rag_store=VertexRagStore(...)
        ... )

        >>> # External API
        >>> source = RetrievalSource(
        ...     external_api=ExternalApi(...)
        ... )
    """

    def __init__(
        self,
        vertex_ai_search: Optional[VertexAISearch] = None,
        vertex_rag_store: Optional[VertexRagStore] = None,
        external_api: Optional[ExternalApi] = None,
    ) -> None:
        """Initialize retrieval source.

        Exactly one source type must be provided.

        Args:
            vertex_ai_search (Optional[VertexAISearch]):
                Vertex AI Search configuration
            vertex_rag_store (Optional[VertexRagStore]):
                Vertex RAG Store configuration
            external_api (Optional[ExternalApi]):
                External API configuration

        Raises:
            TypeError: If configuration is invalid
        """

class Retrieval:
    """Retrieval tool configuration.

    Enables the model to retrieve information from external sources.

    Examples:
        >>> retrieval = Retrieval(
        ...     source=RetrievalSource(
        ...         vertex_ai_search=VertexAISearch(
        ...             datastore="projects/my-project/..."
        ...         )
        ...     ),
        ...     disable_attribution=False
        ... )
    """

    def __init__(
        self,
        source: RetrievalSource,
        disable_attribution: Optional[bool] = None,
    ) -> None:
        """Initialize retrieval configuration.

        Args:
            source (RetrievalSource):
                Retrieval source configuration
            disable_attribution (Optional[bool]):
                Whether to disable attribution
        """

    @property
    def disable_attribution(self) -> Optional[bool]:
        """Whether attribution is disabled."""

    @property
    def source(self) -> RetrievalSource:
        """The retrieval source."""

class Interval:
    """Time interval specification.

    Represents a time range with start and end times.

    Examples:
        >>> interval = Interval(
        ...     start_time="2024-01-01T00:00:00Z",
        ...     end_time="2024-12-31T23:59:59Z"
        ... )
    """

    @property
    def start_time(self) -> str:
        """The start time."""

    @property
    def end_time(self) -> str:
        """The end time."""

class GoogleSearch:
    """Google Search tool configuration (Gemini API).

    Configures Google Search with time range filtering.

    Examples:
        >>> search = GoogleSearch(
        ...     time_range_filter=Interval(
        ...         start_time="2024-01-01T00:00:00Z",
        ...         end_time="2024-12-31T23:59:59Z"
        ...     )
        ... )
    """

    @property
    def time_range_filter(self) -> Interval:
        """The time range filter."""

class VertexGoogleSearch:
    """Google Search tool configuration (Vertex API).

    Configures Google Search with domain blocking and phishing filters.

    Examples:
        >>> search = VertexGoogleSearch(
        ...     exclude_domains=["example.com", "spam.com"],
        ...     blocking_confidence=PhishBlockThreshold.BlockMediumAndAbove
        ... )
    """

    def __init__(
        self,
        exclude_domains: Optional[List[str]] = None,
        blocking_confidence: Optional[PhishBlockThreshold] = None,
    ) -> None:
        """Initialize Vertex Google Search configuration.

        Args:
            exclude_domains (Optional[List[str]]):
                Domains to exclude from results
            blocking_confidence (Optional[PhishBlockThreshold]):
                Phishing blocking threshold
        """

    @property
    def exclude_domains(self) -> Optional[List[str]]:
        """Domains to exclude."""

    @property
    def blocking_confidence(self) -> Optional[PhishBlockThreshold]:
        """Phishing blocking threshold."""

class EnterpriseWebSearch:
    """Enterprise web search tool configuration.

    Configures enterprise-grade web search with compliance features.

    Examples:
        >>> search = EnterpriseWebSearch(
        ...     exclude_domains=["example.com"],
        ...     blocking_confidence=PhishBlockThreshold.BlockHighAndAbove
        ... )
    """

    def __init__(
        self,
        exclude_domains: Optional[List[str]] = None,
        blocking_confidence: Optional[PhishBlockThreshold] = None,
    ) -> None:
        """Initialize enterprise web search configuration.

        Args:
            exclude_domains (Optional[List[str]]):
                Domains to exclude from results
            blocking_confidence (Optional[PhishBlockThreshold]):
                Phishing blocking threshold
        """

    @property
    def exclude_domains(self) -> Optional[List[str]]:
        """Domains to exclude."""

    @property
    def blocking_confidence(self) -> Optional[PhishBlockThreshold]:
        """Phishing blocking threshold."""

class ParallelAiSearch:
    """Parallel.ai search tool configuration.

    Configures search using the Parallel.ai engine.

    Examples:
        >>> search = ParallelAiSearch(
        ...     api_key="my-api-key",
        ...     custom_configs={
        ...         "source_policy": {"include_domains": ["google.com"]},
        ...         "maxResults": 10
        ...     }
        ... )
    """

    def __init__(
        self,
        api_key: Optional[str] = None,
        custom_configs: Optional[Dict[str, Any]] = None,
    ) -> None:
        """Initialize Parallel.ai search configuration.

        Args:
            api_key (Optional[str]):
                Parallel.ai API key
            custom_configs (Optional[Dict[str, Any]]):
                Custom configuration parameters

        Raises:
            TypeError: If configuration is invalid
        """

    @property
    def api_key(self) -> Optional[str]:
        """The API key."""

    @property
    def custom_configs(self) -> Optional[Dict[str, Any]]:
        """Custom configuration parameters."""

class GoogleSearchNum:
    """Union type for Google Search configurations.

    Represents either Gemini or Vertex Google Search configuration.

    Examples:
        >>> # Gemini search
        >>> search = GoogleSearchNum(
        ...     gemini_search=GoogleSearch(...)
        ... )

        >>> # Vertex search
        >>> search = GoogleSearchNum(
        ...     vertex_search=VertexGoogleSearch(...)
        ... )
    """

    def __init__(
        self,
        gemini_search: Optional[GoogleSearch] = None,
        vertex_search: Optional[VertexGoogleSearch] = None,
    ) -> None:
        """Initialize Google Search configuration.

        Exactly one of gemini_search or vertex_search must be provided.

        Args:
            gemini_search (Optional[GoogleSearch]):
                Gemini API search configuration
            vertex_search (Optional[VertexGoogleSearch]):
                Vertex API search configuration
        """

class DynamicRetrievalConfig:
    """Configuration for dynamic retrieval behavior.

    Controls when and how retrieval is triggered.

    Examples:
        >>> config = DynamicRetrievalConfig(
        ...     mode=DynamicRetrievalMode.ModeDynamic,
        ...     dynamic_threshold=0.5
        ... )
    """

    def __init__(
        self,
        mode: Optional[DynamicRetrievalMode] = None,
        dynamic_threshold: Optional[float] = None,
    ) -> None:
        """Initialize dynamic retrieval configuration.

        Args:
            mode (Optional[DynamicRetrievalMode]):
                Retrieval mode
            dynamic_threshold (Optional[float]):
                Threshold for dynamic retrieval
        """

    @property
    def mode(self) -> Optional[DynamicRetrievalMode]:
        """The retrieval mode."""

    @property
    def dynamic_threshold(self) -> Optional[float]:
        """The dynamic threshold."""

class GoogleSearchRetrieval:
    """Google Search retrieval tool configuration.

    Configures Google Search with dynamic retrieval.

    Examples:
        >>> retrieval = GoogleSearchRetrieval(
        ...     dynamic_retrieval_config=DynamicRetrievalConfig(
        ...         mode=DynamicRetrievalMode.ModeDynamic
        ...     )
        ... )
    """

    def __init__(
        self,
        dynamic_retrieval_config: Optional[DynamicRetrievalConfig] = None,
    ) -> None:
        """Initialize Google Search retrieval configuration.

        Args:
            dynamic_retrieval_config (Optional[DynamicRetrievalConfig]):
                Dynamic retrieval configuration
        """

    @property
    def dynamic_retrieval_config(self) -> Optional[DynamicRetrievalConfig]:
        """The dynamic retrieval configuration."""

class GoogleMaps:
    """Google Maps tool configuration.

    Configures Google Maps integration.

    Examples:
        >>> maps = GoogleMaps(enable_widget=True)
    """

    def __init__(
        self,
        enable_widget: bool = False,
    ) -> None:
        """Initialize Google Maps configuration.

        Args:
            enable_widget (bool):
                Whether to enable widget context token
        """

    @property
    def enable_widget(self) -> bool:
        """Whether widget is enabled."""

class CodeExecution:
    """Code execution tool configuration.

    Enables the model to execute generated code.

    This type has no configuration fields.

    Examples:
        >>> code_exec = CodeExecution()
    """

    def __init__(self) -> None:
        """Initialize code execution tool."""

class ComputerUse:
    """Computer use tool configuration.

    Enables the model to interact with computer interfaces.

    Examples:
        >>> computer_use = ComputerUse(
        ...     environment=ComputerUseEnvironment.EnvironmentBrowser,
        ...     excluded_predefined_functions=["take_screenshot"]
        ... )
    """

    def __init__(
        self,
        environment: ComputerUseEnvironment,
        excluded_predefined_functions: List[str],
    ) -> None:
        """Initialize computer use configuration.

        Args:
            environment (ComputerUseEnvironment):
                Operating environment
            excluded_predefined_functions (List[str]):
                Functions to exclude from auto-inclusion
        """

    @property
    def environment(self) -> ComputerUseEnvironment:
        """The operating environment."""

    @property
    def excluded_predefined_functions(self) -> List[str]:
        """Excluded functions."""

class UrlContext:
    """URL context tool configuration.

    Enables retrieval from user-provided URLs.

    This type has no configuration fields.

    Examples:
        >>> url_context = UrlContext()
    """

    def __init__(self) -> None:
        """Initialize URL context tool."""

class FileSearch:
    """File search tool configuration.

    Enables searching in file stores.

    Examples:
        >>> file_search = FileSearch(
        ...     file_search_store_names=["my-store"],
        ...     metadata_filter="category='docs'",
        ...     top_k=5
        ... )
    """

    def __init__(
        self,
        file_search_store_names: List[str],
        metadata_filter: str,
        top_k: int,
    ) -> None:
        """Initialize file search configuration.

        Args:
            file_search_store_names (List[str]):
                File store names to search
            metadata_filter (str):
                Metadata filter expression
            top_k (int):
                Number of top results
        """

    @property
    def file_search_store_names(self) -> List[str]:
        """File store names."""

    @property
    def metadata_filter(self) -> str:
        """Metadata filter."""

    @property
    def top_k(self) -> int:
        """Number of top results."""

class GeminiTool:
    """Tool definition for model use.

    Defines tools/functions that the model can use during generation.
    Tools enable the model to perform actions or retrieve information.

    Examples:
        >>> # Function tool
        >>> tool = Tool(
        ...     function_declarations=[
        ...         FunctionDeclaration(
        ...             name="get_weather",
        ...             description="Get weather for a location",
        ...             parameters=Schema(...)
        ...         )
        ...     ]
        ... )

        >>> # Google Search tool
        >>> tool = Tool(
        ...     google_search=GoogleSearchNum(
        ...         vertex_search=VertexGoogleSearch()
        ...     )
        ... )

        >>> # Code execution tool
        >>> tool = Tool(code_execution=CodeExecution())

        >>> # Multiple tools
        >>> tool = Tool(
        ...     function_declarations=[...],
        ...     google_search=GoogleSearchNum(...),
        ...     code_execution=CodeExecution()
        ... )
    """

    def __init__(
        self,
        function_declarations: Optional[List[FunctionDeclaration]] = None,
        retrieval: Optional[Retrieval] = None,
        google_search_retrieval: Optional[GoogleSearchRetrieval] = None,
        code_execution: Optional[CodeExecution] = None,
        google_search: Optional[GoogleSearchNum] = None,
        google_maps: Optional[GoogleMaps] = None,
        enterprise_web_search: Optional[EnterpriseWebSearch] = None,
        parallel_ai_search: Optional[ParallelAiSearch] = None,
        computer_use: Optional[ComputerUse] = None,
        url_context: Optional[UrlContext] = None,
        file_search: Optional[FileSearch] = None,
    ) -> None:
        """Initialize tool configuration.

        Args:
            function_declarations (Optional[List[FunctionDeclaration]]):
                Function declarations
            retrieval (Optional[Retrieval]):
                Retrieval tool configuration
            google_search_retrieval (Optional[GoogleSearchRetrieval]):
                Google Search retrieval configuration
            code_execution (Optional[CodeExecution]):
                Code execution tool
            google_search (Optional[GoogleSearchNum]):
                Google Search tool
            google_maps (Optional[GoogleMaps]):
                Google Maps tool
            enterprise_web_search (Optional[EnterpriseWebSearch]):
                Enterprise web search tool
            parallel_ai_search (Optional[ParallelAiSearch]):
                Parallel.ai search tool
            computer_use (Optional[ComputerUse]):
                Computer use tool
            url_context (Optional[UrlContext]):
                URL context tool
            file_search (Optional[FileSearch]):
                File search tool
        """

    @property
    def function_declarations(self) -> Optional[List[FunctionDeclaration]]:
        """Function declarations."""

    @property
    def retrieval(self) -> Optional[Retrieval]:
        """Retrieval configuration."""

    @property
    def google_search_retrieval(self) -> Optional[GoogleSearchRetrieval]:
        """Google Search retrieval configuration."""

    @property
    def code_execution(self) -> Optional[CodeExecution]:
        """Code execution tool."""

    @property
    def google_search(self) -> Optional[GoogleSearchNum]:
        """Google Search tool."""

    @property
    def google_maps(self) -> Optional[GoogleMaps]:
        """Google Maps tool."""

    @property
    def enterprise_web_search(self) -> Optional[EnterpriseWebSearch]:
        """Enterprise web search tool."""

    @property
    def parallel_ai_search(self) -> Optional[ParallelAiSearch]:
        """Parallel.ai search tool."""

    @property
    def computer_use(self) -> Optional[ComputerUse]:
        """Computer use tool."""

    @property
    def url_context(self) -> Optional[UrlContext]:
        """URL context tool."""

    @property
    def file_search(self) -> Optional[FileSearch]:
        """File search tool."""

class ModalityTokenCount:
    """Token count by modality.

    Breaks down token usage by content type (text, image, audio, etc.).

    Examples:
        >>> count = ModalityTokenCount(
        ...     modality=Modality.Text,
        ...     token_count=150
        ... )
    """

    @property
    def modality(self) -> Optional[Modality]:
        """The content modality."""

    @property
    def token_count(self) -> Optional[int]:
        """Token count for this modality."""

class UsageMetadata:
    """Token usage metadata for a request/response.

    Provides detailed breakdown of token usage across different components.

    Examples:
        >>> usage = UsageMetadata(
        ...     prompt_token_count=100,
        ...     candidates_token_count=50,
        ...     total_token_count=150,
        ...     cached_content_token_count=20
        ... )
    """

    @property
    def prompt_token_count(self) -> Optional[int]:
        """Tokens in the prompt."""

    @property
    def candidates_token_count(self) -> Optional[int]:
        """Tokens in generated candidates."""

    @property
    def tool_use_prompt_token_count(self) -> Optional[int]:
        """Tokens from tool use results."""

    @property
    def thoughts_token_count(self) -> Optional[int]:
        """Tokens in thinking/reasoning."""

    @property
    def total_token_count(self) -> Optional[int]:
        """Total token count."""

    @property
    def cached_content_token_count(self) -> Optional[int]:
        """Tokens from cached content."""

    @property
    def prompt_tokens_details(self) -> Optional[List[ModalityTokenCount]]:
        """Prompt tokens by modality."""

    @property
    def cache_tokens_details(self) -> Optional[List[ModalityTokenCount]]:
        """Cache tokens by modality."""

    @property
    def candidates_tokens_details(self) -> Optional[List[ModalityTokenCount]]:
        """Candidate tokens by modality."""

    @property
    def tool_use_prompt_tokens_details(self) -> Optional[List[ModalityTokenCount]]:
        """Tool use tokens by modality."""

    @property
    def traffic_type(self) -> Optional[TrafficType]:
        """Traffic type for billing."""

class PromptFeedback:
    """Feedback about prompt blocking.

    Indicates why a prompt was blocked by content filters.

    Examples:
        >>> feedback = PromptFeedback(
        ...     block_reason=BlockedReason.Safety,
        ...     safety_ratings=[...],
        ...     block_reason_message="Prompt contains unsafe content"
        ... )
    """

    @property
    def block_reason(self) -> Optional[BlockedReason]:
        """Why the prompt was blocked."""

    @property
    def safety_ratings(self) -> Optional[List["SafetyRating"]]:
        """Safety ratings for the prompt."""

    @property
    def block_reason_message(self) -> Optional[str]:
        """Human-readable block reason."""

class UrlMetadata:
    """Metadata about URL retrieval.

    Information about a URL retrieved by the URL context tool.

    Examples:
        >>> metadata = UrlMetadata(
        ...     retrieved_url="https://example.com",
        ...     url_retrieval_status=UrlRetrievalStatus.UrlRetrievalStatusSuccess
        ... )
    """

    @property
    def retrieved_url(self) -> Optional[str]:
        """The retrieved URL."""

    @property
    def url_retrieval_status(self) -> Optional[UrlRetrievalStatus]:
        """Retrieval status."""

class UrlContextMetadata:
    """Metadata about URL context tool usage.

    Contains information about URLs retrieved by the tool.

    Examples:
        >>> metadata = UrlContextMetadata(
        ...     url_metadata=[
        ...         UrlMetadata(retrieved_url="https://example.com", ...)
        ...     ]
        ... )
    """

    @property
    def url_metadata(self) -> Optional[List[UrlMetadata]]:
        """List of URL metadata."""

class SourceFlaggingUri:
    """URI flagged as potentially problematic.

    Information about a source that was flagged for review.

    Examples:
        >>> uri = SourceFlaggingUri(
        ...     source_id="source123",
        ...     flag_content_uri="https://example.com/flagged"
        ... )
    """

    @property
    def source_id(self) -> str:
        """Source identifier."""

    @property
    def flag_content_uri(self) -> str:
        """URI of flagged content."""

class RetrievalMetadata:
    """Metadata about retrieval operations.

    Contains scores and information about retrieval behavior.

    Examples:
        >>> metadata = RetrievalMetadata(
        ...     google_search_dynamic_retrieval_score=0.85
        ... )
    """

    @property
    def google_search_dynamic_retrieval_score(self) -> Optional[float]:
        """Score for dynamic retrieval likelihood."""

class SearchEntryPoint:
    """Search entry point information.

    Contains embeddable search widgets and SDK data.

    Examples:
        >>> entry_point = SearchEntryPoint(
        ...     rendered_content="<div>...</div>",
        ...     sdk_blob="base64encodeddata"
        ... )
    """

    @property
    def rendered_content(self) -> Optional[str]:
        """Embeddable HTML content."""

    @property
    def sdk_blob(self) -> Optional[str]:
        """Base64 encoded SDK data."""

class Segment:
    """Text segment within content.

    Identifies a portion of generated content by part index and byte range.

    Examples:
        >>> segment = Segment(
        ...     part_index=0,
        ...     start_index=10,
        ...     end_index=50,
        ...     text="example text"
        ... )
    """

    @property
    def part_index(self) -> Optional[int]:
        """Index of the Part object."""

    @property
    def start_index(self) -> Optional[int]:
        """Start byte index."""

    @property
    def end_index(self) -> Optional[int]:
        """End byte index."""

    @property
    def text(self) -> Optional[str]:
        """The segment text."""

class GroundingSupport:
    """Grounding support information.

    Links generated content to source materials with confidence scores.

    Examples:
        >>> support = GroundingSupport(
        ...     grounding_chunk_indices=[0, 1, 2],
        ...     confidence_scores=[0.9, 0.85, 0.8],
        ...     segment=Segment(...)
        ... )
    """

    @property
    def grounding_chunk_indices(self) -> Optional[List[int]]:
        """Indices into grounding chunks."""

    @property
    def confidence_scores(self) -> Optional[List[float]]:
        """Confidence scores for citations."""

    @property
    def segment(self) -> Optional[Segment]:
        """Content segment being supported."""

class Web:
    """Web source information.

    Information about a web source used for grounding.

    Examples:
        >>> web = Web(
        ...     uri="https://example.com/page",
        ...     title="Example Page",
        ...     domain="example.com"
        ... )
    """

    @property
    def uri(self) -> Optional[str]:
        """The source URI."""

    @property
    def title(self) -> Optional[str]:
        """The page title."""

    @property
    def domain(self) -> Optional[str]:
        """The domain name."""

class PageSpan:
    """Page range in a document.

    Specifies a range of pages in a document.

    Examples:
        >>> span = PageSpan(first_page=1, last_page=5)
    """

    @property
    def first_page(self) -> int:
        """First page number."""

    @property
    def last_page(self) -> int:
        """Last page number."""

class RagChunk:
    """RAG chunk information.

    Text chunk from RAG retrieval with optional page information.

    Examples:
        >>> chunk = RagChunk(
        ...     text="Retrieved text content",
        ...     page_span=PageSpan(first_page=1, last_page=2)
        ... )
    """

    @property
    def text(self) -> str:
        """The chunk text."""

    @property
    def page_span(self) -> Optional[PageSpan]:
        """Page range for this chunk."""

class RetrievedContext:
    """Retrieved context information.

    Context retrieved from a knowledge source.

    Examples:
        >>> context = RetrievedContext(
        ...     uri="https://example.com",
        ...     title="Example",
        ...     text="Retrieved content",
        ...     rag_chunk=RagChunk(...)
        ... )
    """

    @property
    def uri(self) -> Optional[str]:
        """Source URI."""

    @property
    def title(self) -> Optional[str]:
        """Source title."""

    @property
    def text(self) -> Optional[str]:
        """Retrieved text."""

    @property
    def rag_chunk(self) -> Optional[RagChunk]:
        """RAG chunk information."""

class Maps:
    """Google Maps source information.

    Information about a Maps location used for grounding.

    Examples:
        >>> maps = Maps(
        ...     uri="https://maps.google.com/...",
        ...     title="Statue of Liberty",
        ...     place_id="ChIJPTacEpBQwokRKwIlDbbNLlE"
        ... )
    """

    @property
    def uri(self) -> Optional[str]:
        """Maps URI."""

    @property
    def title(self) -> Optional[str]:
        """Location title."""

    @property
    def text(self) -> Optional[str]:
        """Location description."""

    @property
    def place_id(self) -> Optional[str]:
        """Google Maps place ID."""

class GroundingChunkType:
    """Union type for grounding chunk sources.

    Represents different types of grounding sources.

    Examples:
        >>> # Web source
        >>> chunk = GroundingChunkType.Web(Web(...))

        >>> # Retrieved context
        >>> chunk = GroundingChunkType.RetrievedContext(RetrievedContext(...))

        >>> # Maps source
        >>> chunk = GroundingChunkType.Maps(Maps(...))
    """

class GroundingChunk:
    """Grounding chunk wrapper.

    Wraps a grounding chunk source.

    Examples:
        >>> chunk = GroundingChunk(
        ...     chunk_type=GroundingChunkType.Web(Web(...))
        ... )
    """

    @property
    def chunk_type(self) -> GroundingChunkType:
        """The chunk type."""

class GroundingMetadata:
    """Grounding metadata for a response.

    Contains all grounding information including sources, supports, and search queries.

    Examples:
        >>> metadata = GroundingMetadata(
        ...     web_search_queries=["query1", "query2"],
        ...     grounding_chunks=[GroundingChunk(...)],
        ...     grounding_supports=[GroundingSupport(...)]
        ... )
    """

    @property
    def web_search_queries(self) -> Optional[List[str]]:
        """Web search queries used."""

    @property
    def grounding_chunks(self) -> Optional[List[GroundingChunk]]:
        """Grounding source chunks."""

    @property
    def grounding_supports(self) -> Optional[List[GroundingSupport]]:
        """Grounding support information."""

    @property
    def search_entry_point(self) -> Optional[SearchEntryPoint]:
        """Search entry point."""

    @property
    def retrieval_metadata(self) -> Optional[RetrievalMetadata]:
        """Retrieval metadata."""

    @property
    def source_flagging_uris(self) -> Optional[List[SourceFlaggingUri]]:
        """Flagged source URIs."""

    @property
    def google_maps_widget_context_token(self) -> Optional[str]:
        """Maps widget context token."""

class SafetyRating:
    """Safety rating for content.

    Provides detailed safety assessment including probability and severity.

    Examples:
        >>> rating = SafetyRating(
        ...     category=HarmCategory.HarmCategoryHateSpeech,
        ...     probability=HarmProbability.Low,
        ...     probability_score=0.2,
        ...     severity=HarmSeverity.HarmSeverityLow,
        ...     severity_score=0.15,
        ...     blocked=False
        ... )
    """

    @property
    def category(self) -> HarmCategory:
        """Harm category."""

    @property
    def probability(self) -> Optional[HarmProbability]:
        """Harm probability level."""

    @property
    def probability_score(self) -> Optional[float]:
        """Numeric probability score."""

    @property
    def severity(self) -> Optional[HarmSeverity]:
        """Harm severity level."""

    @property
    def severity_score(self) -> Optional[float]:
        """Numeric severity score."""

    @property
    def blocked(self) -> Optional[bool]:
        """Whether content was blocked."""

    @property
    def overwritten_threshold(self) -> Optional[HarmBlockThreshold]:
        """Overwritten threshold for image output."""

class LogprobsCandidate:
    """Log probability information for a token.

    Contains token string, ID, and log probability.

    Examples:
        >>> candidate = LogprobsCandidate(
        ...     token="hello",
        ...     token_id=12345,
        ...     log_probability=-0.5
        ... )
    """

    @property
    def token(self) -> Optional[str]:
        """Token string."""

    @property
    def token_id(self) -> Optional[int]:
        """Token ID."""

    @property
    def log_probability(self) -> Optional[float]:
        """Log probability."""

class TopCandidates:
    """Top token candidates at a decoding step.

    List of top candidates sorted by log probability.

    Examples:
        >>> top = TopCandidates(
        ...     candidates=[
        ...         LogprobsCandidate(token="hello", log_probability=-0.5),
        ...         LogprobsCandidate(token="hi", log_probability=-1.2)
        ...     ]
        ... )
    """

    @property
    def candidates(self) -> Optional[List[LogprobsCandidate]]:
        """List of candidates."""

class LogprobsResult:
    """Complete log probability result.

    Contains both top candidates and chosen tokens with probabilities.

    Examples:
        >>> result = LogprobsResult(
        ...     top_candidates=[TopCandidates(...)],
        ...     chosen_candidates=[LogprobsCandidate(...)]
        ... )
    """

    @property
    def top_candidates(self) -> Optional[List[TopCandidates]]:
        """Top candidates per step."""

    @property
    def chosen_candidates(self) -> Optional[List[LogprobsCandidate]]:
        """Actually chosen tokens."""

class GoogleDate:
    """Date representation.

    Simple date with year, month, and day.

    Examples:
        >>> date = GoogleDate(year=2024, month=12, day=25)
    """

    @property
    def year(self) -> Optional[int]:
        """Year."""

    @property
    def month(self) -> Optional[int]:
        """Month (1-12)."""

    @property
    def day(self) -> Optional[int]:
        """Day of month."""

class Citation:
    """Source citation information.

    Citation for a piece of generated content with source details.

    Examples:
        >>> citation = Citation(
        ...     start_index=10,
        ...     end_index=50,
        ...     uri="https://example.com",
        ...     title="Example Source",
        ...     license="CC-BY-4.0",
        ...     publication_date=GoogleDate(year=2024, month=1, day=1)
        ... )
    """

    @property
    def start_index(self) -> Optional[int]:
        """Start index in content."""

    @property
    def end_index(self) -> Optional[int]:
        """End index in content."""

    @property
    def uri(self) -> Optional[str]:
        """Source URI."""

    @property
    def title(self) -> Optional[str]:
        """Source title."""

    @property
    def license(self) -> Optional[str]:
        """Source license."""

    @property
    def publication_date(self) -> Optional[GoogleDate]:
        """Publication date."""

class CitationMetadata:
    """Collection of citations.

    Contains all citations for a piece of content.

    Examples:
        >>> metadata = CitationMetadata(
        ...     citations=[Citation(...), Citation(...)]
        ... )
    """

    @property
    def citations(self) -> Optional[List[Citation]]:
        """List of citations."""

class Candidate:
    """Response candidate from the model.

    A single generated response option with content and metadata.

    Examples:
        >>> candidate = Candidate(
        ...     index=0,
        ...     content=GeminiContent(...),
        ...     finish_reason=FinishReason.Stop,
        ...     safety_ratings=[SafetyRating(...)],
        ...     citation_metadata=CitationMetadata(...)
        ... )
    """

    @property
    def index(self) -> Optional[int]:
        """Candidate index."""

    @property
    def content(self) -> GeminiContent:
        """Generated content."""

    @property
    def avg_logprobs(self) -> Optional[float]:
        """Average log probability."""

    @property
    def logprobs_result(self) -> Optional[LogprobsResult]:
        """Detailed log probabilities."""

    @property
    def finish_reason(self) -> Optional[FinishReason]:
        """Why generation stopped."""

    @property
    def safety_ratings(self) -> Optional[List[SafetyRating]]:
        """Safety ratings."""

    @property
    def citation_metadata(self) -> Optional[CitationMetadata]:
        """Citation metadata."""

    @property
    def grounding_metadata(self) -> Optional[GroundingMetadata]:
        """Grounding metadata."""

    @property
    def url_context_metadata(self) -> Optional[UrlContextMetadata]:
        """URL context metadata."""

    @property
    def finish_message(self) -> Optional[str]:
        """Detailed finish reason message."""

class GenerateContentResponse:
    """Response from content generation.

    Complete response including candidates, usage, and feedback.

    Examples:
        >>> response = GenerateContentResponse(
        ...     candidates=[Candidate(...)],
        ...     usage_metadata=UsageMetadata(...),
        ...     model_version="gemini-1.5-pro-002"
        ... )
    """

    @property
    def candidates(self) -> List[Candidate]:
        """Generated candidates."""

    @property
    def model_version(self) -> Optional[str]:
        """Model version used."""

    @property
    def create_time(self) -> Optional[str]:
        """Request timestamp."""

    @property
    def response_id(self) -> Optional[str]:
        """Response identifier."""

    @property
    def prompt_feedback(self) -> Optional[PromptFeedback]:
        """Prompt feedback (if blocked)."""

    @property
    def usage_metadata(self) -> Optional[UsageMetadata]:
        """Token usage metadata."""

class PredictRequest:
    """Prediction API request.

    Generic prediction request for embedding and other endpoints.

    Examples:
        >>> request = PredictRequest(
        ...     instances=[{"content": {"parts": [{"text": "Hello"}]}}],
        ...     parameters={"outputDimensionality": 768}
        ... )
    """

    def __init__(
        self,
        instances: Any,
        parameters: Optional[Any] = None,
    ) -> None:
        """Initialize prediction request.

        Args:
            instances (Any):
                Input instances
            parameters (Optional[Any]):
                Request parameters
        """

    @property
    def instances(self) -> Any:
        """Input instances."""

    @property
    def parameters(self) -> Any:
        """Request parameters."""

    def __str__(self) -> str:
        """String representation."""

class PredictResponse:
    """Prediction API response.

    Generic prediction response containing predictions and metadata.

    Examples:
        >>> response = PredictResponse(
        ...     predictions=[{"embedding": {"values": [0.1, 0.2, ...]}}],
        ...     deployed_model_id="12345",
        ...     model="embedding-001"
        ... )
    """

    @property
    def predictions(self) -> Any:
        """Predictions."""

    @property
    def metadata(self) -> Any:
        """Response metadata."""

    @property
    def deployed_model_id(self) -> str:
        """Deployed model ID."""

    @property
    def model(self) -> str:
        """Model name."""

    @property
    def model_version_id(self) -> str:
        """Model version ID."""

    @property
    def model_display_name(self) -> str:
        """Model display name."""

    def __str__(self) -> str:
        """String representation."""

class GeminiEmbeddingConfig:
    """Configuration for Gemini embeddings.

    Configures embedding generation including dimensionality and task type.

    Examples:
        >>> config = GeminiEmbeddingConfig(
        ...     model="text-embedding-004",
        ...     output_dimensionality=768,
        ...     task_type=EmbeddingTaskType.RetrievalDocument
        ... )
    """

    def __init__(
        self,
        model: Optional[str] = None,
        output_dimensionality: Optional[int] = None,
        task_type: Optional[EmbeddingTaskType] = None,
    ) -> None:
        """Initialize embedding configuration.

        Args:
            model (Optional[str]):
                Model name
            output_dimensionality (Optional[int]):
                Output embedding dimensionality
            task_type (Optional[EmbeddingTaskType]):
                Task type for embeddings

        Raises:
            TypeError: If neither model nor task_type is provided
        """

    @property
    def model(self) -> Optional[str]:
        """The model name."""

    @property
    def output_dimensionality(self) -> Optional[int]:
        """Output dimensionality."""

    @property
    def task_type(self) -> Optional[EmbeddingTaskType]:
        """Task type."""

    @property
    def is_configured(self) -> bool:
        """Whether config has parameters set."""

class ContentEmbedding:
    """Content embedding result.

    Contains the embedding vector values.

    Examples:
        >>> embedding = ContentEmbedding(
        ...     values=[0.1, 0.2, 0.3, ...]
        ... )
    """

    @property
    def values(self) -> List[float]:
        """Embedding vector values."""

class GeminiEmbeddingResponse:
    """Response from embedding generation.

    Contains the generated embedding.

    Examples:
        >>> response = GeminiEmbeddingResponse(
        ...     embedding=ContentEmbedding(values=[0.1, 0.2, ...])
        ... )
    """

    @property
    def embedding(self) -> ContentEmbedding:
        """The generated embedding."""

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
    def __init__(
        self, mode: Optional[Mode], allowed_function_names: Optional[list[str]]
    ) -> None: ...

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
    def __init__(
        self, instances: List[dict], parameters: Optional[dict] = None
    ) -> None:
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
    def __init__(
        self, content: str | ImageUrl | AudioUrl | BinaryContent | DocumentUrl
    ) -> None:
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
        model_settings: Optional[
            ModelSettings | OpenAIChatSettings | GeminiSettings
        ] = None,
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
    def model_validate_json(
        json_string: str, output_types: Optional[Dict[str, Any]]
    ) -> "Workflow":
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

class WorkflowTask:
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
    def tasks(self) -> Dict[str, WorkflowTask]:
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
    "WorkflowTask",
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
