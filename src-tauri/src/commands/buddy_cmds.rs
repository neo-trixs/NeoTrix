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

#[command]
pub fn buddy_pet(name: String) -> Result<String, String> {
    let mut lock = BUDDY.lock().map_err(|e| e.to_string())?;
    lock.state.energy = clamp_energy(lock.state.energy.saturating_add(ENERGY_PET));
    lock.state.mood = "happy".into();
    lock.state.last_interaction = Utc::now().timestamp();
    lock.total_pets += 1;
    push_action(&mut lock, "pet", &format!("Petted by {}", name));
    Ok(format!("{} purrs happily! Energy: {}", lock.state.name, lock.state.energy))
}

#[command]
pub fn buddy_feed(food: String) -> Result<String, String> {
    let mut lock = BUDDY.lock().map_err(|e| e.to_string())?;
    lock.total_feedings += 1;
    let known = KNOWN_SNACKS.contains(&food.as_str());
    let gain = if known { ENERGY_FEED_KNOWN } else { ENERGY_FEED_OTHER };
    lock.state.energy = clamp_energy(lock.state.energy.saturating_add(gain));
    lock.state.last_interaction = Utc::now().timestamp();
    push_action(&mut lock, "feed", &format!("Ate {} (known={})", food, known));
    if known {
        Ok(format!("{} devours the {}! +{} energy", lock.state.name, food, gain))
    } else {
        Ok(format!("{} nibbles the {}... +{} energy", lock.state.name, food, gain))
    }
}

#[command]
pub fn buddy_train(task: String) -> Result<String, String> {
    let mut lock = BUDDY.lock().map_err(|e| e.to_string())?;
    lock.state.xp += 10;
    lock.state.mood = "excited".into();
    lock.state.last_interaction = Utc::now().timestamp();
    let leveled = lock.state.xp >= lock.state.level as u64 * XP_PER_LEVEL;
    if leveled {
        lock.state.level += 1;
    }
    push_action(&mut lock, "train", &format!("Trained on: {}", task));
    if leveled {
        Ok(format!("{} mastered {}! Level up! Now level {}", lock.state.name, task, lock.state.level))
    } else {
        Ok(format!("{} learned from {}. +10 XP", lock.state.name, task))
    }
}

#[command]
pub fn buddy_rest() -> Result<String, String> {
    let mut lock = BUDDY.lock().map_err(|e| e.to_string())?;
    lock.state.energy = 100;
    lock.state.mood = "neutral".into();
    lock.state.last_interaction = Utc::now().timestamp();
    push_action(&mut lock, "rest", "Rested to full energy");
    Ok(format!("{} is fully recharged!", lock.state.name))
}

#[command]
pub fn buddy_achievements() -> Vec<Achievement> {
    BUDDY.lock().map(|l| l.achievements.clone()).unwrap_or_default()
}

fn check_auto_achievements(internal: &mut BuddyInternals) {
    let now = Utc::now().timestamp();
    if internal.total_pets >= 10 && !internal.achievements.iter().any(|a| a.id == "first_birthday") {
        internal.achievements.push(Achievement {
            id: "first_birthday".into(),
            name: "First Birthday".into(),
            description: "Petted 10 times".into(),
            unlocked_at: now,
        });
    }
    if internal.total_pets >= 50 && !internal.achievements.iter().any(|a| a.id == "faithful_companion") {
        internal.achievements.push(Achievement {
            id: "faithful_companion".into(),
            name: "Faithful Companion".into(),
            description: "Petted 50 times".into(),
            unlocked_at: now,
        });
    }
    if internal.state.level >= 3 && !internal.achievements.iter().any(|a| a.id == "code_master") {
        internal.achievements.push(Achievement {
            id: "code_master".into(),
            name: "Code Master".into(),
            description: "Reached level 3".into(),
            unlocked_at: now,
        });
    }
}

#[command]
pub fn buddy_idle_tick() -> Result<BuddyState, String> {
    let mut lock = BUDDY.lock().map_err(|e| e.to_string())?;
    lock.state.energy = clamp_energy(lock.state.energy.saturating_sub(ENERGY_DECAY_TICK));
    lock.state.mood = mood_decay(&lock.state.mood).into();
    check_auto_achievements(&mut lock);
    Ok(lock.state.clone())
}

#[command]
pub fn buddy_log(count: usize) -> Vec<BuddyAction> {
    BUDDY.lock().map(|l| {
        l.actions.iter().rev().take(count).cloned().collect()
    }).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buddy_full_lifecycle() {
        {
            let mut lock = BUDDY.lock().unwrap();
            lock.state = BuddyState {
                mood: "neutral".into(), energy: 100, xp: 0, level: 1,
                name: "NeoBuddy".into(), last_interaction: 0, active: true,
            };
            lock.total_pets = 0;
            lock.total_feedings = 0;
            lock.achievements.clear();
        }

        let initial_energy = {
            let lock = BUDDY.lock().unwrap();
            lock.state.energy
        };

        let _ = buddy_pet("tester".into());
        {
            let lock = BUDDY.lock().unwrap();
            assert!(lock.state.energy >= initial_energy, "pet should not decrease energy");
            assert_eq!(lock.state.mood, "happy");
        }

        {
            let mut lock = BUDDY.lock().unwrap();
            lock.state.energy = 50;
        }
        let before_feed = {
            let lock = BUDDY.lock().unwrap();
            lock.state.energy
        };
        buddy_feed("treat".into()).unwrap();
        {
            let lock = BUDDY.lock().unwrap();
            assert!(lock.state.energy > before_feed, "feed should increase energy");
        }

        {
            let mut lock = BUDDY.lock().unwrap();
            lock.state.energy = 90;
        }
        let before_tick = {
            let lock = BUDDY.lock().unwrap();
            lock.state.energy
        };
        buddy_idle_tick().unwrap();
        {
            let lock = BUDDY.lock().unwrap();
            assert!(lock.state.energy <= before_tick, "tick should not increase energy");
        }

        {
            let mut lock = BUDDY.lock().unwrap();
            lock.state.xp = 95;
            lock.state.level = 1;
        }
        buddy_train("test".into()).unwrap();
        {
            let lock = BUDDY.lock().unwrap();
            assert!(lock.state.level >= 2, "should level up after 100 XP");
            assert_eq!(lock.state.mood, "excited");
        }

        {
            let mut lock = BUDDY.lock().unwrap();
            lock.total_pets = 10;
            lock.state.level = 3;
        }
        buddy_idle_tick().unwrap();
        {
            let lock = BUDDY.lock().unwrap();
            assert!(lock.achievements.iter().any(|x| x.id == "first_birthday"), "first_birthday should unlock after 10 pets");
            assert!(lock.achievements.iter().any(|x| x.id == "code_master"), "code_master should unlock at level 3");
        }
    }
}
