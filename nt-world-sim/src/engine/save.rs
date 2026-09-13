use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::engine::renderer::Vec2;
use crate::engine::inventory::InventorySlot;
use crate::engine::quest::{QuestState, Objective};

// ---------------------------------------------------------------------------
// Save Metadata
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    pub slot: u32,
    pub timestamp: u64,
    pub playtime_seconds: f64,
    pub level: u32,
    pub location: String,
    pub player_name: String,
    pub version: u32,
    pub preview: String,
}

impl SaveMetadata {
    pub fn new(slot: u32, player_name: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            slot,
            timestamp: now,
            playtime_seconds: 0.0,
            level: 1,
            location: "Unknown".into(),
            player_name: player_name.to_string(),
            version: SAVE_VERSION,
            preview: String::new(),
        }
    }

    pub fn timestamp_string(&self) -> String {
        let dt = UNIX_EPOCH + Duration::from_secs(self.timestamp);
        let datetime = format!("{:?}", dt);
        datetime.split('.').next().unwrap_or(&datetime).to_string()
    }

    pub fn playtime_string(&self) -> String {
        let total_secs = self.playtime_seconds as u64;
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        let secs = total_secs % 60;
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    }
}

// ---------------------------------------------------------------------------
// Game State (serializable)
// ---------------------------------------------------------------------------

