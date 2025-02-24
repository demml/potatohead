use crate::error::WormTongueError;
use crate::tongues::common::Utils;
use pyo3::{
    prelude::*,
    types::{PyList, PyString},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    #[pyo3(get, set)]
    pub role: String,
    #[pyo3(get, set)]
    pub content: String,

    next_param: usize,
}

#[pymethods]
impl Message {
    #[new]
    #[pyo3(signature = (role, content))]
    pub fn new(role: &str, content: &Bound<'_, PyAny>) -> PyResult<Self> {
        content
            .is_instance_of::<PyString>()
            .then(|| content.extract::<String>())
            .map_or_else(
                || Err(WormTongueError::new_err("Content type must be a string")),
                |result| {
                    result
                        .map(|content| Self {
                            role: role.to_string(),
                            content,
                            next_param: 0,
                        })
                        .map_err(WormTongueError::new_err)
                },
            )
    }

    pub fn bind(&mut self, value: &str) -> PyResult<()> {
        let placeholder = format!("${}", self.next_param);
        self.content = self.content.replace(&placeholder, value);
        self.next_param += 1;
        Ok(())
    }

    pub fn reset_binding(&mut self) {
        self.next_param = 0;
    }
}
