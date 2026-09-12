use crate::core::Resource;

#[derive(Debug, Clone)]
pub struct Fish {
    pub id: u32,
    pub name: String,
    pub difficulty: u32,
    pub value: u32,
    pub xp: u32,
    pub seasons: Vec<super::time::Season>,
    pub times: Vec<u32>,
    pub min_depth: u32,
}

impl Fish {
    pub fn all() -> Vec<Self> {
        use super::time::Season::*;
        vec![
            Self { id: 8001, name: "Basic Thought Fish".to_string(), difficulty: 10, value: 30, xp: 5, seasons: vec![Clarity, Flow], times: (6..=18).collect(), min_depth: 1 },
            Self { id: 8002, name: "Curiosity Carp".to_string(), difficulty: 20, value: 50, xp: 8, seasons: vec![Clarity], times: vec![6,7,8,18,19,20], min_depth: 1 },
            Self { id: 8003, name: "Logic Bass".to_string(), difficulty: 30, value: 70, xp: 12, seasons: vec![Flow, Reflection], times: vec![12,13,14,15], min_depth: 3 },
            Self { id: 8004, name: "Empathy Trout".to_string(), difficulty: 25, value: 60, xp: 10, seasons: vec![Clarity, Flow, Reflection, Stillness], times: (6..=18).collect(), min_depth: 2 },
            Self { id: 8005, name: "Creativity Salmon".to_string(), difficulty: 35, value: 80, xp: 15, seasons: vec![Reflection], times: vec![18,19,20,21], min_depth: 4 },
            Self { id: 8006, name: "Memory Eel".to_string(), difficulty: 40, value: 100, xp: 18, seasons: vec![Stillness], times: vec![20,21,22,23], min_depth: 5 },
            Self { id: 8007, name: "Focus Pike".to_string(), difficulty: 45, value: 120, xp: 20, seasons: vec![Flow], times: vec![10,11,12,13,14,15], min_depth: 6 },
            Self { id: 8008, name: "Wisdom Whale".to_string(), difficulty: 60, value: 200, xp: 30, seasons: vec![Clarity, Flow, Reflection, Stillness], times: vec![0,1,2,3,4,5,6,22,23], min_depth: 8 },
            Self { id: 8009, name: "Void Bass".to_string(), difficulty: 50, value: 150, xp: 25, seasons: vec![Stillness], times: vec![21,22,23], min_depth: 7 },
            Self { id: 8010, name: "Clarity Perch".to_string(), difficulty: 15, value: 40, xp: 6, seasons: vec![Clarity], times: (6..=11).collect(), min_depth: 1 },
        ]
    }

    pub fn available_at(&self, season: &super::time::Season, hour: u32) -> bool {
        self.seasons.contains(season) && self.times.contains(&hour)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FishQuality { Normal, Silver, Gold, Iridium }

impl FishQuality {
    pub fn multiplier(&self) -> f32 {
        match self {
            Self::Normal => 1.0, Self::Silver => 1.5, Self::Gold => 2.0, Self::Iridium => 3.0,
        }
    }
}

#[derive(Resource)]
pub struct FishingSystem {
    pub fish_caught: Vec<(Fish, FishQuality)>,
    pub total_caught: u32,
    pub best_catch: Option<Fish>,
    pub minigame_active: bool,
    pub fish_position: f32,
    pub bar_position: f32,
    pub catch_progress: f32,
}

impl FishingSystem {
    pub fn new() -> Self {
        Self { fish_caught: Vec::new(), total_caught: 0, best_catch: None, minigame_active: false, fish_position: 0.5, bar_position: 0.5, catch_progress: 0.0 }
    }

    pub fn start_minigame(&mut self) -> bool {
        if !self.minigame_active {
            self.minigame_active = true;
            self.fish_position = rand_f32();
            self.bar_position = 0.5;
            self.catch_progress = 0.0;
            true
        } else { false }
    }

    pub fn update_minigame(&mut self, dt: f32, skill_level: u32) -> Option<(Fish, FishQuality)> {
        if !self.minigame_active { return None; }

        self.fish_position += (rand_f32() - 0.5) * dt * 2.0;
        self.fish_position = self.fish_position.clamp(0.0, 1.0);

        let overlap = (self.bar_position - self.fish_position).abs() < 0.15;
        if overlap {
            self.catch_progress += dt * (0.5 + skill_level as f32 * 0.05);
        } else {
            self.catch_progress -= dt * 0.3;
            self.catch_progress = self.catch_progress.max(0.0);
        }

        if self.catch_progress >= 1.0 {
            self.minigame_active = false;
            let fish = Fish::all();
            let caught = fish[rand_f32() as usize % fish.len()].clone();
            let quality = if skill_level >= 8 { FishQuality::Gold }
                else if skill_level >= 5 { FishQuality::Silver }
                else { FishQuality::Normal };
            self.fish_caught.push((caught.clone(), quality));
            self.total_caught += 1;
            if self.best_catch.as_ref().map(|b| b.value).unwrap_or(0) < caught.value {
                self.best_catch = Some(caught.clone());
            }
            return Some((caught, quality));
        }

        None
    }

    pub fn cancel_minigame(&mut self) {
        self.minigame_active = false;
        self.catch_progress = 0.0;
    }
}

fn rand_f32() -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().subsec_nanos();
    (nanos % 10000) as f32 / 10000.0
}

impl Default for FishingSystem {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::time::Season;

    #[test]
    fn test_fish_database() {
        let fish = Fish::all();
        assert_eq!(fish.len(), 10);
    }

    #[test]
    fn test_fish_availability() {
        let fish = Fish::all();
        let clarity_fish: Vec<_> = fish.iter().filter(|f| f.available_at(&Season::Clarity, 8)).collect();
        assert!(!clarity_fish.is_empty());
    }

    #[test]
    fn test_fishing_system_start() {
        let mut system = FishingSystem::new();
        assert!(system.start_minigame());
        assert!(system.minigame_active);
        assert!(!system.start_minigame());
    }

    #[test]
    fn test_fishing_system_cancel() {
        let mut system = FishingSystem::new();
        system.start_minigame();
        system.cancel_minigame();
        assert!(!system.minigame_active);
    }

    #[test]
    fn test_fish_quality_multiplier() {
        assert_eq!(FishQuality::Normal.multiplier(), 1.0);
        assert_eq!(FishQuality::Silver.multiplier(), 1.5);
        assert_eq!(FishQuality::Gold.multiplier(), 2.0);
        assert_eq!(FishQuality::Iridium.multiplier(), 3.0);
    }

    #[test]
    fn test_update_without_minigame() {
        let mut system = FishingSystem::new();
        assert!(system.update_minigame(0.1, 1).is_none());
    }
}
