use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{GameError, GameResult};
use crate::core::UniversalWorld;
use crate::game::inventory::Inventory;
use crate::game::time::GameTime;
use crate::game::energy::Energy;
use crate::game::farming::{CropTile, CropState};
use crate::game::npc::Npc;
use crate::world::tile::WorldMap;

// ---------------------------------------------------------------------------
// Save data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub slot: u32,
    pub player_name: String,
    pub play_time_seconds: f64,
    pub tick_count: u64,

    // Time
    pub day: u32,
    pub season: String,
    pub year: u32,
    pub hour: u32,
    pub minute: u32,

    // Player stats
    pub energy: f32,
    pub max_energy: f32,
    pub insight_points: u32,
    pub resonance: u32,

    // Skills
    pub skills: HashMap<String, u32>,

    // Inventory
    pub inventory: Vec<InventorySlotData>,
    pub hotbar_index: usize,
    pub gold: u32,

    // Farm
    pub farm_tiles: Vec<FarmTileData>,

    // NPCs
    pub npcs: Vec<NpcData>,

    // World state
    pub current_zone: u32,
    pub unlocked_zones: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlotData {
    pub item_id: Option<u32>,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FarmTileData {
    pub x: u32,
    pub y: u32,
    pub state: CropStateData,
    pub watered: bool,
    pub fertilized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CropStateData {
    Empty,
    Tilled,
    Seeded { seed_id: u32, day_planted: u32 },
    Growing { seed_id: u32, day_planted: u32, growth: f32 },
    Ready { seed_id: u32 },
    Withered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcData {
    pub name: String,
    pub role: String,
    pub resonance: u32,
    pub gifts_this_week: u32,
}

// ---------------------------------------------------------------------------
// Conversion: CropState <-> CropStateData
// ---------------------------------------------------------------------------

impl From<&CropState> for CropStateData {
    fn from(state: &CropState) -> Self {
        match state {
            CropState::Empty => CropStateData::Empty,
            CropState::Tilled => CropStateData::Tilled,
            CropState::Seeded { seed_id, day_planted } => CropStateData::Seeded { seed_id: *seed_id, day_planted: *day_planted },
            CropState::Growing { seed_id, day_planted, growth } => CropStateData::Growing { seed_id: *seed_id, day_planted: *day_planted, growth: *growth },
            CropState::Ready { seed_id } => CropStateData::Ready { seed_id: *seed_id },
            CropState::Withered => CropStateData::Withered,
        }
    }
}

impl From<&CropStateData> for CropState {
    fn from(data: &CropStateData) -> Self {
        match data {
            CropStateData::Empty => CropState::Empty,
            CropStateData::Tilled => CropState::Tilled,
            CropStateData::Seeded { seed_id, day_planted } => CropState::Seeded { seed_id: *seed_id, day_planted: *day_planted },
            CropStateData::Growing { seed_id, day_planted, growth } => CropState::Growing { seed_id: *seed_id, day_planted: *day_planted, growth: *growth },
            CropStateData::Ready { seed_id } => CropState::Ready { seed_id: *seed_id },
            CropStateData::Withered => CropState::Withered,
        }
    }
}

// ---------------------------------------------------------------------------
// SaveManager
// ---------------------------------------------------------------------------

pub struct SaveManager {
    save_dir: PathBuf,
}

impl SaveManager {
    pub fn new() -> Self {
        Self { save_dir: PathBuf::from("saves") }
    }

    pub fn with_dir(save_dir: &Path) -> Self {
        Self { save_dir: save_dir.to_path_buf() }
    }

    /// Save game state to a slot
    pub fn save(&self, slot: u32, data: &SaveData) -> GameResult<()> {
        std::fs::create_dir_all(&self.save_dir)?;

        let path = self.save_dir.join(format!("slot_{}.json", slot));
        let json = serde_json::to_string_pretty(data)?;

        std::fs::write(&path, json)?;

        Ok(())
    }

    /// Load game state from a slot
    pub fn load(&self, slot: u32) -> GameResult<SaveData> {
        let path = self.save_dir.join(format!("slot_{}.json", slot));

        if !path.exists() {
            return Err(GameError::Load(format!("Save slot {} does not exist", slot)));
        }

        let json = std::fs::read_to_string(&path)?;
        let data = serde_json::from_str(&json)?;
        Ok(data)
    }

    /// Check if a save slot exists
    pub fn has_save(&self, slot: u32) -> bool {
        let path = self.save_dir.join(format!("slot_{}.json", slot));
        path.exists()
    }

    /// Delete a save slot
    pub fn delete(&self, slot: u32) -> GameResult<()> {
        let path = self.save_dir.join(format!("slot_{}.json", slot));
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// List all save slots
    pub fn list_saves(&self) -> Vec<u32> {
        let mut slots = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.save_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("slot_") && name_str.ends_with(".json") {
                    let num_str = &name_str[5..name_str.len()-5];
                    if let Ok(slot) = num_str.parse::<u32>() {
                        slots.push(slot);
                    }
                }
            }
        }
        slots.sort();
        slots
    }

    /// Save from UniversalWorld + tick_count
    pub fn save_from_world(&self, slot: u32, world: &UniversalWorld, tick_count: u64) -> GameResult<()> {
        let time = world.get_resource::<GameTime>().cloned().unwrap_or_default();
        let energy = world.get_resource::<Energy>().cloned().unwrap_or_default();
        let inventory = world.get_resource::<Inventory>().cloned().unwrap_or_default();

        let data = SaveData {
            version: 1,
            slot,
            player_name: "Player".to_string(),
            play_time_seconds: tick_count as f64,
            tick_count,
            day: time.day,
            season: format!("{:?}", time.season),
            year: time.year,
            hour: time.hour,
            minute: time.minute,
            energy: energy.current,
            max_energy: energy.max,
            insight_points: 0,
            resonance: 0,
            skills: HashMap::new(),
            inventory: inventory.slots.iter().map(|s| {
                InventorySlotData { item_id: s.item_id, count: s.quantity }
            }).collect(),
            hotbar_index: inventory.selected_hotbar,
            gold: inventory.gold as u32,
            farm_tiles: vec![],
            npcs: vec![],
            current_zone: 0,
            unlocked_zones: vec![0, 1, 2, 3, 4],
        };
        self.save(slot, &data)
    }

    /// Load and return tick_count
    pub fn load_tick_count(&self, slot: u32) -> GameResult<u64> {
        let data = self.load(slot)?;
        Ok(data.tick_count)
    }
}

// ---------------------------------------------------------------------------
// Serialization helpers: WorldMap <-> SaveData
// ---------------------------------------------------------------------------

impl SaveData {
    /// Create a new save from current game state
    pub fn from_game_state(
        player_name: &str,
        game_time: &GameTime,
        energy: &Energy,
        inventory: &Inventory,
        farm_tiles: &[CropTile],
        npcs: &[Npc],
        world_map: &WorldMap,
        play_time: f64,
    ) -> Self {
        let skills = HashMap::from([
            ("awareness".into(), 0),
            ("focus".into(), 0),
            ("creativity".into(), 0),
            ("empathy".into(), 0),
            ("logic".into(), 0),
        ]);

        let farm_data: Vec<FarmTileData> = farm_tiles.iter().enumerate().map(|(i, tile)| {
            FarmTileData {
                x: (i as u32) % world_map.width,
                y: (i as u32) / world_map.width,
                state: CropStateData::from(&tile.state),
                watered: tile.watered,
                fertilized: tile.fertilized,
            }
        }).collect();

        let npc_data: Vec<NpcData> = npcs.iter().map(|npc| {
            NpcData {
                name: npc.name.clone(),
                role: format!("{:?}", npc.role),
                resonance: npc.resonance,
                gifts_this_week: npc.gifts_this_week,
            }
        }).collect();

        Self {
            version: 1,
            slot: 0,
            player_name: player_name.to_string(),
            play_time_seconds: play_time,
            tick_count: 0,
            day: game_time.day,
            season: format!("{:?}", game_time.season),
            year: game_time.year,
            hour: game_time.hour,
            minute: game_time.minute,
            energy: energy.current,
            max_energy: energy.max,
            insight_points: 0,
            resonance: 0,
            skills,
            inventory: inventory.slots.iter().map(|s| {
                InventorySlotData {
                    item_id: s.item_id,
                    count: s.quantity,
                }
            }).collect(),
            hotbar_index: inventory.selected_hotbar,
            gold: inventory.gold as u32,
            farm_tiles: farm_data,
            npcs: npc_data,
            current_zone: 0,
            unlocked_zones: vec![0, 1, 2, 3, 4],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_data_serialization() {
        let data = SaveData {
            version: 1,
            slot: 1,
            player_name: "TestPlayer".into(),
            play_time_seconds: 3600.0,
            day: 1,
            season: "Clarity".into(),
            year: 1,
            hour: 6,
            minute: 0,
            energy: 100.0,
            max_energy: 100.0,
            insight_points: 50,
            resonance: 100,
            skills: HashMap::new(),
            inventory: vec![],
            hotbar_index: 0,
            gold: 500,
            farm_tiles: vec![],
            npcs: vec![],
            current_zone: 0,
            unlocked_zones: vec![0],
        };

        let json = serde_json::to_string(&data).unwrap();
        let restored: SaveData = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.player_name, "TestPlayer");
        assert_eq!(restored.gold, 500);
    }

    #[test]
    fn test_crop_state_roundtrip() {
        let states = vec![
            CropStateData::Empty,
            CropStateData::Tilled,
            CropStateData::Seeded { seed_id: 1, day_planted: 5 },
            CropStateData::Growing { seed_id: 1, day_planted: 5, growth: 0.5 },
            CropStateData::Ready { seed_id: 1 },
            CropStateData::Withered,
        ];

        for data in &states {
            let json = serde_json::to_string(data).unwrap();
            let restored: CropStateData = serde_json::from_str(&json).unwrap();
            let crop_state = CropState::from(&restored);
            let back = CropStateData::from(&crop_state);
            let json2 = serde_json::to_string(&back).unwrap();
            assert_eq!(json, json2);
        }
    }
}
