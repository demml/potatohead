use crate::anthropic::v1::request::{AnthropicSettings, MessageParam as AnthropicMessage};
use crate::error::TypeError;
use crate::google::v1::generate::request::{GeminiContent, GeminiSettings};
use crate::openai::v1::chat::request::ChatMessage as OpenAIChatMessage;
use crate::openai::v1::chat::settings::OpenAIChatSettings;
use crate::prompt::builder::{to_provider_request, ProviderRequest};
use crate::prompt::settings::ModelSettings;
use crate::prompt::types::parse_response_to_json;
use crate::prompt::types::ResponseType;
use crate::prompt::types::Role;
use crate::prompt::MessageNum;
use crate::traits::MessageFactory;
use crate::SettingsType;
use crate::{Provider, SaveName};
use potato_macro::try_extract_message;
use potato_util::utils::extract_string_value;
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString, PyTuple};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

fn create_message_for_provider(
    content: String,
    provider: &Provider,
    role: &str,
) -> Result<MessageNum, TypeError> {
    match provider {
        Provider::OpenAI => {
            OpenAIChatMessage::from_text(content, role).map(MessageNum::OpenAIMessageV1)
        }
        Provider::Anthropic => {
            AnthropicMessage::from_text(content, role).map(MessageNum::AnthropicMessageV1)
        }
        Provider::Gemini | Provider::Google | Provider::Vertex => {
            GeminiContent::from_text(content, role).map(MessageNum::GeminiContentV1)
        }
        _ => Err(TypeError::Error(format!(
            "Unsupported provider for message creation: {:?}",
            provider
        ))),
    }
}

fn parse_single_message(
    message: &Bound<'_, PyAny>,
    provider: &Provider,
    default_role: &str,
) -> Result<MessageNum, TypeError> {
    // String conversion (most common case)
    if message.is_instance_of::<PyString>() {
        let text = message.extract::<String>()?;
        return create_message_for_provider(text, provider, default_role);
    }

    // Try each message type using macro
    try_extract_message!(
        message,
        OpenAIChatMessage => MessageNum::OpenAIMessageV1,
        AnthropicMessage => MessageNum::AnthropicMessageV1,
        GeminiContent => MessageNum::GeminiContentV1,
    );

    Err(TypeError::InvalidMessageTypeInList(
        message.get_type().name()?.to_string(),
    ))
}

fn parse_messages(
    messages: &Bound<'_, PyAny>,
    provider: &Provider,
    default_role: &str,
) -> Result<Vec<MessageNum>, TypeError> {
    // Single message
    let mut messages =
        if !messages.is_instance_of::<PyList>() && !messages.is_instance_of::<PyTuple>() {
            vec![parse_single_message(messages, provider, default_role)?]
        } else {
            // List/tuple of messages
            messages
                .try_iter()?
                .map(|item| {
                    let item = item?;
                    parse_single_message(&item, provider, default_role)
                })
                .collect::<Result<Vec<_>, _>>()?
        };

    // Convert Anthropic system messages to TextBlockParam format
    // optimize this later - maybe
    if provider == &Provider::Anthropic
        && (default_role == Role::System.as_str()
            || default_role == Role::Assistant.as_str()
            || default_role == Role::Developer.as_str())
    {
        for msg in messages.iter_mut() {
            msg.anthropic_message_to_system_message()?;
        }
    }

    Ok(messages)
}

fn get_system_role(provider: &Provider) -> &'static str {
    match provider {
        Provider::OpenAI | Provider::Gemini | Provider::Vertex | Provider::Google => {
            Role::Developer.into()
        }
        Provider::Anthropic => Role::Assistant.into(),
        _ => Role::System.into(),
    }
}

