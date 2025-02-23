use crate::error::WormTongueError;
use crate::OpenAIPrompt;
use crate::{tongues::common::TongueType, tongues::openai::OpenAIInterface};
use pyo3::prelude::*;
use tracing::{debug, info, instrument};

#[derive(Debug, Clone)]
pub enum Interface {
    OpenAI(OpenAIInterface),
}

#[pyclass]
#[derive(Debug)]
pub struct Tongue {
    pub interface: Interface,

    pub prompt: PyObject,
}

#[pymethods]
impl Tongue {
    #[new]
    #[pyo3(signature = (prompt, url=None, api_key=None, organization=None, project=None))]
    pub fn new(
        prompt: &Bound<'_, PyAny>,
        url: Option<&str>,
        api_key: Option<&str>,
        organization: Option<&str>,
        project: Option<&str>,
    ) -> PyResult<Self> {
        let tongue_type = prompt
            .getattr("tongue_type")
            .map_err(|e| WormTongueError::new_err(e))?
            .extract::<TongueType>()?;

        match tongue_type {
            TongueType::OpenAI => {
                let rust_prompt = prompt.extract::<OpenAIPrompt>()?;
                let interface =
                    OpenAIInterface::new(&rust_prompt, url, api_key, organiation, project, None)?;

                return Ok(Self {
                    interface: Interface::OpenAI(interface),
                    prompt: prompt.clone().unbind(),
                });
            }
            _ => Err(WormTongueError::new_err("Invalid tongue type")),
        }
    }

    #[getter]
    pub fn get_prompt<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        Ok(self.prompt.bind(py).clone())
    }

    #[instrument(skip(self))]
    pub fn send(&self) -> PyResult<String> {
        debug!("Sending message");
        match &self.interface {
            Interface::OpenAI(interface) => {
                let response = interface.send()?;
                let response: serde_json::Value = response
                    .json()
                    .map_err(|e| WormTongueError::new_err(e.to_string()))?;
                Ok(response.to_string())
            }
        }
    }

    pub fn add_message(&mut self, message: &Bound<'_, PyAny>) -> PyResult<()> {
        match &mut self.interface {
            Interface::OpenAI(interface) => {
                let message = message.extract()?;
                interface.prompt.add_message(message);
            }
        }

        Ok(())
    }
}
