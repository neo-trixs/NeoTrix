use super::save_state::GameStateSnapshot;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct ReplayFrame {
    pub tick: u64,
    pub action: String,
    pub state: GameStateSnapshot,
    pub reward: f64,
}

pub struct ReplayBuffer {
    frames: VecDeque<ReplayFrame>,
    max_size: usize,
}

impl ReplayBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            frames: VecDeque::new(),
            max_size,
        }
    }

    pub fn record(&mut self, frame: ReplayFrame) {
        if self.frames.len() >= self.max_size {
            self.frames.pop_front();
        }
        self.frames.push_back(frame);
    }

    pub fn get_frame(&self, tick: u64) -> Option<&ReplayFrame> {
        self.frames.iter().find(|f| f.tick == tick)
    }

    pub fn recent(&self, n: usize) -> Vec<&ReplayFrame> {
        self.frames.iter().rev().take(n).collect()
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn clear(&mut self) {
        self.frames.clear();
    }

    pub fn total_reward(&self) -> f64 {
        self.frames.iter().map(|f| f.reward).sum()
    }

    pub fn avg_reward(&self) -> f64 {
        if self.frames.is_empty() {
            0.0
        } else {
            self.total_reward() / self.frames.len() as f64
        }
    }
}

impl Default for ReplayBuffer {
    fn default() -> Self {
        Self::new(1000)
    }
}
