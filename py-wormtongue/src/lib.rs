use ::wormtongue::tongues::{ChatPrompt, Message, PromptType, Tongue};
use pyo3::prelude::*;
pub mod openai;
use pyo3::wrap_pymodule;

#[pymodule]
fn wormtongue(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ChatPrompt>()?;
    m.add_class::<Tongue>()?;
    m.add_class::<Message>()?;
    m.add_class::<PromptType>()?;
    m.add_wrapped(wrap_pymodule!(openai::openai))?;
    Ok(())
}
