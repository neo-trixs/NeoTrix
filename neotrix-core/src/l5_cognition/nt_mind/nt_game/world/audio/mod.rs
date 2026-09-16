use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioChannel {
    Master,
    Music,
    Sfx,
    Voice,
    Ambient,
}

#[derive(Debug, Clone)]
pub struct AudioTrack {
    pub id: u32,
    pub name: String,
    pub path: String,
    pub channel: AudioChannel,
    pub volume: f64,
    pub looping: bool,
}

pub struct AudioManager {
    tracks: HashMap<u32, AudioTrack>,
    volumes: HashMap<AudioChannel, f64>,
    current_music: Option<u32>,
    muted: bool,
}

impl AudioManager {
    pub fn new() -> Self {
        let mut volumes = HashMap::new();
        for ch in [
            AudioChannel::Master,
            AudioChannel::Music,
            AudioChannel::Sfx,
            AudioChannel::Voice,
            AudioChannel::Ambient,
        ] {
            volumes.insert(ch, 1.0);
        }
        Self {
            tracks: HashMap::new(),
            volumes,
            current_music: None,
            muted: false,
        }
    }

    pub fn register_track(&mut self, track: AudioTrack) {
        self.tracks.insert(track.id, track);
    }

    pub fn play_music(&mut self, track_id: u32) {
        if self.tracks.contains_key(&track_id) {
            self.current_music = Some(track_id);
        }
    }

    pub fn stop_music(&mut self) {
        self.current_music = None;
    }

    pub fn set_volume(&mut self, channel: AudioChannel, volume: f64) {
        self.volumes.insert(channel, volume.clamp(0.0, 1.0));
    }

    pub fn get_volume(&self, channel: AudioChannel) -> f64 {
        self.volumes.get(&channel).copied().unwrap_or(1.0)
    }

    pub fn mute(&mut self) {
        self.muted = true;
    }

    pub fn unmute(&mut self) {
        self.muted = false;
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn current_music(&self) -> Option<&AudioTrack> {
        self.current_music.and_then(|id| self.tracks.get(&id))
    }

    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}
