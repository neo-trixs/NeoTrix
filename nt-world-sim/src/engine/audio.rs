use std::collections::HashMap;

/// Audio handle for referencing loaded sounds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioHandle(pub u32);

// ---------------------------------------------------------------------------
// Sound Categories
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundCategory {
    Music,
    Sfx,
    Voice,
    Ambient,
}

impl SoundCategory {
    pub fn label(&self) -> &'static str {
        match self {
            SoundCategory::Music => "music",
            SoundCategory::Sfx => "sfx",
            SoundCategory::Voice => "voice",
            SoundCategory::Ambient => "ambient",
        }
    }
}

// ---------------------------------------------------------------------------
// Sound playback state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PlaybackInstance {
    pub handle: AudioHandle,
    pub playing: bool,
    pub volume: f32,
    pub category: SoundCategory,
    pub position: Option<[f32; 2]>,
    pub looped: bool,
    pub fade_progress: f32,
    pub fade_target: f32,
    pub sound_id: String,
}

// ---------------------------------------------------------------------------
// Audio Backend trait
// ---------------------------------------------------------------------------

pub trait AudioBackend: Send + Sync {
    fn load_sound(&mut self, path: &str) -> Result<AudioHandle, String>;
    fn play_sound(&mut self, handle: AudioHandle, volume: f32) -> Result<(), String>;
    fn stop_sound(&mut self, handle: AudioHandle) -> Result<(), String>;
    fn set_volume(&mut self, handle: AudioHandle, volume: f32) -> Result<(), String>;
    fn set_position(&mut self, handle: AudioHandle, position: [f32; 2]) -> Result<(), String>;
    fn is_playing(&self, handle: AudioHandle) -> bool;
    fn active_sounds(&self) -> Vec<PlaybackInstance>;
}

// ---------------------------------------------------------------------------
// Audio Trigger
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AudioTrigger {
    CombatStart,
    CombatEnd,
    PlayerHit,
    EnemyHit,
    PlayerDeath,
    LevelUp,
    QuestComplete,
    ItemPickup,
    MenuOpen,
    MenuClose,
    DialogueStart,
    DialogueEnd,
    Exploration,
    EnvironmentChange(String),
    Custom(String),
}

// ---------------------------------------------------------------------------
// Sound Pool — reuse sound instances
// ---------------------------------------------------------------------------

pub struct SoundPool {
    pool_size: usize,
    available: Vec<AudioHandle>,
    in_use: Vec<AudioHandle>,
}

impl SoundPool {
    pub fn new(pool_size: usize) -> Self {
        Self {
            pool_size,
            available: Vec::new(),
            in_use: Vec::new(),
        }
    }

    pub fn acquire(&mut self, handle: AudioHandle) -> bool {
        if self.in_use.len() >= self.pool_size {
            return false;
        }
        self.in_use.push(handle);
        true
    }

    pub fn release(&mut self, handle: AudioHandle) {
        self.in_use.retain(|&h| h != handle);
        self.available.push(handle);
    }

    pub fn active_count(&self) -> usize { self.in_use.len() }
    pub fn available_count(&self) -> usize { self.available.len() }
    pub fn capacity(&self) -> usize { self.pool_size }
}

// ---------------------------------------------------------------------------
// Music Crossfade Controller
// ---------------------------------------------------------------------------

pub struct MusicCrossfader {
    current_track: Option<String>,
    next_track: Option<String>,
    fade_duration: f32,
    fade_timer: f32,
    fading_in: bool,
    fading_out: bool,
    master_volume: f32,
}

impl MusicCrossfader {
    pub fn new(fade_duration: f32) -> Self {
        Self {
            current_track: None,
            next_track: None,
            fade_duration,
            fade_timer: 0.0,
            fading_in: false,
            fading_out: false,
            master_volume: 1.0,
        }
    }

    pub fn crossfade(&mut self, new_track: &str) {
        if self.current_track.as_deref() == Some(new_track) {
            return;
        }
        self.next_track = Some(new_track.to_string());
        self.fade_timer = 0.0;
        self.fading_out = true;
        self.fading_in = false;
    }

