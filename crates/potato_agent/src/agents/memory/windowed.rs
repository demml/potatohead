use super::{Memory, MemoryTurn};
use potato_type::prompt::MessageNum;
use std::collections::VecDeque;

/// Sliding-window memory that keeps only the last `capacity` turns.
#[derive(Debug)]
pub struct WindowedMemory {
    capacity: usize,
    turns: VecDeque<MemoryTurn>,
}

impl WindowedMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            turns: VecDeque::with_capacity(capacity),
        }
    }
}

impl Memory for WindowedMemory {
    fn push_turn(&mut self, turn: MemoryTurn) {
        if self.capacity == 0 {
            return;
        }
        while self.turns.len() >= self.capacity {
            self.turns.pop_front();
        }
        self.turns.push_back(turn);
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
