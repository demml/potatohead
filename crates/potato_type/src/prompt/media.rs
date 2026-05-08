use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD, Engine};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use crate::error::TypeError;

#[pyclass(from_py_object, eq, eq_int)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Image,
    Document,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MediaSource {
    Url {
        url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
    },
    Base64 {
        mime_type: String,
        data: String,
    },
}

#[pyclass(from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaRef {
    #[pyo3(get)]
    pub kind: MediaKind,
    pub source: MediaSource,
}

impl MediaRef {
    pub fn new_url(kind: MediaKind, url: String, mime_type: Option<String>) -> Self {
        Self {
            kind,
            source: MediaSource::Url { url, mime_type },
        }
    }

    pub fn new_base64(kind: MediaKind, mime_type: String, data: String) -> Self {
        Self {
            kind,
            source: MediaSource::Base64 { mime_type, data },
        }
    }

    pub fn from_bytes(kind: MediaKind, mime_type: String, data: &[u8]) -> Self {
        let encoded = STANDARD.encode(data);
        Self::new_base64(kind, mime_type, encoded)
    }

    pub fn from_path(kind: MediaKind, path: &Path) -> Result<Self, TypeError> {
        let mime = infer_mime_from_path(path)?;
        let bytes = std::fs::read(path)?;
        Ok(Self::from_bytes(kind, mime, &bytes))
    }
}

#[pymethods]
impl MediaRef {
    #[staticmethod]
    #[pyo3(signature = (url, mime_type=None))]
    pub fn image_url(url: String, mime_type: Option<String>) -> Self {
        Self::new_url(MediaKind::Image, url, mime_type)
    }

    #[staticmethod]
    pub fn image_bytes(mime_type: String, data: &[u8]) -> Self {
        Self::from_bytes(MediaKind::Image, mime_type, data)
    }

    #[staticmethod]
    pub fn image_path(path: PathBuf) -> Result<Self, TypeError> {
        Self::from_path(MediaKind::Image, &path)
    }

    #[staticmethod]
    #[pyo3(signature = (url, mime_type=None))]
    pub fn document_url(url: String, mime_type: Option<String>) -> Self {
        Self::new_url(MediaKind::Document, url, mime_type)
    }

    #[staticmethod]
    pub fn document_bytes(mime_type: String, data: &[u8]) -> Self {
        Self::from_bytes(MediaKind::Document, mime_type, data)
    }

    #[staticmethod]
    pub fn document_path(path: PathBuf) -> Result<Self, TypeError> {
        Self::from_path(MediaKind::Document, &path)
    }

    fn __repr__(&self) -> String {
        format!("MediaRef(kind={:?})", self.kind)
    }
}

fn infer_mime_from_path(path: &Path) -> Result<String, TypeError> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(str::to_ascii_lowercase);
    match ext.as_deref() {
        Some("png") => Ok("image/png".into()),
        Some("jpg") | Some("jpeg") => Ok("image/jpeg".into()),
        Some("gif") => Ok("image/gif".into()),
        Some("webp") => Ok("image/webp".into()),
        Some("pdf") => Ok("application/pdf".into()),
        Some("txt") => Ok("text/plain".into()),
        Some(other) => Err(TypeError::InvalidMediaType(format!(
            "unrecognized extension: {other}"
        ))),
        None => Err(TypeError::InvalidMediaType(
            "path has no extension; pass mime_type explicitly via image_bytes".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_bytes_eager_encodes() {
        let m = MediaRef::from_bytes(MediaKind::Image, "image/png".into(), b"hello");
        match m.source {
            MediaSource::Base64 { data, mime_type } => {
                assert_eq!(data, "aGVsbG8=");
                assert_eq!(mime_type, "image/png");
            }
            _ => panic!("expected base64 source"),
        }
    }

    #[test]
    fn url_source_carries_optional_mime() {
        let m = MediaRef::new_url(
            MediaKind::Image,
            "gs://b/c.png".into(),
            Some("image/png".into()),
        );
        match m.source {
            MediaSource::Url { url, mime_type } => {
                assert_eq!(url, "gs://b/c.png");
                assert_eq!(mime_type.as_deref(), Some("image/png"));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn image_path_infers_mime_from_extension() {
        let tmp_dir = std::env::temp_dir().join("potatohead_media_test");
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let p = tmp_dir.join("chart.png");
        std::fs::write(&p, b"FAKEPNG").unwrap();
        let m = MediaRef::from_path(MediaKind::Image, &p).unwrap();
        match &m.source {
            MediaSource::Base64 { mime_type, .. } => assert_eq!(mime_type, "image/png"),
            _ => panic!(),
        }
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn image_path_unknown_extension_errors() {
        let tmp_dir = std::env::temp_dir().join("potatohead_media_test2");
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let p = tmp_dir.join("chart.xyz");
        std::fs::write(&p, b"X").unwrap();
        assert!(matches!(
            MediaRef::from_path(MediaKind::Image, &p),
            Err(TypeError::InvalidMediaType(_))
        ));
        std::fs::remove_file(&p).ok();
    }
}
