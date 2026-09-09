use super::types::*;
use super::playback::PlaybackState;

pub struct NowPlaying {
    item: Option<MediaItem>,
    lyric: Option<Lyric>,
    position: std::time::Duration,
}

impl NowPlaying {
    pub fn new() -> Self {
        Self {
            item: None,
            lyric: None,
            position: std::time::Duration::ZERO,
        }
    }

    pub fn set_item(&mut self, item: MediaItem) {
        self.item = Some(item);
    }

    pub fn set_lyric(&mut self, lyric: Lyric) {
        self.lyric = Some(lyric);
    }

    pub fn item(&self) -> Option<&MediaItem> {
        self.item.as_ref()
    }

    pub fn lyric(&self) -> Option<&Lyric> {
        self.lyric.as_ref()
    }

    pub fn position(&self) -> std::time::Duration {
        self.position
    }

    pub fn set_position(&mut self, position: std::time::Duration) {
        self.position = position;
    }

    pub fn clear(&mut self) {
        self.item = None;
        self.lyric = None;
        self.position = std::time::Duration::ZERO;
    }
}

impl Default for NowPlaying {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PlayerDisplay;

impl PlayerDisplay {
    pub fn new() -> Self {
        Self
    }
}
