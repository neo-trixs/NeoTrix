//! Playback control — play/pause/stop/queue/history/retry.
//!
//! Consolidated from L2 perception skeletons (R-P42: strengthen existing nodes).
//! Origin: `l2_perception::nt_world::source::playback{,_queue,_state,_history,_retry}`

use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

// ============================================================================
// PlaybackController (from playback.rs)
// ============================================================================

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
    position: Duration,
}

impl PlaybackController {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            current: None,
            state: PlaybackState::Idle,
            mode: PlayMode::Sequential,
            position: Duration::ZERO,
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

    pub fn position(&self) -> Duration {
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
        self.position = Duration::ZERO;
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

// ============================================================================
// PlaybackQueue (from playback_queue.rs)
// ============================================================================

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

// ============================================================================
// PlaybackEngine + EnginePlaybackState (from playback_state.rs)
// ============================================================================

/// Playback state with associated item/position data.
/// Aliased as `EnginePlaybackState` in L2 re-exports to avoid collision with
/// the simple `PlaybackState` enum above.
#[derive(Debug, Clone)]
pub enum EnginePlaybackState {
    Idle,
    Loading { item: MediaItem },
    Playing { item: MediaItem, position: Duration },
    Paused { item: MediaItem, position: Duration },
    Error { item: MediaItem, error: String },
}

impl EnginePlaybackState {
    pub fn _is_idle(&self) -> bool {
        matches!(self, EnginePlaybackState::Idle)
    }

    pub fn current_item(&self) -> Option<&MediaItem> {
        match self {
            EnginePlaybackState::Loading { item } => Some(item),
            EnginePlaybackState::Playing { item, .. } => Some(item),
            EnginePlaybackState::Paused { item, .. } => Some(item),
            EnginePlaybackState::Error { item, .. } => Some(item),
            _ => None,
        }
    }

    pub fn position(&self) -> Duration {
        match self {
            EnginePlaybackState::Playing { position, .. } => *position,
            EnginePlaybackState::Paused { position, .. } => *position,
            _ => Duration::ZERO,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            EnginePlaybackState::Idle => "idle",
            EnginePlaybackState::Loading { .. } => "loading",
            EnginePlaybackState::Playing { .. } => "playing",
            EnginePlaybackState::Paused { .. } => "paused",
            EnginePlaybackState::Error { .. } => "error",
        }
    }
}

pub struct PlaybackEngine {
    state: EnginePlaybackState,
}

impl PlaybackEngine {
    pub fn new() -> Self {
        Self {
            state: EnginePlaybackState::Idle,
        }
    }

    pub fn transition(&mut self, new_state: EnginePlaybackState) {
        self.state = new_state;
    }

    pub fn current(&self) -> &EnginePlaybackState {
        &self.state
    }

    pub fn load(&mut self, item: MediaItem) {
        self.state = EnginePlaybackState::Loading { item };
    }

    pub fn play(&mut self, item: MediaItem, position: Duration) {
        self.state = EnginePlaybackState::Playing { item, position };
    }

    pub fn pause(&mut self) {
        if let EnginePlaybackState::Playing { item, position } = self.state.clone() {
            self.state = EnginePlaybackState::Paused { item, position };
        }
    }

    pub fn resume(&mut self) {
        if let EnginePlaybackState::Paused { item, position } = self.state.clone() {
            self.state = EnginePlaybackState::Playing { item, position };
        }
    }

    pub fn stop(&mut self) {
        self.state = EnginePlaybackState::Idle;
    }

    pub fn error(&mut self, item: MediaItem, error: String) {
        self.state = EnginePlaybackState::Error { item, error };
    }
}

impl Default for PlaybackEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PlaybackHistory (from playback_history.rs)
// ============================================================================

#[derive(Debug, Clone)]
pub struct _PlaybackRecord {
    pub item: MediaItem,
    pub quality: Quality,
    pub source: String,
    pub played_at: Instant,
    pub duration: Option<Duration>,
}

pub struct PlaybackHistory {
    records: Vec<_PlaybackRecord>,
    max_size: usize,
}

impl PlaybackHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            records: Vec::new(),
            max_size,
        }
    }

    pub fn record(&mut self, item: MediaItem, quality: Quality, source: String) {
        if self.records.len() >= self.max_size {
            self.records.remove(0);
        }
        self.records.push(_PlaybackRecord {
            item,
            quality,
            source,
            played_at: Instant::now(),
            duration: None,
        });
    }

    pub fn recent(&self, limit: usize) -> Vec<&_PlaybackRecord> {
        self.records.iter().rev().take(limit).collect()
    }

    pub fn search(&self, query: &str) -> Vec<&_PlaybackRecord> {
        let query_lower = query.to_lowercase();
        self.records.iter()
            .filter(|r| r.item.title.to_lowercase().contains(&query_lower)
                || r.item.artist.to_lowercase().contains(&query_lower))
            .collect()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

// ============================================================================
// PlaybackRetry (from playback_retry.rs)
// ============================================================================

pub struct PlaybackRetry;

impl PlaybackRetry {
    pub async fn get_play_url_with_fallback(
        item: &MediaItem,
        quality: Quality,
        sources: &[Box<dyn MediaSource>],
    ) -> Result<ViewSource, String> {
        for source in sources {
            if let Ok(view) = source.play_url(item, quality).await {
                return Ok(view);
            }
        }

        let fallback_qualities = match quality {
            Quality::Flac => vec![Quality::High, Quality::Standard],
            Quality::High => vec![Quality::Standard, Quality::Flac],
            Quality::Standard => vec![Quality::High, Quality::Flac],
            _ => vec![Quality::Standard, Quality::High],
        };

        for fallback_quality in fallback_qualities {
            for source in sources {
                if let Ok(view) = source.play_url(item, fallback_quality).await {
                    return Ok(view);
                }
            }
        }

        Err(format!("All sources failed for: {}", item.title))
    }
}
