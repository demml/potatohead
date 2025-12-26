use ::potato_head::prompt_types::*;
use ::potato_head::PyEmbedder;
use potato_head::{
    EventDetails, Provider, PyAgent, PyAgentResponse, PyTask, PyWorkflow, Task, TaskEvent,
    TaskList, TaskStatus, WorkflowResult,
};
pub mod anthropic;
pub mod google;
pub mod logging;
pub mod mock;
pub mod openai;
use pyo3::prelude::*;

#[pymodule]
pub fn _potato_head(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Provider>()?;
    m.add_class::<PyAgent>()?;
    m.add_class::<PyWorkflow>()?;
    m.add_class::<Task>()?;
    m.add_class::<PyTask>()?;
    m.add_class::<Prompt>()?;
    m.add_class::<ModelSettings>()?;
    m.add_class::<Score>()?;
    m.add_class::<AudioUrl>()?;
    m.add_class::<BinaryContent>()?;
    m.add_class::<DocumentUrl>()?;
    m.add_class::<ImageUrl>()?;
    m.add_class::<EventDetails>()?;
    m.add_class::<TaskEvent>()?;
    m.add_class::<WorkflowResult>()?;
    m.add_class::<TaskList>()?;
    m.add_class::<TaskStatus>()?;
    m.add_class::<PyAgentResponse>()?;
    m.add_class::<PyEmbedder>()?;
    mock::add_mock_module(m)?;
    logging::add_logging_module(m)?;
    google::add_google_module(m)?;
    openai::add_openai_module(m)?;
    anthropic::add_anthropic_module(m)?;
    Ok(())
}