/// Helper for extracting system instructions from optional parameter
pub fn extract_system_instructions(
    system_instruction: Option<&Bound<'_, PyAny>>,
    provider: &Provider,
) -> Result<Option<Vec<MessageNum>>, TypeError> {
    let system_instructions = if let Some(sys_inst) = system_instruction {
        Some(parse_messages(
            sys_inst,
            provider,
            get_system_role(provider),
        )?)
    } else {
        None
    };

    Ok(system_instructions)
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Prompt {
    pub request: ProviderRequest,

    #[pyo3(get)]
    pub model: String,

    #[pyo3(get)]
    pub provider: Provider,

    pub version: String,

    #[pyo3(get)]
    pub parameters: Vec<String>,

    pub response_type: ResponseType,
}

/// ModelSettings variant based on the type of settings provided.
fn extract_model_settings(model_settings: &Bound<'_, PyAny>) -> Result<ModelSettings, TypeError> {
    let settings_type = model_settings
        .call_method0("settings_type")?
        .extract::<SettingsType>()?;

    match settings_type {
        SettingsType::OpenAIChat => model_settings
            .extract::<OpenAIChatSettings>()
            .map(ModelSettings::OpenAIChat),
        SettingsType::GoogleChat => model_settings
            .extract::<GeminiSettings>()
            .map(ModelSettings::GoogleChat),
        SettingsType::Anthropic => model_settings
            .extract::<AnthropicSettings>()
            .map(ModelSettings::AnthropicChat),
        SettingsType::ModelSettings => model_settings.extract::<ModelSettings>(),
    }
    .map_err(Into::into)
}

#[pymethods]
impl Prompt {
    /// Creates a new Prompt object.
    /// Main parsing logic is as follows:
    /// 1. Extract model settings if provided, otherwise use provider default settings.
    /// 2. Message and system instructions are expected to be a variant of MessageNum (OpenAIChatMessage, AnthropicMessage or GeminiContent).
    /// 3. On instantiation, message will be check if is_instance_of pystring. If pystring, provider will be used to map to appropriate message Text type
    /// 4. If message is a pylist, each item will be checked for is_instance_of pystring or MessageNum variant and converted accordingly.
    /// 5. If message is a single MessageNum variant, it will be extracted and wrapped in a vec.
    /// 6. After messages are parsed, a full provider request struct will by built using to_provider_request function.
    /// # Arguments:
    /// * `message`: A single message or list of messages representing user input.
    /// * `model`: The model identifier to use for the prompt.
    /// * `provider`: The provider to use for the prompt.
    /// * `system_instruction`: Optional system instruction message or list of messages.
    /// * `model_settings`: Optional model settings to use for the prompt.
    /// * `output_type`: Optional output type to enforce structured output.
    #[new]
    #[pyo3(signature = (message, model, provider, system_instruction=None, model_settings=None, output_type=None))]
    pub fn new(
        py: Python<'_>,
        message: &Bound<'_, PyAny>,
        model: &str,
        provider: &Bound<'_, PyAny>,
        system_instruction: Option<&Bound<'_, PyAny>>,
        model_settings: Option<&Bound<'_, PyAny>>,
        output_type: Option<&Bound<'_, PyAny>>, // can be a pydantic model or one of Opsml's predefined outputs
    ) -> Result<Self, TypeError> {
        // 1. get model settings if provided
        let model_settings = model_settings
            .as_ref()
            .map(|s| extract_model_settings(s))
            .transpose()?;

        // 2. extract provider
        let provider = Provider::extract_provider(provider)?;

        // 3. Parse user messages with "user" role
        // We'll use this to figure out the type of request struct to create
        let messages = parse_messages(message, &provider, Role::User.into())?;
        let system_instructions = if let Some(sys_inst) = system_instruction {
            parse_messages(sys_inst, &provider, get_system_role(&provider))?
        } else {
            vec![]
        };

        // 4.  validate response_json_schema
        let (response_type, response_json_schema) = match output_type {
            Some(output_type) => {
                // check if output_type is a pydantic model and extract the model json schema
                parse_response_to_json(py, output_type)?
            }
            None => (ResponseType::Null, None),
        };

        Self::new_rs(
            messages,
            model,
            provider,
            system_instructions,
            model_settings,
            response_json_schema,
            response_type,
        )
    }

    #[getter]
    pub fn model_settings<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        self.request.model_settings(py)
    }

    #[getter]
    pub fn model_identifier(&self) -> String {
        format!("{}:{}", self.provider.as_str(), self.model)
    }

    #[pyo3(signature = (path = None))]
    pub fn save_prompt(&self, path: Option<PathBuf>) -> PyResult<PathBuf> {
        let save_path = path.unwrap_or_else(|| PathBuf::from(SaveName::Prompt));
        PyHelperFuncs::save_to_json(self, &save_path)?;
        Ok(save_path)
    }

    #[staticmethod]
    pub fn from_path(path: PathBuf) -> Result<Self, TypeError> {
        // Load the JSON file from the path
        let file = std::fs::read_to_string(&path)?;

        // Parse the JSON file into a Prompt
        Ok(serde_json::from_str(&file)?)
    }

    #[staticmethod]
    pub fn model_validate_json(json_string: String) -> Result<Self, TypeError> {
        let json_value: Value = serde_json::from_str(&json_string)?;
        let model: Self = serde_json::from_value(json_value)?;

        Ok(model)
    }

    pub fn model_dump_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }

    #[getter]
    pub fn messages<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyList>, TypeError> {
        self.request.get_py_messages(py)
    }

    #[getter]
    pub fn system_instructions<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyList>, TypeError> {
        self.request.get_py_system_instructions(py)
    }

    /// Binds a variable in the prompt to a value. This will return a new Prompt with the variable bound to the value.
    /// This will iterate over all user messages and bind the variable in each message.
    /// # Arguments:
    /// * `name`: The name of the variable to bind.
    /// * `value`: The value to bind the variable to.
    /// # Returns:
    /// * `Result<Self, PromptError>`: Returns a new Prompt with the variable bound to the value.
    #[pyo3(signature = (name=None, value=None, **kwargs))]
    pub fn bind(
        &self,
        name: Option<&str>,
        value: Option<&Bound<'_, PyAny>>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> Result<Self, TypeError> {
        let mut new_prompt = self.clone();

        if let (Some(name), Some(value)) = (name, value) {
            let var_value = extract_string_value(value)?;
            for message in new_prompt.request.messages_mut() {
                message.bind_mut(name, &var_value)?;
            }
        }

        if let Some(kwargs) = kwargs {
            for (key, val) in kwargs.iter() {
                let var_name = key.extract::<String>()?;
                let var_value = extract_string_value(&val)?;

                for message in new_prompt.request.messages_mut() {
                    message.bind_mut(&var_name, &var_value)?;
                }
            }
        }

        if name.is_none() && kwargs.is_none_or(|k| k.is_empty()) {
            return Err(TypeError::Error(
                "Must provide either (name, value) or keyword arguments for binding".to_string(),
            ));
        }

        Ok(new_prompt)
    }

    /// Binds a variable in the prompt to a value. This will mutate the current Prompt and bind the variable in each user message.
    /// # Arguments:
    /// * `name`: The name of the variable to bind.
    /// * `value`: The value to bind the variable to.
    /// # Returns:
    /// * `Result<(), PromptError>`: Returns Ok(()) on success or an error if the binding fails.
    #[pyo3(signature = (name=None, value=None, **kwargs))]
    pub fn bind_mut(
        &mut self,
        name: Option<&str>,
        value: Option<&Bound<'_, PyAny>>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> Result<(), TypeError> {
        if let (Some(name), Some(value)) = (name, value) {
            let var_value = extract_string_value(value)?;
            for message in self.request.messages_mut() {
                message.bind_mut(name, &var_value)?;
            }
        }

        if let Some(kwargs) = kwargs {
            for (key, val) in kwargs.iter() {
                let var_name = key.extract::<String>()?;
                let var_value = extract_string_value(&val)?;

                for message in self.request.messages_mut() {
                    message.bind_mut(&var_name, &var_value)?;
                }
            }
        }

        if name.is_none() && kwargs.is_none_or(|k| k.is_empty()) {
            return Err(TypeError::Error(
                "Must provide either (name, value) or keyword arguments for binding".to_string(),
            ));
        }

        Ok(())
    }

    #[getter]
    pub fn response_json_schema(&self) -> Option<String> {
        Some(PyHelperFuncs::__str__(
            self.request.response_json_schema().as_ref()?,
        ))
    }
}

