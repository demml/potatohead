use crate::agents::error::AgentError;
use potato_provider::ChatResponse;

use potato_type::prompt::ResponseContent;
use potato_type::traits::LogProbExt;
use potato_type::traits::ResponseAdapter;
use potato_type::traits::{MessageResponseExt, TokenUsage};
use potato_util::json_to_pyobject;
use potato_util::utils::{LogProbs, ResponseLogProbs};
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;
use tracing::warn;
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentResponse {
    pub id: String,
    pub response: ChatResponse,
}

#[pymethods]
impl AgentResponse {
    pub fn token_usage<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        match &self.response {
            ChatResponse::OpenAIV1(resp) => Ok(resp.usage(py)?),
            ChatResponse::GeminiV1(resp) => Ok(resp.usage(py)?),
            ChatResponse::VertexGenerateV1(resp) => Ok(resp.usage(py)?),
            ChatResponse::AnthropicMessageV1(resp) => Ok(resp.usage(py)?),
            _ => Err(AgentError::NotSupportedError(
                "Token usage not supported for the vertex predict response type".to_string(),
            )),
        }
    }

    /// Returns the response as a Python object
    pub fn response<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        Ok(self.response.to_python(py)?)
    }
}

impl AgentResponse {
    pub fn new(id: String, response: ChatResponse) -> Self {
        Self { id, response }
    }

    /// WIll retrieve the first content choice from the response
    #[instrument(skip_all)]
    pub fn content(&self) -> ResponseContent {
        match &self.response {
            ChatResponse::OpenAIV1(resp) => resp.get_content(),
            ChatResponse::GeminiV1(resp) => resp.get_content(),
            ChatResponse::VertexGenerateV1(resp) => resp.get_content(),
            ChatResponse::AnthropicMessageV1(resp) => resp.get_content(),
            _ => {
                warn!("Content not available for this response type");
                None
            }
        }
    }

    pub fn log_probs(&self) -> Vec<ResponseLogProbs> {
        match &self.response {
            ChatResponse::OpenAIV1(resp) => resp.get_log_probs(),
            ChatResponse::GeminiV1(resp) => resp.get_log_probs(),
            ChatResponse::VertexGenerateV1(resp) => resp.get_log_probs(),
            ChatResponse::AnthropicMessageV1(resp) => resp.get_log_probs(),
            _ => {
                warn!("Log probabilities not available for this response type");
                vec![]
            }
        }
    }
}

#[pyclass(name = "AgentResponse")]
#[derive(Debug, Serialize)]
pub struct PyAgentResponse {
    pub response: AgentResponse,

    #[serde(skip_serializing)]
    pub output_type: Option<Py<PyAny>>,

    #[pyo3(get)]
    pub failed_conversion: bool,
}

#[pymethods]
impl PyAgentResponse {
    #[getter]
    pub fn id(&self) -> &str {
        &self.response.id
    }

    /// Return the token usage of the response
    #[getter]
    pub fn token_usage<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        self.response.token_usage(py)
    }

    /// Returns the actual response object from the provider
    #[getter]
    pub fn response<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, AgentError> {
        self.response.response(py)
    }

    #[getter]
    pub fn log_probs(&self) -> LogProbs {
        LogProbs {
            tokens: self.response.log_probs(),
        }
    }

    /// This will map a the content of the response to a python object.
    /// A python object in this case will be either a passed pydantic model or support potatohead types.
    /// If neither is porvided, an attempt is made to parse the serde Value into an appropriate Python type.
    /// Types:
    /// - Serde Null -> Python None
    /// - Serde Bool -> Python bool
    /// - Serde String -> Python str
    /// - Serde Number -> Python int or float
    /// - Serde Array -> Python list (with each item converted to Python type)
    /// - Serde Object -> Python dict (with each key-value pair converted to Python type)
    #[getter]
    #[instrument(skip_all)]
    pub fn structured_output<'py>(
        &mut self,
        py: Python<'py>,
    ) -> Result<Bound<'py, PyAny>, AgentError> {
        let content_value = self.response.content();

        // If the content is None, return None
        if content_value.is_none() {
            return Ok(py.None().into_bound_py_any(py)?);
        }
        // convert content_value to string
        let content_value = content_value.unwrap();

        match &self.output_type {
            Some(output_type) => {
                // Match the value. For loading into pydantic models, it's expected that the api response is a JSON string.

                let bound = output_type
                    .bind(py)
                    .call_method1("model_validate_json", (&content_value,));

                match bound {
                    Ok(obj) => {
                        // Successfully validated the model
                        Ok(obj)
                    }
                    Err(err) => {
                        // Model validation failed
                        // convert string to json and then to python object
                        warn!("Failed to validate model: {}", err);
                        self.failed_conversion = true;
                        let val = serde_json::from_str::<Value>(&content_value)?;
                        Ok(json_to_pyobject(py, &val)?.into_bound_py_any(py)?)
                    }
                }
            }
            None => {
                // If no output type is provided, attempt to parse the content as JSON
                let val = Value::String(content_value);
                Ok(json_to_pyobject(py, &val)?.into_bound_py_any(py)?)
            }
        }
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

impl PyAgentResponse {
    pub fn new(response: AgentResponse, output_type: Option<Py<PyAny>>) -> Self {
        Self {
            response,
            output_type,
            failed_conversion: false,
        }
    }
}