    pub fn update(&mut self, dt: f32) -> MusicCrossfadeState {
        if self.fade_timer >= self.fade_duration {
            if self.fading_out && self.next_track.is_some() {
                self.current_track = self.next_track.take();
                self.fading_out = false;
                self.fading_in = true;
                self.fade_timer = 0.0;
            } else if self.fading_in {
                self.fading_in = false;
            }
        }

        if self.fading_out || self.fading_in {
            self.fade_timer += dt;
        }

        let progress = (self.fade_timer / self.fade_duration).clamp(0.0, 1.0);

        let out_vol = if self.fading_out {
            self.master_volume * (1.0 - progress)
        } else {
            0.0
        };

        let in_vol = if self.fading_in {
            self.master_volume * progress
        } else if !self.fading_out && self.current_track.is_some() {
            self.master_volume
        } else {
            0.0
        };

        MusicCrossfadeState {
            fade_out_track: self.current_track.clone(),
            fade_out_volume: out_vol,
            fade_in_track: self.next_track.clone().or_else(|| self.current_track.clone()),
            fade_in_volume: in_vol,
            is_crossfading: self.fading_out || self.fading_in,
        }
    }

    pub fn set_volume(&mut self, vol: f32) { self.master_volume = vol.clamp(0.0, 1.0); }
    pub fn current_track(&self) -> Option<&str> { self.current_track.as_deref() }
    pub fn is_crossfading(&self) -> bool { self.fading_out || self.fading_in }
}

#[derive(Debug, Clone)]
pub struct MusicCrossfadeState {
    pub fade_out_track: Option<String>,
    pub fade_out_volume: f32,
    pub fade_in_track: Option<String>,
    pub fade_in_volume: f32,
    pub is_crossfading: bool,
}

// ---------------------------------------------------------------------------
// Spatial Audio Controller
// ---------------------------------------------------------------------------

pub struct SpatialAudioController {
    listener_position: [f32; 2],
    max_distance: f32,
    rolloff_factor: f32,
    reference_distance: f32,
}

impl SpatialAudioController {
    pub fn new() -> Self {
        Self {
            listener_position: [0.0, 0.0],
            max_distance: 1000.0,
            rolloff_factor: 1.0,
            reference_distance: 100.0,
        }
    }

    pub fn set_listener_position(&mut self, pos: [f32; 2]) {
        self.listener_position = pos;
    }

    pub fn set_max_distance(&mut self, dist: f32) { self.max_distance = dist; }
    pub fn set_rolloff(&mut self, factor: f32) { self.rolloff_factor = factor; }
    pub fn set_reference_distance(&mut self, dist: f32) { self.reference_distance = dist; }

    /// Calculate volume based on distance from listener
    pub fn distance_volume(&self, source_pos: [f32; 2]) -> f32 {
        let dx = source_pos[0] - self.listener_position[0];
        let dy = source_pos[1] - self.listener_position[1];
        let distance = (dx * dx + dy * dy).sqrt();

        if distance <= self.reference_distance {
            return 1.0;
        }
        if distance >= self.max_distance {
            return 0.0;
        }

        let attenuation = self.reference_distance
            / (self.reference_distance + self.rolloff_factor * (distance - self.reference_distance));
        attenuation.clamp(0.0, 1.0)
    }

    /// Calculate stereo panning (-1.0 left, 0.0 center, 1.0 right)
    pub fn pan(&self, source_pos: [f32; 2]) -> f32 {
        let dx = source_pos[0] - self.listener_position[0];
        let distance = dx.abs().max(1.0);
        (dx / distance).clamp(-1.0, 1.0)
    }
}

impl Default for SpatialAudioController {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Audio Trigger System
// ---------------------------------------------------------------------------

pub struct AudioTriggerSystem {
    triggers: HashMap<AudioTrigger, Vec<String>>,
    active_zone: String,
}

impl AudioTriggerSystem {
    pub fn new() -> Self {
        Self {
            triggers: HashMap::new(),
            active_zone: String::new(),
        }
    }

    pub fn register(&mut self, trigger: AudioTrigger, sound_ids: Vec<String>) {
        self.triggers.insert(trigger, sound_ids);
    }

    pub fn set_zone(&mut self, zone: &str) {
        self.active_zone = zone.to_string();
    }

    pub fn get_sounds(&self, trigger: &AudioTrigger) -> Vec<&str> {
        self.triggers.get(trigger)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }
}

impl Default for AudioTriggerSystem {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Audio Manager — full-featured
// ---------------------------------------------------------------------------

pub struct AudioManager {
    backends: Vec<Box<dyn AudioBackend>>,
    loaded_sounds: HashMap<String, AudioHandle>,
    next_handle: u32,

    // Volumes per category
    master_volume: f32,
    category_volumes: HashMap<SoundCategory, f32>,

    // Sound pools
    pools: HashMap<SoundCategory, SoundPool>,

    // Music crossfade
    crossfader: MusicCrossfader,

    // Spatial audio
    spatial: SpatialAudioController,

    // Trigger system
    trigger_system: AudioTriggerSystem,

