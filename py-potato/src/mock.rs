use ::potato_head::OpenAITestServer;
use pyo3::prelude::*;

#[pymodule]
pub fn mock(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<OpenAITestServer>()?;

    Ok(())
}
