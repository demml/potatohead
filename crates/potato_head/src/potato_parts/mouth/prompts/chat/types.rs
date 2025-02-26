use pyo3::prelude::*;
use pyo3::types::PyString;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Messages {
    Base(Message),
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatPartText {
    #[pyo3(get)]
    pub text: String,
    #[pyo3(get)]
    pub r#type: String,
}

#[pymethods]
impl ChatPartText {
    #[new]
    #[pyo3(signature = (text, r#type))]
    pub fn new(text: &str, r#type: &str) -> Self {
        Self {
            text: text.to_string(),
            r#type: r#type.to_string(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MessageContent {
    Text(String),
    Part(ChatPartText),
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    #[pyo3(get)]
    pub role: String,

    #[pyo3(get)]
    pub content: MessageContent,

    #[pyo3(get)]
    pub name: Option<String>,

    next_param: usize,
}

#[pymethods]
impl Message {
    #[new]
    #[pyo3(signature = (role, content, name=None))]
    pub fn new(role: &str, content: &Bound<'_, PyAny>, name: Option<&str>) -> PyResult<Self> {
        // check if content isinstance of PyString
        if content.is_instance_of::<PyString>() {
            let content = content.extract::<String>()?;
            return Ok(Self {
                role: role.to_string(),
                content: MessageContent::Text(content),
                name: name.map(|s| s.to_string()),
                next_param: 1,
            });
        }

        Ok(Self {
            role: role.to_string(),
            content: MessageContent::Part(content.extract()?),
            name: name.map(|s| s.to_string()),
            next_param: 1,
        })
    }

    pub fn bind(&mut self, value: &str) -> PyResult<()> {
        let placeholder = format!("${}", self.next_param);

        match &mut self.content {
            MessageContent::Text(content) => {
                *content = content.replace(&placeholder, value);
            }
            MessageContent::Part(part) => {
                let ChatPartText { text, .. } = part;
                *text = text.replace(&placeholder, value);
            }
        }

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
