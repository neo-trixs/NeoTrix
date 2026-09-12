use std::collections::HashMap;

/// Audio handle for referencing loaded sounds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioHandle(pub u32);

/// Sound effect playback state
#[derive(Debug, Clone)]
pub struct PlaybackInstance {
    pub handle: AudioHandle,
    pub playing: bool,
    pub volume: f32,
    pub position: Option<[f32; 2]>, // For spatial audio
}

/// Audio backend trait
pub trait AudioBackend: Send + Sync {
    fn load_sound(&mut self, path: &str) -> Result<AudioHandle, String>;
    fn play_sound(&mut self, handle: AudioHandle, volume: f32) -> Result<(), String>;
    fn stop_sound(&mut self, handle: AudioHandle) -> Result<(), String>;
    fn set_volume(&mut self, handle: AudioHandle, volume: f32) -> Result<(), String>;
    fn set_position(&mut self, handle: AudioHandle, position: [f32; 2]) -> Result<(), String>;
    fn is_playing(&self, handle: AudioHandle) -> bool;
    fn active_sounds(&self) -> Vec<PlaybackInstance>;
}

/// Audio manager with resource tracking
pub struct AudioManager {
    backends: Vec<Box<dyn AudioBackend>>,
    loaded_sounds: HashMap<String, AudioHandle>,
    next_handle: u32,
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
            loaded_sounds: HashMap::new(),
            next_handle: 1,
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 1.0,
        }
    }

    /// Register an audio backend
    pub fn add_backend(&mut self, backend: Box<dyn AudioBackend>) {
        self.backends.push(backend);
    }

    /// Load a sound file
    pub fn load(&mut self, path: &str) -> Result<AudioHandle, String> {
        if let Some(&handle) = self.loaded_sounds.get(path) {
            return Ok(handle);
        }

        let backend = self.backends.first_mut()
            .ok_or_else(|| "No audio backend registered".to_string())?;

        let handle = backend.load_sound(path)?;
        self.loaded_sounds.insert(path.to_string(), handle);
        Ok(handle)
    }

    /// Play a sound
    pub fn play(&mut self, handle: AudioHandle) -> Result<(), String> {
        let volume = self.master_volume * self.sfx_volume;
        let backend = self.backends.first_mut()
            .ok_or_else(|| "No audio backend registered".to_string())?;
        backend.play_sound(handle, volume)
    }

    /// Stop a sound
    pub fn stop(&mut self, handle: AudioHandle) -> Result<(), String> {
        let backend = self.backends.first_mut()
            .ok_or_else(|| "No audio backend registered".to_string())?;
        backend.stop_sound(handle)
    }

    /// Set master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Set music volume
    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
    }

    /// Set SFX volume
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
    }

    /// Get volumes
    pub fn volumes(&self) -> (f32, f32, f32) {
        (self.master_volume, self.music_volume, self.sfx_volume)
    }
}

impl Default for AudioManager {
    fn default() -> Self { Self::new() }
}

/// Stub backend for testing (no-op)
pub struct StubAudioBackend;

impl AudioBackend for StubAudioBackend {
    fn load_sound(&mut self, _path: &str) -> Result<AudioHandle, String> {
        Ok(AudioHandle(1))
    }
    fn play_sound(&mut self, _handle: AudioHandle, _volume: f32) -> Result<(), String> { Ok(()) }
    fn stop_sound(&mut self, _handle: AudioHandle) -> Result<(), String> { Ok(()) }
    fn set_volume(&mut self, _handle: AudioHandle, _volume: f32) -> Result<(), String> { Ok(()) }
    fn set_position(&mut self, _handle: AudioHandle, _position: [f32; 2]) -> Result<(), String> { Ok(()) }
    fn is_playing(&self, _handle: AudioHandle) -> bool { false }
    fn active_sounds(&self) -> Vec<PlaybackInstance> { vec![] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_manager_load_play() {
        let mut manager = AudioManager::new();
        manager.add_backend(Box::new(StubAudioBackend));

        let handle = manager.load("test.wav").unwrap();
        assert!(manager.play(handle).is_ok());
        assert!(manager.stop(handle).is_ok());
    }

    #[test]
    fn test_volume_controls() {
        let mut manager = AudioManager::new();
        manager.set_master_volume(0.5);
        manager.set_music_volume(0.3);
        manager.set_sfx_volume(0.8);

        let (master, music, sfx) = manager.volumes();
        assert!((master - 0.5).abs() < 0.01);
        assert!((music - 0.3).abs() < 0.01);
        assert!((sfx - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_volume_clamping() {
        let mut manager = AudioManager::new();
        manager.set_master_volume(2.0); // Should clamp to 1.0
        manager.set_master_volume(-0.5); // Should clamp to 0.0

        let (master, _, _) = manager.volumes();
        assert!((master - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_no_backend_error() {
        let mut manager = AudioManager::new();
        let result = manager.load("test.wav");
        assert!(result.is_err());
    }

    #[test]
    fn test_cached_load() {
        let mut manager = AudioManager::new();
        manager.add_backend(Box::new(StubAudioBackend));

        let h1 = manager.load("test.wav").unwrap();
        let h2 = manager.load("test.wav").unwrap();
        assert_eq!(h1, h2); // Same handle returned
    }
}
