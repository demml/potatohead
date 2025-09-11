use crate::TypeError;
use potato_util::{json_to_pyobject, pyobject_to_json};
use pyo3::prelude::*;
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
    pub fn instances<'py>(&self, py: Python<'py>) -> Result<PyObject, TypeError> {
        let obj = json_to_pyobject(py, &self.instances)?;
        Ok(obj)
    }

    #[getter]
    pub fn parameters<'py>(&self, py: Python<'py>) -> Result<PyObject, TypeError> {
        let obj = json_to_pyobject(py, &self.parameters)?;
        Ok(obj)
    }

    #[new]
    #[pyo3(signature = (instances, parameters=None))]
    pub fn new(instances: Bound<'_, PyAny>, parameters: Option<Bound<'_, PyAny>>) -> Self {
        // check if instances is a PyList, if not,
        let instances = pyobject_to_json(&instances).unwrap_or(Value::Null);
        let parameters =
            parameters.map_or(Value::Null, |p| pyobject_to_json(&p).unwrap_or(Value::Null));

        Self {
            instances,
            parameters,
        }
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
    pub fn predictions<'py>(&self, py: Python<'py>) -> Result<PyObject, TypeError> {
        let obj = json_to_pyobject(py, &self.predictions)?;
        Ok(obj)
    }

    #[getter]
    pub fn metadata<'py>(&self, py: Python<'py>) -> Result<PyObject, TypeError> {
        let obj = json_to_pyobject(py, &self.metadata)?;
        Ok(obj)
    }
}
