use ::potato_head::common::PromptType;
use ::potato_head::mouth::{ChatPrompt, Message, Mouth};
use pyo3::prelude::*;
use pyo3::wrap_pymodule;

pub mod logging;
pub mod openai;

#[pymodule]
fn potato_head(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ChatPrompt>()?;
    m.add_class::<Mouth>()?;
    m.add_class::<Message>()?;
    m.add_class::<PromptType>()?;
    m.add_wrapped(wrap_pymodule!(openai::openai))?;
    m.add_wrapped(wrap_pymodule!(logging::logging))?;
    Ok(())
}
