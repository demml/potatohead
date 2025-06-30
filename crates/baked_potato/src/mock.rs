use crate::error::MockError;
use mockito;
use potato_agent::agents::provider::openai::OpenAIChatResponse;
use serde_json;

use pyo3::prelude::*;

const OPENAI_CHAT_COMPLETION_RESPONSE: &str =
    include_str!("assets/openai_chat_completion_response.json");

pub struct OpenAIMock {
    pub url: String,
    pub server: mockito::ServerGuard,
}

impl OpenAIMock {
    pub fn new() -> Self {
        let mut server = mockito::Server::new();
        // load the OpenAI chat completion response
        let chat_msg_response: OpenAIChatResponse =
            serde_json::from_str(OPENAI_CHAT_COMPLETION_RESPONSE).unwrap();

        // Openai chat completion mock
        server
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chat_msg_response).unwrap())
            .create();

        Self {
            url: server.url(),
            server,
        }
    }
}

impl Default for OpenAIMock {
    fn default() -> Self {
        Self::new()
    }
}

#[pyclass]
#[allow(dead_code)]
pub struct OpenAITestServer {
    openai_server: Option<OpenAIMock>,
}

#[pymethods]
impl OpenAITestServer {
    #[new]
    pub fn new() -> Self {
        OpenAITestServer {
            openai_server: None,
        }
    }

    pub fn start_mock_server(&mut self) -> Result<(), MockError> {
        let openai_server = OpenAIMock::new();
        println!("Mock OpenAI Server started at {}", openai_server.url);
        self.openai_server = Some(openai_server);
        Ok(())
    }

    pub fn stop_mock_server(&mut self) {
        if let Some(server) = self.openai_server.take() {
            drop(server);
            std::env::remove_var("OPENAI_API_URL");
            std::env::remove_var("OPENAI_API_KEY");
        }
        println!("Mock OpenAI Server stopped");
    }

    pub fn set_env_vars_for_client(&self) -> Result<(), MockError> {
        {
            std::env::set_var("APP_ENV", "dev_client");
            std::env::set_var("OPENAI_API_KEY", "test_key");
            std::env::set_var(
                "OPENAI_API_URL",
                self.openai_server.as_ref().unwrap().url.clone(),
            );
            Ok(())
        }
    }

    pub fn start_server(&mut self) -> Result<(), MockError> {
        self.cleanup()?;

        println!("Starting Mock GenAI Server...");
        self.start_mock_server()?;
        self.set_env_vars_for_client()?;

        // set server env vars
        std::env::set_var("APP_ENV", "dev_server");

        Ok(())
    }

    pub fn stop_server(&mut self) -> Result<(), MockError> {
        self.cleanup()?;

        Ok(())
    }

    pub fn remove_env_vars_for_client(&self) -> Result<(), MockError> {
        std::env::remove_var("OPENAI_API_URI");
        std::env::remove_var("OPENAI_API_KEY");
        Ok(())
    }

    fn cleanup(&self) -> Result<(), MockError> {
        // unset env vars
        self.remove_env_vars_for_client()?;

        Ok(())
    }

    fn __enter__(mut self_: PyRefMut<Self>) -> Result<PyRefMut<Self>, MockError> {
        self_.start_server()?;
        Ok(self_)
    }

    fn __exit__(
        &mut self,
        _exc_type: PyObject,
        _exc_value: PyObject,
        _traceback: PyObject,
    ) -> Result<(), MockError> {
        self.stop_server()
    }
}

impl Default for OpenAITestServer {
    fn default() -> Self {
        Self::new()
    }
}
