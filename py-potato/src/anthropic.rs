use potato_head::anthropic_types::v1::message::{
    AnthropicSettings, CacheControl, Metadata, ThinkingConfig, Tool, ToolChoice,
};
use pyo3::prelude::*;

pub fn add_anthropic_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<AnthropicSettings>()?;
    m.add_class::<Metadata>()?;
    m.add_class::<CacheControl>()?;
    m.add_class::<Tool>()?;
    m.add_class::<ThinkingConfig>()?;
    m.add_class::<ToolChoice>()?;

    Ok(())
}
