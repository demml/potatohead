use potato_providers::anthropic::AnthropicConfig;
use pyo3::prelude::*;
#[pymodule]
pub fn anthropic(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<AnthropicConfig>()?;
    Ok(())
}