    // Tracking
    active_instances: Vec<PlaybackInstance>,
}

impl AudioManager {
    pub fn new() -> Self {
        let mut category_volumes = HashMap::new();
        category_volumes.insert(SoundCategory::Music, 0.7);
        category_volumes.insert(SoundCategory::Sfx, 1.0);
        category_volumes.insert(SoundCategory::Voice, 1.0);
        category_volumes.insert(SoundCategory::Ambient, 0.5);

        let mut pools = HashMap::new();
        pools.insert(SoundCategory::Music, SoundPool::new(4));
        pools.insert(SoundCategory::Sfx, SoundPool::new(32));
        pools.insert(SoundCategory::Voice, SoundPool::new(8));
        pools.insert(SoundCategory::Ambient, SoundPool::new(8));

        Self {
            backends: Vec::new(),
            loaded_sounds: HashMap::new(),
            next_handle: 1,
            master_volume: 1.0,
            category_volumes,
            pools,
            crossfader: MusicCrossfader::new(2.0),
            spatial: SpatialAudioController::new(),
            trigger_system: AudioTriggerSystem::new(),
            active_instances: Vec::new(),
        }
    }

    /// Register an audio backend
    pub fn add_backend(&mut self, backend: Box<dyn AudioBackend>) {
        self.backends.push(backend);
    }

    /// Load a sound file (cached by path)
    pub fn load(&mut self, path: &str) -> Result<AudioHandle, String> {
        if let Some(&handle) = self.loaded_sounds.get(path) {
            return Ok(handle);
        }
        let backend = self.backends.first_mut()
            .ok_or_else(|| "No audio backend".to_string())?;
        let handle = backend.load_sound(path)?;
        self.loaded_sounds.insert(path.to_string(), handle);
        Ok(handle)
    }

    /// Play a sound in a category
    pub fn play_categorized(&mut self, handle: AudioHandle, category: SoundCategory) -> Result<(), String> {
        let vol = self.effective_volume(category);
        let pool = self.pools.get_mut(&category).unwrap();
        if !pool.acquire(handle) {
            return Err("Sound pool full".into());
        }
        let backend = self.backends.first_mut()
            .ok_or_else(|| "No audio backend".to_string())?;
        backend.play_sound(handle, vol)
    }

    /// Play a sound with spatial positioning
    pub fn play_spatial(&mut self, handle: AudioHandle, category: SoundCategory, position: [f32; 2]) -> Result<(), String> {
        let vol = self.effective_volume(category) * self.spatial.distance_volume(position);
        let backend = self.backends.first_mut()
            .ok_or_else(|| "No audio backend".to_string())?;
        backend.set_position(handle, position)?;
        backend.play_sound(handle, vol)
    }

    /// Play a sound (default SFX category)
    pub fn play(&mut self, handle: AudioHandle) -> Result<(), String> {
        self.play_categorized(handle, SoundCategory::Sfx)
    }

    /// Stop a sound
    pub fn stop(&mut self, handle: AudioHandle) -> Result<(), String> {
        for pool in self.pools.values_mut() {
            pool.release(handle);
        }
        let backend = self.backends.first_mut()
            .ok_or_else(|| "No audio backend".to_string())?;
        backend.stop_sound(handle)
    }

    /// Set master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Set volume for a specific category
    pub fn set_category_volume(&mut self, category: SoundCategory, volume: f32) {
        self.category_volumes.insert(category, volume.clamp(0.0, 1.0));
    }

    /// Get effective volume for a category
    pub fn effective_volume(&self, category: SoundCategory) -> f32 {
        let cat_vol = self.category_volumes.get(&category).copied().unwrap_or(1.0);
        self.master_volume * cat_vol
    }

    /// Get volumes tuple (master, music, sfx)
    pub fn volumes(&self) -> (f32, f32, f32) {
        (
            self.master_volume,
            *self.category_volumes.get(&SoundCategory::Music).unwrap_or(&0.7),
            *self.category_volumes.get(&SoundCategory::Sfx).unwrap_or(&1.0),
        )
    }

    /// Get all category volumes
    pub fn all_volumes(&self) -> &HashMap<SoundCategory, f32> {
        &self.category_volumes
    }

    /// Crossfade music to a new track
    pub fn crossfade_music(&mut self, track: &str) {
        self.crossfader.set_volume(self.effective_volume(SoundCategory::Music));
        self.crossfader.crossfade(track);
    }

    /// Set listener position for spatial audio
    pub fn set_listener_position(&mut self, pos: [f32; 2]) {
        self.spatial.set_listener_position(pos);
    }

    /// Register an audio trigger mapping
    pub fn register_trigger(&mut self, trigger: AudioTrigger, sound_ids: Vec<String>) {
        self.trigger_system.register(trigger, sound_ids);
    }

