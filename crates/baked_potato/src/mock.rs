use crate::error::MockError;
use mockito;
use potato_type::google::v1::generate::{DataNum, GenerateContentResponse};
use potato_type::google::GeminiEmbeddingResponse;
use potato_type::openai::v1::embedding::OpenAIEmbeddingResponse;
use potato_type::openai::v1::OpenAIChatResponse;
use pyo3::prelude::*;
use rand::Rng;
use serde_json;

pub const OPENAI_EMBEDDING_RESPONSE: &str = include_str!("assets/openai/embedding_response.json");

pub const GEMINI_EMBEDDING_RESPONSE: &str = include_str!("assets/gemini/embedding_response.json");

pub const OPENAI_CHAT_COMPLETION_RESPONSE: &str =
    include_str!("assets/openai/openai_chat_completion_response.json");

pub const OPENAI_CHAT_STRUCTURED_RESPONSE: &str =
    include_str!("assets/openai/chat_completion_structured_response.json");

pub const OPENAI_CHAT_STRUCTURED_SCORE_RESPONSE: &str =
    include_str!("assets/openai/chat_completion_structured_score_response.json");

pub const OPENAI_CHAT_STRUCTURED_RESPONSE_PARAMS: &str =
    include_str!("assets/openai/chat_completion_structured_response_params.json");

pub const OPENAI_CHAT_STRUCTURED_TASK_OUTPUT: &str =
    include_str!("assets/openai/chat_completion_structured_task_output.json");

pub const GEMINI_CHAT_COMPLETION_RESPONSE: &str =
    include_str!("assets/gemini/chat_completion.json");

pub const GEMINI_CHAT_COMPLETION_RESPONSE_WITH_SCORE: &str =
    include_str!("assets/gemini/chat_completion_with_score.json");

fn randomize_openai_embedding_response(
    response: OpenAIEmbeddingResponse,
) -> OpenAIEmbeddingResponse {
    // create random Vec<f32> of length 512
    let mut cloned_response = response.clone();
    let mut rng = rand::rng();
    let embedding: Vec<f32> = (0..512).map(|_| rng.random_range(-1.0..1.0)).collect();
    cloned_response.data[0].embedding = embedding;
    cloned_response
}

fn randomize_gemini_embedding_response(
    response: GeminiEmbeddingResponse,
) -> GeminiEmbeddingResponse {
    let mut cloned_response = response.clone();
    let mut rng = rand::rng();
    let embedding: Vec<f32> = (0..512).map(|_| rng.random_range(-1.0..1.0)).collect();
    cloned_response.embedding.values = embedding;
    cloned_response
}

fn randomize_structured_openai_score_response(response: &OpenAIChatResponse) -> OpenAIChatResponse {
    let mut cloned_response = response.clone();
    let mut rng = rand::rng();

    // Generate random score between 1 and 5
    let score = rng.random_range(1..=5);

    // Generate random reason from a set of predefined reasons
    let reasons = [
        "The code is excellent and follows best practices.",
        "The implementation is solid with minor improvements possible.",
        "The code works but could use some optimization.",
        "The solution is functional but needs refactoring.",
        "The code has significant issues that need addressing.",
    ];
    let reason = reasons[rng.random_range(0..reasons.len())];

    cloned_response.choices[0].message.content = Some(format!(
        "{{ \"score\": {}, \"reason\": \"{}\" }}",
        score, reason
    ));

    cloned_response
}

fn randomize_gemini_score_response(response: GenerateContentResponse) -> GenerateContentResponse {
    let mut cloned_response = response.clone();
    let mut rng = rand::rng();

    // Generate random score between 1 and 100 (typical for Gemini scoring)
    let score = rng.random_range(1..=100);

    // Generate random reason from a set of predefined reasons
    let reasons = [
        "The model performed exceptionally well on the evaluation.",
        "Good performance with room for minor improvements.",
        "Satisfactory results with some areas for optimization.",
        "Adequate performance but needs significant improvements.",
        "Performance below expectations, requires major adjustments.",
    ];
    let reason = reasons[rng.random_range(0..reasons.len())];

    // Update the first candidate's content
    if let Some(candidate) = cloned_response.candidates.get_mut(0) {
        if let Some(part) = candidate.content.parts.get_mut(0) {
            part.data = DataNum::Text(format!(
                "{{\"score\": {}, \"reason\": \"{}\"}}",
                score, reason
            ));
        }
    }

    cloned_response
}