impl Prompt {
    pub fn new_rs(
        messages: Vec<MessageNum>,
        model: &str,
        provider: Provider,
        system_instructions: Vec<MessageNum>,
        model_settings: Option<ModelSettings>,
        response_json_schema: Option<Value>,
        response_type: ResponseType,
    ) -> Result<Self, TypeError> {
        let model = model.to_string();
        // get version from crate
        let version = potato_util::version();
        // If model_settings is not provided, set model and provider to undefined if missing
        let model_settings = match model_settings {
            Some(settings) => {
                // validates if provider and settings are compatible
                settings.validate_provider(&provider)?;
                settings
            }
            None => ModelSettings::provider_default_settings(&provider),
        };

        // extract named parameters in prompt
        let parameters = Self::extract_variables(&messages, &system_instructions);

        // Build the provider request
        let request = to_provider_request(
            messages,
            system_instructions,
            model.clone(),
            model_settings,
            response_json_schema,
        )?;

        Ok(Self {
            request,
            version,
            parameters,
            response_type,
            model,
            provider,
        })
    }

    pub fn extract_variables(
        messages: &[MessageNum],
        system_instructions: &[MessageNum],
    ) -> Vec<String> {
        let mut variables = HashSet::new();

        // Extract from system instructions
        for msg in system_instructions {
            variables.extend(msg.extract_variables());
        }

        // Extract from user messages
        for msg in messages {
            variables.extend(msg.extract_variables());
        }

        variables.into_iter().collect()
    }

