use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const CURRENT_SAVE_VERSION: u32 = 1;

// ─── SaveGame ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveGame {
    pub version: u32,
    pub timestamp: u64,
    pub player_name: String,
    pub player_position: (f32, f32),
    pub player_health: f32,
    pub level: u32,
    pub experience: u64,
    pub flags: HashMap<String, String>,
    pub inventory: Vec<String>,
    pub quests_completed: Vec<String>,
    pub play_time_seconds: u64,
}

impl SaveGame {
    pub fn new(player_name: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            version: CURRENT_SAVE_VERSION,
            timestamp,
            player_name: player_name.to_string(),
            player_position: (0.0, 0.0),
            player_health: 100.0,
            level: 1,
            experience: 0,
            flags: HashMap::new(),
            inventory: Vec::new(),
            quests_completed: Vec::new(),
            play_time_seconds: 0,
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), SaveError> {
        let data = bincode::serialize(self).map_err(|e| SaveError::Serialization(e.to_string()))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(SaveError::Io)?;
        }
        fs::write(path, data).map_err(SaveError::Io)
    }

    pub fn load(path: &Path) -> Result<Self, SaveError> {
        let data = fs::read(path).map_err(SaveError::Io)?;
        let mut save: SaveGame =
            bincode::deserialize(&data).map_err(|e| SaveError::Serialization(e.to_string()))?;
        save.migrate()?;
        Ok(save)
    }

    fn migrate(&mut self) -> Result<(), SaveError> {
        if self.version > CURRENT_SAVE_VERSION {
            return Err(SaveError::VersionMismatch {
                expected: CURRENT_SAVE_VERSION,
                found: self.version,
            });
        }

        while self.version < CURRENT_SAVE_VERSION {
            self.migrate_once()?;
        }
        Ok(())
    }

    fn migrate_once(&mut self) -> Result<(), SaveError> {
        match self.version {
            0 => {
                // v0 → v1: initialize empty defaults for fields added in v1
                self.flags = HashMap::new();
                self.inventory = Vec::new();
                self.quests_completed = Vec::new();
                self.version = 1;
            }
            _ => {
                return Err(SaveError::VersionMismatch {
                    expected: CURRENT_SAVE_VERSION,
                    found: self.version,
                });
            }
        }
        Ok(())
    }
}

// ─── SaveManager ────────────────────────────────────────────────────────────

pub struct SaveManager {
    save_dir: PathBuf,
    max_slots: usize,
    auto_save_interval: Duration,
    last_auto_save: Instant,
}

impl SaveManager {
    pub fn new(save_dir: PathBuf) -> Self {
        Self {
            save_dir,
            max_slots: 10,
            auto_save_interval: Duration::from_secs(300),
            last_auto_save: Instant::now(),
        }
    }

    pub fn with_max_slots(mut self, max_slots: usize) -> Self {
        self.max_slots = max_slots;
        self
    }

    pub fn with_auto_save_interval(mut self, interval: Duration) -> Self {
        self.auto_save_interval = interval;
        self
    }

    fn slot_path(&self, slot: usize) -> PathBuf {
        self.save_dir.join(format!("slot_{}.sav", slot))
    }

    pub fn save_game(&self, slot: usize, save: &SaveGame) -> Result<(), SaveError> {
        if slot >= self.max_slots {
            return Err(SaveError::SlotNotFound(slot));
        }
        fs::create_dir_all(&self.save_dir).map_err(SaveError::Io)?;
        save.save(&self.slot_path(slot))
    }

    pub fn load_game(&self, slot: usize) -> Result<SaveGame, SaveError> {
        if slot >= self.max_slots {
            return Err(SaveError::SlotNotFound(slot));
        }
        let path = self.slot_path(slot);
        if !path.exists() {
            return Err(SaveError::SlotNotFound(slot));
        }
        SaveGame::load(&path)
    }

