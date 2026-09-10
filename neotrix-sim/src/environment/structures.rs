use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StructureType {
    Shelter, Farm, Workshop, Watchtower, Market, Wall, Road,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructureEffect {
    RestBonus(f32),
    FoodGeneration(f32),
    Visibility(f32),
    MovementSpeed(f32),
    DangerReduction(f32),
    TradeRadius(f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Structure {
    pub id: u64,
    pub structure_type: StructureType,
    pub position: [f32; 2],
    pub builder_id: String,
    pub built_tick: u64,
    pub health: f32,
    pub max_health: f32,
    pub effects: Vec<StructureEffect>,
}

pub struct StructureManager {
    structures: Vec<Structure>,
    next_id: u64,
    grid: HashMap<(i32, i32), Vec<u64>>,
    cell_size: f32,
}

impl StructureManager {
    pub fn new(cell_size: f32) -> Self {
        Self { structures: Vec::new(), next_id: 0, grid: HashMap::new(), cell_size }
    }

    fn cell_key(&self, pos: [f32; 2]) -> (i32, i32) {
        ((pos[0] / self.cell_size).floor() as i32, (pos[1] / self.cell_size).floor() as i32)
    }

    fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
    }

    pub fn build(
        &mut self,
        structure_type: StructureType,
        position: [f32; 2],
        builder_id: &str,
        tick: u64,
    ) -> Result<u64, String> {
        // Prevent duplicate of same type within 30 units
        if self.structures.iter().any(|s| {
            s.structure_type == structure_type && Self::dist(s.position, position) < 30.0
        }) {
            return Err("similar structure too close".to_string());
        }

        let id = self.next_id;
        self.next_id += 1;
        let (max_hp, effects) = match structure_type {
            StructureType::Shelter => (100.0, vec![StructureEffect::RestBonus(1.5), StructureEffect::DangerReduction(0.3)]),
            StructureType::Farm => (60.0, vec![StructureEffect::FoodGeneration(2.0)]),
            StructureType::Workshop => (80.0, vec![]),
            StructureType::Watchtower => (50.0, vec![StructureEffect::Visibility(50.0)]),
            StructureType::Market => (70.0, vec![StructureEffect::TradeRadius(100.0)]),
            StructureType::Wall => (200.0, vec![StructureEffect::DangerReduction(0.5)]),
            StructureType::Road => (40.0, vec![StructureEffect::MovementSpeed(1.5)]),
        };

        let key = self.cell_key(position);
        let s = Structure {
            id, structure_type, position,
            builder_id: builder_id.to_string(),
            built_tick: tick, health: max_hp, max_health: max_hp, effects,
        };
        self.structures.push(s);
        self.grid.entry(key).or_default().push(id);
        Ok(id)
    }

    pub fn effects_at(&self, position: [f32; 2], radius: f32) -> Vec<&StructureEffect> {
        self.structures.iter()
            .filter(|s| Self::dist(s.position, position) <= radius)
            .flat_map(|s| s.effects.iter())
            .collect()
    }

    pub fn near(&self, position: [f32; 2], radius: f32) -> Vec<&Structure> {
        self.structures.iter().filter(|s| Self::dist(s.position, position) <= radius).collect()
    }

    pub fn damage(&mut self, id: u64, amount: f32) -> bool {
        if let Some(s) = self.structures.iter_mut().find(|s| s.id == id) {
            s.health = (s.health - amount).max(0.0);
            s.health <= 0.0
        } else {
            false
        }
    }

    pub fn rest_bonus_at(&self, position: [f32; 2]) -> f32 {
        self.structures.iter()
            .filter(|s| Self::dist(s.position, position) < 50.0)
            .flat_map(|s| s.effects.iter())
            .filter_map(|e| if let StructureEffect::RestBonus(v) = e { Some(*v) } else { None })
            .product()
    }

    pub fn food_generation_at(&self, position: [f32; 2]) -> f32 {
        self.structures.iter()
            .filter(|s| Self::dist(s.position, position) < 50.0)
            .flat_map(|s| s.effects.iter())
            .filter_map(|e| if let StructureEffect::FoodGeneration(v) = e { Some(*v) } else { None })
            .sum()
    }

    pub fn count_by_type(&self) -> HashMap<StructureType, usize> {
        let mut map = HashMap::new();
        for s in &self.structures {
            *map.entry(s.structure_type.clone()).or_insert(0) += 1;
        }
        map
    }

    pub fn total_structures(&self) -> usize { self.structures.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_creates_structure() {
        let mut sm = StructureManager::new(50.0);
        let id = sm.build(StructureType::Shelter, [0.0, 0.0], "alice", 0).unwrap();
        assert_eq!(id, 0);
        assert_eq!(sm.total_structures(), 1);
    }

    #[test]
    fn build_duplicate_prevented() {
        let mut sm = StructureManager::new(50.0);
        sm.build(StructureType::Shelter, [0.0, 0.0], "alice", 0).unwrap();
        let r = sm.build(StructureType::Shelter, [5.0, 0.0], "bob", 1);
        assert!(r.is_err());
    }

    #[test]
    fn effects_at_returns_effects() {
        let mut sm = StructureManager::new(50.0);
        sm.build(StructureType::Farm, [10.0, 10.0], "alice", 0).unwrap();
        let effects = sm.effects_at([12.0, 12.0], 50.0);
        assert!(!effects.is_empty());
    }

    #[test]
    fn near_returns_nearby() {
        let mut sm = StructureManager::new(50.0);
        sm.build(StructureType::Wall, [0.0, 0.0], "alice", 0).unwrap();
        sm.build(StructureType::Wall, [200.0, 200.0], "bob", 1).unwrap();
        assert_eq!(sm.near([5.0, 5.0], 50.0).len(), 1);
    }

    #[test]
    fn damage_reduces_health() {
        let mut sm = StructureManager::new(50.0);
        let id = sm.build(StructureType::Shelter, [0.0, 0.0], "alice", 0).unwrap();
        assert!(!sm.damage(id, 50.0));
        assert!(sm.damage(id, 60.0)); // destroyed
    }

    #[test]
    fn count_by_type_works() {
        let mut sm = StructureManager::new(50.0);
        sm.build(StructureType::Shelter, [0.0, 0.0], "a", 0).unwrap();
        sm.build(StructureType::Farm, [100.0, 0.0], "b", 1).unwrap();
        sm.build(StructureType::Farm, [200.0, 0.0], "c", 2).unwrap();
        let counts = sm.count_by_type();
        assert_eq!(counts[&StructureType::Shelter], 1);
        assert_eq!(counts[&StructureType::Farm], 2);
    }

    #[test]
    fn food_generation_at_works() {
        let mut sm = StructureManager::new(50.0);
        sm.build(StructureType::Farm, [10.0, 10.0], "alice", 0).unwrap();
        assert!(sm.food_generation_at([12.0, 12.0]) > 0.0);
        assert_eq!(sm.food_generation_at([500.0, 500.0]), 0.0);
    }
}
