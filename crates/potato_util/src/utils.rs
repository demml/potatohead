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
use std::path::Path;
use uuid::Uuid;

pub fn create_uuid7() -> String {
    Uuid::now_v7().to_string()
}

pub struct PyHelperFuncs {}

impl PyHelperFuncs {
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

pub fn json_to_pyobject<'py>(
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
                            let py_item = json_to_pyobject_value(py, item)?;
                            py_list.append(py_item)?;
                        }
                        py_list.into_py_any(py)?
                    }
                    Value::Object(_) => {
                        let nested_dict = PyDict::new(py);
                        json_to_pyobject(py, v, &nested_dict)?;
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

pub fn json_to_pyobject_value(py: Python, value: &Value) -> Result<PyObject, UtilError> {
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
                let py_item = json_to_pyobject_value(py, item)?;
                py_list.append(py_item)?;
            }
            py_list.into_py_any(py)?
        }
        Value::Object(_) => {
            let nested_dict = PyDict::new(py);
            json_to_pyobject(py, value, &nested_dict)?;
            nested_dict.into_py_any(py)?
        }
    })
}

pub fn pyobject_to_json(obj: &Bound<'_, PyAny>) -> Result<Value, UtilError> {
    if obj.is_instance_of::<PyDict>() {
        let dict = obj.downcast::<PyDict>()?;
        let mut map = serde_json::Map::new();
        for (key, value) in dict.iter() {
            let key_str = key.extract::<String>()?;
            let json_value = pyobject_to_json(&value)?;
            map.insert(key_str, json_value);
        }
        Ok(Value::Object(map))
    } else if obj.is_instance_of::<PyList>() {
        let list = obj.downcast::<PyList>()?;
        let mut vec = Vec::new();
        for item in list.iter() {
            vec.push(pyobject_to_json(&item)?);
        }
        Ok(Value::Array(vec))
    } else if obj.is_instance_of::<PyTuple>() {
        let tuple = obj.downcast::<PyTuple>()?;
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
