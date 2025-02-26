use pyo3::prelude::*;
use pyo3::wrap_pymodule;

pub mod logging;
pub mod openai;
pub mod prompts;

#[pymodule]
fn potato_head(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(openai::openai))?;
    m.add_wrapped(wrap_pymodule!(logging::logging))?;
    m.add_wrapped(wrap_pymodule!(prompts::prompts))?;
    Ok(())
}
