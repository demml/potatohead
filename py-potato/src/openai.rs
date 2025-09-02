use potato_head::agents::provider::openai::{
    AllowedTools, AudioParam, Content, ContentPart, CustomChoice, CustomDefinition, CustomTool,
    CustomToolChoice, CustomToolFormat, FunctionChoice, FunctionChoice, FunctionDefinition,
    FunctionTool, Grammar, OpenAIChatSettings, Prediction, StreamOptions, Tool, ToolChoice,
};
use pyo3::prelude::*;

#[pymodule]
pub fn openai(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<AllowedTools>()?;
    m.add_class::<AudioParam>()?;
    m.add_class::<Content>()?;
    m.add_class::<ContentPart>()?;
    m.add_class::<CustomChoice>()?;
    m.add_class::<CustomDefinition>()?;
    m.add_class::<CustomTool>()?;
    m.add_class::<CustomToolChoice>()?;
    m.add_class::<CustomToolFormat>()?;
    m.add_class::<FunctionChoice>()?;
    m.add_class::<FunctionDefinition>()?;
    m.add_class::<FunctionTool>()?;
    m.add_class::<Grammar>()?;
    m.add_class::<OpenAIChatSettings>()?;
    m.add_class::<Prediction>()?;
    m.add_class::<StreamOptions>()?;
    m.add_class::<Tool>()?;
    m.add_class::<ToolChoice>()?;
    Ok(())
}
