use crate::core::Resource;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ForageItem {
    pub id: u32,
    pub name: String,
    pub spawn_rate: f32,
    pub seasons: Vec<super::time::Season>,
    pub zones: Vec<String>,
    pub value: u32,
    pub xp: u32,
}

impl ForageItem {
    pub fn all() -> Vec<Self> {
        use super::time::Season::*;
        vec![
            Self { id: 9001, name: "Wild Thought".to_string(), spawn_rate: 0.3, seasons: vec![Clarity, Flow], zones: vec!["Farm".to_string(), "Forest".to_string()], value: 20, xp: 5 },
            Self { id: 9002, name: "Forest Memory".to_string(), spawn_rate: 0.2, seasons: vec![Reflection], zones: vec!["Forest".to_string()], value: 30, xp: 8 },
            Self { id: 9003, name: "Mountain Root".to_string(), spawn_rate: 0.15, seasons: vec![Flow, Stillness], zones: vec!["Mine".to_string(), "Mountain".to_string()], value: 40, xp: 10 },
            Self { id: 9004, name: "Lakeside Flower".to_string(), spawn_rate: 0.25, seasons: vec![Clarity], zones: vec!["Lake".to_string()], value: 25, xp: 6 },
            Self { id: 9005, name: "Crystal Fragment".to_string(), spawn_rate: 0.1, seasons: vec![Clarity, Flow, Reflection, Stillness], zones: vec!["Mine".to_string()], value: 50, xp: 12 },
            Self { id: 9006, name: "Ancient Fossil".to_string(), spawn_rate: 0.05, seasons: vec![Stillness], zones: vec!["Mine".to_string(), "Mountain".to_string()], value: 100, xp: 20 },
        ]
    }

    pub fn available_in_season(&self, season: &super::time::Season) -> bool {
        self.seasons.contains(season)
    }
}

#[derive(Debug, Clone)]
pub struct ForagingSystem {
    pub items_collected: HashMap<u32, u32>,
    pub total_collected: u32,
    pub spawn_points: Vec<(u32, u32, u32)>,
}

impl ForagingSystem {
    pub fn new() -> Self {
        Self { items_collected: HashMap::new(), total_collected: 0, spawn_points: Vec::new() }
    }

    pub fn spawn_items(&mut self, width: u32, height: u32, season: super::time::Season) {
        self.spawn_points.clear();
        let items = ForageItem::all();
        let mut idx = 0u32;
        for item in &items {
            if item.seasons.contains(&season) {
                let count = (width as f32 * height as f32 * item.spawn_rate / 100.0) as u32;
                for _ in 0..count {
                    let x = pseudo_random(self.total_collected + idx * 1000) % width;
                    let y = pseudo_random(self.total_collected + idx * 2000) % height;
                    self.spawn_points.push((x, y, item.id));
                    idx += 1;
                }
            }
        }
    }

    pub fn collect(&mut self, x: u32, y: u32) -> Option<ForageItem> {
        if let Some(idx) = self.spawn_points.iter().position(|(sx, sy, _)| *sx == x && *sy == y) {
            let (_, _, item_id) = self.spawn_points.remove(idx);
            *self.items_collected.entry(item_id).or_insert(0) += 1;
            self.total_collected += 1;
            ForageItem::all().into_iter().find(|i| i.id == item_id)
        } else { None }
    }

    pub fn advance_day(&mut self, season: super::time::Season) {
        self.spawn_points.clear();
        self.spawn_items(100, 80, season);
    }
}

fn pseudo_random(seed: u32) -> u32 {
    let mut x = seed;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}

impl Default for ForagingSystem {
    fn default() -> Self { Self::new() }
}

impl Resource for ForagingSystem {}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::time::Season;

    #[test]
    fn test_forage_items_database() {
        let items = ForageItem::all();
        assert_eq!(items.len(), 6);
    }

    #[test]
    fn test_forage_availability_by_season() {
        let items = ForageItem::all();
        let clarity_items: Vec<_> = items.iter().filter(|i| i.available_in_season(&Season::Clarity)).collect();
        assert_eq!(clarity_items.len(), 3);
    }

    #[test]
    fn test_foraging_system_spawn() {
        let mut system = ForagingSystem::new();
        system.spawn_items(100, 80, Season::Clarity);
        assert!(!system.spawn_points.is_empty());
    }

    #[test]
    fn test_foraging_collect() {
        let mut system = ForagingSystem::new();
        system.spawn_items(100, 80, Season::Clarity);
        if let Some((x, y, _)) = system.spawn_points.first().cloned() {
            let item = system.collect(x, y);
            assert!(item.is_some());
            assert_eq!(system.total_collected, 1);
        }
    }

    #[test]
    fn test_foraging_collect_nothing() {
        let mut system = ForagingSystem::new();
        system.spawn_items(100, 80, Season::Clarity);
        let item = system.collect(9999, 9999);
        assert!(item.is_none());
    }

    #[test]
    fn test_advance_day_respawns() {
        let mut system = ForagingSystem::new();
        system.spawn_items(100, 80, Season::Clarity);
        let _initial = system.spawn_points.len();
        system.advance_day(Season::Flow);
        assert!(system.spawn_points.len() > 0);
    }
}
