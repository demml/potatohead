use super::{Memory, MemoryTurn};
use potato_type::prompt::MessageNum;

/// Simple unbounded in-memory conversation history.
#[derive(Debug, Default)]
pub struct InMemoryMemory {
    turns: Vec<MemoryTurn>,
}

impl InMemoryMemory {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Memory for InMemoryMemory {
    fn push_turn(&mut self, turn: MemoryTurn) {
        self.turns.push(turn);
    }

    fn messages(&self) -> Vec<MessageNum> {
        let mut msgs = Vec::with_capacity(self.turns.len() * 2);
        for turn in &self.turns {
            msgs.push(turn.user.clone());
            msgs.push(turn.assistant.clone());
        }
        msgs
    }

    fn clear(&mut self) {
        self.turns.clear();
    }

    fn len(&self) -> usize {
        self.turns.len()
    }
}