    pub fn delete_save(&self, slot: usize) -> Result<(), SaveError> {
        if slot >= self.max_slots {
            return Err(SaveError::SlotNotFound(slot));
        }
        let path = self.slot_path(slot);
        if !path.exists() {
            return Err(SaveError::SlotNotFound(slot));
        }
        fs::remove_file(path).map_err(SaveError::Io)
    }

    pub fn list_saves(&self) -> Vec<SaveSlotInfo> {
        let mut slots = Vec::new();
        for i in 0..self.max_slots {
            let path = self.slot_path(i);
            if let Ok(data) = fs::read(&path) {
                if let Ok(save) = bincode::deserialize::<SaveGame>(&data) {
                    slots.push(SaveSlotInfo {
                        slot: i,
                        player_name: save.player_name,
                        timestamp: save.timestamp,
                        play_time: save.play_time_seconds,
                    });
                }
            }
        }
        slots
    }

    pub fn check_auto_save(&mut self, current_save: &SaveGame) -> Option<Result<(), SaveError>> {
        if self.last_auto_save.elapsed() >= self.auto_save_interval {
            self.last_auto_save = Instant::now();
            Some(self.save_game(0, current_save))
        } else {
            None
        }
    }
}

// ─── SaveSlotInfo ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SaveSlotInfo {
    pub slot: usize,
    pub player_name: String,
    pub timestamp: u64,
    pub play_time: u64,
}

