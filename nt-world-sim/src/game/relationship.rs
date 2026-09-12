use crate::core::world::Resource;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipStage { Stranger, Acquaintance, Friend, GoodFriend, BestFriend, Soulmate }

#[derive(Debug, Clone)]
pub struct Relationship {
    pub npc_name: String,
    pub resonance: u32,
    pub max_resonance: u32,
    pub stage: RelationshipStage,
    pub gifts_given: u32,
    pub days_known: u32,
    pub is_married: bool,
    pub has_children: bool,
    pub children_count: u32,
}

impl Relationship {
    pub fn new(npc_name: &str) -> Self {
        Self {
            npc_name: npc_name.to_string(),
            resonance: 0,
            max_resonance: 1000,
            stage: RelationshipStage::Stranger,
            gifts_given: 0,
            days_known: 0,
            is_married: false,
            has_children: false,
            children_count: 0,
        }
    }

    pub fn add_resonance(&mut self, amount: u32) {
        self.resonance = (self.resonance + amount).min(self.max_resonance);
        self.update_stage();
    }

    fn update_stage(&mut self) {
        self.stage = if self.resonance >= 800 { RelationshipStage::Soulmate }
            else if self.resonance >= 500 { RelationshipStage::BestFriend }
            else if self.resonance >= 300 { RelationshipStage::GoodFriend }
            else if self.resonance >= 150 { RelationshipStage::Friend }
            else if self.resonance >= 50 { RelationshipStage::Acquaintance }
            else { RelationshipStage::Stranger };
    }

    pub fn can_marry(&self) -> bool {
        self.stage == RelationshipStage::Soulmate && !self.is_married && self.resonance >= 800
    }

    pub fn marry(&mut self) -> bool {
        if self.can_marry() {
            self.is_married = true;
            true
        } else { false }
    }

    pub fn can_have_child(&self) -> bool {
        self.is_married && !self.has_children && self.resonance >= 900
    }

    pub fn have_child(&mut self, child_name: &str) -> Option<Child> {
        if self.can_have_child() || (self.has_children && self.children_count < 2) {
            self.has_children = true;
            self.children_count += 1;
            Some(Child {
                name: child_name.to_string(),
                parent: self.npc_name.clone(),
                age_days: 0,
                happiness: 100,
            })
        } else { None }
    }

    pub fn stage_name(&self) -> &str {
        match self.stage {
            RelationshipStage::Stranger => "Stranger",
            RelationshipStage::Acquaintance => "Acquaintance",
            RelationshipStage::Friend => "Friend",
            RelationshipStage::GoodFriend => "Good Friend",
            RelationshipStage::BestFriend => "Best Friend",
            RelationshipStage::Soulmate => "Soulmate",
        }
    }

    pub fn resonance_hearts(&self) -> u32 {
        self.resonance / 100 // 0-10 hearts
    }
}

#[derive(Debug, Clone)]
pub struct Child {
    pub name: String,
    pub parent: String,
    pub age_days: u32,
    pub happiness: u32,
}

impl Child {
    pub fn age_up(&mut self) {
        self.age_days += 1;
        self.happiness = self.happiness.saturating_sub(1);
    }

    pub fn play(&mut self) {
        self.happiness = (self.happiness + 10).min(100);
    }

    pub fn is_adult(&self) -> bool { self.age_days >= 14 }
}

pub struct RelationshipSystem {
    pub relationships: HashMap<String, Relationship>,
    pub children: Vec<Child>,
    pub proposals_pending: Vec<String>,
}

impl Resource for RelationshipSystem {}

impl RelationshipSystem {
    pub fn new() -> Self {
        Self { relationships: HashMap::new(), children: Vec::new(), proposals_pending: Vec::new() }
    }

    pub fn get_relationship(&self, npc_name: &str) -> Option<&Relationship> {
        self.relationships.get(npc_name)
    }

    pub fn get_relationship_mut(&mut self, npc_name: &str) -> &mut Relationship {
        self.relationships.entry(npc_name.to_string()).or_insert_with(|| Relationship::new(npc_name))
    }

    pub fn propose(&mut self, npc_name: &str) -> bool {
        if let Some(rel) = self.relationships.get(npc_name) {
            if rel.can_marry() {
                self.proposals_pending.push(npc_name.to_string());
                return true;
            }
        }
        false
    }

    pub fn accept_proposal(&mut self, npc_name: &str) -> bool {
        if let Some(rel) = self.relationships.get_mut(npc_name) {
            if rel.marry() {
                self.proposals_pending.retain(|n| n != npc_name);
                return true;
            }
        }
        false
    }

