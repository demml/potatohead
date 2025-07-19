pub trait TokenUsage {
    fn total_tokens(&self) -> u64;

    fn prompt_tokens(&self) -> u64;

    fn completion_tokens(&self) -> u64;
}

pub trait ResponseExtensions {
    fn get_content(&self) -> Option<String>;
}
