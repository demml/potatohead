use crate::error::UtilError;

use colored_json::{Color, ColorMode, ColoredFormatter, PrettyFormatter, Styler};
use pyo3::prelude::*;

use pyo3::types::{
    PyAny, PyBool, PyDict, PyDictMethods, PyFloat, PyInt, PyList, PyString, PyTuple,
};
use pyo3::IntoPyObjectExt;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use serde_json::Value::{Null, Object};
use std::ops::RangeInclusive;
use std::path::Path;
use uuid::Uuid;
pub fn create_uuid7() -> String {
    Uuid::now_v7().to_string()
}
use tracing::warn;
pub struct PyHelperFuncs {}

impl PyHelperFuncs {
    /// Convert any type implementing `IntoPyObject` to a Python object
    /// # Arguments
    /// * `py` - A Python interpreter instance
    /// * `object` - A reference to an object implementing `IntoPyObject`
    /// # Returns
    /// * `Result<Bound<'py, PyAny>, UtilError>` - A result containing the Python object or an error
    pub fn to_bound_py_object<'py, T>(
        py: Python<'py>,
        object: &T,
    ) -> Result<Bound<'py, PyAny>, UtilError>
    where
        T: IntoPyObject<'py> + Clone,
    {
        Ok(object.clone().into_bound_py_any(py)?)
    }
    pub fn __str__<T: Serialize>(object: T) -> String {
        match ColoredFormatter::with_styler(
            PrettyFormatter::default(),
            Styler {
                key: Color::Rgb(75, 57, 120).foreground(),
                string_value: Color::Rgb(4, 205, 155).foreground(),
                float_value: Color::Rgb(4, 205, 155).foreground(),
                integer_value: Color::Rgb(4, 205, 155).foreground(),
                bool_value: Color::Rgb(4, 205, 155).foreground(),
                nil_value: Color::Rgb(4, 205, 155).foreground(),
                ..Default::default()
            },
        )
        .to_colored_json(&object, ColorMode::On)
        {
            Ok(json) => json,
            Err(e) => format!("Failed to serialize to json: {e}"),
        }
        // serialize the struct to a string
    }

    pub fn __json__<T: Serialize>(object: T) -> String {
        match serde_json::to_string_pretty(&object) {
            Ok(json) => json,
            Err(e) => format!("Failed to serialize to json: {e}"),
        }
    }

    /// Save a struct to a JSON file
    ///
    /// # Arguments
    ///
    /// * `model` - A reference to a struct that implements the `Serialize` trait
    /// * `path` - A reference to a `Path` object that holds the path to the file
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` or a `UtilError`
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The struct cannot be serialized to a string
    pub fn save_to_json<T>(model: T, path: &Path) -> Result<(), UtilError>
    where
        T: Serialize,
    {
        // serialize the struct to a string
        let json =
            serde_json::to_string_pretty(&model).map_err(|_| UtilError::SerializationError)?;

        // ensure .json extension
        let path = path.with_extension("json");

        if !path.exists() {
            // ensure path exists, create if not
            let parent_path = path.parent().ok_or(UtilError::GetParentPathError)?;

            std::fs::create_dir_all(parent_path).map_err(|_| UtilError::CreateDirectoryError)?;
        }

        std::fs::write(path, json).map_err(|_| UtilError::WriteError)?;

        Ok(())
    }
}

pub fn json_to_pydict<'py>(
    py: Python,
    value: &Value,
    dict: &Bound<'py, PyDict>,
) -> Result<Bound<'py, PyDict>, UtilError> {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let py_value = match v {
                    Value::Null => py.None(),
                    Value::Bool(b) => b.into_py_any(py)?,
                    Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            i.into_py_any(py)?
                        } else if let Some(f) = n.as_f64() {
                            f.into_py_any(py)?
                        } else {
                            return Err(UtilError::InvalidNumber);
                        }
                    }
                    Value::String(s) => s.into_py_any(py)?,
                    Value::Array(arr) => {
                        let py_list = PyList::empty(py);
                        for item in arr {
                            let py_item = json_to_pyobject(py, item)?;
                            py_list.append(py_item)?;
                        }
                        py_list.into_py_any(py)?
                    }
                    Value::Object(_) => {
                        let nested_dict = PyDict::new(py);
                        json_to_pydict(py, v, &nested_dict)?;
                        nested_dict.into_py_any(py)?
                    }
                };
                dict.set_item(k, py_value)?;
            }
        }
        _ => return Err(UtilError::RootMustBeObjectError),
    }

    Ok(dict.clone())
}

/// Converts a serde_json::Value to a PyObject. Including support for nested objects and arrays.
/// This function handles all Serde JSON types:
/// - Serde Null -> Python None
/// - Serde Bool -> Python bool
/// - Serde String -> Python str
/// - Serde Number -> Python int or float
/// - Serde Array -> Python list (with each item converted to Python type)
/// - Serde Object -> Python dict (with each key-value pair converted to Python type)
////// # Arguments
/// * `py` - A Python interpreter instance.
/// * `value` - A reference to a serde_json::Value object.
/// # Returns
/// * `Ok(PyObject)` if the conversion was successful.
/// * `Err(UtilError)` if the conversion failed.
pub fn json_to_pyobject(py: Python, value: &Value) -> Result<Py<PyAny>, UtilError> {
    Ok(match value {
        Value::Null => py.None(),
        Value::Bool(b) => b.into_py_any(py)?,
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into_py_any(py)?
            } else if let Some(f) = n.as_f64() {
                f.into_py_any(py)?
            } else {
                return Err(UtilError::InvalidNumber);
            }
        }
        Value::String(s) => s.into_py_any(py)?,
        Value::Array(arr) => {
            let py_list = PyList::empty(py);
            for item in arr {
                let py_item = json_to_pyobject(py, item)?;
                py_list.append(py_item)?;
            }
            py_list.into_py_any(py)?
        }
        Value::Object(_) => {
            let nested_dict = PyDict::new(py);
            json_to_pydict(py, value, &nested_dict)?;
            nested_dict.into_py_any(py)?
        }
    })
}

pub fn vec_to_py_object<'py>(
    py: Python<'py>,
    vec: &Vec<Value>,
) -> Result<Bound<'py, PyList>, UtilError> {
    let py_list = PyList::empty(py);
    for item in vec {
        let py_item = json_to_pyobject(py, item)?;
        py_list.append(py_item)?;
    }
    Ok(py_list)
}

pub fn pyobject_to_json(obj: &Bound<'_, PyAny>) -> Result<Value, UtilError> {
    if obj.is_instance_of::<PyDict>() {
        let dict = obj.cast::<PyDict>()?;
        let mut map = serde_json::Map::new();
        for (key, value) in dict.iter() {
            let key_str = key.extract::<String>()?;
            let json_value = pyobject_to_json(&value)?;
            map.insert(key_str, json_value);
        }
        Ok(Value::Object(map))
    } else if obj.is_instance_of::<PyList>() {
        let list = obj.cast::<PyList>()?;
        let mut vec = Vec::new();
        for item in list.iter() {
            vec.push(pyobject_to_json(&item)?);
        }
        Ok(Value::Array(vec))
    } else if obj.is_instance_of::<PyTuple>() {
        let tuple = obj.cast::<PyTuple>()?;
        let mut vec = Vec::new();
        for item in tuple.iter() {
            vec.push(pyobject_to_json(&item)?);
        }
        Ok(Value::Array(vec))
    } else if obj.is_instance_of::<PyString>() {
        let s = obj.extract::<String>()?;
        Ok(Value::String(s))
    } else if obj.is_instance_of::<PyFloat>() {
        let f = obj.extract::<f64>()?;
        Ok(json!(f))
    } else if obj.is_instance_of::<PyBool>() {
        let b = obj.extract::<bool>()?;
        Ok(json!(b))
    } else if obj.is_instance_of::<PyInt>() {
        let i = obj.extract::<i64>()?;
        Ok(json!(i))
    } else if obj.is_none() {
        Ok(Value::Null)
    } else {
        // display "cant show" for unsupported types
        // call obj.str to get the string representation
        // if error, default to "unsupported type"
        let obj_str = match obj.str() {
            Ok(s) => s
                .extract::<String>()
                .unwrap_or_else(|_| "unsupported type".to_string()),
            Err(_) => "unsupported type".to_string(),
        };

        Ok(Value::String(obj_str))
    }
}

pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn update_serde_value(value: &mut Value, key: &str, new_value: Value) -> Result<(), UtilError> {
    if let Value::Object(map) = value {
        map.insert(key.to_string(), new_value);
        Ok(())
    } else {
        Err(UtilError::RootMustBeObjectError)
    }
}

/// Updates a serde_json::Value object with another serde_json::Value object.
/// Both types must be of the `Object` variant.
/// If a key in the source object does not exist in the destination object,
/// it will be added with the value from the source object.
/// # Arguments
/// * `dest` - A mutable reference to the destination serde_json::Value object.
/// * `src` - A reference to the source serde_json::Value object.
/// # Returns
/// * `Ok(())` if the update was successful.    
/// * `Err(UtilError::RootMustBeObjectError)` if either `dest` or `src` is not an `Object`.
pub fn update_serde_map_with(
    dest: &mut serde_json::Value,
    src: &serde_json::Value,
) -> Result<(), UtilError> {
    match (dest, src) {
        (&mut Object(ref mut map_dest), Object(ref map_src)) => {
            // map_dest and map_src both are Map<String, Value>
            for (key, value) in map_src {
                // if key is not in map_dest, create a Null object
                // then only, update the value
                *map_dest.entry(key.clone()).or_insert(Null) = value.clone();
            }
            Ok(())
        }
        (_, _) => Err(UtilError::RootMustBeObjectError),
    }
}

/// Extracts a string value from a Python object.
pub fn extract_string_value(py_value: &Bound<'_, PyAny>) -> Result<String, UtilError> {
    // Try to extract as string first (most common case)
    if let Ok(string_val) = py_value.extract::<String>() {
        return Ok(string_val);
    }

    // Try to extract as integer
    if let Ok(int_val) = py_value.extract::<i64>() {
        return Ok(int_val.to_string());
    }

    // Try to extract as float
    if let Ok(float_val) = py_value.extract::<f64>() {
        return Ok(float_val.to_string());
    }

    // Try to extract as boolean
    if let Ok(bool_val) = py_value.extract::<bool>() {
        return Ok(bool_val.to_string());
    }

    // For complex objects, convert to JSON but extract the value without quotes
    let json_value = pyobject_to_json(py_value)?;

    match json_value {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Null => Ok("null".to_string()),
        _ => {
            // For arrays and objects, serialize to JSON string
            let json_string = serde_json::to_string(&json_value)?;
            Ok(json_string)
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Clone)]
pub struct TokenLogProbs {
    #[pyo3(get)]
    pub token: String,

    #[pyo3(get)]
    pub logprob: f64,
}

#[pyclass]
#[derive(Debug, Serialize, Clone)]
pub struct ResponseLogProbs {
    #[pyo3(get)]
    pub tokens: Vec<TokenLogProbs>,
}

#[pymethods]
impl ResponseLogProbs {
    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

/// Calculate a weighted score base on the log probabilities of tokens 1-5.
pub fn calculate_weighted_score(log_probs: &[TokenLogProbs]) -> Result<Option<f64>, UtilError> {
    let score_range = RangeInclusive::new(1, 5);
    let mut score_probs = Vec::new();
    let mut weighted_sum = 0.0;
    let mut total_prob = 0.0;

    for log_prob in log_probs {
        let token = log_prob.token.parse::<u64>().ok();

        if let Some(token_val) = token {
            if score_range.contains(&token_val) {
                let prob = log_prob.logprob.exp();
                score_probs.push((token_val, prob));
            }
        }
    }

    for (score, logprob) in score_probs {
        weighted_sum += score as f64 * logprob;
        total_prob += logprob;
    }

    if total_prob > 0.0 {
        Ok(Some(weighted_sum / total_prob))
    } else {
        Ok(None)
    }
}

/// Generic function to convert text to a structured output model
/// It is expected that output_model is a pydantic model or a potatohead type that implements serde json deserialization
/// via model_validate_json method.
/// Flow:
/// 1. Attempt to validate the model using model_validate_json
/// 2. If validation fails, attempt to parse the text as JSON and convert to python object
/// # Arguments
/// * `py` - A Python interpreter instance
/// * `text` - The text to be converted (typically from an LLM response that returns structured output)
/// * `output_model` - A bound python object representing the output model
/// # Returns
/// * `Result<Bound<'py, PyAny>, UtilError>` - A result containing the structured output or an error
pub fn convert_text_to_structured_output<'py>(
    py: Python<'py>,
    text: String,
    output_model: &Bound<'py, PyAny>,
) -> Result<Bound<'py, PyAny>, UtilError> {
    let output = output_model.call_method1("model_validate_json", (&text,));
    match output {
        Ok(obj) => {
            // Successfully validated the model
            Ok(obj)
        }
        Err(err) => {
            // Model validation failed
            // convert string to json and then to python object
            warn!(
                "Failed to validate model: {}, Attempting fallback to JSON parsing",
                err
            );
            let val = serde_json::from_str::<serde_json::Value>(&text)?;
            Ok(json_to_pyobject(py, &val)?.into_bound_py_any(py)?)
        }
    }
}

