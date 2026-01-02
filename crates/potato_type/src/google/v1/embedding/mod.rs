use crate::google::v1::generate::{GeminiContent, Part};
use crate::prompt::MessageNum;
use crate::prompt::ResponseContent;
use crate::prompt::Role;
use crate::traits::ResponseAdapter;
use crate::TypeError;
use potato_util::utils::TokenLogProbs;
use potato_util::PyHelperFuncs;
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
use pythonize::{depythonize, pythonize};
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[pyclass]
#[serde(rename_all = "camelCase", default)]
pub struct PredictRequest {
    pub instances: Value,
    pub parameters: Value,
}

#[pymethods]
impl PredictRequest {
    #[getter]
    pub fn instances<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        let obj = pythonize(py, &self.instances)?;
        Ok(obj)
    }

    #[getter]
    pub fn parameters<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        let obj = pythonize(py, &self.parameters)?;
        Ok(obj)
    }

    #[new]
    #[pyo3(signature = (instances, parameters=None))]
    pub fn new(instances: Bound<'_, PyAny>, parameters: Option<Bound<'_, PyAny>>) -> Self {
        // check if instances is a PyList, if not,
        let instances = depythonize(&instances).unwrap_or(Value::Null);
        let parameters = parameters.map_or(Value::Null, |p| depythonize(&p).unwrap_or(Value::Null));

        Self {
            instances,
            parameters,
        }
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[pyclass]
#[serde(rename_all = "camelCase", default)]
pub struct PredictResponse {
    pub predictions: Value,
    pub metadata: Value,
    #[pyo3(get)]
    pub deployed_model_id: String,
    #[pyo3(get)]
    pub model: String,
    #[pyo3(get)]
    pub model_version_id: String,
    #[pyo3(get)]
    pub model_display_name: String,
}

#[pymethods]
impl PredictResponse {
    #[getter]
    pub fn predictions<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        let obj = pythonize(py, &self.predictions)?;
        Ok(obj)
    }

    #[getter]
    pub fn metadata<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        let obj = pythonize(py, &self.metadata)?;
        Ok(obj)
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

impl PredictResponse {
    pub fn into_py_bound_any<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        let bound = Py::new(py, self.clone())?;
        Ok(bound.into_bound_py_any(py)?)
    }
}

impl ResponseAdapter for PredictResponse {
    fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }

    fn is_empty(&self) -> bool {
        match &self.predictions {
            Value::Array(arr) => arr.is_empty(),
            _ => false,
        }
    }

    fn to_bound_py_object<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        Ok(PyHelperFuncs::to_bound_py_object(py, self)?)
    }

    fn id(&self) -> &str {
        &self.deployed_model_id
    }

    fn to_message_num(&self) -> Result<Vec<MessageNum>, TypeError> {
        Err(TypeError::Error(
            "Cannot convert PredictResponse to MessageNum".to_string(),
        ))
    }

    fn get_content(&self) -> ResponseContent {
        ResponseContent::PredictResponse(self.clone())
    }

    fn structured_output<'py>(
        &self,
        py: Python<'py>,
        _output_model: Option<&Bound<'py, PyAny>>,
    ) -> Result<Bound<'py, PyAny>, TypeError> {
        if self.is_empty() {
            // return Py None if no content
            return Ok(py.None().into_bound_py_any(py)?);
        }

        let val = self.predictions.clone();
        Ok(pythonize(py, &val)?)
    }

    fn usage<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        Ok(py.None().into_bound_py_any(py)?)
    }

    fn get_log_probs(&self) -> Vec<TokenLogProbs> {
        vec![]
    }

    fn structured_output_value(&self) -> Option<Value> {
        None
    }

    fn tool_call_output(&self) -> Option<Value> {
        None
    }

    fn response_text(&self) -> String {
        String::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[pyclass(eq, eq_int)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EmbeddingTaskType {
    TaskTypeUnspecified,
    RetrievalQuery,
    RetrievalDocument,
    SemanticSimilarity,
    Classification,
    Clustering,
    QuestionAnswering,
    FactVerification,
    CodeRetrievalQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[pyclass]
pub struct GeminiEmbeddingConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_dimensionality: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<EmbeddingTaskType>,

    #[serde(skip_serializing)]
    pub is_configured: bool,
}

#[pymethods]
impl GeminiEmbeddingConfig {
    #[new]
    #[pyo3(signature = (model=None, output_dimensionality=None, task_type=None))]
    pub fn new(
        model: Option<String>,
        output_dimensionality: Option<i32>,
        task_type: Option<EmbeddingTaskType>,
    ) -> Result<Self, TypeError> {
        if model.is_none() && task_type.is_none() {
            return Err(TypeError::GeminiEmbeddingConfigError(
                "Either 'model' or 'task_type' must be provided.".to_string(),
            ));
        }

        let is_configured = output_dimensionality.is_some() || task_type.is_some();

        Ok(Self {
            model,
            output_dimensionality,
            task_type,
            is_configured,
        })
    }
}

impl GeminiEmbeddingConfig {
    pub fn get_parameters_for_predict(&self) -> serde_json::Value {
        let mut params = serde_json::Map::new();
        if let Some(dim) = self.output_dimensionality {
            params.insert("outputDimensionality".to_string(), serde_json::json!(dim));
        }
        if let Some(task) = &self.task_type {
            params.insert("task_type".to_string(), serde_json::json!(task));
        }
        if params.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::Value::Object(params)
        }
    }
}

pub trait EmbeddingConfigTrait {
    fn get_model(&self) -> &str;
}

impl EmbeddingConfigTrait for GeminiEmbeddingConfig {
    fn get_model(&self) -> &str {
        self.model.as_deref().unwrap_or("embedding-001")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[pyclass]
pub struct ContentEmbedding {
    pub values: Vec<f32>,
}

#[pymethods]
impl ContentEmbedding {
    #[getter]
    pub fn values(&self) -> &Vec<f32> {
        &self.values
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[pyclass]
pub struct GeminiEmbeddingResponse {
    #[pyo3(get)]
    pub embedding: ContentEmbedding,
}

impl GeminiEmbeddingResponse {
    pub fn into_py_bound_any<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyAny>, TypeError> {
        let bound = Py::new(py, self.clone())?;
        Ok(bound.into_bound_py_any(py)?)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GeminiEmbeddingRequest<T>
where
    T: Serialize,
{
    pub content: GeminiContent,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub embedding_config: Option<T>,
}

impl<T> GeminiEmbeddingRequest<T>
where
    T: Serialize,
{
    pub fn new(inputs: Vec<String>, config: T) -> Self {
        let parts = inputs.into_iter().map(Part::from_text).collect();

        Self {
            content: GeminiContent {
                parts,
                role: Role::User.to_string(),
            },
            embedding_config: Some(config),
        }
    }
}
