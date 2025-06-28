use crate::prompt::error::PromptError;
use crate::prompt::types::parse_pydantic_model;

use crate::prompt::types::{Message, Role};
use potato_types::SaveName;
use potato_utils::{json_to_pyobject, pyobject_to_json, PyHelperFuncs};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString, PyTuple};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]

pub struct ModelSettings {
    #[pyo3(get, set)]
    pub model: String,

    #[pyo3(get, set)]
    pub provider: String,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<f32>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_body: Option<Value>,
}

#[pymethods]
impl ModelSettings {
    #[new]
    #[pyo3(signature = (model, provider, max_tokens=None, temperature=None, top_p=None, frequency_penalty=None, presence_penalty=None, timeout=0.0, parallel_tool_calls=None, seed=None, logit_bias=None, stop_sequences=None, extra_body=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        model: &str,
        provider: &str,
        max_tokens: Option<usize>,
        temperature: Option<f32>,
        top_p: Option<f32>,
        frequency_penalty: Option<f32>,
        presence_penalty: Option<f32>,
        timeout: Option<f32>,
        parallel_tool_calls: Option<bool>,
        seed: Option<u64>,
        logit_bias: Option<HashMap<String, i32>>,
        stop_sequences: Option<Vec<String>>,
        extra_body: Option<&Bound<'_, PyAny>>,
    ) -> Result<Self, PromptError> {
        // check if extra body is not none.
        // if not none, conver to py any and attempt pyobject_to_json
        let extra_body =
            if let Some(extra_body) = extra_body {
                Some(pyobject_to_json(extra_body).map_err(|e| {
                    PromptError::Error(format!("Failed to convert extra body: {}", e))
                })?)
            } else {
                None
            };

        Ok(Self {
            model: model.to_string(),
            provider: provider.to_string(),
            max_tokens,
            temperature,
            top_p,
            frequency_penalty,
            presence_penalty,
            timeout,
            parallel_tool_calls,
            seed,
            logit_bias,
            stop_sequences,
            extra_body,
        })
    }

    #[getter]
    pub fn extra_body<'py>(
        &self,
        py: Python<'py>,
    ) -> Result<Option<Bound<'py, PyDict>>, PromptError> {
        // error if extra body is None
        self.extra_body
            .as_ref()
            .map(|v| {
                let pydict = PyDict::new(py);
                json_to_pyobject(py, v, &pydict)
            })
            .transpose()
            .map_err(|e| PromptError::Error(format!("Failed to get extra body: {}", e)))
    }

    pub fn model_dump<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyDict>, PromptError> {
        // iterate over each field in model_settings and add to the dict if it is not None
        let pydict = PyDict::new(py);

        pydict.set_item("model", &self.model)?;
        pydict.set_item("provider", &self.provider)?;

        if let Some(max_tokens) = self.max_tokens {
            pydict.set_item("max_tokens", max_tokens)?;
        }
        if let Some(temperature) = self.temperature {
            pydict.set_item("temperature", temperature)?;
        }
        if let Some(top_p) = self.top_p {
            pydict.set_item("top_p", top_p)?;
        }
        if let Some(frequency_penalty) = self.frequency_penalty {
            pydict.set_item("frequency_penalty", frequency_penalty)?;
        }
        if let Some(presence_penalty) = self.presence_penalty {
            pydict.set_item("presence_penalty", presence_penalty)?;
        }
        if let Some(parallel_tool_calls) = self.parallel_tool_calls {
            pydict.set_item("parallel_tool_calls", parallel_tool_calls)?;
        }
        if let Some(seed) = self.seed {
            pydict.set_item("seed", seed)?;
        }
        if let Some(logit_bias) = &self.logit_bias {
            pydict.set_item("logit_bias", logit_bias)?;
        }
        if let Some(stop_sequences) = &self.stop_sequences {
            pydict.set_item("stop_sequences", stop_sequences)?;
        }
        let extra = self.extra_body(py)?;

        if let Some(extra) = extra {
            pydict.set_item("extra_body", extra)?;
        }

        Ok(pydict)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Prompt {
    #[pyo3(get)]
    pub user_message: Vec<Message>,

    #[pyo3(get)]
    pub system_message: Vec<Message>,

    pub version: String,

    #[pyo3(get)]
    pub model_settings: ModelSettings,

    pub response_format: Option<Value>,

    pub parameters: Vec<String>,
}

pub fn parse_prompt(messages: &Bound<'_, PyAny>) -> PyResult<Vec<Message>> {
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
                    return Err(e);
                }
            }
        }
        Ok(revised_messages)
    } else {
        // Not a list or tuple, try to convert directly to Message
        Ok(vec![Message::new(messages)?])
    }
}