/// Helper function to extract result from LLM response text
/// If an output model is provided, it will attempt to convert the text to the structured output
/// using the provided model. If no model is provided, it will attempt to convert the response to an appropriate
/// Python type directly.
/// # Arguments
/// * `py` - A Python interpreter instance
/// * `text` - The text to be converted (typically from an LLM response)
/// * `output_model` - An optional bound python object representing the output model
/// # Returns
/// * `Result<Bound<'py, PyAny>, UtilError>` - A result containing the structured output or an error
pub fn construct_structured_response<'py>(
    py: Python<'py>,
    text: String,
    output_model: Option<&Bound<'py, PyAny>>,
) -> Result<Bound<'py, PyAny>, UtilError> {
    match output_model {
        Some(model) => convert_text_to_structured_output(py, text, model),
        None => {
            // No output model provided, return the text as a Python string
            let val = Value::String(text);
            Ok(json_to_pyobject(py, &val)?.into_bound_py_any(py)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_calculate_weighted_score() {
        let log_probs = vec![
            TokenLogProbs {
                token: "1".into(),
                logprob: 0.9,
            },
            TokenLogProbs {
                token: "2".into(),
                logprob: 0.8,
            },
            TokenLogProbs {
                token: "3".into(),
                logprob: 0.7,
            },
        ];

        let result = calculate_weighted_score(&log_probs);
        assert!(result.is_ok());

        let val = result.unwrap().unwrap();
        // round to int
        assert_eq!(val.round(), 2.0);
    }
    #[test]
    fn test_calculate_weighted_score_empty() {
        let log_probs: Vec<TokenLogProbs> = vec![];
        let result = calculate_weighted_score(&log_probs);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }
}
