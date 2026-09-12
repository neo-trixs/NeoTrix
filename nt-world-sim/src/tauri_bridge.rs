use crate::game::game_loop::{GameLoop, GameAction};
use crate::game::time::GameTime;
use crate::game::weather::Weather;
use crate::game::inventory::Inventory;
use crate::game::npc::Position;
use once_cell::sync::Lazy;
use std::fs;
use std::sync::Mutex;

static GAME_STATE: Lazy<Mutex<Option<GameLoop>>> = Lazy::new(|| Mutex::new(None));

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct GameSnapshot {
    pub state: String,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub season: String,
    pub weather: String,
    pub energy: u32,
    pub max_energy: u32,
    pub gold: i32,
    pub tick: u64,
    pub player_x: u32,
    pub player_y: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct InventorySnapshot {
    pub slots: Vec<InventorySlotSnapshot>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct InventorySlotSnapshot {
    pub item_name: String,
    pub quantity: u32,
    pub icon_color: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FarmPlotSnapshot {
    pub x: u32,
    pub y: u32,
    pub tilled: bool,
    pub watered: bool,
    pub crop_name: String,
    pub growth_stage: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct NpcSnapshot {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub resonance: u32,
    pub dialogue_hint: String,
}

pub fn init_game() -> Result<String, String> {
    let mut game = GameLoop::new();
    game.start_game();

    let mut state = GAME_STATE.lock().map_err(|e| e.to_string())?;
    *state = Some(game);

    Ok("Game initialized".to_string())
}

pub fn get_game_snapshot() -> Result<GameSnapshot, String> {
    let state = GAME_STATE.lock().map_err(|e| e.to_string())?;
    let game = state.as_ref().ok_or("Game not initialized")?;

    let time = game.world.get_resource::<GameTime>();
    let weather = game.world.get_resource::<Weather>();

    let (player_x, player_y) = if let Some(player) = game.player_entity {
        if let Some(pos) = game.world.get_component::<Position>(player) {
            (pos.x, pos.y)
        } else {
            (50, 40)
        }
    } else {
        (50, 40)
    };

    Ok(GameSnapshot {
        state: format!("{:?}", game.state),
        day: time.map(|t| t.day).unwrap_or(1),
        hour: time.map(|t| t.hour).unwrap_or(6),
        minute: time.map(|t| t.minute).unwrap_or(0),
        season: time
            .map(|t| t.season_name().to_string())
            .unwrap_or_else(|| "Clarity".to_string()),
        weather: weather
            .map(|w| w.current.name().to_string())
            .unwrap_or_else(|| "Clear".to_string()),
        energy: 100,
        max_energy: 100,
        gold: 500,
        tick: game.tick_count,
        player_x,
        player_y,
    })
}

pub fn get_inventory_snapshot() -> Result<InventorySnapshot, String> {
    let state = GAME_STATE.lock().map_err(|e| e.to_string())?;
    let game = state.as_ref().ok_or("Game not initialized")?;

    let inv = game.world.get_resource::<Inventory>();

    let mut slots = Vec::new();
    if let Some(inventory) = inv {
        for (i, slot) in inventory.hotbar.iter().enumerate() {
            if let Some(_item_id) = slot.item_id {
                let name = format!("Hotbar Slot {}", i + 1);
                let color = match i {
                    0 => "#4FC3F7",
                    1 => "#FFD54F",
                    2 => "#AB47BC",
                    _ => "#78909C",
                };
                slots.push(InventorySlotSnapshot {
                    item_name: name,
                    quantity: slot.quantity,
                    icon_color: color.to_string(),
                });
            }
        }
        for (i, slot) in inventory.slots.iter().enumerate() {
            if let Some(_item_id) = slot.item_id {
                let name = format!("Inventory Slot {}", i + 1);
                slots.push(InventorySlotSnapshot {
                    item_name: name,
                    quantity: slot.quantity,
                    icon_color: "#78909C".to_string(),
                });
            }
        }
    }

    if slots.is_empty() {
        slots.push(InventorySlotSnapshot {
            item_name: "Thought Seed".to_string(),
            quantity: 5,
            icon_color: "#4FC3F7".to_string(),
        });
        slots.push(InventorySlotSnapshot {
            item_name: "Curiosity Seed".to_string(),
            quantity: 3,
            icon_color: "#FFD54F".to_string(),
        });
        slots.push(InventorySlotSnapshot {
            item_name: "Logic Seed".to_string(),
            quantity: 2,
            icon_color: "#AB47BC".to_string(),
        });
    }

    Ok(InventorySnapshot { slots })
}

pub fn get_farm_snapshot() -> Result<Vec<FarmPlotSnapshot>, String> {
    let state = GAME_STATE.lock().map_err(|e| e.to_string())?;
    let _game = state.as_ref().ok_or("Game not initialized")?;

    Ok(vec![
        FarmPlotSnapshot {
            x: 10,
            y: 10,
            tilled: true,
            watered: true,
            crop_name: "Basic Thought".to_string(),
            growth_stage: "Sprout".to_string(),
        },
        FarmPlotSnapshot {
            x: 11,
            y: 10,
            tilled: true,
            watered: false,
            crop_name: "Curiosity".to_string(),
            growth_stage: "Seed".to_string(),
        },
    ])
}

pub fn get_npc_snapshot() -> Result<Vec<NpcSnapshot>, String> {
    let _state = GAME_STATE.lock().map_err(|e| e.to_string())?;

    Ok(vec![
        NpcSnapshot {
            name: "Awareness".to_string(),
            x: 55,
            y: 32,
            resonance: 100,
            dialogue_hint: "The meadow remembers...".to_string(),
        },
        NpcSnapshot {
            name: "Focus".to_string(),
            x: 58,
            y: 30,
            resonance: 50,
            dialogue_hint: "Sharp mind cuts through...".to_string(),
        },
        NpcSnapshot {
            name: "Creativity".to_string(),
            x: 52,
            y: 35,
            resonance: 75,
            dialogue_hint: "Every pattern was once...".to_string(),
        },
        NpcSnapshot {
            name: "Empathy".to_string(),
            x: 60,
            y: 33,
            resonance: 120,
            dialogue_hint: "To understand another...".to_string(),
        },
    ])
}

pub fn game_action(action: String, params: Option<String>) -> Result<String, String> {
    let mut state = GAME_STATE.lock().map_err(|e| e.to_string())?;
    let game = state.as_mut().ok_or("Game not initialized")?;

    let game_action = match action.as_str() {
        "move" => {
            let p = params.unwrap_or_else(|| "0,0".to_string());
            let parts: Vec<i32> = p.split(',').filter_map(|s| s.parse().ok()).collect();
            let dx = parts.get(0).copied().unwrap_or(0);
            let dy = parts.get(1).copied().unwrap_or(0);
            GameAction::Move { dx, dy }
        }
        "use_tool" => {
            let tool = params
                .unwrap_or_else(|| "1".to_string())
                .parse()
                .unwrap_or(1u32);
            GameAction::UseTool(tool)
        }
        "interact" => GameAction::Interact,
        "toggle_inventory" => GameAction::ToggleInventory,
        "toggle_crafting" => GameAction::ToggleCrafting,
        "advance_dialogue" => GameAction::AdvanceDialogue,
        "sleep" => GameAction::Sleep,
        "save" => GameAction::Save,
        "load" => GameAction::Load,
        _ => return Err(format!("Unknown action: {}", action)),
    };

    game.handle_input(game_action);

    Ok(format!("Action '{}' executed", action))
}

pub fn game_tick(dt: f32) -> Result<String, String> {
    let mut state = GAME_STATE.lock().map_err(|e| e.to_string())?;
    let game = state.as_mut().ok_or("Game not initialized")?;

    game.update(dt);

    Ok(game.get_state_summary())
}

pub fn cleanup() {
    let mut state = GAME_STATE.lock().unwrap();
    *state = None;
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SaveData {
    pub snapshot: GameSnapshot,
    pub inventory: InventorySnapshot,
    pub farm: Vec<FarmPlotSnapshot>,
    pub npcs: Vec<NpcSnapshot>,
    pub tick: u64,
    pub timestamp: u64,
}

pub fn save_game_to_file(slot: u32) -> Result<String, String> {
    let snapshot = get_game_snapshot()?;
    let inventory = get_inventory_snapshot()?;
    let farm = get_farm_snapshot()?;
    let npcs = get_npc_snapshot()?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let state = GAME_STATE.lock().map_err(|e| e.to_string())?;
    let game = state.as_ref().ok_or("Game not initialized")?;

    let data = SaveData {
        snapshot,
        inventory,
        farm,
        npcs,
        tick: game.tick_count,
        timestamp,
    };

    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    let path = format!("save_slot_{}.json", slot);
    fs::write(&path, json).map_err(|e| e.to_string())?;

    Ok(format!("Saved to {}", path))
}

pub fn load_game_from_file(slot: u32) -> Result<String, String> {
    let path = format!("save_slot_{}.json", slot);
    let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let _data: SaveData = serde_json::from_str(&json).map_err(|e| e.to_string())?;

    // TODO: Restore game state from data
    Ok(format!("Loaded from {}", path))
}