    pub fn model_dump_value(&self) -> Value {
        // Convert the Prompt to a JSON Value
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    pub fn to_request_json(&self) -> Result<Value, TypeError> {
        // Convert the Prompt to a JSON Value
        let json_value = serde_json::to_value(self)?;

        Ok(json_value)
    }

    pub fn set_response_json_schema(
        &mut self,
        response_json_schema: Option<Value>,
        response_type: ResponseType,
    ) {
        self.request.set_response_json_schema(response_json_schema);
        self.response_type = response_type;
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::anthropic::v1::request::{
        Base64ImageSource, Base64PDFSource, ContentBlockParam, DocumentBlockParam, ImageBlockParam,
        MessageParam, PlainTextSource, TextBlockParam, UrlImageSource, UrlPDFSource,
    };
    use crate::google::{DataNum, GeminiContent, Part};
    use crate::openai::v1::chat::request::{
        ChatMessage as OpenAIChatMessage, ContentPart, FileContentPart, ImageContentPart,
        TextContentPart,
    };
    use crate::prompt::types::Score;
    use crate::StructuredOutput;

    fn create_openai_chat_message() -> OpenAIChatMessage {
        let text_part = TextContentPart::new("What company is this logo from?".to_string());
        let text_content_part = ContentPart::Text(text_part);
        OpenAIChatMessage {
            role: "user".to_string(),
            content: vec![text_content_part],
            name: None,
        }
    }

    fn create_system_openai_chat_message() -> OpenAIChatMessage {
        let text_part = TextContentPart::new("system_prompt".to_string());
        let text_content_part = ContentPart::Text(text_part);
        OpenAIChatMessage {
            role: "developer".to_string(),
            content: vec![text_content_part],
            name: None,
        }
    }

    fn create_openai_image_message() -> OpenAIChatMessage {
        let image_part = ImageContentPart::new("https://iili.io/3Hs4FMg.png".to_string(), None);
        let image_content_part = ContentPart::ImageUrl(image_part);
        OpenAIChatMessage {
            role: "user".to_string(),
            content: vec![image_content_part],
            name: None,
        }
    }

    fn create_openai_file_message() -> OpenAIChatMessage {
        let file_part = FileContentPart::new(
            Some("filedata".to_string()),
            Some("fileid".to_string()),
            Some("filename".to_string()),
        );
        let file_content_part = ContentPart::FileContent(file_part);
        OpenAIChatMessage {
            role: "user".to_string(),
            content: vec![file_content_part],
            name: None,
        }
    }

    fn create_anthropic_text_message() -> MessageParam {
        let text_block =
            TextBlockParam::new_rs("What company is this logo from?".to_string(), None, None);
        MessageParam {
            role: "user".to_string(),
            content: vec![ContentBlockParam {
                inner: crate::anthropic::v1::request::ContentBlock::Text(text_block),
            }],
        }
    }

    fn create_anthropic_system_message() -> MessageParam {
        let text_block = TextBlockParam::new_rs("system_prompt".to_string(), None, None);
        MessageParam {
            role: "assistant".to_string(),
            content: vec![ContentBlockParam {
                inner: crate::anthropic::v1::request::ContentBlock::Text(text_block),
            }],
        }
    }

    fn create_anthropic_base64_image_message() -> MessageParam {
        let image_source =
            Base64ImageSource::new("image/png".to_string(), "base64data".to_string()).unwrap();
        let image_block = ImageBlockParam {
            source: crate::anthropic::v1::request::ImageSource::Base64(image_source),
            cache_control: None,
            r#type: "image".to_string(),
        };
        MessageParam {
            role: "user".to_string(),
            content: vec![ContentBlockParam {
                inner: crate::anthropic::v1::request::ContentBlock::Image(image_block),
            }],
        }
    }

    fn create_anthropic_url_image_message() -> MessageParam {
        let image_source = UrlImageSource::new("https://iili.io/3Hs4FMg.png".to_string());
        let image_block = ImageBlockParam {
            source: crate::anthropic::v1::request::ImageSource::Url(image_source),
            cache_control: None,
            r#type: "image".to_string(),
        };
        MessageParam {
            role: "user".to_string(),
            content: vec![ContentBlockParam {
                inner: crate::anthropic::v1::request::ContentBlock::Image(image_block),
            }],
        }
    }

    fn create_anthropic_base64_pdf_message() -> MessageParam {
        let pdf_source = Base64PDFSource::new("base64pdfdata".to_string()).unwrap();
        let document_block = DocumentBlockParam {
            source: crate::anthropic::v1::request::DocumentSource::Base64(pdf_source),
            cache_control: None,
            title: Some("test_document.pdf".to_string()),
            context: None,
            r#type: "document".to_string(),
            citations: None,
        };
        MessageParam {
            role: "user".to_string(),
            content: vec![ContentBlockParam {
                inner: crate::anthropic::v1::request::ContentBlock::Document(document_block),
            }],
        }
    }

    fn create_anthropic_url_pdf_message() -> MessageParam {
        let pdf_source = UrlPDFSource::new("https://example.com/document.pdf".to_string());
        let document_block = DocumentBlockParam {
            source: crate::anthropic::v1::request::DocumentSource::Url(pdf_source),
            cache_control: None,
            title: Some("test_document.pdf".to_string()),
            context: None,
            r#type: "document".to_string(),
            citations: None,
        };
        MessageParam {
            role: "user".to_string(),
            content: vec![ContentBlockParam {
                inner: crate::anthropic::v1::request::ContentBlock::Document(document_block),
            }],
        }
    }

    fn create_anthropic_plain_text_document_message() -> MessageParam {
        let text_source = PlainTextSource::new("Plain text document content".to_string());
        let document_block = DocumentBlockParam {
            source: crate::anthropic::v1::request::DocumentSource::Text(text_source),
            cache_control: None,
            title: Some("text_document.txt".to_string()),
            context: Some("Context for the document".to_string()),
            r#type: "document".to_string(),
            citations: None,
        };
        MessageParam {
            role: "user".to_string(),
            content: vec![ContentBlockParam {
                inner: crate::anthropic::v1::request::ContentBlock::Document(document_block),
            }],
        }
    }

    #[test]
    fn test_task_list_add_and_get() {
        let text_part = TextContentPart::new("Test prompt. ${param1} ${param2}".to_string());
        let content_part = ContentPart::Text(text_part);
        let message = OpenAIChatMessage {
            role: "user".to_string(),
            content: vec![content_part],
            name: None,
        };

        let prompt = Prompt::new_rs(
            vec![MessageNum::OpenAIMessageV1(message)],
            "gpt-4o",
            Provider::OpenAI,
            vec![],
            None,
            None,
            ResponseType::Null,
        )
        .unwrap();

        // Check if the prompt was created successfully
        assert_eq!(prompt.request.messages().len(), 1);

        // check prompt parameters
        assert!(prompt.parameters.len() == 2);

        // sort parameters to ensure order does not affect the test
        let mut parameters = prompt.parameters.clone();
        parameters.sort();

        assert_eq!(parameters[0], "param1");
        assert_eq!(parameters[1], "param2");

        // bind parameter
        let bound_msg = prompt.request.messages()[0]
            .bind("param1", "Value1")
            .unwrap();
        let bound_msg = bound_msg.bind("param2", "Value2").unwrap();

        // Check if the bound message contains the correct values
        match bound_msg.clone() {
            MessageNum::OpenAIMessageV1(msg) => {
                if let ContentPart::Text(text_part) = &msg.content[0] {
                    assert_eq!(text_part.text, "Test prompt. Value1 Value2");
                } else {
                    panic!("Expected TextContentPart");
                }
            }
            _ => panic!("Expected OpenAIMessageV1"),
        }
    }

    #[test]
    fn test_image_prompt() {
        let text_message = create_openai_chat_message();
        let image_message = create_openai_image_message();

        let system_text_part = TextContentPart::new("system_prompt".to_string());
        let system_text_content_part = ContentPart::Text(system_text_part);

        let system_text_message = OpenAIChatMessage {
            role: "assistant".to_string(),
            content: vec![system_text_content_part],
            name: None,
        };

        let prompt = Prompt::new_rs(
            vec![
                MessageNum::OpenAIMessageV1(text_message),
                MessageNum::OpenAIMessageV1(image_message),
            ],
            "gpt-4o",
            Provider::OpenAI,
            vec![MessageNum::OpenAIMessageV1(system_text_message)],
            None,
            None,
            ResponseType::Null,
        )
        .unwrap();

        // Check the first user message
        if let MessageNum::OpenAIMessageV1(msg) = &prompt.request.messages()[1] {
            if let ContentPart::Text(text_part) = &msg.content[0] {
                assert_eq!(text_part.text, "What company is this logo from?");
            } else {
                panic!("Expected TextContentPart for the first user message");
            }
        } else {
            panic!("Expected OpenAIMessageV1 for the first user message");
        }

        // Check the second user message (ImageUrl)
        if let MessageNum::OpenAIMessageV1(msg) = &prompt.request.messages()[2] {
            if let ContentPart::ImageUrl(image_url) = &msg.content[0] {
                assert_eq!(image_url.image_url.url, "https://iili.io/3Hs4FMg.png");
                assert_eq!(image_url.r#type, "image_url");
            } else {
                panic!("Expected ContentPart::Image for the second user message");
            }
        } else {
            panic!("Expected OpenAIMessageV1 for the second user message");
        }
    }

    #[test]
    fn test_document_prompt() {
        let text_message = create_openai_chat_message();
        let file_message = create_openai_file_message();
        let system_message = create_system_openai_chat_message();

        let prompt = Prompt::new_rs(
            vec![
                MessageNum::OpenAIMessageV1(text_message),
                MessageNum::OpenAIMessageV1(file_message),
            ],
            "gpt-4o",
            Provider::OpenAI,
            vec![MessageNum::OpenAIMessageV1(system_message)],
            None,
            None,
            ResponseType::Null,
        )
        .unwrap();

        // Check the 2nd user message (file)
        if let MessageNum::OpenAIMessageV1(msg) = &prompt.request.messages()[2] {
            if let ContentPart::FileContent(file_content) = &msg.content[0] {
                assert_eq!(file_content.file.file_id.as_ref().unwrap(), "fileid");
                assert_eq!(file_content.file.filename.as_ref().unwrap(), "filename");
            } else {
                panic!("Expected ContentPart::FileContent for the second user message");
            }
        } else {
            panic!("Expected OpenAIMessageV1 for the first user message");
        }
    }

    #[test]
    fn test_response_format_score() {
        let text_message = create_openai_chat_message();
        let prompt = Prompt::new_rs(
            vec![MessageNum::OpenAIMessageV1(text_message)],
            "gpt-4o",
            Provider::OpenAI,
            vec![],
            None,
            Some(Score::get_structured_output_schema()),
            ResponseType::Null,
        )
        .unwrap();

        // Check if the response json schema is set correctly
        assert!(prompt.response_json_schema().is_some());
    }

    #[test]
    fn test_anthropic_text_message_binding() {
        let text_block =
            TextBlockParam::new_rs("Test prompt. ${param1} ${param2}".to_string(), None, None);
        let message = MessageParam {
            role: "user".to_string(),
            content: vec![ContentBlockParam {
                inner: crate::anthropic::v1::request::ContentBlock::Text(text_block),
            }],
        };

        let prompt = Prompt::new_rs(
            vec![MessageNum::AnthropicMessageV1(message)],
            "claude-3-5-sonnet-20241022",
            Provider::Anthropic,
            vec![],
            None,
            None,
            ResponseType::Null,
        )
        .unwrap();

        assert_eq!(prompt.request.messages().len(), 1);
        assert_eq!(prompt.parameters.len(), 2);

        let mut parameters = prompt.parameters.clone();
        parameters.sort();
        assert_eq!(parameters[0], "param1");
        assert_eq!(parameters[1], "param2");

        // Test parameter binding
        let bound_msg = prompt.request.messages()[0]
            .bind("param1", "Value1")
            .unwrap();
        let bound_msg = bound_msg.bind("param2", "Value2").unwrap();

        match bound_msg {
            MessageNum::AnthropicMessageV1(msg) => {
                if let crate::anthropic::v1::request::ContentBlock::Text(text_block) =
                    &msg.content[0].inner
                {
                    assert_eq!(text_block.text, "Test prompt. Value1 Value2");
                } else {
                    panic!("Expected TextBlockParam");
                }
            }
            _ => panic!("Expected AnthropicMessageV1"),
        }
    }

    #[test]
    fn test_anthropic_url_image_prompt() {
        let text_message = create_anthropic_text_message();
        let image_message = create_anthropic_url_image_message();
        let system_message = create_anthropic_system_message();

        let prompt = Prompt::new_rs(
            vec![
                MessageNum::AnthropicMessageV1(text_message),
                MessageNum::AnthropicMessageV1(image_message),
            ],
            "claude-3-5-sonnet-20241022",
            Provider::Anthropic,
            vec![MessageNum::AnthropicMessageV1(system_message)],
            None,
            None,
            ResponseType::Null,
        )
        .unwrap();

        // Check first message (text)
        if let MessageNum::AnthropicMessageV1(msg) = &prompt.request.messages()[0] {
            if let crate::anthropic::v1::request::ContentBlock::Text(text_block) =
                &msg.content[0].inner
            {
                assert_eq!(text_block.text, "What company is this logo from?");
            } else {
                panic!("Expected TextBlock for first message");
            }
        } else {
            panic!("Expected AnthropicMessageV1");
        }

        // Check second message (image URL)
        if let MessageNum::AnthropicMessageV1(msg) = &prompt.request.messages()[1] {
            if let crate::anthropic::v1::request::ContentBlock::Image(image_block) =
                &msg.content[0].inner
            {
                match &image_block.source {
                    crate::anthropic::v1::request::ImageSource::Url(url_source) => {
                        assert_eq!(url_source.url, "https://iili.io/3Hs4FMg.png");
                        assert_eq!(url_source.r#type, "url");
                    }
                    _ => panic!("Expected URL image source"),
                }
                assert_eq!(image_block.r#type, "image");
            } else {
                panic!("Expected ImageBlock for second message");
            }
        } else {
            panic!("Expected AnthropicMessageV1");
        }
    }

    #[test]
    fn test_anthropic_base64_image_prompt() {
        let text_message = create_anthropic_text_message();
        let image_message = create_anthropic_base64_image_message();

        let prompt = Prompt::new_rs(
            vec![
                MessageNum::AnthropicMessageV1(text_message),
                MessageNum::AnthropicMessageV1(image_message),
            ],
            "claude-3-5-sonnet-20241022",
            Provider::Anthropic,
            vec![],
            None,
            None,
            ResponseType::Null,
        )
        .unwrap();

        // Check second message (base64 image)
        if let MessageNum::AnthropicMessageV1(msg) = &prompt.request.messages()[1] {
            if let crate::anthropic::v1::request::ContentBlock::Image(image_block) =
                &msg.content[0].inner
            {
                match &image_block.source {
                    crate::anthropic::v1::request::ImageSource::Base64(base64_source) => {
                        assert_eq!(base64_source.media_type, "image/png");
                        assert_eq!(base64_source.data, "base64data");
                        assert_eq!(base64_source.r#type, "base64");
                    }
                    _ => panic!("Expected Base64 image source"),
                }
            } else {
                panic!("Expected ImageBlock");
            }
        } else {
            panic!("Expected AnthropicMessageV1");
        }
    }

    // Test: Anthropic PDF document (base64)
    #[test]
    fn test_anthropic_base64_pdf_document_prompt() {
        let text_message = create_anthropic_text_message();
        let pdf_message = create_anthropic_base64_pdf_message();
        let system_message = create_anthropic_system_message();

        let prompt = Prompt::new_rs(
            vec![
                MessageNum::AnthropicMessageV1(text_message),
                MessageNum::AnthropicMessageV1(pdf_message),
            ],
            "claude-3-5-sonnet-20241022",
            Provider::Anthropic,
            vec![MessageNum::AnthropicMessageV1(system_message)],
            None,
            None,
            ResponseType::Null,
        )
        .unwrap();

        // Check second message (PDF document)
        if let MessageNum::AnthropicMessageV1(msg) = &prompt.request.messages()[1] {
            if let crate::anthropic::v1::request::ContentBlock::Document(document_block) =
                &msg.content[0].inner
            {
                match &document_block.source {
                    crate::anthropic::v1::request::DocumentSource::Base64(pdf_source) => {
                        assert_eq!(pdf_source.media_type, "application/pdf");
                        assert_eq!(pdf_source.data, "base64pdfdata");
                        assert_eq!(pdf_source.r#type, "base64");
                    }
                    _ => panic!("Expected Base64 PDF source"),
                }
                assert_eq!(document_block.r#type, "document");
                assert_eq!(document_block.title.as_ref().unwrap(), "test_document.pdf");
            } else {
                panic!("Expected DocumentBlock");
            }
        } else {
            panic!("Expected AnthropicMessageV1");
        }
    }

    // Test: Anthropic URL PDF document
    #[test]
    fn test_anthropic_url_pdf_document_prompt() {
        let text_message = create_anthropic_text_message();
        let pdf_message = create_anthropic_url_pdf_message();

        let prompt = Prompt::new_rs(
            vec![
                MessageNum::AnthropicMessageV1(text_message),
                MessageNum::AnthropicMessageV1(pdf_message),
            ],
            "claude-3-5-sonnet-20241022",
            Provider::Anthropic,
            vec![],
            None,
            None,
            ResponseType::Null,
        )
        .unwrap();

        // Check second message (URL PDF)
        if let MessageNum::AnthropicMessageV1(msg) = &prompt.request.messages()[1] {
            if let crate::anthropic::v1::request::ContentBlock::Document(document_block) =
                &msg.content[0].inner
            {
                match &document_block.source {
                    crate::anthropic::v1::request::DocumentSource::Url(url_source) => {
                        assert_eq!(url_source.url, "https://example.com/document.pdf");
                        assert_eq!(url_source.r#type, "url");
                    }
                    _ => panic!("Expected URL PDF source"),
                }
            } else {
                panic!("Expected DocumentBlock");
            }
        } else {
            panic!("Expected AnthropicMessageV1");
        }
    }

    // Test: Anthropic plain text document
    #[test]
    fn test_anthropic_plain_text_document_prompt() {
        let text_message = create_anthropic_text_message();
        let text_doc_message = create_anthropic_plain_text_document_message();

        let prompt = Prompt::new_rs(
            vec![
                MessageNum::AnthropicMessageV1(text_message),
                MessageNum::AnthropicMessageV1(text_doc_message),
            ],
            "claude-3-5-sonnet-20241022",
            Provider::Anthropic,
            vec![],
            None,
            None,
            ResponseType::Null,
        )
        .unwrap();

        // Check second message (plain text document)
        if let MessageNum::AnthropicMessageV1(msg) = &prompt.request.messages()[1] {
            if let crate::anthropic::v1::request::ContentBlock::Document(document_block) =
                &msg.content[0].inner
            {
                match &document_block.source {
                    crate::anthropic::v1::request::DocumentSource::Text(text_source) => {
                        assert_eq!(text_source.media_type, "text/plain");
                        assert_eq!(text_source.data, "Plain text document content");
                        assert_eq!(text_source.r#type, "text");
                    }
                    _ => panic!("Expected Text document source"),
                }
                assert_eq!(
                    document_block.context.as_ref().unwrap(),
                    "Context for the document"
                );
            } else {
                panic!("Expected DocumentBlock");
            }
        } else {
            panic!("Expected AnthropicMessageV1");
        }
    }

    // Test: Mixed Anthropic content (text + multiple documents)
    #[test]
    fn test_anthropic_mixed_content_prompt() {
        let text_message = create_anthropic_text_message();
        let pdf_message = create_anthropic_base64_pdf_message();
        let text_doc_message = create_anthropic_plain_text_document_message();
        let system_message = create_anthropic_system_message();

        let prompt = Prompt::new_rs(
            vec![
                MessageNum::AnthropicMessageV1(text_message),
                MessageNum::AnthropicMessageV1(pdf_message),
                MessageNum::AnthropicMessageV1(text_doc_message),
            ],
            "claude-3-5-sonnet-20241022",
            Provider::Anthropic,
            vec![MessageNum::AnthropicMessageV1(system_message)],
            None,
            None,
            ResponseType::Null,
        )
        .unwrap();

        assert_eq!(prompt.request.messages().len(), 3);
        assert_eq!(prompt.request.system_instructions().len(), 1);
        assert_eq!(prompt.provider, Provider::Anthropic);
        assert_eq!(prompt.model, "claude-3-5-sonnet-20241022");
    }

    // gemini test
    #[test]
    fn test_gemini_chat_message() {
        let text = Part::from_text("Test prompt. ${param1} ${param2}".to_string());
        let message = GeminiContent {
            role: "user".to_string(),
            parts: vec![text],
        };

        let prompt = Prompt::new_rs(
            vec![MessageNum::GeminiContentV1(message)],
            "gemini-1.5-pro",
            Provider::Google,
            vec![],
            None,
            None,
            ResponseType::Null,
        )
        .unwrap();

        assert_eq!(prompt.request.messages().len(), 1);
        assert_eq!(prompt.parameters.len(), 2);

        let mut parameters = prompt.parameters.clone();
        parameters.sort();
        assert_eq!(parameters[0], "param1");
        assert_eq!(parameters[1], "param2");

        // Test parameter binding
        let bound_msg = prompt.request.messages()[0]
            .bind("param1", "Value1")
            .unwrap();
        let bound_msg = bound_msg.bind("param2", "Value2").unwrap();

        match bound_msg {
            MessageNum::GeminiContentV1(msg) => {
                if let DataNum::Text(text_part) = &msg.parts[0].data {
                    assert_eq!(text_part, "Test prompt. Value1 Value2");
                } else {
                    panic!("Expected Text Part");
                }
            }
            _ => panic!("Expected GeminiContentV1"),
        }
    }
}
