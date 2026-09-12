#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .setup(|_app| {
            // Initialize game state
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            init_game,
            get_game_snapshot,
            get_inventory_snapshot,
            get_farm_snapshot,
            get_npc_snapshot,
            game_action,
            game_tick,
            save_game,
            load_game,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn init_game() -> Result<String, String> {
    nt_world_sim::tauri_bridge::init_game()
}

#[tauri::command]
fn get_game_snapshot() -> Result<nt_world_sim::tauri_bridge::GameSnapshot, String> {
    nt_world_sim::tauri_bridge::get_game_snapshot()
}

#[tauri::command]
fn get_inventory_snapshot() -> Result<nt_world_sim::tauri_bridge::InventorySnapshot, String> {
    nt_world_sim::tauri_bridge::get_inventory_snapshot()
}

#[tauri::command]
fn get_farm_snapshot() -> Result<Vec<nt_world_sim::tauri_bridge::FarmPlotSnapshot>, String> {
    nt_world_sim::tauri_bridge::get_farm_snapshot()
}

#[tauri::command]
fn get_npc_snapshot() -> Result<Vec<nt_world_sim::tauri_bridge::NpcSnapshot>, String> {
    nt_world_sim::tauri_bridge::get_npc_snapshot()
}

#[tauri::command]
fn game_action(action: String, params: Option<String>) -> Result<String, String> {
    nt_world_sim::tauri_bridge::game_action(action, params)
}

#[tauri::command]
fn game_tick(dt: f32) -> Result<String, String> {
    nt_world_sim::tauri_bridge::game_tick(dt)
}

#[tauri::command]
fn save_game(slot: u32) -> Result<String, String> {
    nt_world_sim::tauri_bridge::save_game_to_file(slot)
}

#[tauri::command]
fn load_game(slot: u32) -> Result<String, String> {
    nt_world_sim::tauri_bridge::load_game_from_file(slot)
}
