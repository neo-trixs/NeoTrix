use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A structure built by agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Structure {
    pub id: u64,
    pub structure_type: StructureType,
    pub position: [f32; 2],
    pub builder_id: String,
    pub built_tick: u64,
    pub health: f32,
    pub max_health: f32,
    /// What this structure provides
    pub effects: Vec<StructureEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StructureType {
    Shelter,
    Farm,
    Workshop,
    Watchtower,
    Market,
    Wall,
    Road,
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

/// Manages all structures in the world
pub struct StructureManager {
    structures: Vec<Structure>,
    next_id: u64,
    /// Structures indexed by grid cell for spatial queries
    grid: HashMap<(i32, i32), Vec<u64>>,
    cell_size: f32,
}

impl StructureManager {
    pub fn new(cell_size: f32) -> Self {
        Self {
            structures: Vec::new(),
            next_id: 1,
            grid: HashMap::new(),
            cell_size,
        }
    }

    /// Build a new structure
    pub fn build(
        &mut self,
        structure_type: StructureType,
        position: [f32; 2],
        builder_id: &str,
        tick: u64,
    ) -> Result<u64, String> {
        // Check if same structure type already exists within 30 units (prevent duplicates)
        let min_distance_sq = 30.0 * 30.0;
        for s in &self.structures {
            if s.structure_type == structure_type {
                let dx = s.position[0] - position[0];
                let dy = s.position[1] - position[1];
                if dx * dx + dy * dy < min_distance_sq {
                    return Err(format!(
                        "{:?} already exists nearby at ({:.1}, {:.1})",
                        structure_type, s.position[0], s.position[1]
                    ));
                }
            }
        }

        let id = self.next_id;
        self.next_id += 1;

        let effects = default_effects(&structure_type);
        let (max_health, health) = match structure_type {
            StructureType::Wall => (150.0, 150.0),
            StructureType::Watchtower => (80.0, 80.0),
            _ => (100.0, 100.0),
        };

        let structure = Structure {
            id,
            structure_type,
            position,
            builder_id: builder_id.to_string(),
            built_tick: tick,
            health,
            max_health,
            effects,
        };

        // Insert into grid
        let cell = self.cell_key(position);
        self.grid.entry(cell).or_default().push(id);

        self.structures.push(structure);
        Ok(id)
    }

    /// Get effects at a position (all structures within radius)
    pub fn effects_at(&self, position: [f32; 2], radius: f32) -> Vec<&StructureEffect> {
        let mut result = Vec::new();
        let radius_sq = radius * radius;
        for s in &self.structures {
            let dx = s.position[0] - position[0];
            let dy = s.position[1] - position[1];
            if dx * dx + dy * dy <= radius_sq {
                result.extend(&s.effects);
            }
        }
        result
    }

    /// Get structures near a position
    pub fn near(&self, position: [f32; 2], radius: f32) -> Vec<&Structure> {
        let radius_sq = radius * radius;
        self.structures
            .iter()
            .filter(|s| {
                let dx = s.position[0] - position[0];
                let dy = s.position[1] - position[1];
                dx * dx + dy * dy <= radius_sq
            })
            .collect()
    }

    /// Damage a structure. Returns true if structure was destroyed.
    pub fn damage(&mut self, id: u64, amount: f32) -> bool {
        let destroyed = if let Some(s) = self.structures.iter_mut().find(|s| s.id == id) {
            s.health = (s.health - amount).max(0.0);
            s.health <= 0.0
        } else {
            false
        };
        if destroyed {
            let pos = self.structures.iter().find(|s| s.id == id).map(|s| s.position);
            if let Some(pos) = pos {
                let cell = self.cell_key(pos);
                if let Some(ids) = self.grid.get_mut(&cell) {
                    ids.retain(|&x| x != id);
                }
            }
            return true;
        }
        false
    }

    /// Get total rest bonus at position
    pub fn rest_bonus_at(&self, position: [f32; 2]) -> f32 {
        let mut total = 1.0;
        for s in &self.structures {
            let dx = s.position[0] - position[0];
            let dy = s.position[1] - position[1];
            if dx * dx + dy * dy <= 50.0 * 50.0 {
                for effect in &s.effects {
                    if let StructureEffect::RestBonus(bonus) = effect {
                        total *= bonus;
                    }
                }
            }
        }
        total
    }

    /// Get total food generation at position
    pub fn food_generation_at(&self, position: [f32; 2]) -> f32 {
        let mut total = 0.0;
        for s in &self.structures {
            let dx = s.position[0] - position[0];
            let dy = s.position[1] - position[1];
            if dx * dx + dy * dy <= 50.0 * 50.0 {
                for effect in &s.effects {
                    if let StructureEffect::FoodGeneration(gen) = effect {
                        total += gen;
                    }
                }
            }
        }
        total
    }

    /// Get structure count by type
    pub fn count_by_type(&self) -> HashMap<StructureType, usize> {
        let mut counts = HashMap::new();
        for s in &self.structures {
            *counts.entry(s.structure_type.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn total_structures(&self) -> usize {
        self.structures.len()
    }

    fn cell_key(&self, position: [f32; 2]) -> (i32, i32) {
        (
            (position[0] / self.cell_size).floor() as i32,
            (position[1] / self.cell_size).floor() as i32,
        )
    }
}

/// Get default effects for a structure type
pub fn default_effects(structure_type: &StructureType) -> Vec<StructureEffect> {
    match structure_type {
        StructureType::Shelter => vec![
            StructureEffect::RestBonus(1.5),
            StructureEffect::DangerReduction(0.3),
        ],
        StructureType::Farm => vec![StructureEffect::FoodGeneration(2.0)],
        StructureType::Workshop => vec![],
        StructureType::Watchtower => vec![StructureEffect::Visibility(50.0)],
        StructureType::Market => vec![StructureEffect::TradeRadius(100.0)],
        StructureType::Wall => vec![StructureEffect::DangerReduction(0.5)],
        StructureType::Road => vec![StructureEffect::MovementSpeed(1.5)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_creates_structure() {
        let mut mgr = StructureManager::new(50.0);
        let id = mgr
            .build(StructureType::Shelter, [10.0, 20.0], "agent_0", 100)
            .unwrap();
        assert_eq!(id, 1);
        assert_eq!(mgr.total_structures(), 1);
    }

    #[test]
    fn test_effects_at_returns_structure_effects() {
        let mut mgr = StructureManager::new(50.0);
        mgr.build(StructureType::Farm, [10.0, 10.0], "agent_0", 1)
            .unwrap();
        let effects = mgr.effects_at([10.0, 10.0], 50.0);
        assert!(!effects.is_empty());
        assert!(effects
            .iter()
            .any(|e| matches!(e, StructureEffect::FoodGeneration(_))));
    }

    #[test]
    fn test_near_returns_nearby_structures() {
        let mut mgr = StructureManager::new(50.0);
        mgr.build(StructureType::Wall, [10.0, 10.0], "a", 1)
            .unwrap();
        mgr.build(StructureType::Wall, [500.0, 500.0], "b", 1)
            .unwrap();
        let nearby = mgr.near([12.0, 12.0], 20.0);
        assert_eq!(nearby.len(), 1);
    }

    #[test]
    fn test_damage_reduces_health() {
        let mut mgr = StructureManager::new(50.0);
        let id = mgr
            .build(StructureType::Shelter, [10.0, 10.0], "a", 1)
            .unwrap();
        let destroyed = mgr.damage(id, 30.0);
        assert!(!destroyed);
        assert_eq!(mgr.near([10.0, 10.0], 1.0)[0].health, 70.0);
    }

    #[test]
    fn test_damage_destroys_at_zero() {
        let mut mgr = StructureManager::new(50.0);
        let id = mgr
            .build(StructureType::Shelter, [10.0, 10.0], "a", 1)
            .unwrap();
        let destroyed = mgr.damage(id, 200.0);
        assert!(destroyed);
        assert_eq!(mgr.total_structures(), 0);
    }

    #[test]
    fn test_count_by_type() {
        let mut mgr = StructureManager::new(50.0);
        mgr.build(StructureType::Shelter, [0.0, 0.0], "a", 1)
            .unwrap();
        mgr.build(StructureType::Shelter, [100.0, 0.0], "a", 1)
            .unwrap();
        mgr.build(StructureType::Farm, [50.0, 0.0], "b", 1)
            .unwrap();
        let counts = mgr.count_by_type();
        assert_eq!(counts[&StructureType::Shelter], 2);
        assert_eq!(counts[&StructureType::Farm], 1);
    }

    #[test]
    fn test_duplicate_prevention() {
        let mut mgr = StructureManager::new(50.0);
        mgr.build(StructureType::Shelter, [10.0, 10.0], "a", 1)
            .unwrap();
        let result = mgr.build(StructureType::Shelter, [15.0, 15.0], "b", 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_types_can_be_near() {
        let mut mgr = StructureManager::new(50.0);
        mgr.build(StructureType::Shelter, [10.0, 10.0], "a", 1)
            .unwrap();
        let result = mgr.build(StructureType::Farm, [12.0, 12.0], "b", 2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_effects_cover_all_types() {
        for t in [
            StructureType::Shelter,
            StructureType::Farm,
            StructureType::Workshop,
            StructureType::Watchtower,
            StructureType::Market,
            StructureType::Wall,
            StructureType::Road,
        ] {
            let _ = default_effects(&t);
        }
    }
}