#[pymethods]
impl Prompt {
    #[new]
    #[pyo3(signature = (user_message, model=None, provider=None, system_message=None, model_settings=None, response_format=None))]
    pub fn new(
        py: Python<'_>,
        user_message: &Bound<'_, PyAny>,
        model: Option<&str>,
        provider: Option<&str>,
        system_message: Option<&Bound<'_, PyAny>>,
        model_settings: Option<ModelSettings>,
        response_format: Option<&Bound<'_, PyAny>>, // can be a pydantic model or one of Opsml's predefined outputs
    ) -> Result<Self, PromptError> {
        // extract messages

        let system_message = if let Some(system_message) = system_message {
            parse_prompt(system_message)?
                .into_iter()
                .map(|mut msg| {
                    msg.role = Role::Developer.to_string();
                    msg
                })
                .collect::<Vec<Message>>()
        } else {
            vec![]
        };

        let user_message = parse_prompt(user_message)?
            .into_iter()
            .map(|mut msg| {
                msg.role = Role::User.to_string();
                msg
            })
            .collect::<Vec<Message>>();

        // validate response_format
        let response_format = match response_format {
            Some(response_format) => {
                // check if response_format is a pydantic model and extract the model json schema
                parse_pydantic_model(py, response_format)?

                // we don't store response_format in the prompt, but we validate it
            }
            None => None,
        };

        Self::new_rs(
            user_message,
            model,
            provider,
            system_message,
            model_settings,
            response_format,
        )
    }

    #[getter]
    pub fn model(&self) -> &str {
        // error if model is None
        &self.model_settings.model
    }

    #[getter]
    pub fn provider(&self) -> &str {
        // error if model is None
        &self.model_settings.provider
    }

    #[getter]
    pub fn model_identifier(&self) -> String {
        format!(
            "{}:{}",
            self.model_settings.provider, self.model_settings.model
        )
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
            .map_err(|e| PromptError::Error(format!("Failed to read file: {}", e)))?;

        // Parse the JSON file into a Prompt
        serde_json::from_str(&file)
            .map_err(|e| PromptError::Error(format!("Failed to parse JSON: {}", e)))
    }

    #[staticmethod]
    pub fn model_validate_json(json_string: String) -> Result<Self, PromptError> {
        let json_value: Value = serde_json::from_str(&json_string)
            .map_err(|e| PromptError::Error(format!("Failed to parse JSON string: {}", e)))?;
        let model: Self = serde_json::from_value(json_value)
            .map_err(|e| PromptError::Error(format!("Failed to parse JSON value: {}", e)))?;

        Ok(model)
    }

    pub fn model_dump_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }

    #[getter]
    pub fn response_format(&self) -> Option<String> {
        Some(PyHelperFuncs::__str__(self.response_format.as_ref()))
    }
}

impl Prompt {
    pub fn new_rs(
        user_message: Vec<Message>,
        model: Option<&str>,
        provider: Option<&str>,
        system_message: Vec<Message>,
        model_settings: Option<ModelSettings>,
        response_format: Option<Value>,
    ) -> Result<Self, PromptError> {
        // get version from crate
        let version = potato_utils::version();

        // either model and provider or model_settings must be provided
        if (model.is_none() || provider.is_none()) && model_settings.is_none() {
            return Err(PromptError::Error(
                "Either model and provider or model_settings must be provided".to_string(),
            ));
        }

        let model_settings = match model_settings {
            Some(settings) => settings,
            None => ModelSettings {
                model: model.unwrap().to_string(),
                provider: provider.unwrap().to_string(),
                ..Default::default()
            },
        };

        // extract named parameters in prompt
        let parameters = Self::extract_variables(&user_message, &system_message);

        Ok(Self {
            user_message,
            version,
            system_message,
            model_settings,
            response_format,
            parameters,
        })
    }

    fn extract_variables(
        user_message: &Vec<Message>,
        system_message: &Vec<Message>,
    ) -> Vec<String> {
        let mut variables = HashSet::new();

        // Check system messages
        for message in system_message {
            for var in message.extract_variables() {
                variables.insert(var);
            }
        }

        // Check user messages
        for message in user_message {
            for var in message.extract_variables() {
                variables.insert(var);
            }
        }

        // Convert HashSet to Vec for return
        variables.into_iter().collect()
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::PromptContent;

    #[test]
    fn test_task_list_add_and_get() {
        let prompt_content = PromptContent::Str("Test prompt. ${param1} ${param2}".to_string());
        let prompt = Prompt::new_rs(
            vec![Message::new_rs(prompt_content)],
            Some("gpt-4o"),
            Some("openai"),
            vec![],
            None,
            None,
        )
        .unwrap();

        // Check if the prompt was created successfully
        assert_eq!(prompt.user_message.len(), 1);

        // check prompt parameters
        assert!(prompt.parameters.len() == 2);

        // sort parameters to ensure order does not affect the test
        let mut parameters = prompt.parameters.clone();
        parameters.sort();

        assert_eq!(parameters[0], "param1");
        assert_eq!(parameters[1], "param2");

        // bind parameter
        let bound_msg = prompt.user_message[0].bind("param1", "Value1").unwrap();
        let bound_msg = bound_msg.bind("param2", "Value2").unwrap();

        // Check if the bound message contains the correct values
        match bound_msg.content {
            PromptContent::Str(content) => {
                assert_eq!(content, "Test prompt. Value1 Value2");
            }
            _ => panic!("Expected PromptContent::Str"),
        }
    }
}
