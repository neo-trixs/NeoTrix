use tauri::Emitter;

#[tauri::command]
pub fn get_pet_state() -> serde_json::Value {
    serde_json::json!({
        "energy": 0.5,
        "mood": "neutral",
        "level": 1,
        "experience": 0,
        "valence": 0.5,
        "arousal": 0.5,
    })
}

#[tauri::command]
pub fn feed_pet_conversation(app: tauri::AppHandle, _text: String) {
    let _ = app.emit("pet:updated", serde_json::json!({
        "energy": 0.5,
        "mood": "neutral",
        "level": 1,
        "experience": 0,
        "valence": 0.5,
        "arousal": 0.5,
    }));
}

/// Sync pet state with consciousness metrics. Called periodically by the background loop.
#[tauri::command]
pub fn sync_pet_consciousness(app: tauri::AppHandle, _valence: f64, _arousal: f64, _curiosity: f64) {
    let _ = app.emit("pet:updated", serde_json::json!({
        "energy": 0.5,
        "mood": "neutral",
        "level": 1,
        "experience": 0,
        "valence": 0.5,
        "arousal": 0.5,
    }));
}
