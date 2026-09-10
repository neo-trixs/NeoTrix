use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use crate::foundation::math_bridge::Vec2;

/// Agent action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentAction {
    Move { target: Vec2 },
    Harvest { resource_id: String },
    Eat { resource_id: String },
    Attack { target_id: String },
    Trade {
        target_id: String,
        item: String,
        amount: u32,
    },
    Talk {
        target_id: String,
        message: String,
    },
    Build {
        position: Vec2,
        structure_type: String,
    },
    Explore { direction: Vec2 },
    Rest,
    Think,
}

/// Agent observation of the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentObservation {
    pub position: Vec2,
    pub nearby_agents: Vec<NearbyAgent>,
    pub nearby_resources: Vec<NearbyResource>,
    pub terrain_type: String,
    pub time_of_day: String,
    pub season: String,
    pub threats: Vec<Threat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearbyAgent {
    pub id: String,
    pub distance: f32,
    pub relationship: f32,
    pub apparent_health: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearbyResource {
    pub id: String,
    pub resource_type: String,
    pub distance: f32,
    pub amount: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threat {
    pub threat_type: String,
    pub position: Vec2,
    pub severity: f32,
}

/// Core agent state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCore {
    pub id: String,
    pub position: Vec2,
    pub energy: f32,
    pub health: f32,
    pub hunger: f32,
    pub age: u64,
    pub alive: bool,
}

impl AgentCore {
    pub fn new(id: &str, position: Vec2) -> Self {
        Self {
            id: id.to_string(),
            position,
            energy: 100.0,
            health: 100.0,
            hunger: 0.0,
            age: 0,
            alive: true,
        }
    }

    /// Metabolism: energy decreases each tick, hunger increases
    pub fn metabolize(&mut self, energy_cost: f32, hunger_rate: f32) {
        self.energy = (self.energy - energy_cost).max(0.0);
        self.hunger = (self.hunger + hunger_rate).min(100.0);
        self.age += 1;

        // Starvation damage
        if self.hunger > 80.0 {
            self.health -= (self.hunger - 80.0) * 0.1;
        }

        // Death check
        if self.energy <= 0.0 || self.health <= 0.0 {
            self.alive = false;
        }
    }

    pub fn eat(&mut self, nutrition: f32) {
        self.hunger = (self.hunger - nutrition).max(0.0);
        self.energy = (self.energy + nutrition * 0.5).min(100.0);
    }

    pub fn rest(&mut self, amount: f32) {
        self.energy = (self.energy + amount).min(100.0);
        self.health = (self.health + amount * 0.2).min(100.0);
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
        if self.health <= 0.0 {
            self.alive = false;
        }
    }
}

/// The complete SimAgent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimAgent {
    pub id: u64,
    pub core: AgentCore,
    pub skills: Vec<String>,
    pub personality: Personality,
    pub recent_actions: Vec<String>,
    pub last_action: AgentAction,
    pub phi: f32,
    /// Maslow needs (CivSim pattern): [physiological, safety, love, esteem, self_actualization]
    pub needs: [f32; 5],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    pub openness: f32,
    pub sociability: f32,
    pub aggression: f32,
    pub cooperativeness: f32,
    pub curiosity: f32,
}

impl Default for Personality {
    fn default() -> Self {
        Self {
            openness: 0.5,
            sociability: 0.5,
            aggression: 0.3,
            cooperativeness: 0.5,
            curiosity: 0.5,
        }
    }
}

impl Personality {
    pub fn dominant_trait(&self) -> String {
        let traits = [
            ("openness", self.openness),
            ("sociability", self.sociability),
            ("aggression", self.aggression),
            ("cooperativeness", self.cooperativeness),
            ("curiosity", self.curiosity),
        ];
        traits
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(name, _)| name.to_string())
            .unwrap_or_else(|| "balanced".to_string())
    }
}

impl SimAgent {
    pub fn new(id: u64, position: Vec2) -> Self {
        Self {
            id,
            core: AgentCore::new(&id.to_string(), position),
            skills: Vec::new(),
            personality: Personality::default(),
            recent_actions: Vec::new(),
            last_action: AgentAction::Rest,
            phi: 0.0,
            needs: [1.0, 1.0, 0.5, 0.3, 0.2],
        }
    }

    /// Create from a string ID (for evolution pipeline compatibility).
    /// Hashes the string to produce a u64 id.
    pub fn from_string_id(string_id: &str, position: Vec2) -> Self {
        let numeric_id = {
            let mut hasher = DefaultHasher::new();
            string_id.hash(&mut hasher);
            hasher.finish()
        };
        Self {
            id: numeric_id,
            core: AgentCore::new(string_id, position),
            skills: Vec::new(),
            personality: Personality::default(),
            recent_actions: Vec::new(),
            last_action: AgentAction::Rest,
            phi: 0.0,
            needs: [1.0, 1.0, 0.5, 0.3, 0.2],
        }
    }

