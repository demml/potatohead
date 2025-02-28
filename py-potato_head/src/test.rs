use potato_error::PotatoHeadError;
use pyo3::prelude::*;

#[cfg(feature = "dev")]
use baked_potato::{start_server_in_background, stop_server};
#[cfg(feature = "dev")]
use std::sync::Arc;
#[cfg(feature = "dev")]
use std::thread::sleep;
#[cfg(feature = "dev")]
use std::time::Duration;
#[cfg(feature = "dev")]
use tokio::{runtime::Runtime, sync::Mutex, task::JoinHandle};

#[pyclass]
#[allow(dead_code)]
pub struct LLMTestServer {
    #[cfg(feature = "dev")]
    handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    #[cfg(feature = "dev")]
    runtime: Arc<Runtime>,
    cleanup: bool,
}

#[pymethods]
impl LLMTestServer {
    #[new]
    #[pyo3(signature = (cleanup = true))]
    fn new(cleanup: bool) -> Self {
        LLMTestServer {
            #[cfg(feature = "dev")]
            handle: Arc::new(Mutex::new(None)),
            #[cfg(feature = "dev")]
            runtime: Arc::new(Runtime::new().unwrap()),
            cleanup,
        }
    }

    fn set_env_vars_for_client(&self) -> PyResult<()> {
        #[cfg(feature = "dev")]
        {
            std::env::set_var("POTATO_HEAD_URL", "http://localhost:3000");
            std::env::set_var("POTATO_HEAD_API_KEY", "key");
            std::env::set_var("APP_ENV", "dev_client");
            Ok(())
        }
        #[cfg(not(feature = "dev"))]
        {
            Err(PotatoHeadError::new_err("Opsml Server feature not enabled"))
        }
    }

    fn start_server(&mut self) -> PyResult<()> {
        #[cfg(feature = "dev")]
        {
            self.cleanup()?;

            // set server env vars
            std::env::set_var("APP_ENV", "dev_server");
            let handle = self.handle.clone();
            let runtime = self.runtime.clone();
            runtime.spawn(async move {
                let server_handle = start_server_in_background();
                *handle.lock().await = server_handle.lock().await.take();
            });

            let client = reqwest::blocking::Client::new();
            let mut attempts = 0;
            let max_attempts = 20;

            while attempts < max_attempts {
                let res = client.get("http://localhost:3000/healthcheck").send();
                if let Ok(response) = res {
                    if response.status() == 200 {
                        self.set_env_vars_for_client()?;
                        println!("LLM Dev server started successfully");
                        return Ok(());
                    }
                }
                attempts += 1;
                sleep(Duration::from_millis(100));
            }

            Err(PotatoHeadError::new_err("Failed to start LLM Dev server"))
        }
        #[cfg(not(feature = "dev"))]
        {
            Err(PotatoHeadError::new_err(
                "LLM Dev server feature not enabled",
            ))
        }
    }

    fn stop_server(&self) -> PyResult<()> {
        #[cfg(feature = "dev")]
        {
            let handle = self.handle.clone();
            let runtime = self.runtime.clone();
            runtime.spawn(async move {
                stop_server(handle).await;
            });

            if self.cleanup {
                self.cleanup()?;
            }

            Ok(())
        }
        #[cfg(not(feature = "dev"))]
        {
            Err(PotatoHeadError::new_err(
                "LLM Dev server feature not enabled",
            ))
        }
    }

    fn cleanup(&self) -> PyResult<()> {
        Ok(())
    }

    fn __enter__(mut self_: PyRefMut<Self>) -> PyResult<PyRefMut<Self>> {
        self_.start_server()?;
        Ok(self_)
    }

    fn __exit__(
        &self,
        _exc_type: PyObject,
        _exc_value: PyObject,
        _traceback: PyObject,
    ) -> PyResult<()> {
        self.stop_server()
    }
}

#[pymodule]
pub fn test(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LLMTestServer>()?;
    Ok(())
}
