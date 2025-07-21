use ::potato_head::LLMTestServer;
use pyo3::prelude::*;

#[pymodule]
pub fn mock(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LLMTestServer>()?;

    Ok(())
}
