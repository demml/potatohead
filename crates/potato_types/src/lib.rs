use pyo3::prelude::*;
use std::fmt;
use std::fmt::Display;
use std::path::{Path, PathBuf};

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Clone)]
pub enum SaveName {
    Prompt,
}

#[pymethods]
impl SaveName {
    #[staticmethod]
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "prompt" => Some(SaveName::Prompt),

            _ => None,
        }
    }

    pub fn as_string(&self) -> &str {
        match self {
            SaveName::Prompt => "prompt",
        }
    }

    pub fn __str__(&self) -> String {
        self.to_string()
    }
}

impl Display for SaveName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl AsRef<Path> for SaveName {
    fn as_ref(&self) -> &Path {
        match self {
            SaveName::Prompt => Path::new("prompt"),
        }
    }
}

// impl PathBuf: From<SaveName>
impl From<SaveName> for PathBuf {
    fn from(save_name: SaveName) -> Self {
        PathBuf::from(save_name.as_ref())
    }
}
