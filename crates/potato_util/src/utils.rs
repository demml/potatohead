use crate::error::UtilError;

use colored_json::{Color, ColorMode, ColoredFormatter, PrettyFormatter, Styler};
use pyo3::prelude::*;

use pyo3::types::{PyAny, PyDict, PyList};
use pyo3::IntoPyObjectExt;
use serde::Serialize;
use serde_json::Value;
use serde_json::Value::{Null, Object};
use std::ops::RangeInclusive;
use std::path::Path;
use uuid::Uuid;
pub fn create_uuid7() -> String {
    Uuid::now_v7().to_string()
}
use pythonize::{depythonize, pythonize};
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

pub fn vec_to_py_object<'py>(
    py: Python<'py>,
    vec: &Vec<Value>,
) -> Result<Bound<'py, PyList>, UtilError> {
    let py_list = PyList::empty(py);
    for item in vec {
        let py_item = pythonize(py, item)?;
        py_list.append(py_item)?;
    }
    Ok(py_list)
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
    // Try to extract as boolean
    if let Ok(bool_val) = py_value.extract::<bool>() {
        return Ok(bool_val.to_string());
    }

    // Try to extract as integer
    if let Ok(int_val) = py_value.extract::<i64>() {
        return Ok(int_val.to_string());
    }

    // Try to extract as float
    if let Ok(float_val) = py_value.extract::<f64>() {
        return Ok(float_val.to_string());
    }

    // For complex objects, convert to JSON but extract the value without quotes
    let json_value = depythonize(py_value)?;

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
            Ok(pythonize(py, &val)?)
        }
    }
}

pub fn is_pydantic_basemodel(py: Python, obj: &Bound<'_, PyAny>) -> Result<bool, UtilError> {
    let pydantic = match py.import("pydantic") {
        Ok(module) => module,
        // return false if pydantic cannot be imported
        Err(_) => return Ok(false),
    };

    let basemodel = pydantic.getattr("BaseModel")?;

    // check if context is a pydantic model
    let is_basemodel = obj
        .is_instance(&basemodel)
        .map_err(|e| UtilError::FailedToCheckPydanticModel(e.to_string()))?;

    Ok(is_basemodel)
}

fn process_dict_with_nested_models(
    py: Python<'_>,
    dict: &Bound<'_, PyAny>,
) -> Result<Value, UtilError> {
    let py_dict = dict.cast::<PyDict>()?;
    let mut result = serde_json::Map::new();

    for (key, value) in py_dict.iter() {
        let key_str: String = key.extract()?;
        let processed_value = depythonize_object_to_value(py, &value)?;
        result.insert(key_str, processed_value);
    }

    Ok(Value::Object(result))
}

pub fn depythonize_object_to_value<'py>(
    py: Python<'py>,
    value: &Bound<'py, PyAny>,
) -> Result<Value, UtilError> {
    let py_value = if is_pydantic_basemodel(py, value)? {
        let model = value.call_method0("model_dump")?;
        depythonize(&model)?
    } else if value.is_instance_of::<PyDict>() {
        process_dict_with_nested_models(py, value)?
    } else {
        depythonize(value)?
    };
    Ok(py_value)
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
            Ok(pythonize(py, &val)?)
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