pub struct LLMApiMock {
    pub url: String,
    pub server: mockito::ServerGuard,
}

impl LLMApiMock {
    pub fn new() -> Self {
        let mut server = mockito::Server::new();
        // load the OpenAI chat completion response
        let openai_embedding_response: OpenAIEmbeddingResponse =
            serde_json::from_str(OPENAI_EMBEDDING_RESPONSE).unwrap();
        let chat_msg_response: OpenAIChatResponse =
            serde_json::from_str(OPENAI_CHAT_COMPLETION_RESPONSE).unwrap();
        let chat_structured_response: OpenAIChatResponse =
            serde_json::from_str(OPENAI_CHAT_STRUCTURED_RESPONSE).unwrap();
        let chat_structured_score_response: OpenAIChatResponse =
            serde_json::from_str(OPENAI_CHAT_STRUCTURED_SCORE_RESPONSE).unwrap();
        let chat_structured_response_params: OpenAIChatResponse =
            serde_json::from_str(OPENAI_CHAT_STRUCTURED_RESPONSE_PARAMS).unwrap();
        let chat_structured_task_output: OpenAIChatResponse =
            serde_json::from_str(OPENAI_CHAT_STRUCTURED_TASK_OUTPUT).unwrap();

        // load the Gemini chat completion response
        let gemini_chat_response: GenerateContentResponse =
            serde_json::from_str(GEMINI_CHAT_COMPLETION_RESPONSE).unwrap();
        let gemini_chat_response_with_score: GenerateContentResponse =
            serde_json::from_str(GEMINI_CHAT_COMPLETION_RESPONSE_WITH_SCORE).unwrap();
        let gemini_embedding_response: GeminiEmbeddingResponse =
            serde_json::from_str(GEMINI_EMBEDDING_RESPONSE).unwrap();

        server
            .mock("POST", "/chat/completions")
            .match_body(mockito::Matcher::PartialJson(serde_json::json!({
                "response_format": {
                    "type": "json_schema",
                    "json_schema": {
                        "name": "Parameters",
                         "schema": {
                              "$schema": "https://json-schema.org/draft/2020-12/schema",
                              "properties": {
                                  "variable1": {
                                  "format": "int32",
                                  "type": "integer"
                                  },
                                  "variable2": {
                                  "format": "int32",
                                  "type": "integer"
                                  }
                              },
                              "required": [
                                  "variable1",
                                  "variable2"
                              ],
                              "title": "Parameters",
                              "type": "object"
                              },
                        "strict": true
                    }

                }
            })))
            .expect(usize::MAX)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chat_structured_response_params).unwrap())
            .create();

        server
            .mock("POST", "/chat/completions")
            .match_body(mockito::Matcher::PartialJson(serde_json::json!({
               "response_format": {
                    "type": "json_schema",
                    "json_schema": {
                        "name": "TaskOutput",
                    }
                }
            })))
            .expect(usize::MAX)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chat_structured_task_output).unwrap())
            .create();

        server
            .mock("POST", "/chat/completions")
            .match_body(mockito::Matcher::PartialJson(serde_json::json!({
                "response_format": {
                    "type": "json_schema",
                    "json_schema": {
                        "name": "Score",
                    }
                }
            })))
            .expect(usize::MAX)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chat_structured_score_response).unwrap())
            .with_body_from_request({
                let chat_structured_score_response = chat_structured_score_response.clone();
                move |_request| {
                    let randomized_response = randomize_structured_openai_score_response(
                        &chat_structured_score_response.clone(),
                    );
                    serde_json::to_string(&randomized_response).unwrap().into()
                }
            })
            .create();

        server
            .mock("POST", "/chat/completions")
            .match_body(mockito::Matcher::Regex(
                r#".*"name"\s*:\s*"Score".*"#.to_string(),
            ))
            .expect(usize::MAX)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body_from_request({
                let chat_structured_score_response = chat_structured_score_response.clone();
                move |_request| {
                    let randomized_response = randomize_structured_openai_score_response(
                        &chat_structured_score_response.clone(),
                    );
                    serde_json::to_string(&randomized_response).unwrap().into()
                }
            })
            .create();

        server
            .mock("POST", "/chat/completions")
            .match_body(mockito::Matcher::PartialJson(serde_json::json!({
                "response_format": {
                    "type": "json_schema"
                }
            })))
            .expect(usize::MAX)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chat_structured_response).unwrap())
            .create();

        // mock the Gemini chat completion response
        server
            .mock(
                "POST",
                mockito::Matcher::Regex(r".*/.*:generateContent$".to_string()),
            )
            .match_header("x-goog-api-key", mockito::Matcher::Any)
            .match_header("content-type", "application/json")
            .match_body(mockito::Matcher::PartialJson(serde_json::json!({
                "contents": [
                    {
                        "parts": [
                            {
                                "text":  "You are a helpful assistant"
                            }
                        ]
                    }
                ]
            })))
            .expect(usize::MAX) // More specific expectation than usize::MAX
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&gemini_chat_response).unwrap())
            .create();

        // mock structured response
        server
            .mock(
                "POST",
                mockito::Matcher::Regex(r".*/.*:generateContent$".to_string()),
            )
            .match_header("x-goog-api-key", mockito::Matcher::Any)
            .match_header("content-type", "application/json")
            .match_body(mockito::Matcher::PartialJson(serde_json::json!({
                "generation_config": {
                    "responseMimeType": "application/json"
                }
            })))
            .expect(usize::MAX)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body_from_request(move |_request| {
                let randomized_response =
                    randomize_gemini_score_response(gemini_chat_response_with_score.clone());
                serde_json::to_string(&randomized_response).unwrap().into()
            })
            .create();

        // Openai chat completion mock
        server
            .mock("POST", "/chat/completions")
            .expect(usize::MAX)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chat_msg_response).unwrap())
            .create();

        server
            .mock("POST", "/embeddings")
            .expect(usize::MAX)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body_from_request(move |_request| {
                let randomized_response =
                    randomize_openai_embedding_response(openai_embedding_response.clone());
                serde_json::to_string(&randomized_response).unwrap().into()
            })
            .create();

        server
            .mock(
                "POST",
                mockito::Matcher::Regex(r".*/.*:embedContent$".to_string()),
            )
            .expect(usize::MAX)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body_from_request(move |_request| {
                let randomized_response =
                    randomize_gemini_embedding_response(gemini_embedding_response.clone());
                serde_json::to_string(&randomized_response).unwrap().into()
            })
            .create();

        Self {
            url: server.url(),
            server,
        }
    }
}

