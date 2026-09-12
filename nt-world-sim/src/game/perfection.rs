use crate::core::Resource;

#[derive(Debug, Clone)]
pub struct PerfectionCategory {
    pub name: String,
    pub current: u32,
    pub target: u32,
    pub weight: f32,
}

impl PerfectionCategory {
    pub fn new(name: &str, target: u32, weight: f32) -> Self {
        Self { name: name.to_string(), current: 0, target, weight }
    }

    pub fn progress(&self) -> f32 {
        if self.target == 0 { 1.0 }
        else { (self.current as f32 / self.target as f32).min(1.0) }
    }

    pub fn is_complete(&self) -> bool { self.current >= self.target }
}

#[derive(Resource)]
pub struct PerfectionTracker {
    pub categories: Vec<PerfectionCategory>,
    pub total_gold_earned: u32,
    pub total_crops_sold: u32,
    pub total_fish_caught: u32,
    pub total_minerals_mined: u32,
    pub total_crafts: u32,
    pub total_monsters_defeated: u32,
    pub npc_max_resonance: u32,
    pub skill_max_level: u32,
    pub golden_clock_built: bool,
    pub obelisk_water: bool,
    pub obelisk_earth: bool,
    pub obelisk_sky: bool,
    pub obelisk_desert: bool,
}

impl PerfectionTracker {
    pub fn new() -> Self {
        Self {
            categories: vec![
                PerfectionCategory::new("Shipping", 100, 0.15),
                PerfectionCategory::new("Farming", 100, 0.15),
                PerfectionCategory::new("Mining", 100, 0.10),
                PerfectionCategory::new("Fishing", 100, 0.10),
                PerfectionCategory::new("Foraging", 100, 0.10),
                PerfectionCategory::new("Crafting", 100, 0.10),
                PerfectionCategory::new("Cooking", 100, 0.10),
                PerfectionCategory::new("Community Center", 100, 0.10),
                PerfectionCategory::new("NPCs", 100, 0.10),
            ],
            total_gold_earned: 0,
            total_crops_sold: 0,
            total_fish_caught: 0,
            total_minerals_mined: 0,
            total_crafts: 0,
            total_monsters_defeated: 0,
            npc_max_resonance: 0,
            skill_max_level: 0,
            golden_clock_built: false,
            obelisk_water: false,
            obelisk_earth: false,
            obelisk_sky: false,
            obelisk_desert: false,
        }
    }

    pub fn update_category(&mut self, name: &str, value: u32) {
        if let Some(cat) = self.categories.iter_mut().find(|c| c.name == name) {
            cat.current = value;
        }
    }

    pub fn increment_category(&mut self, name: &str, amount: u32) {
        if let Some(cat) = self.categories.iter_mut().find(|c| c.name == name) {
            cat.current = (cat.current + amount).min(cat.target);
        }
    }

    pub fn overall_progress(&self) -> f32 {
        let total_weight: f32 = self.categories.iter().map(|c| c.weight).sum();
        let weighted_progress: f32 = self.categories.iter()
            .map(|c| c.progress() * c.weight)
            .sum();
        if total_weight > 0.0 { weighted_progress / total_weight } else { 0.0 }
    }

    pub fn is_perfect(&self) -> bool {
        self.categories.iter().all(|c| c.is_complete())
            && self.golden_clock_built
    }

    pub fn get_achievements(&self) -> Vec<String> {
        let mut achievements = Vec::new();
        if self.total_gold_earned >= 100000 { achievements.push("Millionaire".to_string()); }
        if self.total_crops_sold >= 500 { achievements.push("Master Farmer".to_string()); }
        if self.total_fish_caught >= 100 { achievements.push("Master Angler".to_string()); }
        if self.total_minerals_mined >= 200 { achievements.push("Master Miner".to_string()); }
        if self.total_monsters_defeated >= 100 { achievements.push("Monster Slayer".to_string()); }
        if self.is_perfect() { achievements.push("Perfection".to_string()); }
        achievements
    }

    pub fn summary(&self) -> String {
        format!(
            "Perfection: {:.1}% | Gold: {} | Crops: {} | Fish: {} | Minerals: {} | Monsters: {}",
            self.overall_progress() * 100.0,
            self.total_gold_earned,
            self.total_crops_sold,
            self.total_fish_caught,
            self.total_minerals_mined,
            self.total_monsters_defeated,
        )
    }
}

impl Default for PerfectionTracker {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_progress() {
        let cat = PerfectionCategory::new("Test", 100, 1.0);
        assert_eq!(cat.progress(), 0.0);
        assert!(!cat.is_complete());
    }

