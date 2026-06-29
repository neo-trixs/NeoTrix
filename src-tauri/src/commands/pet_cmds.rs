use neotrix::neotrix::nt_world_pet::{PetEngine, PetState};
use std::sync::Mutex;
use tauri::{Emitter, State};

#[tauri::command]
pub fn get_pet_state(engine: State<'_, Mutex<PetEngine>>) -> PetState {
    engine.lock().expect("PetEngine mutex poisoned").state_snapshot().clone()
}

#[tauri::command]
pub fn feed_pet_conversation(app: tauri::AppHandle, engine: State<'_, Mutex<PetEngine>>, text: String) {
    let mut pet = engine.lock().expect("PetEngine mutex poisoned");
    pet.process_conversation(&text);
    let state = pet.state_snapshot().clone();
    let _ = app.emit("pet:updated", state);
}

/// Initialize pet engine with a snapshot of current KB nodes.
pub fn init_pet_engine() -> PetEngine {
    let kb = neotrix::neotrix::nt_memory_kb::KnowledgeBase::open(None)
        .map_err(|e| log::warn!("init_pet_engine: cannot open KB: {}", e))
        .ok();
    let node_count = kb.as_ref().and_then(|k| k.stats().ok()).map(|s| s.total_nodes as usize).unwrap_or(0);
    PetEngine::with_kb_snapshot(node_count)
}

/// Sync pet state with consciousness metrics. Called periodically by the background loop.
#[tauri::command]
pub fn sync_pet_consciousness(app: tauri::AppHandle, engine: State<'_, Mutex<PetEngine>>, valence: f64, arousal: f64, curiosity: f64) {
    let mut pet = engine.lock().expect("PetEngine mutex poisoned");
    let total_nodes = neotrix::neotrix::nt_memory_kb::KnowledgeBase::open(None)
        .map_err(|e| log::warn!("sync_pet_consciousness: cannot open KB: {}", e))
        .ok()
        .and_then(|k| k.stats().ok())
        .map(|s| s.total_nodes as usize)
        .unwrap_or(0);
    pet.set_valence(valence, arousal);
    pet.set_energy(curiosity);
    pet.check_growth(total_nodes);
    pet.tick();
    let state = pet.state_snapshot().clone();
    let _ = app.emit("pet:updated", state);
}
