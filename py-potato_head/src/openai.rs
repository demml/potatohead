use potato_providers::openai::{
    stream::{
        ChatCompletionChunk, ChoiceDelta, ChoiceDeltaFunctionCall, ChoiceDeltaToolCall,
        ChoiceDeltaToolCallFunction, ChunkChoice,
    },
    ChatCompletion, ChatCompletionAudio, ChatCompletionMessage, ChatCompletionTokenLogprob, Choice,
    ChoiceLogprobs, CompletionTokensDetails, CompletionUsage, FunctionCall, OpenAIConfig,
    ParsedChatCompletion, PromptTokensDetails, TopLogProb,
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

    m.add_class::<ChoiceDelta>()?;
    m.add_class::<ChoiceDeltaFunctionCall>()?;
    m.add_class::<ChoiceDeltaToolCall>()?;
    m.add_class::<ChatCompletionChunk>()?;
    m.add_class::<ChunkChoice>()?;
    m.add_class::<ChoiceDeltaToolCallFunction>()?;
    Ok(())
}
