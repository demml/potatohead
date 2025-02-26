use potato_error::PotatoHeadError;
use potato_tools::Utils;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::types::{PyList, PyString};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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
    #[pyo3(signature = (text, r#type="text"))]
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
    #[pyo3(signature = (url, detail="auto"))]
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
    #[pyo3(signature = (image_url, r#type="image_url"))]
    pub fn new(image_url: ImageUrl, r#type: &str) -> Self {
        Self {
            image_url,
            r#type: r#type.to_string(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputAudio {
    data: String,

    format: String,
}

#[pymethods]
impl InputAudio {
    #[new]
    #[pyo3(signature = (data, format="wav"))]
    pub fn new(data: &str, format: &str) -> Self {
        Self {
            data: data.to_string(),
            format: format.to_string(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatPartAudio {
    #[pyo3(get)]
    pub input_audio: InputAudio,
    #[pyo3(get)]
    pub r#type: String,
}

#[pymethods]
impl ChatPartAudio {
    #[new]
    #[pyo3(signature = (input_audio, r#type="input_audio"))]
    pub fn new(input_audio: InputAudio, r#type: &str) -> Self {
        Self {
            input_audio,
            r#type: r#type.to_string(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MessagePart {
    Text(ChatPartText),
    Image(ChatPartImage),
    Audio(ChatPartAudio),
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<MessagePart>),
}

/// Generic helper for extracting a value from a dictionary
fn extract_value_from_dict(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<String> {
    match dict.get_item(key)? {
        Some(value) => value.extract::<String>(),
        None => Err(PotatoHeadError::new_err(format!("Missing key: {}", key))),
    }
}

/// convert audio content from a dictionary to a MessagePart
fn extract_audio_from_dict(dict: &Bound<'_, PyDict>, r#type: &str) -> PyResult<MessagePart> {
    let data = extract_value_from_dict(dict, "data")?;
    let format = extract_value_from_dict(dict, "format")?;

    Ok(MessagePart::Audio(ChatPartAudio::new(
        InputAudio { data, format },
        r#type,
    )))
}

/// convert text content from a dictionary to a MessagePart
fn extract_text_from_dict(dict: &Bound<'_, PyDict>, r#type: &str) -> PyResult<MessagePart> {
    let text = extract_value_from_dict(dict, "text")?;
    Ok(MessagePart::Text(ChatPartText::new(&text, r#type)))
}

/// convert image content from a dictionary to a MessagePart
fn extract_image_from_dict(dict: &Bound<'_, PyDict>, r#type: &str) -> PyResult<MessagePart> {
    let image_dict = dict.get_item("image_url")?.unwrap();
    let image_url_dict = image_dict.downcast::<PyDict>()?;

    let url = extract_value_from_dict(image_url_dict, "url")?;
    let detail = match extract_value_from_dict(image_url_dict, "detail") {
        Ok(value) => value,
        Err(_) => "auto".to_string(),
    };

    Ok(MessagePart::Image(ChatPartImage::new(
        ImageUrl { url, detail },
        r#type,
    )))
}

/// Extract a MessagePart from a PyAny object
fn extract_part(item: &Bound<'_, PyAny>) -> PyResult<Option<MessagePart>> {
    // First try direct instance checks
    if item.is_instance_of::<ChatPartText>() {
        return Ok(Some(MessagePart::Text(item.extract::<ChatPartText>()?)));
    } else if item.is_instance_of::<ChatPartImage>() {
        return Ok(Some(MessagePart::Image(item.extract::<ChatPartImage>()?)));
    } else if item.is_instance_of::<ChatPartAudio>() {
        return Ok(Some(MessagePart::Audio(item.extract::<ChatPartAudio>()?)));
    }

    // If not a direct instance, try dictionary conversion
    if item.is_instance_of::<PyDict>() {
        let dict = item.downcast::<PyDict>()?;

        // Try to determine the type from the dictionary
        let type_str = match extract_value_from_dict(dict, "type") {
            Ok(value) => value,
            Err(_) => return Ok(None),
        };

        match type_str.as_str() {
            // parse text content
            "text" => match extract_text_from_dict(dict, &type_str) {
                Ok(part) => return Ok(Some(part)),
                Err(_) => return Ok(None),
            },

            // parse image content
            "image_url" => match extract_image_from_dict(dict, &type_str) {
                Ok(part) => return Ok(Some(part)),
                Err(_) => return Ok(None),
            },

            // parse audio content
            "input_audio" => {
                if let Ok(part) = extract_audio_from_dict(dict, &type_str) {
                    return Ok(Some(part));
                }
            }
            _ => return Ok(None),
        }
    }

    Ok(None)
}

fn extract_content(content: &Bound<'_, PyAny>) -> PyResult<Option<MessageContent>> {
    // Handle text content
    if content.is_instance_of::<PyString>() {
        return Ok(Some(MessageContent::Text(content.extract::<String>()?)));
    }

    // Handle list content
    if content.is_instance_of::<PyList>() {
        let list = content.downcast::<PyList>()?;
        let mut parts = Vec::new();

        for item in list.iter() {
            // attempt to extract a part from the item
            if let Some(part) = extract_part(&item)? {
                parts.push(part);
            } else {
                return Ok(None);
            }
        }

        if !parts.is_empty() {
            return Ok(Some(MessageContent::Parts(parts)));
        }
        return Ok(None);
    }

    // Handle single part content
    if let Some(part) = extract_part(content)? {
        return Ok(Some(MessageContent::Parts(vec![part])));
    }

    Ok(None)
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
        // Extracting content to ensure it is a valid type
        let content = match extract_content(content)? {
            Some(content) => content,
            None => return Err(PotatoHeadError::new_err("Invalid content type")),
        };

        Ok(Self {
            role: role.to_string(),
            content,
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
            MessageContent::Parts(parts) => {
                for part in parts {
                    match part {
                        MessagePart::Text(text_part) => {
                            text_part.text = text_part.text.replace(&placeholder, value);
                        }
                        MessagePart::Image(_) => {
                            return Err(PotatoHeadError::new_err(
                                "Cannot bind value to image content",
                            ));
                        }
                        MessagePart::Audio(_) => {
                            return Err(PotatoHeadError::new_err(
                                "Cannot bind value to audio content",
                            ));
                        }
                    }
                }
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

    pub fn to_api_spec(&self) -> String {
        let val = match &self.content {
            MessageContent::Text(text) => json!({
                "role": self.role,
                "content": text,
                "name": self.name,
            }),
            MessageContent::Parts(parts) => {
                let content: Vec<Value> = parts
                    .iter()
                    .map(|part| match part {
                        MessagePart::Text(text) => json!({
                            "text": text.text,
                            "type": text.r#type,
                        }),
                        MessagePart::Image(image) => json!({
                            "image_url": {
                                "url": image.image_url.url,
                                "detail": image.image_url.detail,
                            },
                            "type": image.r#type,
                        }),
                        MessagePart::Audio(audio) => json!({
                            "input_audio": {
                                "data": audio.input_audio.data,
                                "format": audio.input_audio.format,
                            },
                            "type": audio.r#type,
                        }),
                    })
                    .collect();

                json!({
                    "role": self.role,
                    "content": content,
                    "name": self.name,
                })
            }
        };

        Utils::__str__(val)
    }
}

impl Message {
    pub fn to_spec(&self) -> Value {
        match &self.content {
            MessageContent::Text(text) => json!({
                "role": self.role,
                "content": text,
                "name": self.name,
            }),
            MessageContent::Parts(parts) => {
                let content: Vec<Value> = parts
                    .iter()
                    .map(|part| match part {
                        MessagePart::Text(text) => json!({
                            "text": text.text,
                            "type": text.r#type,
                        }),
                        MessagePart::Image(image) => json!({
                            "image_url": {
                                "url": image.image_url.url,
                                "detail": image.image_url.detail,
                            },
                            "type": image.r#type,
                        }),
                        MessagePart::Audio(audio) => json!({
                            "input_audio": {
                                "data": audio.input_audio.data,
                                "format": audio.input_audio.format,
                            },
                            "type": audio.r#type,
                        }),
                    })
                    .collect();

                json!({
                    "role": self.role,
                    "content": content,
                    "name": self.name,
                })
            }
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
            content: MessageContent::Parts(vec![MessagePart::Text(chat_part)]),
            name: Some("bot".to_string()),
            next_param: 1,
        };

        match &message.content {
            MessageContent::Parts(part) => {
                assert_eq!(part.len(), 1);
                match &part[0] {
                    MessagePart::Text(text) => {
                        assert_eq!(text.text, "Hello");
                        assert_eq!(text.r#type, "text");
                    }
                    _ => panic!("Expected Text part"),
                }
            }
            _ => panic!("Expected Part content"),
        }
        assert_eq!(message.role, "assistant");
        assert_eq!(message.name, Some("bot".to_string()));

        println!("{}", message.to_spec());
    }
}
