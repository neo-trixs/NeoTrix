use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcRole {
    Merchant,
    Guard,
    QuestGiver,
    Villager,
    Enemy,
    Ally,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelationshipLevel {
    Stranger,
    Acquaintance,
    Friend,
    CloseFriend,
    BestFriend,
}

impl RelationshipLevel {
    pub fn next(&self) -> Option<RelationshipLevel> {
        match self {
            RelationshipLevel::Stranger => Some(RelationshipLevel::Acquaintance),
            RelationshipLevel::Acquaintance => Some(RelationshipLevel::Friend),
            RelationshipLevel::Friend => Some(RelationshipLevel::CloseFriend),
            RelationshipLevel::CloseFriend => Some(RelationshipLevel::BestFriend),
            RelationshipLevel::BestFriend => None,
        }
    }

    pub fn trust_modifier(&self) -> f64 {
        match self {
            RelationshipLevel::Stranger => 0.5,
            RelationshipLevel::Acquaintance => 0.7,
            RelationshipLevel::Friend => 0.9,
            RelationshipLevel::CloseFriend => 1.1,
            RelationshipLevel::BestFriend => 1.3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScheduleEntry {
    pub hour: u32,
    pub action: String,
    pub location: (f64, f64),
}

#[derive(Debug, Clone)]
pub struct NpcState {
    pub id: u64,
    pub name: String,
    pub role: NpcRole,
    pub position: (f64, f64),
    pub health: f64,
    pub max_health: f64,
    pub relationships: HashMap<u64, RelationshipLevel>,
    pub inventory: Vec<String>,
    pub mood: f64,
    pub schedule: Vec<ScheduleEntry>,
}

impl NpcState {
    pub fn new(id: u64, name: &str, role: NpcRole) -> Self {
        Self {
            id,
            name: name.to_string(),
            role,
            position: (0.0, 0.0),
            health: 100.0,
            max_health: 100.0,
            relationships: HashMap::new(),
            inventory: Vec::new(),
            mood: 0.5,
            schedule: Vec::new(),
        }
    }

    pub fn with_position(mut self, x: f64, y: f64) -> Self {
        self.position = (x, y);
        self
    }

    pub fn with_health(mut self, health: f64) -> Self {
        self.health = health;
        self.max_health = health;
        self
    }

    pub fn update_relationship(&mut self, other_id: u64, change: f64) {
        let current = self
            .relationships
            .get(&other_id)
            .copied()
            .unwrap_or(RelationshipLevel::Stranger);

        if change > 0.2 {
            if let Some(next) = current.next() {
                self.relationships.insert(other_id, next);
            }
        } else if change < -0.2 {
            let prev = match current {
                RelationshipLevel::Acquaintance => Some(RelationshipLevel::Stranger),
                RelationshipLevel::Friend => Some(RelationshipLevel::Acquaintance),
                RelationshipLevel::CloseFriend => Some(RelationshipLevel::Friend),
                RelationshipLevel::BestFriend => Some(RelationshipLevel::CloseFriend),
                _ => None,
            };
            if let Some(p) = prev {
                self.relationships.insert(other_id, p);
            }
        }
    }

    pub fn relationship_with(&self, other_id: u64) -> RelationshipLevel {
        self.relationships
            .get(&other_id)
            .copied()
            .unwrap_or(RelationshipLevel::Stranger)
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn heal(&mut self, amount: f64) {
        self.health = (self.health + amount).min(self.max_health);
    }

    pub fn damage(&mut self, amount: f64) {
        self.health = (self.health - amount).max(0.0);
    }
}
