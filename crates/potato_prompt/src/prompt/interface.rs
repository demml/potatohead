use crate::prompt::error::PromptError;
use crate::prompt::types::parse_response_to_json;
use crate::prompt::types::{Message, Role};
use crate::prompt::ResponseType;
use potato_type::google::chat::GeminiSettings;
use potato_type::openai::chat::OpenAIChatSettings;
use potato_type::SettingsType;
use potato_type::{Provider, SaveName};

use crate::prompt::settings::ModelSettings;
use potato_util::utils::extract_string_value;
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString, PyTuple};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Prompt {
    #[pyo3(get, set)]
    pub message: Vec<Message>,

    #[pyo3(get, set)]
    pub system_instruction: Vec<Message>,

    #[pyo3(get, set)]
    pub model_settings: ModelSettings,

    #[pyo3(get, set)]
    pub model: String,

    #[pyo3(get, set)]
    pub provider: Provider,

    pub version: String,

    pub response_json_schema: Option<Value>,

    pub parameters: Vec<String>,

    pub response_type: ResponseType,
}

pub fn parse_prompt(messages: &Bound<'_, PyAny>) -> Result<Vec<Message>, PromptError> {
    if messages.is_instance_of::<Message>() {
        return Ok(vec![messages.extract::<Message>()?]);
    }

    if messages.is_instance_of::<PyString>() {
        return Ok(vec![Message::new(messages)?]);
    }

    let initial_capacity = messages.len().unwrap_or(1);
    let mut revised_messages = Vec::with_capacity(initial_capacity);

    // Explicitly check for list or tuple
    if messages.is_instance_of::<PyList>() || messages.is_instance_of::<PyTuple>() {
        for item in messages.try_iter()? {
            match item {
                Ok(item) => {
                    revised_messages.push(if item.is_instance_of::<Message>() {
                        item.extract::<Message>()?
                    } else {
                        Message::new(&item)?
                    });
                }
                Err(e) => {
                    return Err(PromptError::ParseError(e.to_string()));
                }
            }
        }
        Ok(revised_messages)
    } else {
        // Not a list or tuple, try to convert directly to Message
        Ok(vec![Message::new(messages)?])
    }
}
/// Extract model settings from the settings_type attribute. This function will return the appropriate
/// ModelSettings variant based on the type of settings provided.
fn extract_model_settings(model_settings: &Bound<'_, PyAny>) -> Result<ModelSettings, PromptError> {
    let settings_type = model_settings.call_method0("settings_type")?;
    match settings_type.extract::<SettingsType>()? {
        SettingsType::OpenAIChat => {
            let openai_settings = model_settings.extract::<OpenAIChatSettings>()?;
            Ok(ModelSettings::OpenAIChat(openai_settings))
        }
        SettingsType::GoogleChat => {
            let gemini_settings = model_settings.extract::<GeminiSettings>()?;
            Ok(ModelSettings::GoogleChat(gemini_settings))
        }
        SettingsType::ModelSettings => {
            let model_settings = model_settings.extract::<ModelSettings>()?;
            Ok(model_settings)
        }
    }
}