// ─── SaveError ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum SaveError {
    Io(io::Error),
    Serialization(String),
    VersionMismatch { expected: u32, found: u32 },
    SlotNotFound(usize),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Io(e) => write!(f, "IO error: {}", e),
            SaveError::Serialization(e) => write!(f, "Serialization error: {}", e),
            SaveError::VersionMismatch { expected, found } => {
                write!(f, "Version mismatch: expected {}, found {}", expected, found)
            }
            SaveError::SlotNotFound(s) => write!(f, "Slot not found: {}", s),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<io::Error> for SaveError {
    fn from(e: io::Error) -> Self {
        SaveError::Io(e)
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_save_dir() -> PathBuf {
        let dir = PathBuf::from(format!(
            "/tmp/neotrix_save_test_{}",
            std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn new_save_has_correct_defaults() {
        let save = SaveGame::new("Hero");
        assert_eq!(save.version, CURRENT_SAVE_VERSION);
        assert_eq!(save.player_name, "Hero");
        assert_eq!(save.player_health, 100.0);
        assert_eq!(save.level, 1);
        assert!(save.flags.is_empty());
        assert!(save.inventory.is_empty());
        assert!(save.quests_completed.is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = temp_save_dir();
        let path = dir.join("test.sav");

        let mut save = SaveGame::new("Tester");
        save.player_position = (42.0, 7.5);
        save.level = 5;
        save.experience = 1234;
        save.inventory.push("sword".into());
        save.quests_completed.push("find_dragon".into());
        save.flags.insert("door_opened".into(), "true".into());
        save.play_time_seconds = 9999;

        save.save(&path).unwrap();
        let loaded = SaveGame::load(&path).unwrap();

        assert_eq!(loaded.player_name, "Tester");
        assert_eq!(loaded.player_position, (42.0, 7.5));
        assert_eq!(loaded.level, 5);
        assert_eq!(loaded.experience, 1234);
        assert_eq!(loaded.inventory, vec!["sword"]);
        assert_eq!(loaded.quests_completed, vec!["find_dragon"]);
        assert_eq!(loaded.flags.get("door_opened").unwrap(), "true");
        assert_eq!(loaded.play_time_seconds, 9999);
        assert_eq!(loaded.version, CURRENT_SAVE_VERSION);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn save_creates_parent_dirs() {
        let dir = temp_save_dir();
        let path = dir.join("nested/deep/save.sav");

        let save = SaveGame::new("Nested");
        save.save(&path).unwrap();
        assert!(path.exists());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn load_nonexistent_returns_slot_not_found() {
        let dir = temp_save_dir();
        let result = SaveGame::load(&dir.join("nope.sav"));
        assert!(matches!(result, Err(SaveError::Io(_))));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn manager_save_and_load() {
        let dir = temp_save_dir();
        let mgr = SaveManager::new(dir.clone());
        let mut save = SaveGame::new("Mgr");
        save.level = 10;

        mgr.save_game(0, &save).unwrap();
        let loaded = mgr.load_game(0).unwrap();
        assert_eq!(loaded.level, 10);
        assert_eq!(loaded.player_name, "Mgr");

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn manager_slot_out_of_range() {
        let dir = temp_save_dir();
        let mgr = SaveManager::new(dir.clone());
        let save = SaveGame::new("Bad");

        assert!(matches!(
            mgr.save_game(99, &save),
            Err(SaveError::SlotNotFound(99))
        ));
        assert!(matches!(
            mgr.load_game(99),
            Err(SaveError::SlotNotFound(99))
        ));
        assert!(matches!(
            mgr.delete_save(99),
            Err(SaveError::SlotNotFound(99))
        ));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn manager_delete_save() {
        let dir = temp_save_dir();
        let mgr = SaveManager::new(dir.clone());
        let save = SaveGame::new("Deleter");

        mgr.save_game(2, &save).unwrap();
        assert!(mgr.load_game(2).is_ok());

        mgr.delete_save(2).unwrap();
        assert!(matches!(mgr.load_game(2), Err(SaveError::SlotNotFound(2))));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn manager_list_saves() {
        let dir = temp_save_dir();
        let mgr = SaveManager::new(dir.clone());

        let mut s1 = SaveGame::new("Alice");
        s1.play_time_seconds = 100;
        mgr.save_game(0, &s1).unwrap();

        let mut s2 = SaveGame::new("Bob");
        s2.play_time_seconds = 200;
        mgr.save_game(3, &s2).unwrap();

        let list = mgr.list_saves();
        assert_eq!(list.len(), 2);

        let names: Vec<&str> = list.iter().map(|s| s.player_name.as_str()).collect();
        assert!(names.contains(&"Alice"));
        assert!(names.contains(&"Bob"));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn manager_auto_save_triggers() {
        let dir = temp_save_dir();
        let mut mgr = SaveManager::new(dir.clone())
            .with_auto_save_interval(Duration::from_millis(0));
        let save = SaveGame::new("Auto");

        // Should trigger immediately (interval = 0)
        let result = mgr.check_auto_save(&save);
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        // Verify it actually saved
        let loaded = mgr.load_game(0).unwrap();
        assert_eq!(loaded.player_name, "Auto");

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn manager_auto_save_not_yet() {
        let dir = temp_save_dir();
        let mut mgr = SaveManager::new(dir.clone())
            .with_auto_save_interval(Duration::from_secs(600));
        let save = SaveGame::new("NotYet");

        let result = mgr.check_auto_save(&save);
        assert!(result.is_none());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn version_rejects_future_version() {
        let dir = temp_save_dir();
        let path = dir.join("Future.sav");

        let mut save = SaveGame::new("Future");
        save.version = 999;
        save.save(&path).unwrap();

        let result = SaveGame::load(&path);
        assert!(matches!(
            result,
            Err(SaveError::VersionMismatch {
                expected: CURRENT_SAVE_VERSION,
                found: 999
            })
        ));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn save_error_display() {
        let e = SaveError::SlotNotFound(5);
        assert_eq!(format!("{}", e), "Slot not found: 5");

        let e = SaveError::VersionMismatch {
            expected: 1,
            found: 0,
        };
        assert_eq!(format!("{}", e), "Version mismatch: expected 1, found 0");

        let e = SaveError::Serialization("bad data".into());
        assert_eq!(format!("{}", e), "Serialization error: bad data");
    }
}
