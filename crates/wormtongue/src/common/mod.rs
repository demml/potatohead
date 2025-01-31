use crate::error::TongueError;
use chrono::Utc;
use colored_json::{Color, ColorMode, ColoredFormatter, PrettyFormatter, Styler};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub struct Utils {}

impl Utils {
    pub fn __str__<T: Serialize>(object: T) -> String {
        match ColoredFormatter::with_styler(
            PrettyFormatter::default(),
            Styler {
                key: Color::Rgb(245, 77, 85).bold(),
                string_value: Color::Rgb(249, 179, 93).foreground(),
                float_value: Color::Rgb(249, 179, 93).foreground(),
                integer_value: Color::Rgb(249, 179, 93).foreground(),
                bool_value: Color::Rgb(249, 179, 93).foreground(),
                nil_value: Color::Rgb(249, 179, 93).foreground(),
                ..Default::default()
            },
        )
        .to_colored_json(&object, ColorMode::On)
        {
            Ok(json) => json,
            Err(e) => format!("Failed to serialize to json: {}", e),
        }
        // serialize the struct to a string
    }

    pub fn __json__<T: Serialize>(object: T) -> String {
        match serde_json::to_string_pretty(&object) {
            Ok(json) => json,
            Err(e) => format!("Failed to serialize to json: {}", e),
        }
    }

    pub fn save_to_json<T>(
        model: T,
        path: Option<PathBuf>,
        filename: &str,
    ) -> Result<(), TongueError>
    where
        T: Serialize,
    {
        // serialize the struct to a string
        let json = serde_json::to_string_pretty(&model).map_err(|_| TongueError::SerializeError)?;

        // check if path is provided
        let write_path = if path.is_some() {
            let mut new_path = path.ok_or(TongueError::CreatePathError)?;

            // ensure .json extension
            new_path.set_extension("json");

            if !new_path.exists() {
                // ensure path exists, create if not
                let parent_path = new_path.parent().ok_or(TongueError::GetParentPathError)?;

                std::fs::create_dir_all(parent_path)
                    .map_err(|_| TongueError::CreateDirectoryError)?;
            }

            new_path
        } else {
            PathBuf::from(filename)
        };

        std::fs::write(write_path, json).map_err(|_| TongueError::WriteError)?;

        Ok(())
    }
}

pub enum FileName {
    OpenAIPrompt,
    ClaudePrompt,
    Prompt,
}

impl FileName {
    pub fn to_str(&self) -> String {
        // add current timestamp to filename
        let now = Utc::now().naive_utc().to_string();
        match self {
            FileName::OpenAIPrompt => format!("openai_prompt_{}", now),
            FileName::ClaudePrompt => format!("claude_prompt_{}", now),
            FileName::Prompt => format!("prompt_{}", now),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    InProgress,
    Completed,
    Failed,
    NotStarted,
}

#[pyclass(eq)]
#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub enum PromptType {
    OpenAI,
    OpenAICompatible,
    Claude,
}
