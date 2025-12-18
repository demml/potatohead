use ::potato_head::LLMTestServer;
use pyo3::prelude::*;

pub fn add_mock_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LLMTestServer>()?;

    Ok(())
}
