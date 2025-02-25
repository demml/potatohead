use potato_head::client::OpenAIConfig;
use potato_head::mouth::responses::openai::{
    ChatCompletion, ChatCompletionAudio, ChatCompletionMessage, ChatCompletionTokenLogprob, Choice,
    ChoiceLogprobs, CompletionTokensDetails, CompletionUsage, FunctionCall, ParsedChatCompletion,
    PromptTokensDetails, TopLogProb,
};
use pyo3::prelude::*;
#[pymodule]
pub fn openai(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ChatCompletion>()?;
    m.add_class::<ParsedChatCompletion>()?;
    m.add_class::<ChatCompletionMessage>()?;
    m.add_class::<ChatCompletionAudio>()?;
    m.add_class::<ChatCompletionTokenLogprob>()?;
    m.add_class::<Choice>()?;
    m.add_class::<ChoiceLogprobs>()?;
    m.add_class::<CompletionTokensDetails>()?;
    m.add_class::<CompletionUsage>()?;
    m.add_class::<FunctionCall>()?;
    m.add_class::<ParsedChatCompletion>()?;
    m.add_class::<PromptTokensDetails>()?;
    m.add_class::<TopLogProb>()?;
    m.add_class::<OpenAIConfig>()?;
    Ok(())
}
