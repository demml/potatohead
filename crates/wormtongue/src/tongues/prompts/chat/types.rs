use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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
    pub fn new(role: &str, content: &str) -> PyResult<Self> {
        Ok(Self {
            role: role.to_string(),
            content: content.to_string(),
            next_param: 1,
        })
    }

    pub fn bind(&mut self, value: &str) -> PyResult<()> {
        let placeholder = format!("${}", self.next_param);
        self.content = self.content.replace(&placeholder, value);
        self.next_param += 1;
        Ok(())
    }

    pub fn reset_binding(&mut self) {
        self.next_param = 1;
    }

    pub fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}: {}", self.role, self.content))
    }
}

impl Message {
    pub fn to_spec(&self) -> Value {
        json!({
            "role": self.role,
            "content": self.content,
        })
    }
}
