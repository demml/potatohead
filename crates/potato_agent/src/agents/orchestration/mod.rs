pub mod parallel;
pub mod sequential;

pub use parallel::{MergeStrategy, ParallelAgent, ParallelAgentBuilder};
pub use sequential::{SequentialAgent, SequentialAgentBuilder};
