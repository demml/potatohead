pub use potato_util::utils::ResponseLogProbs;
pub trait TokenUsage {
    fn total_tokens(&self) -> u64;

    fn prompt_tokens(&self) -> u64;

    fn completion_tokens(&self) -> u64;
}

pub trait ResponseExt {
    fn get_content(&self) -> Option<String>;
}
pub trait LogProbExt {
    fn get_log_probs(&self) -> Vec<ResponseLogProbs>;
}
