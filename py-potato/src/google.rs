use potato_head::agents::provider::gemini::{
    GenerationConfig, HarmBlockMethod, HarmBlockThreshold, HarmCategory, MediaResolution, Modality,
    PrebuiltVoiceConfig, SafetySetting, SpeechConfig, ThinkingConfig, VoiceConfig, VoiceConfigMode,
};
use pyo3::prelude::*;

#[pymodule]
pub fn google(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<GenerationConfig>()?;
    m.add_class::<MediaResolution>()?;
    m.add_class::<Modality>()?;
    m.add_class::<SpeechConfig>()?;
    m.add_class::<ThinkingConfig>()?;
    m.add_class::<VoiceConfig>()?;
    m.add_class::<VoiceConfigMode>()?;
    m.add_class::<PrebuiltVoiceConfig>()?;
    m.add_class::<HarmCategory>()?;
    m.add_class::<HarmBlockMethod>()?;
    m.add_class::<HarmBlockThreshold>()?;
    m.add_class::<SafetySetting>()?;
    Ok(())
}
