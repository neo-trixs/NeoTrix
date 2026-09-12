use super::types::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    None,
    All,
    One,
}

pub struct PlaybackQueue {
    queue: VecDeque<MediaItem>,
    current_index: usize,
    repeat_mode: RepeatMode,
    shuffle: bool,
}

impl PlaybackQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            current_index: 0,
            repeat_mode: RepeatMode::None,
            shuffle: false,
        }
    }

    pub fn add(&mut self, item: MediaItem) {
        self.queue.push_back(item);
    }

    pub fn next(&mut self) -> Option<&MediaItem> {
        if self.queue.is_empty() {
            return None;
        }
        match self.repeat_mode {
            RepeatMode::One => {}
            RepeatMode::None => {
                if self.current_index + 1 < self.queue.len() {
                    self.current_index += 1;
                } else {
                    return None;
                }
            }
            RepeatMode::All => {
                self.current_index = (self.current_index + 1) % self.queue.len();
            }
        }
        self.queue.get(self.current_index)
    }

    pub fn prev(&mut self) -> Option<&MediaItem> {
        if self.queue.is_empty() {
            return None;
        }
        match self.repeat_mode {
            RepeatMode::One => {}
            RepeatMode::None => {
                if self.current_index > 0 {
                    self.current_index -= 1;
                } else {
                    return None;
                }
            }
            RepeatMode::All => {
                self.current_index = if self.current_index == 0 {
                    self.queue.len() - 1
                } else {
                    self.current_index - 1
                };
            }
        }
        self.queue.get(self.current_index)
    }

    pub fn _set_repeat(&mut self, mode: RepeatMode) {
        self.repeat_mode = mode;
    }

    pub fn _toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
    }

    pub fn current(&self) -> Option<&MediaItem> {
        self.queue.get(self.current_index)
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.current_index = 0;
    }

    pub fn remove(&mut self, index: usize) -> Option<MediaItem> {
        if index < self.queue.len() {
            let item = self.queue.remove(index)?;
            if self.current_index >= self.queue.len() && self.current_index > 0 {
                self.current_index -= 1;
            }
            Some(item)
        } else {
            None
        }
    }
}

impl Default for PlaybackQueue {
    fn default() -> Self {
        Self::new()
    }
}