impl Default for LLMApiMock {
    fn default() -> Self {
        Self::new()
    }
}

#[pyclass]
#[allow(dead_code)]
pub struct LLMTestServer {
    openai_server: Option<LLMApiMock>,

    #[pyo3(get)]
    pub url: Option<String>,
}

#[pymethods]
impl LLMTestServer {
    #[new]
    pub fn new() -> Self {
        LLMTestServer {
            openai_server: None,
            url: None,
        }
    }

    pub fn start_mock_server(&mut self) -> Result<(), MockError> {
        let llm_server = LLMApiMock::new();
        println!("Mock LLM Server started at {}", llm_server.url);
        self.openai_server = Some(llm_server);
        Ok(())
    }

    pub fn stop_mock_server(&mut self) {
        if let Some(server) = self.openai_server.take() {
            drop(server);
            std::env::remove_var("OPENAI_API_URL");
            std::env::remove_var("OPENAI_API_KEY");
        }
        println!("Mock LLM Server stopped");
    }

    pub fn set_env_vars_for_client(&self) -> Result<(), MockError> {
        {
            std::env::set_var("APP_ENV", "dev_client");
            std::env::set_var("OPENAI_API_KEY", "test_key");
            std::env::set_var("GEMINI_API_KEY", "gemini");
            std::env::set_var(
                "OPENAI_API_URL",
                self.openai_server.as_ref().unwrap().url.clone(),
            );
            std::env::set_var(
                "GEMINI_API_URL",
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

        self.url = Some(self.openai_server.as_ref().unwrap().url.clone());

        Ok(())
    }

    pub fn stop_server(&mut self) -> Result<(), MockError> {
        self.cleanup()?;

        Ok(())
    }

    pub fn remove_env_vars_for_client(&self) -> Result<(), MockError> {
        std::env::remove_var("OPENAI_API_URI");
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("GEMINI_API_KEY");
        std::env::remove_var("GEMINI_API_URL");
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
        _exc_type: Py<PyAny>,
        _exc_value: Py<PyAny>,
        _traceback: Py<PyAny>,
    ) -> Result<(), MockError> {
        self.stop_server()
    }
}

impl Default for LLMTestServer {
    fn default() -> Self {
        Self::new()
    }
}
