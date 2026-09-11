use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FactionId(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    pub id: FactionId,
    pub name: String,
    pub ideology: String,
    pub territory_center: (f32, f32),
    pub territory_radius: f32,
    pub member_ids: Vec<u32>,
    pub reputation: HashMap<FactionId, f32>,
    pub resources: f32,
    pub aggression: f32,
    pub cooperation: f32,
    pub age: u64,
}

impl Faction {
    pub fn new(id: u32, name: &str, ideology: &str, center: (f32, f32)) -> Self {
        Self {
            id: FactionId(id),
            name: name.to_string(),
            ideology: ideology.to_string(),
            territory_center: center,
            territory_radius: 50.0,
            member_ids: Vec::new(),
            reputation: HashMap::new(),
            resources: 100.0,
            aggression: 0.5,
            cooperation: 0.5,
            age: 0,
        }
    }

    pub fn add_member(&mut self, agent_id: u32) {
        if !self.member_ids.contains(&agent_id) {
            self.member_ids.push(agent_id);
        }
    }

    pub fn remove_member(&mut self, agent_id: u32) {
        self.member_ids.retain(|&id| id != agent_id);
    }

    pub fn is_in_territory(&self, x: f32, y: f32) -> bool {
        let dx = x - self.territory_center.0;
        let dy = y - self.territory_center.1;
        (dx * dx + dy * dy).sqrt() <= self.territory_radius
    }

    pub fn get_reputation(&self, other: &FactionId) -> f32 {
        *self.reputation.get(other).unwrap_or(&0.0)
    }

    pub fn modify_reputation(&mut self, other: &FactionId, delta: f32) {
        let current = self.get_reputation(other);
        let new_rep = (current + delta).clamp(-1.0, 1.0);
        self.reputation.insert(other.clone(), new_rep);
    }

    pub fn tick(&mut self) {
        self.age += 1;
        self.resources += self.member_ids.len() as f32 * 0.1;
    }
}

pub struct FactionManager {
    pub factions: Vec<Faction>,
    pub agent_factions: HashMap<u32, FactionId>,
}

impl FactionManager {
    pub fn new() -> Self {
        Self {
            factions: Vec::new(),
            agent_factions: HashMap::new(),
        }
    }

    pub fn create_faction(&mut self, name: &str, ideology: &str, center: (f32, f32)) -> FactionId {
        let id = FactionId(self.factions.len() as u32);
        let faction = Faction::new(id.0, name, ideology, center);
        self.factions.push(faction);
        id
    }

    pub fn assign_agent(&mut self, agent_id: u32, faction_id: &FactionId) {
        if let Some(old_faction) = self.agent_factions.get(&agent_id).cloned() {
            if let Some(f) = self.factions.iter_mut().find(|f| f.id == old_faction) {
                f.remove_member(agent_id);
            }
        }
        self.agent_factions.insert(agent_id, faction_id.clone());
        if let Some(f) = self.factions.iter_mut().find(|f| f.id == *faction_id) {
            f.add_member(agent_id);
        }
    }

    pub fn get_faction_for_agent(&self, agent_id: u32) -> Option<&Faction> {
        let faction_id = self.agent_factions.get(&agent_id)?;
        self.factions.iter().find(|f| f.id == *faction_id)
    }

    pub fn get_faction_mut(&mut self, id: &FactionId) -> Option<&mut Faction> {
        self.factions.iter_mut().find(|f| f.id == *id)
    }

    pub fn tick(&mut self) {
        for faction in &mut self.factions {
            faction.tick();
        }
    }

    pub fn get_closest_faction(&self, x: f32, y: f32) -> Option<&Faction> {
        self.factions.iter().min_by(|a, b| {
            let da = ((a.territory_center.0 - x).powi(2) + (a.territory_center.1 - y).powi(2)).sqrt();
            let db = ((b.territory_center.0 - x).powi(2) + (b.territory_center.1 - y).powi(2)).sqrt();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faction_creation() {
        let faction = Faction::new(0, "Warriors", "aggressive", (50.0, 50.0));
        assert_eq!(faction.name, "Warriors");
        assert_eq!(faction.member_ids.len(), 0);
    }

    #[test]
    fn test_faction_territory() {
        let faction = Faction::new(0, "Test", "neutral", (50.0, 50.0));
        assert!(faction.is_in_territory(50.0, 50.0));
        assert!(!faction.is_in_territory(100.0, 100.0));
    }

    #[test]
    fn test_faction_manager() {
        let mut manager = FactionManager::new();
        let id = manager.create_faction("A", "peaceful", (0.0, 0.0));
        manager.assign_agent(1, &id);
        let faction = manager.get_faction_for_agent(1);
        assert!(faction.is_some());
        assert_eq!(faction.unwrap().member_ids, vec![1]);
    }

    #[test]
    fn test_faction_reputation() {
        let mut manager = FactionManager::new();
        let a = manager.create_faction("A", "peaceful", (0.0, 0.0));
        let b = manager.create_faction("B", "aggressive", (100.0, 100.0));
        
        if let Some(faction_a) = manager.get_faction_mut(&a) {
            faction_a.modify_reputation(&b, 0.5);
            assert_eq!(faction_a.get_reputation(&b), 0.5);
        }
    }
}
