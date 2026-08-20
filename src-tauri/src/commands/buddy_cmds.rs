use std::collections::VecDeque;
use std::sync::Mutex;
use serde::Serialize;
use tauri::command;
use chrono::Utc;

const KNOWN_SNACKS: &[&str] = &["treat", "data", "code", "feedback"];
const MAX_ACTIONS: usize = 50;
const XP_PER_LEVEL: u64 = 100;
const ENERGY_PET: u32 = 5;
const ENERGY_FEED_KNOWN: u32 = 20;
const ENERGY_FEED_OTHER: u32 = 5;
const ENERGY_DECAY_TICK: u32 = 2;

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

#[derive(Debug, Clone, Serialize)]
pub struct BuddyAction {
    pub action: String,
    pub timestamp: i64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub unlocked_at: i64,
}

struct BuddyInternals {
    state: BuddyState,
    actions: VecDeque<BuddyAction>,
    achievements: Vec<Achievement>,
    total_pets: u64,
    total_feedings: u64,
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
        actions: VecDeque::with_capacity(MAX_ACTIONS),
        achievements: Vec::new(),
        total_pets: 0,
        total_feedings: 0,
    })
});

fn push_action(internal: &mut BuddyInternals, action: &str, desc: &str) {
    if internal.actions.len() >= MAX_ACTIONS {
        internal.actions.pop_front();
    }
    internal.actions.push_back(BuddyAction {
        action: action.into(),
        timestamp: Utc::now().timestamp(),
        description: desc.into(),
    });
}

fn mood_decay(mood: &str) -> &str {
    match mood {
        "happy" | "excited" => "curious",
        "curious" | "tired" => "neutral",
        _ => "neutral",
    }
}

fn clamp_energy(v: u32) -> u32 {
    v.min(100)
}

#[command]
pub fn buddy_status() -> Result<BuddyState, String> {
    let lock = BUDDY.lock().map_err(|e| e.to_string())?;
    Ok(lock.state.clone())
}