    pub fn with_personality(mut self, p: Personality) -> Self {
        self.personality = p;
        self
    }

    pub fn is_alive(&self) -> bool {
        self.core.alive
    }

    /// Composite fitness: energy + health + needs satisfaction
    pub fn fitness(&self) -> f64 {
        let needs_avg = (self.needs[0] + self.needs[1] + self.needs[2] + self.needs[3] + self.needs[4]) / 5.0;
        ((self.core.energy + self.core.health) / 200.0 + needs_avg * 0.3) as f64
    }

    /// Update Maslow needs each tick (CivSim pattern).
    /// Lower-level needs decay faster; satisfying lower needs enables higher ones.
    pub fn update_needs(&mut self, _dt: f32) {
        // Physiological: decays with hunger, restored by eating
        self.needs[0] = (1.0 - self.core.hunger / 100.0).max(0.0);
        // Safety: decays with low health, restored by resting
        self.needs[1] = (self.core.health / 100.0).max(0.0);
        // Love/belonging: managed externally via RelationshipGraph
        // self.needs[2] is set by WorldSim using relationship_graph data
        // Esteem: grows with fitness and skills
        self.needs[3] = (self.skills.len() as f32 / 5.0 + self.fitness() as f32 * 0.5).min(1.0);
        // Self-actualization: grows with phi and curiosity
        self.needs[4] = (self.phi * 0.5 + self.personality.curiosity * 0.5).min(1.0);
    }

    /// Record an action for coherence tracking
    pub fn record_action(&mut self, action: &AgentAction) {
        self.last_action = action.clone();
        let desc = format!("{:?}", action);
        self.recent_actions.push(desc);
        if self.recent_actions.len() > 20 {
            self.recent_actions.remove(0);
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_creation() {
        let agent = SimAgent::new(1, Vec2::new(10.0, 20.0));
        assert_eq!(agent.id, 1);
        assert_eq!(agent.core.position, Vec2::new(10.0, 20.0));
        assert!(agent.is_alive());
        assert_eq!(agent.core.energy, 100.0);
    }

    #[test]
    fn agent_metabolism() {
        let mut core = AgentCore::new("test", Vec2::zero());
        core.metabolize(10.0, 5.0);
        assert_eq!(core.energy, 90.0);
        assert_eq!(core.hunger, 5.0);
        assert_eq!(core.age, 1);

        // Starvation damage
        core.hunger = 90.0;
        core.metabolize(0.0, 0.0);
        assert!(core.health < 100.0);
    }

    #[test]
    fn agent_eat_rest() {
        let mut core = AgentCore::new("test", Vec2::zero());
        core.hunger = 50.0;
        core.eat(30.0);
        assert_eq!(core.hunger, 20.0);
        assert_eq!(core.energy, 100.0); // clamped

        core.energy = 50.0;
        core.rest(20.0);
        assert_eq!(core.energy, 70.0);
    }

    #[test]
    fn agent_death() {
        let mut core = AgentCore::new("test", Vec2::zero());
        core.take_damage(100.0);
        assert!(!core.alive);

        let mut core2 = AgentCore::new("test2", Vec2::zero());
        core2.energy = 0.0;
        core2.metabolize(0.0, 0.0);
        assert!(!core2.alive);
    }

    #[test]
    fn agent_action_record() {
        let mut agent = SimAgent::new(1, Vec2::zero());
        agent.record_action(&AgentAction::Rest);
        agent.record_action(&AgentAction::Think);
        assert_eq!(agent.recent_actions.len(), 2);
        assert!(matches!(agent.last_action, AgentAction::Think));
    }

    #[test]
    fn agent_fitness() {
        let mut agent = SimAgent::new(1, Vec2::zero());
        let f1 = agent.fitness();
        agent.core.energy = 50.0;
        let f2 = agent.fitness();
        assert!(f2 < f1);
    }

    #[test]
    fn agent_needs_decay() {
        let mut agent = SimAgent::new(1, Vec2::zero());
        agent.core.hunger = 80.0;
        agent.core.health = 40.0;
        agent.update_needs(1.0);
        assert!(agent.needs[0] < 0.5); // physiological low
        assert!(agent.needs[1] < 0.5); // safety low
    }

    #[test]
    fn agent_dominant_trait() {
        let agent = SimAgent::new(1, Vec2::zero());
        let trait_name = agent.personality.dominant_trait();
        assert!(!trait_name.is_empty());
    }
}
