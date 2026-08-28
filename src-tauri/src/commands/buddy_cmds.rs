use std::sync::Mutex;
use serde::Serialize;
use tauri::command;
use chrono::Utc;

#[derive(Debug, Clone, Serialize)]
pub struct BuddyState {
    pub mood: String,
    pub energy: u32,
    pub xp: u64,
    pub level: u32,
    pub name: String,
    pub last_interaction: i64,
    pub active: bool,
}

struct BuddyInternals {
    state: BuddyState,
}

static BUDDY: std::sync::LazyLock<Mutex<BuddyInternals>> = std::sync::LazyLock::new(|| {
    let now = Utc::now().timestamp();
    Mutex::new(BuddyInternals {
        state: BuddyState {
            mood: "neutral".into(),
            energy: 100,
            xp: 0,
            level: 1,
            name: "NeoBuddy".into(),
            last_interaction: now,
            active: true,
        },
    })
});

#[command]
pub fn buddy_status() -> Result<BuddyState, String> {
    let lock = BUDDY.lock().map_err(|e| e.to_string())?;
    Ok(lock.state.clone())
}