    #[test]
    fn test_category_progress_partial() {
        let mut cat = PerfectionCategory::new("Test", 100, 1.0);
        cat.current = 50;
        assert!((cat.progress() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_category_complete() {
        let mut cat = PerfectionCategory::new("Test", 100, 1.0);
        cat.current = 100;
        assert!(cat.is_complete());
        assert_eq!(cat.progress(), 1.0);
    }

    #[test]
    fn test_category_overcomplete() {
        let mut cat = PerfectionCategory::new("Test", 100, 1.0);
        cat.current = 200;
        assert!(cat.is_complete());
        assert_eq!(cat.progress(), 1.0);
    }

    #[test]
    fn test_category_zero_target() {
        let cat = PerfectionCategory::new("Test", 0, 1.0);
        assert_eq!(cat.progress(), 1.0);
        assert!(cat.is_complete());
    }

    #[test]
    fn test_tracker_new() {
        let tracker = PerfectionTracker::new();
        assert_eq!(tracker.categories.len(), 9);
        assert_eq!(tracker.total_gold_earned, 0);
        assert!(!tracker.golden_clock_built);
        assert_eq!(tracker.overall_progress(), 0.0);
    }

    #[test]
    fn test_update_category() {
        let mut tracker = PerfectionTracker::new();
        tracker.update_category("Shipping", 50);
        assert_eq!(tracker.categories[0].current, 50);
    }

    #[test]
    fn test_update_category_unknown() {
        let mut tracker = PerfectionTracker::new();
        tracker.update_category("Nonexistent", 50);
        assert_eq!(tracker.categories[0].current, 0);
    }

    #[test]
    fn test_increment_category() {
        let mut tracker = PerfectionTracker::new();
        tracker.increment_category("Mining", 30);
        assert_eq!(tracker.categories[2].current, 30);
        tracker.increment_category("Mining", 20);
        assert_eq!(tracker.categories[2].current, 50);
    }

    #[test]
    fn test_increment_capped_at_target() {
        let mut tracker = PerfectionTracker::new();
        tracker.increment_category("Mining", 80);
        tracker.increment_category("Mining", 50);
        assert_eq!(tracker.categories[2].current, 100);
    }

    #[test]
    fn test_overall_progress() {
        let mut tracker = PerfectionTracker::new();
        tracker.update_category("Shipping", 100);
        tracker.update_category("Farming", 100);
        let progress = tracker.overall_progress();
        assert!(progress > 0.0);
        assert!(progress < 1.0);
    }

    #[test]
    fn test_perfect_requires_all_categories() {
        let mut tracker = PerfectionTracker::new();
        for cat in &mut tracker.categories {
            cat.current = cat.target;
        }
        assert!(!tracker.is_perfect());
    }

    #[test]
    fn test_perfect_requires_golden_clock() {
        let mut tracker = PerfectionTracker::new();
        for cat in &mut tracker.categories {
            cat.current = cat.target;
        }
        tracker.golden_clock_built = true;
        assert!(tracker.is_perfect());
    }

    #[test]
    fn test_achievements_empty() {
        let tracker = PerfectionTracker::new();
        assert!(tracker.get_achievements().is_empty());
    }

    #[test]
    fn test_achievements_millionaire() {
        let mut tracker = PerfectionTracker::new();
        tracker.total_gold_earned = 100000;
        let achievements = tracker.get_achievements();
        assert!(achievements.contains(&"Millionaire".to_string()));
    }

    #[test]
    fn test_achievements_master_farmer() {
        let mut tracker = PerfectionTracker::new();
        tracker.total_crops_sold = 500;
        let achievements = tracker.get_achievements();
        assert!(achievements.contains(&"Master Farmer".to_string()));
    }

    #[test]
    fn test_achievements_master_angler() {
        let mut tracker = PerfectionTracker::new();
        tracker.total_fish_caught = 100;
        let achievements = tracker.get_achievements();
        assert!(achievements.contains(&"Master Angler".to_string()));
    }

    #[test]
    fn test_achievements_master_miner() {
        let mut tracker = PerfectionTracker::new();
        tracker.total_minerals_mined = 200;
        let achievements = tracker.get_achievements();
        assert!(achievements.contains(&"Master Miner".to_string()));
    }

    #[test]
    fn test_achievements_monster_slayer() {
        let mut tracker = PerfectionTracker::new();
        tracker.total_monsters_defeated = 100;
        let achievements = tracker.get_achievements();
        assert!(achievements.contains(&"Monster Slayer".to_string()));
    }

    #[test]
    fn test_achievements_perfection() {
        let mut tracker = PerfectionTracker::new();
        for cat in &mut tracker.categories {
            cat.current = cat.target;
        }
        tracker.golden_clock_built = true;
        let achievements = tracker.get_achievements();
        assert!(achievements.contains(&"Perfection".to_string()));
    }

    #[test]
    fn test_summary() {
        let tracker = PerfectionTracker::new();
        let s = tracker.summary();
        assert!(s.contains("Perfection:"));
        assert!(s.contains("Gold: 0"));
    }

    #[test]
    fn test_default() {
        let tracker = PerfectionTracker::default();
        assert_eq!(tracker.categories.len(), 9);
    }
}
