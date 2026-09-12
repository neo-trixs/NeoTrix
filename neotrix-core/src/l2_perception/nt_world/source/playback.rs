use super::types::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayMode {
    Sequential,
    LoopOne,
    LoopAll,
    Shuffle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaybackState {
    Idle,
    Playing,
    Paused,
    Stopped,
}

pub struct PlaybackController {
    queue: VecDeque<MediaItem>,
    current: Option<MediaItem>,
    state: PlaybackState,
    mode: PlayMode,
    position: std::time::Duration,
}

impl PlaybackController {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            current: None,
            state: PlaybackState::Idle,
            mode: PlayMode::Sequential,
            position: std::time::Duration::ZERO,
        }
    }

    pub fn enqueue(&mut self, item: MediaItem) {
        self.queue.push_back(item);
    }

    pub fn _play_next(&mut self) -> Option<&MediaItem> {
        if self.current.is_none() {
            self.current = self.queue.pop_front();
        }
        self.current.as_ref()
    }

    pub fn state(&self) -> PlaybackState {
        self.state.clone()
    }

    pub fn mode(&self) -> PlayMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: PlayMode) {
        self.mode = mode;
    }

    pub fn position(&self) -> std::time::Duration {
        self.position
    }

    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            self.state = PlaybackState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == PlaybackState::Paused {
            self.state = PlaybackState::Playing;
        }
    }

    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
        self.current = None;
        self.position = std::time::Duration::ZERO;
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    pub fn _clear_queue(&mut self) {
        self.queue.clear();
    }
}

impl Default for PlaybackController {
    fn default() -> Self {
        Self::new()
    }
}
