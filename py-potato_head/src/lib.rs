use ::potato_head::common::PromptType;
use ::potato_head::mouth::{ChatPrompt, Message, Mouth};
use pyo3::prelude::*;
pub mod openai;
use pyo3::wrap_pymodule;

#[pymodule]
fn potato_head(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ChatPrompt>()?;
    m.add_class::<Mouth>()?;
    m.add_class::<Message>()?;
    m.add_class::<PromptType>()?;
    m.add_wrapped(wrap_pymodule!(openai::openai))?;
    Ok(())
}
