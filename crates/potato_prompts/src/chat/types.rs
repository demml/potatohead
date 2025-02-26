use potato_error::PotatoHeadError;
use potato_tools::Utils;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::types::{PyList, PyString};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MessageEnum {
    Base(Message),
}

impl MessageEnum {
    pub fn to_spec(&self) -> Value {
        match self {
            MessageEnum::Base(msg) => msg.to_spec(),
        }
    }

    pub fn bind(&mut self, value: &str) -> PyResult<()> {
        match self {
            MessageEnum::Base(msg) => msg.bind(value),
        }
    }

    pub fn reset_binding(&mut self) {
        match self {
            MessageEnum::Base(msg) => msg.reset_binding(),
        }
    }
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
pub struct ImageUrl {
    url: String,

    detail: String,
}

#[pymethods]
impl ImageUrl {
    #[new]
    #[pyo3(signature = (url, detail))]
    pub fn new(url: &str, detail: &str) -> Self {
        Self {
            url: url.to_string(),
            detail: detail.to_string(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatPartImage {
    #[pyo3(get)]
    pub image_url: ImageUrl,
    #[pyo3(get)]
    pub r#type: String,
}

#[pymethods]
impl ChatPartImage {
    #[new]
    #[pyo3(signature = (image_url, r#type))]
    pub fn new(image_url: ImageUrl, r#type: &str) -> Self {
        Self {
            image_url,
            r#type: r#type.to_string(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MessageContent {
    Text(String),
    Part(Vec<ChatPartText>),
    Image(Vec<ChatPartImage>),
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

impl Message {
    pub fn from_dict(py: Python, dict: &PyDict) -> PyResult<Self> {
        let role = dict.bind(py, "role")?;

        Self::new(&role, content, name.as_deref())
    }
}

#[pymethods]
impl Message {
    #[new]
    #[pyo3(signature = (role, content, name=None))]
    pub fn new(role: &str, content: &Bound<'_, PyAny>, name: Option<&str>) -> PyResult<Self> {
        // check if content isinstance of PyString (text)
        if content.is_instance_of::<PyString>() {
            let content = content.extract::<String>()?;
            return Ok(Self {
                role: role.to_string(),
                content: MessageContent::Text(content),
                name: name.map(|s| s.to_string()),
                next_param: 1,
            });
        }

        // check if content is a list (part, image)
        if content.is_instance_of::<PyList>() {
            // attempt to extract the first element of the list
            let first_elem = content.get_item(0)?;

            // check if the first element is a ChatPartText,
            // if it is extract pylist into Vec<ChatPartText>
            // if not, check if it is a ChatPartImage
            // if it is extract pylist into Vec<ChatPartImage>
            if first_elem.is_instance_of::<ChatPartText>() {
                // extract pylist into Vec<ChatPartText>
                let content = content.extract::<Vec<ChatPartText>>()?;
                return Ok(Self {
                    role: role.to_string(),
                    content: MessageContent::Part(content),
                    name: name.map(|s| s.to_string()),
                    next_param: 1,
                });
            } else if content.is_instance_of::<ChatPartImage>() {
                // extract pylist into Vec<ChatPartImage>
                let content = content.extract::<Vec<ChatPartImage>>()?;
                return Ok(Self {
                    role: role.to_string(),
                    content: MessageContent::Image(content),
                    name: name.map(|s| s.to_string()),
                    next_param: 1,
                });
            }
        }

        if content.is_instance_of::<ChatPartText>() {
            let content = content.extract::<ChatPartText>()?;
            return Ok(Self {
                role: role.to_string(),
                content: MessageContent::Part(vec![content]),
                name: name.map(|s| s.to_string()),
                next_param: 1,
            });
        } else if content.is_instance_of::<ChatPartImage>() {
            let content = content.extract::<ChatPartImage>()?;
            return Ok(Self {
                role: role.to_string(),
                content: MessageContent::Image(vec![content]),
                name: name.map(|s| s.to_string()),
                next_param: 1,
            });
        }

        Err(PotatoHeadError::new_err("Invalid content type"))
    }

    pub fn bind(&mut self, value: &str) -> PyResult<()> {
        let placeholder = format!("${}", self.next_param);

        match &mut self.content {
            MessageContent::Text(content) => {
                *content = content.replace(&placeholder, value);
            }
            MessageContent::Part(part) => {
                for p in part {
                    p.text = p.text.replace(&placeholder, value);
                }
            }

            // fail for image
            MessageContent::Image(_) => {
                return Err(PotatoHeadError::new_err(
                    "Cannot bind value to image content",
                ));
            }
        }

        self.next_param += 1;
        Ok(())
    }

    pub fn reset_binding(&mut self) {
        self.next_param = 1;
    }

    pub fn __str__(&self) -> String {
        Utils::__str__(self)
    }
}

impl Message {
    pub fn to_spec(&self) -> Value {
        // If you need to further customize the output, you can use a custom serializer
        match &self.content {
            MessageContent::Text(text) => json!({
                "role": self.role,
                "content": text,
                "name": self.name,
            }),
            MessageContent::Part(parts) => json!({
                "role": self.role,
                "content": parts,
                "name": self.name,
            }),

            MessageContent::Image(images) => json!({
                "role": self.role,
                "content": images,
                "name": self.name,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_part_text_creation() {
        let chat_part = ChatPartText::new("Hello", "text");
        assert_eq!(chat_part.text, "Hello");
        assert_eq!(chat_part.r#type, "text");
    }

    #[test]
    fn test_message_text_content() {
        let content = "Hello, world!";
        let message = Message {
            role: "user".to_string(),
            content: MessageContent::Text(content.to_string()),
            name: None,
            next_param: 1,
        };

        match &message.content {
            MessageContent::Text(text) => assert_eq!(text, content),
            _ => panic!("Expected Text content"),
        }

        assert_eq!(message.role, "user");
        assert_eq!(message.name, None);
    }

    #[test]
    fn test_message_part_content() {
        let chat_part = ChatPartText::new("Hello", "text");
        let message = Message {
            role: "assistant".to_string(),
            content: MessageContent::Part(vec![chat_part]),
            name: Some("bot".to_string()),
            next_param: 1,
        };

        match &message.content {
            MessageContent::Part(part) => {
                assert_eq!(part.len(), 1);
                assert_eq!(part[0].text, "Hello");
                assert_eq!(part[0].r#type, "text");
            }
            _ => panic!("Expected Part content"),
        }
        assert_eq!(message.role, "assistant");
        assert_eq!(message.name, Some("bot".to_string()));

        println!("{}", message.to_spec());
    }
}