    pub fn advance_day(&mut self) {
        for child in &mut self.children {
            child.age_up();
        }

        for rel in self.relationships.values_mut() {
            if rel.resonance > 0 && !rel.is_married {
                rel.resonance = rel.resonance.saturating_sub(1);
                rel.update_stage();
            }
        }
    }

    pub fn married_npc(&self) -> Option<&str> {
        self.relationships.values()
            .find(|r| r.is_married)
            .map(|r| r.npc_name.as_str())
    }

    pub fn all_children(&self) -> &[Child] {
        &self.children
    }
}

impl Default for RelationshipSystem {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_stages() {
        let mut rel = Relationship::new("Test");
        assert_eq!(rel.stage, RelationshipStage::Stranger);

        rel.add_resonance(50);
        assert_eq!(rel.stage, RelationshipStage::Acquaintance);

        rel.add_resonance(100);
        assert_eq!(rel.stage, RelationshipStage::Friend);

        rel.add_resonance(150);
        assert_eq!(rel.stage, RelationshipStage::GoodFriend);

        rel.add_resonance(200);
        assert_eq!(rel.stage, RelationshipStage::BestFriend);

        rel.add_resonance(300);
        assert_eq!(rel.stage, RelationshipStage::Soulmate);
    }

    #[test]
    fn test_marry() {
        let mut rel = Relationship::new("Test");
        assert!(!rel.can_marry());

        rel.add_resonance(800);
        assert!(rel.can_marry());
        assert!(rel.marry());
        assert!(rel.is_married);
        assert!(!rel.can_marry());
    }

    #[test]
    fn test_have_child() {
        let mut rel = Relationship::new("Test");
        assert!(!rel.can_have_child());

        rel.add_resonance(900);
        rel.marry();
        assert!(rel.can_have_child());

        let child = rel.have_child("Baby");
        assert!(child.is_some());
        assert_eq!(rel.children_count, 1);
        assert!(rel.has_children);

        let child2 = rel.have_child("Child2");
        assert!(child2.is_some());
        assert_eq!(rel.children_count, 2);

        let child3 = rel.have_child("Child3");
        assert!(child3.is_none());
    }

    #[test]
    fn test_child_aging() {
        let mut child = Child { name: "Test".into(), parent: "NPC".into(), age_days: 0, happiness: 100 };
        child.age_up();
        assert_eq!(child.age_days, 1);
        assert_eq!(child.happiness, 99);
        assert!(!child.is_adult());

        for _ in 0..14 { child.age_up(); }
        assert!(child.is_adult());
    }

    #[test]
    fn test_child_play() {
        let mut child = Child { name: "Test".into(), parent: "NPC".into(), age_days: 0, happiness: 50 };
        child.play();
        assert_eq!(child.happiness, 60);
    }

    #[test]
    fn test_resonance_decay() {
        let mut sys = RelationshipSystem::new();
        sys.get_relationship_mut("NPC1").add_resonance(100);
        assert_eq!(sys.relationships["NPC1"].resonance, 100);

        sys.advance_day();
        assert_eq!(sys.relationships["NPC1"].resonance, 99);
    }

    #[test]
    fn test_married_no_decay() {
        let mut sys = RelationshipSystem::new();
        let rel = sys.get_relationship_mut("NPC1");
        rel.add_resonance(800);
        rel.marry();

        sys.advance_day();
        assert_eq!(sys.relationships["NPC1"].resonance, 800);
    }

    #[test]
    fn test_proposal_flow() {
        let mut sys = RelationshipSystem::new();
        sys.get_relationship_mut("NPC1").add_resonance(800);

        assert!(sys.propose("NPC1"));
        assert!(sys.proposals_pending.contains(&"NPC1".to_string()));

        assert!(sys.accept_proposal("NPC1"));
        assert!(sys.proposals_pending.is_empty());
        assert!(sys.relationships["NPC1"].is_married);
    }

    #[test]
    fn test_married_npc() {
        let mut sys = RelationshipSystem::new();
        assert!(sys.married_npc().is_none());

        sys.get_relationship_mut("Alice").add_resonance(800);
        sys.accept_proposal("Alice");
        assert_eq!(sys.married_npc(), Some("Alice"));
    }

    #[test]
    fn test_resonance_hearts() {
        let mut rel = Relationship::new("Test");
        assert_eq!(rel.resonance_hearts(), 0);
        rel.add_resonance(500);
        assert_eq!(rel.resonance_hearts(), 5);
    }

    #[test]
    fn test_stage_name() {
        let mut rel = Relationship::new("Test");
        assert_eq!(rel.stage_name(), "Stranger");
        rel.add_resonance(800);
        assert_eq!(rel.stage_name(), "Soulmate");
    }
}
