use pyo3::prelude::*;
use pyo3::wrap_pymodule;

pub mod anthropic;
pub mod logging;
pub mod openai;
pub mod parts;
pub mod prompts;
pub mod test;

#[pymodule]
fn potato_head(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(openai::openai))?;
    m.add_wrapped(wrap_pymodule!(logging::logging))?;
    m.add_wrapped(wrap_pymodule!(prompts::prompts))?;
    m.add_wrapped(wrap_pymodule!(parts::parts))?;
    m.add_wrapped(wrap_pymodule!(anthropic::anthropic))?;
    m.add_wrapped(wrap_pymodule!(test::test))?;
    Ok(())
}