    /// Fire an audio trigger
    pub fn fire_trigger(&mut self, trigger: &AudioTrigger) -> Vec<Result<(), String>> {
        let sound_ids: Vec<String> = self.trigger_system.get_sounds(trigger)
            .into_iter().map(|s| s.to_string()).collect();
        let mut results = Vec::new();
        for id in &sound_ids {
            if let Ok(handle) = self.load(id) {
                results.push(self.play(handle));
            }
        }
        results
    }

    /// Update crossfader (call each frame)
    pub fn update(&mut self, dt: f32) {
        let crossfade_state = self.crossfader.update(dt);
        // Volume updates would be applied to backend here
        let _ = crossfade_state;
    }

    /// Pool statistics
    pub fn pool_stats(&self) -> HashMap<SoundCategory, (usize, usize)> {
        self.pools.iter()
            .map(|(&cat, pool)| (cat, (pool.active_count(), pool.capacity())))
            .collect()
    }

    pub fn set_crossfade_duration(&mut self, duration: f32) {
        self.crossfader = MusicCrossfader::new(duration);
    }
}

impl Default for AudioManager {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Stub backend for testing (no-op)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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
        manager.set_category_volume(SoundCategory::Music, 0.3);
        manager.set_category_volume(SoundCategory::Sfx, 0.8);
        let (master, music, sfx) = manager.volumes();
        assert!((master - 0.5).abs() < 0.01);
        assert!((music - 0.3).abs() < 0.01);
        assert!((sfx - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_volume_clamping() {
        let mut manager = AudioManager::new();
        manager.set_master_volume(2.0);
        manager.set_master_volume(-0.5);
        let (master, _, _) = manager.volumes();
        assert!((master - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_cached_load() {
        let mut manager = AudioManager::new();
        manager.add_backend(Box::new(StubAudioBackend));
        let h1 = manager.load("test.wav").unwrap();
        let h2 = manager.load("test.wav").unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_effective_volume() {
        let manager = AudioManager::new();
        let vol = manager.effective_volume(SoundCategory::Sfx);
        assert!((vol - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_spatial_audio_distance_volume() {
        let spatial = SpatialAudioController::new();
        let vol = spatial.distance_volume([0.0, 0.0]);
        assert!((vol - 1.0).abs() < 0.01);
        let vol = spatial.distance_volume([2000.0, 0.0]);
        assert!(vol < 0.01);
    }

    #[test]
    fn test_spatial_audio_pan() {
        let spatial = SpatialAudioController::new();
        let pan = spatial.pan([100.0, 0.0]);
        assert!(pan > 0.0);
        let pan = spatial.pan([-100.0, 0.0]);
        assert!(pan < 0.0);
    }

    #[test]
    fn test_crossfader() {
        let mut xfade = MusicCrossfader::new(1.0);
        assert!(xfade.current_track().is_none());
        xfade.crossfade("track1");
        xfade.update(0.5);
        assert!(xfade.is_crossfading());
        xfade.update(1.0);
        assert!(!xfade.is_crossfading());
        assert_eq!(xfade.current_track(), Some("track1"));
    }

    #[test]
    fn test_sound_pool() {
        let mut pool = SoundPool::new(2);
        assert_eq!(pool.capacity(), 2);
        assert!(pool.acquire(AudioHandle(1)));
        assert!(pool.acquire(AudioHandle(2)));
        assert!(!pool.acquire(AudioHandle(3)));
        pool.release(AudioHandle(1));
        assert!(pool.acquire(AudioHandle(4)));
    }

    #[test]
    fn test_audio_trigger_system() {
        let mut sys = AudioTriggerSystem::new();
        sys.register(AudioTrigger::CombatStart, vec!["combat.ogg".into()]);
        let sounds = sys.get_sounds(&AudioTrigger::CombatStart);
        assert_eq!(sounds.len(), 1);
        assert!(sys.get_sounds(&AudioTrigger::PlayerDeath).is_empty());
    }

    #[test]
    fn test_crossfade_same_track_noop() {
        let mut xfade = MusicCrossfader::new(1.0);
        xfade.crossfade("same");
        // Update to completion
        xfade.update(2.0);
        xfade.crossfade("same"); // same track, should be no-op
        assert!(!xfade.is_crossfading());
    }

    #[test]
    fn test_category_volume_default() {
        let manager = AudioManager::new();
        assert!((manager.effective_volume(SoundCategory::Music) - 0.7).abs() < 0.01);
        assert!((manager.effective_volume(SoundCategory::Ambient) - 0.5).abs() < 0.01);
    }
}