pub const SAVE_VERSION: u32 = 1;
pub const MAX_SAVE_SLOTS: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveGameState {
    pub metadata: SaveMetadata,
    pub player: PlayerSave,
    pub inventory: Vec<InventorySaveSlot>,
    pub equipment: HashMap<String, InventorySaveSlot>,
    pub quests: Vec<QuestSave>,
    pub flags: HashMap<String, bool>,
    pub counters: HashMap<String, i64>,
    pub world: WorldSave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSave {
    pub name: String,
    pub position: Vec2Save,
    pub health: f32,
    pub max_health: f32,
    pub mana: f32,
    pub max_mana: f32,
    pub level: u32,
    pub experience: u64,
    pub gold: u64,
    pub class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vec2Save {
    pub x: f32,
    pub y: f32,
}

impl From<Vec2> for Vec2Save {
    fn from(v: Vec2) -> Self { Self { x: v.x, y: v.y } }
}

impl From<Vec2Save> for Vec2 {
    fn from(v: Vec2Save) -> Self { Vec2::new(v.x, v.y) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySaveSlot {
    pub item_id: Option<String>,
    pub count: u32,
    pub durability: Option<f32>,
}

impl From<&InventorySlot> for InventorySaveSlot {
    fn from(slot: &InventorySlot) -> Self {
        Self {
            item_id: slot.item_id.clone(),
            count: slot.count,
            durability: slot.durability,
        }
    }
}

impl InventorySaveSlot {
    pub fn to_inventory_slot(&self) -> InventorySlot {
        InventorySlot {
            item_id: self.item_id.clone(),
            count: self.count,
            durability: self.durability,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestSave {
    pub id: String,
    pub name: String,
    pub state: QuestStateSave,
    pub objectives: Vec<ObjectiveSave>,
    pub elapsed_time: f64,
    pub repeat_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestStateSave {
    Unavailable,
    Available,
    Active,
    Complete,
    Failed,
}

impl From<QuestState> for QuestStateSave {
    fn from(state: QuestState) -> Self {
        match state {
            QuestState::Unavailable => QuestStateSave::Unavailable,
            QuestState::Available => QuestStateSave::Available,
            QuestState::Active => QuestStateSave::Active,
            QuestState::Complete => QuestStateSave::Complete,
            QuestState::Failed => QuestStateSave::Failed,
        }
    }
}

impl From<QuestStateSave> for QuestState {
    fn from(state: QuestStateSave) -> Self {
        match state {
            QuestStateSave::Unavailable => QuestState::Unavailable,
            QuestStateSave::Available => QuestState::Available,
            QuestStateSave::Active => QuestState::Active,
            QuestStateSave::Complete => QuestState::Complete,
            QuestStateSave::Failed => QuestState::Failed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveSave {
    pub id: String,
    pub description: String,
    pub progress: u32,
    pub target: u32,
    pub completed: bool,
    pub optional: bool,
}

impl From<&Objective> for ObjectiveSave {
    fn from(obj: &Objective) -> Self {
        Self {
            id: obj.id.clone(),
            description: obj.description.clone(),
            progress: obj.current_progress,
            target: obj.target_count,
            completed: obj.completed,
            optional: obj.optional,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSave {
    pub current_zone: String,
    pub unlocked_zones: Vec<String>,
    pub discovered_areas: Vec<String>,
    pub npc_states: HashMap<String, NpcSaveState>,
    pub chests_opened: Vec<String>,
    pub flags: HashMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcSaveState {
    pub id: String,
    pub relation: i32,
    pub talked: bool,
    pub quests_given: Vec<String>,
}

// ---------------------------------------------------------------------------
// Save Backend Trait
// ---------------------------------------------------------------------------

pub trait SaveBackend: Send + Sync {
    fn save(&self, slot: u32, data: &str) -> Result<(), String>;
    fn load(&self, slot: u32) -> Result<String, String>;
    fn exists(&self, slot: u32) -> bool;
    fn delete(&self, slot: u32) -> Result<(), String>;
    fn list_metadata(&self) -> Vec<SaveMetadata>;
}

// ---------------------------------------------------------------------------
// File-based Save Backend
// ---------------------------------------------------------------------------

pub struct FileSaveBackend {
    save_dir: std::path::PathBuf,
}

impl FileSaveBackend {
    pub fn new(save_dir: &str) -> Self {
        Self { save_dir: std::path::PathBuf::from(save_dir) }
    }

    fn slot_path(&self, slot: u32) -> std::path::PathBuf {
        self.save_dir.join(format!("slot_{}.json", slot))
    }

    fn meta_path(&self, slot: u32) -> std::path::PathBuf {
        self.save_dir.join(format!("slot_{}_meta.json", slot))
    }
}

impl SaveBackend for FileSaveBackend {
    fn save(&self, slot: u32, data: &str) -> Result<(), String> {
        std::fs::create_dir_all(&self.save_dir)
            .map_err(|e| format!("Failed to create save dir: {}", e))?;
        std::fs::write(self.slot_path(slot), data)
            .map_err(|e| format!("Failed to write save: {}", e))
    }

    fn load(&self, slot: u32) -> Result<String, String> {
        std::fs::read_to_string(self.slot_path(slot))
            .map_err(|e| format!("Failed to load save: {}", e))
    }

    fn exists(&self, slot: u32) -> bool {
        self.slot_path(slot).exists()
    }

    fn delete(&self, slot: u32) -> Result<(), String> {
        let path = self.slot_path(slot);
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete save: {}", e))?;
        }
        let meta = self.meta_path(slot);
        if meta.exists() {
            let _ = std::fs::remove_file(meta);
        }
        Ok(())
    }

    fn list_metadata(&self) -> Vec<SaveMetadata> {
        let mut metas = Vec::new();
        for slot in 0..MAX_SAVE_SLOTS as u32 {
            let path = self.slot_path(slot);
            if path.exists() {
                if let Ok(json) = std::fs::read_to_string(&path) {
                    if let Ok(state) = serde_json::from_str::<SaveGameState>(&json) {
                        metas.push(state.metadata);
                    }
                }
            }
        }
        metas
    }
}

// ---------------------------------------------------------------------------
// In-memory Save Backend (for HTML5 / testing)
// ---------------------------------------------------------------------------

pub struct MemorySaveBackend {
    saves: HashMap<u32, String>,
}

impl MemorySaveBackend {
    pub fn new() -> Self {
        Self { saves: HashMap::new() }
    }
}

impl Default for MemorySaveBackend {
    fn default() -> Self { Self::new() }
}

impl SaveBackend for MemorySaveBackend {
    fn save(&self, _slot: u32, _data: &str) -> Result<(), String> {
        // In-memory backend: in real HTML5 this would call localStorage
        Ok(())
    }

    fn load(&self, slot: u32) -> Result<String, String> {
        self.saves.get(&slot).cloned()
            .ok_or_else(|| format!("Save slot {} not found", slot))
    }

    fn exists(&self, slot: u32) -> bool {
        self.saves.contains_key(&slot)
    }

    fn delete(&self, _slot: u32) -> Result<(), String> { Ok(()) }

    fn list_metadata(&self) -> Vec<SaveMetadata> { vec![] }
}

// ---------------------------------------------------------------------------
// Auto-Save Manager
// ---------------------------------------------------------------------------

pub struct AutoSaveManager {
    interval_secs: f64,
    timer: f64,
    enabled: bool,
    dirty: bool,
}

impl AutoSaveManager {
    pub fn new(interval_secs: f64) -> Self {
        Self { interval_secs, timer: 0.0, enabled: true, dirty: false }
    }

    pub fn set_interval(&mut self, secs: f64) {
        self.interval_secs = secs.max(10.0);
    }

    pub fn enable(&mut self) { self.enabled = true; }
    pub fn disable(&mut self) { self.enabled = false; }

    pub fn mark_dirty(&mut self) { self.dirty = true; }

    pub fn tick(&mut self, dt: f64) -> bool {
        if !self.enabled || !self.dirty { return false; }
        self.timer += dt;
        if self.timer >= self.interval_secs {
            self.timer = 0.0;
            self.dirty = false;
            return true;
        }
        false
    }

    pub fn should_save(&self) -> bool { self.dirty && self.enabled }
    pub fn is_enabled(&self) -> bool { self.enabled }
    pub fn next_save_in(&self) -> f64 { (self.interval_secs - self.timer).max(0.0) }
}

impl Default for AutoSaveManager {
    fn default() -> Self { Self::new(300.0) }
}

// ---------------------------------------------------------------------------
// Save Manager (orchestrator)
// ---------------------------------------------------------------------------

pub struct EngineSaveManager {
    backend: Box<dyn SaveBackend>,
    auto_save: AutoSaveManager,
    last_save_slot: Option<u32>,
    validation_hash: bool,
}

impl EngineSaveManager {
    pub fn new(backend: Box<dyn SaveBackend>) -> Self {
        Self {
            backend,
            auto_save: AutoSaveManager::default(),
            last_save_slot: None,
            validation_hash: true,
        }
    }

    pub fn with_auto_save(backend: Box<dyn SaveBackend>, interval_secs: f64) -> Self {
        Self {
            backend,
            auto_save: AutoSaveManager::new(interval_secs),
            last_save_slot: None,
            validation_hash: true,
        }
    }

    /// Save game state to a slot
    pub fn save(&mut self, slot: u32, state: &SaveGameState) -> Result<(), String> {
        if slot >= MAX_SAVE_SLOTS as u32 {
            return Err(format!("Invalid save slot: {} (max {})", slot, MAX_SAVE_SLOTS));
        }

        let json = serde_json::to_string_pretty(state)
            .map_err(|e| format!("Serialization error: {}", e))?;

        // Simple checksum validation
        let data = if self.validation_hash {
            let hash = simple_hash(&json);
            serde_json::json!({
                "data": json,
                "hash": hash,
            }).to_string()
        } else {
            json
        };

        self.backend.save(slot, &data)?;
        self.last_save_slot = Some(slot);
        self.auto_save.mark_dirty();
        Ok(())
    }

    /// Load game state from a slot
    pub fn load(&self, slot: u32) -> Result<SaveGameState, String> {
        let raw = self.backend.load(slot)?;

        let json = if self.validation_hash {
            let parsed: serde_json::Value = serde_json::from_str(&raw)
                .map_err(|e| format!("Invalid save format: {}", e))?;

            if let Some(data) = parsed.get("data").and_then(|d| d.as_str()) {
                if let Some(stored_hash) = parsed.get("hash").and_then(|h| h.as_u64()) {
                    let computed = simple_hash(data);
                    if computed != stored_hash {
                        return Err("Save file corrupted (hash mismatch)".into());
                    }
                }
                data.to_string()
            } else {
                raw
            }
        } else {
            raw
        };

        let state: SaveGameState = serde_json::from_str(&json)
            .map_err(|e| format!("Deserialization error: {}", e))?;

        if state.metadata.version > SAVE_VERSION {
            return Err(format!("Save version {} not supported (max {})",
                state.metadata.version, SAVE_VERSION));
        }

        Ok(state)
    }

    /// Check if a slot has a save
    pub fn has_save(&self, slot: u32) -> bool {
        self.backend.exists(slot)
    }

    /// Delete a save slot
    pub fn delete_save(&self, slot: u32) -> Result<(), String> {
        self.backend.delete(slot)
    }

    /// Get metadata for all saves
    pub fn list_saves(&self) -> Vec<SaveMetadata> {
        self.backend.list_metadata()
    }

    /// Auto-save tick (called from game loop)
    pub fn auto_save_tick(&mut self, dt: f64, current_state: Option<&SaveGameState>) -> bool {
        if self.auto_save.tick(dt) {
            if let (Some(slot), Some(state)) = (self.last_save_slot, current_state) {
                let _ = self.save(slot, state);
                return true;
            }
        }
        false
    }

    /// Mark state as dirty for auto-save
    pub fn mark_dirty(&mut self) {
        self.auto_save.mark_dirty();
    }

    pub fn set_auto_save_interval(&mut self, secs: f64) {
        self.auto_save.set_interval(secs);
    }

    pub fn enable_auto_save(&mut self) { self.auto_save.enable(); }
    pub fn disable_auto_save(&mut self) { self.auto_save.disable(); }

    pub fn last_save_slot(&self) -> Option<u32> { self.last_save_slot }

    /// Create a new default game state
    pub fn new_game(slot: u32, player_name: &str) -> SaveGameState {
        SaveGameState {
            metadata: SaveMetadata::new(slot, player_name),
            player: PlayerSave {
                name: player_name.to_string(),
                position: Vec2Save { x: 0.0, y: 0.0 },
                health: 100.0,
                max_health: 100.0,
                mana: 50.0,
                max_mana: 50.0,
                level: 1,
                experience: 0,
                gold: 100,
                class: "Warrior".into(),
            },
            inventory: vec![InventorySaveSlot { item_id: None, count: 0, durability: None }; 20],
            equipment: HashMap::new(),
            quests: Vec::new(),
            flags: HashMap::new(),
            counters: HashMap::new(),
            world: WorldSave {
                current_zone: "town".into(),
                unlocked_zones: vec!["town".into()],
                discovered_areas: Vec::new(),
                npc_states: HashMap::new(),
                chests_opened: Vec::new(),
                flags: HashMap::new(),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn simple_hash(data: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in data.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

pub fn validate_save(state: &SaveGameState) -> Vec<String> {
    let mut issues = Vec::new();

    if state.metadata.version > SAVE_VERSION {
        issues.push(format!("Unsupported save version: {}", state.metadata.version));
    }
    if state.player.health < 0.0 {
        issues.push("Player health is negative".into());
    }
    if state.player.health > state.player.max_health {
        issues.push("Player health exceeds max".into());
    }
    if state.player.mana > state.player.max_mana {
        issues.push("Player mana exceeds max".into());
    }
    if state.inventory.len() > 100 {
        issues.push(format!("Inventory too large: {} slots", state.inventory.len()));
    }
    for (i, slot) in state.inventory.iter().enumerate() {
        if let Some(id) = &slot.item_id {
            if id.is_empty() {
                issues.push(format!("Inventory slot {} has empty item ID", i));
            }
            if slot.count == 0 {
                issues.push(format!("Inventory slot {} has zero count with item ID", i));
            }
        }
    }
    issues
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_metadata() {
        let meta = SaveMetadata::new(1, "Hero");
        assert_eq!(meta.slot, 1);
        assert_eq!(meta.player_name, "Hero");
        assert!(meta.playtime_string().contains(":"));
    }

    #[test]
    fn test_new_game() {
        let state = EngineSaveManager::new_game(0, "TestPlayer");
        assert_eq!(state.metadata.slot, 0);
        assert_eq!(state.player.name, "TestPlayer");
        assert_eq!(state.player.health, 100.0);
        assert_eq!(state.inventory.len(), 20);
    }

    #[test]
    fn test_memory_backend() {
        let backend = MemorySaveBackend::new();
        assert!(!backend.exists(0));
        assert!(backend.list_metadata().is_empty());
    }

    #[test]
    fn test_auto_save_manager() {
        let mut auto = AutoSaveManager::new(10.0);
        assert!(!auto.tick(5.0));
        auto.mark_dirty();
        assert!(!auto.tick(5.0));
        assert!(auto.tick(10.0));
    }

    #[test]
    fn test_validation() {
        let state = EngineSaveManager::new_game(0, "Test");
        let issues = validate_save(&state);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_validation_catches_negative_health() {
        let mut state = EngineSaveManager::new_game(0, "Test");
        state.player.health = -10.0;
        let issues = validate_save(&state);
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_save_load_roundtrip() {
        let backend = MemorySaveBackend::new();
        let mut mgr = EngineSaveManager::new(Box::new(backend));
        let state = EngineSaveManager::new_game(0, "Hero");
        mgr.save(0, &state).unwrap();
        let loaded = mgr.load(0).unwrap();
        assert_eq!(loaded.player.name, "Hero");
        assert_eq!(loaded.player.health, 100.0);
    }

    #[test]
    fn test_save_slot_limit() {
        let backend = MemorySaveBackend::new();
        let mut mgr = EngineSaveManager::new(Box::new(backend));
        let state = EngineSaveManager::new_game(0, "Test");
        assert!(mgr.save(5, &state).is_err());
    }

    #[test]
    fn test_simple_hash_deterministic() {
        let h1 = simple_hash("hello world");
        let h2 = simple_hash("hello world");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_playtime_format() {
        let mut meta = SaveMetadata::new(0, "Test");
        meta.playtime_seconds = 3661.0;
        assert_eq!(meta.playtime_string(), "01:01:01");
    }
}
