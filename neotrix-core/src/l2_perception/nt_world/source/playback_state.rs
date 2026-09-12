use super::types::*;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum PlaybackState {
    Idle,
    Loading { item: MediaItem },
    Playing { item: MediaItem, position: Duration },
    Paused { item: MediaItem, position: Duration },
    Error { item: MediaItem, error: String },
}

impl PlaybackState {
    pub fn _is_idle(&self) -> bool {
        matches!(self, PlaybackState::Idle)
    }

    pub fn current_item(&self) -> Option<&MediaItem> {
        match self {
            PlaybackState::Loading { item } => Some(item),
            PlaybackState::Playing { item, .. } => Some(item),
            PlaybackState::Paused { item, .. } => Some(item),
            PlaybackState::Error { item, .. } => Some(item),
            _ => None,
        }
    }

    pub fn position(&self) -> Duration {
        match self {
            PlaybackState::Playing { position, .. } => *position,
            PlaybackState::Paused { position, .. } => *position,
            _ => Duration::ZERO,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            PlaybackState::Idle => "idle",
            PlaybackState::Loading { .. } => "loading",
            PlaybackState::Playing { .. } => "playing",
            PlaybackState::Paused { .. } => "paused",
            PlaybackState::Error { .. } => "error",
        }
    }
}

pub struct PlaybackEngine {
    state: PlaybackState,
}

impl PlaybackEngine {
    pub fn new() -> Self {
        Self {
            state: PlaybackState::Idle,
        }
    }

    pub fn transition(&mut self, new_state: PlaybackState) {
        self.state = new_state;
    }

    pub fn current(&self) -> &PlaybackState {
        &self.state
    }

    pub fn load(&mut self, item: MediaItem) {
        self.state = PlaybackState::Loading { item };
    }

    pub fn play(&mut self, item: MediaItem, position: Duration) {
        self.state = PlaybackState::Playing { item, position };
    }

    pub fn pause(&mut self) {
        if let PlaybackState::Playing { item, position } = self.state.clone() {
            self.state = PlaybackState::Paused { item, position };
        }
    }

    pub fn resume(&mut self) {
        if let PlaybackState::Paused { item, position } = self.state.clone() {
            self.state = PlaybackState::Playing { item, position };
        }
    }

    pub fn stop(&mut self) {
        self.state = PlaybackState::Idle;
    }

    pub fn error(&mut self, item: MediaItem, error: String) {
        self.state = PlaybackState::Error { item, error };
    }
}

impl Default for PlaybackEngine {
    fn default() -> Self {
        Self::new()
    }
}