#[pymethods]
impl Prompt {
    #[new]
    #[pyo3(signature = (message, model, provider, system_instruction=None, model_settings=None, response_format=None))]
    pub fn new(
        py: Python<'_>,
        message: &Bound<'_, PyAny>,
        model: &str,
        provider: &Bound<'_, PyAny>,
        system_instruction: Option<&Bound<'_, PyAny>>,
        model_settings: Option<&Bound<'_, PyAny>>,
        response_format: Option<&Bound<'_, PyAny>>, // can be a pydantic model or one of Opsml's predefined outputs
    ) -> Result<Self, PromptError> {
        let model_settings = model_settings
            .as_ref()
            .map(|s| extract_model_settings(s))
            .transpose()?;
        let provider = Provider::extract_provider(provider)?;
        let system_instruction = if let Some(system_instruction) = system_instruction {
            parse_prompt(system_instruction)?
                .into_iter()
                .map(|mut msg| {
                    msg.role = Role::Developer.to_string();
                    msg
                })
                .collect::<Vec<Message>>()
        } else {
            vec![]
        };

        let message = parse_prompt(message)?
            .into_iter()
            .map(|mut msg| {
                msg.role = Role::User.to_string();
                msg
            })
            .collect::<Vec<Message>>();

        // validate response_json_schema
        let (response_type, response_json_schema) = match response_format {
            Some(response_format) => {
                // check if response_format is a pydantic model and extract the model json schema
                parse_response_to_json(py, response_format)?
            }
            None => (ResponseType::Null, None),
        };

        Self::new_rs(
            message,
            model,
            provider,
            system_instruction,
            model_settings,
            response_json_schema,
            response_type,
        )
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
    pub fn from_path(path: PathBuf) -> Result<Self, PromptError> {
        // Load the JSON file from the path
        let file = std::fs::read_to_string(&path)
            .map_err(|e| PromptError::Error(format!("Failed to read file: {e}")))?;

        // Parse the JSON file into a Prompt
        serde_json::from_str(&file)
            .map_err(|e| PromptError::Error(format!("Failed to parse JSON: {e}")))
    }

    #[staticmethod]
    pub fn model_validate_json(json_string: String) -> Result<Self, PromptError> {
        let json_value: Value = serde_json::from_str(&json_string)
            .map_err(|e| PromptError::Error(format!("Failed to parse JSON string: {e}")))?;
        let model: Self = serde_json::from_value(json_value)
            .map_err(|e| PromptError::Error(format!("Failed to parse JSON value: {e}")))?;

        Ok(model)
    }

    pub fn model_dump_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
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
    ) -> Result<Self, PromptError> {
        let mut new_prompt = self.clone();
        // Create a new Prompt with the bound value
        if let (Some(name), Some(value)) = (name, value) {
            // Bind in both user and system messages
            for message in &mut new_prompt.message {
                let var_value = extract_string_value(value)?;
                message.bind_mut(name, &var_value)?;
            }
        }

        if let Some(kwargs) = kwargs {
            for (key, val) in kwargs.iter() {
                let var_name = key.extract::<String>()?;
                let var_value = extract_string_value(&val)?;

                // Bind in both user and system messages
                for message in &mut new_prompt.message {
                    message.bind_mut(&var_name, &var_value)?;
                }
            }
        }

        // Validate that at least one binding method was used
        if name.is_none() && kwargs.is_none_or(|k| k.is_empty()) {
            return Err(PromptError::Error(
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
    ) -> Result<(), PromptError> {
        // Create a new Prompt with the bound value
        if let (Some(name), Some(value)) = (name, value) {
            // Bind in both user and system messages
            for message in &mut self.message {
                let var_value = extract_string_value(value)?;
                message.bind_mut(name, &var_value)?;
            }
        }

        if let Some(kwargs) = kwargs {
            for (key, val) in kwargs.iter() {
                let var_name = key.extract::<String>()?;
                let var_value = extract_string_value(&val)?;

                // Bind in both user and system messages
                for message in &mut self.message {
                    message.bind_mut(&var_name, &var_value)?;
                }
            }
        }

        // Validate that at least one binding method was used
        if name.is_none() && kwargs.is_none_or(|k| k.is_empty()) {
            return Err(PromptError::Error(
                "Must provide either (name, value) or keyword arguments for binding".to_string(),
            ));
        }
        Ok(())
    }

    #[getter]
    pub fn response_json_schema(&self) -> Option<String> {
        Some(PyHelperFuncs::__str__(self.response_json_schema.as_ref()))
    }
}

impl Prompt {
    pub fn new_rs(
        message: Vec<Message>,
        model: &str,
        provider: Provider,
        system_instruction: Vec<Message>,
        model_settings: Option<ModelSettings>,
        response_json_schema: Option<Value>,
        response_type: ResponseType,
    ) -> Result<Self, PromptError> {
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
        let parameters = Self::extract_variables(&message, &system_instruction);

        Ok(Self {
            message,
            version,
            system_instruction,
            model_settings,
            response_json_schema,
            parameters,
            response_type,
            model: model.to_string(),
            provider,
        })
    }

    fn extract_variables(message: &Vec<Message>, system_instruction: &Vec<Message>) -> Vec<String> {
        let mut variables = HashSet::new();

        // Check system messages
        for message in system_instruction {
            for var in Message::extract_variables(&message.content) {
                variables.insert(var);
            }
        }

        // Check user messages
        for message in message {
            for var in Message::extract_variables(&message.content) {
                variables.insert(var);
            }
        }

        // Convert HashSet to Vec for return
        variables.into_iter().collect()
    }

    pub fn model_dump_value(&self) -> Value {
        // Convert the Prompt to a JSON Value
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::{
        types::{BinaryContent, DocumentUrl, ImageUrl, PromptContent},
        Score,
    };
    use potato_type::StructuredOutput;

    #[test]
    fn test_task_list_add_and_get() {
        let prompt_content = PromptContent::Str("Test prompt. ${param1} ${param2}".to_string());
        let prompt = Prompt::new_rs(
            vec![Message::new_rs(prompt_content)],
            "gpt-4o",
            Provider::OpenAI,
            vec![],
            None,
            None,
            ResponseType::Null,
        )
        .unwrap();

        // Check if the prompt was created successfully
        assert_eq!(prompt.message.len(), 1);

        // check prompt parameters
        assert!(prompt.parameters.len() == 2);

        // sort parameters to ensure order does not affect the test
        let mut parameters = prompt.parameters.clone();
        parameters.sort();

        assert_eq!(parameters[0], "param1");
        assert_eq!(parameters[1], "param2");

        // bind parameter
        let bound_msg = prompt.message[0].bind("param1", "Value1").unwrap();
        let bound_msg = bound_msg.bind("param2", "Value2").unwrap();

        // Check if the bound message contains the correct values
        match bound_msg.content {
            PromptContent::Str(content) => {
                assert_eq!(content, "Test prompt. Value1 Value2");
            }
            _ => panic!("Expected PromptContent::Str"),
        }
    }

    #[test]
    fn test_image_prompt() {
        let prompt = Prompt::new_rs(
            vec![
                Message::new_rs(PromptContent::Str(
                    "What company is this logo from?".to_string(),
                )),
                Message::new_rs(PromptContent::Image(ImageUrl {
                    url: "https://iili.io/3Hs4FMg.png".to_string(),
                    kind: "image-url".to_string(),
                })),
            ],
            "gpt-4o",
            Provider::OpenAI,
            vec![Message::new_rs(PromptContent::Str(
                "system_prompt".to_string(),
            ))],
            None,
            None,
            ResponseType::Null,
        )
        .unwrap();

        // Check the first user message
        if let PromptContent::Str(content) = &prompt.message[0].content {
            assert_eq!(content, "What company is this logo from?");
        } else {
            panic!("Expected PromptContent::Str for the first user message");
        }

        // Check the second user message (ImageUrl)
        if let PromptContent::Image(image_url) = &prompt.message[1].content {
            assert_eq!(image_url.url, "https://iili.io/3Hs4FMg.png");
            assert_eq!(image_url.kind, "image-url");
        } else {
            panic!("Expected PromptContent::Image for the second user message");
        }
    }

    #[test]
    fn test_binary_prompt() {
        let image_data = vec![137, 80, 78, 71, 13, 10, 26, 10]; // Example PNG header bytes

        let prompt = Prompt::new_rs(
            vec![
                Message::new_rs(PromptContent::Str(
                    "What company is this logo from?".to_string(),
                )),
                Message::new_rs(PromptContent::Binary(BinaryContent {
                    data: image_data.clone(),
                    media_type: "image/png".to_string(),
                    kind: "binary".to_string(),
                })),
            ],
            "gpt-4o",
            Provider::OpenAI,
            vec![Message::new_rs(PromptContent::Str(
                "system_prompt".to_string(),
            ))],
            None,
            None,
            ResponseType::Null,
        )
        .unwrap();

        // Check the first user message
        if let PromptContent::Str(content) = &prompt.message[0].content {
            assert_eq!(content, "What company is this logo from?");
        } else {
            panic!("Expected PromptContent::Str for the first user message");
        }

        // Check the second user message (BinaryContent)
        if let PromptContent::Binary(binary_content) = &prompt.message[1].content {
            assert_eq!(binary_content.data, image_data);
            assert_eq!(binary_content.media_type, "image/png");
            assert_eq!(binary_content.kind, "binary");
        } else {
            panic!("Expected PromptContent::Binary for the second user message");
        }
    }

    #[test]
    fn test_document_prompt() {
        let prompt = Prompt::new_rs(
            vec![
                Message::new_rs(PromptContent::Str(
                    "What is the main content of this document?".to_string(),
                )),
                Message::new_rs(PromptContent::Document(DocumentUrl {
                    url: "https://storage.googleapis.com/cloud-samples-data/generative-ai/pdf/2403.05530.pdf".to_string(),
                    kind: "document-url".to_string(),
                })),
            ],
             "gpt-4o",
            Provider::OpenAI,
            vec![Message::new_rs(PromptContent::Str(
                "system_prompt".to_string(),
            ))],
            None,
            None,
              ResponseType::Null,
        )
        .unwrap();

        // Check the first user message
        if let PromptContent::Str(content) = &prompt.message[0].content {
            assert_eq!(content, "What is the main content of this document?");
        } else {
            panic!("Expected PromptContent::Str for the first user message");
        }

        // Check the second user message (DocumentUrl)
        if let PromptContent::Document(document_url) = &prompt.message[1].content {
            assert_eq!(
                document_url.url,
                "https://storage.googleapis.com/cloud-samples-data/generative-ai/pdf/2403.05530.pdf"
            );
            assert_eq!(document_url.kind, "document-url");
        } else {
            panic!("Expected PromptContent::Document for the second user message");
        }
    }

    #[test]
    fn test_response_format_score() {
        let prompt = Prompt::new_rs(
            vec![Message::new_rs(PromptContent::Str(
                "Rate the quality of this response.".to_string(),
            ))],
            "gpt-4o",
            Provider::OpenAI,
            vec![],
            None,
            Some(Score::get_structured_output_schema()),
            ResponseType::Null,
        )
        .unwrap();

        // Check if the response json schema is set correctly
        assert!(prompt.response_json_schema.is_some());
    }
}
