use serde::{Serialize, Deserialize};
use crate::foundation::math_bridge::Vec2;
use crate::agents::sim_agent::{SimAgent, Personality};

/// Spawn template types defining agent archetypes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpawnTemplate {
    Gatherer,
    Warrior,
    Trader,
    Explorer,
    Builder,
}

impl SpawnTemplate {
    /// Initial resources [energy, health, hunger]
    pub fn initial_resources(&self) -> [f32; 3] {
        match self {
            SpawnTemplate::Gatherer => [90.0, 100.0, 0.0],
            SpawnTemplate::Warrior => [100.0, 100.0, 10.0],
            SpawnTemplate::Trader => [85.0, 95.0, 5.0],
            SpawnTemplate::Explorer => [95.0, 90.0, 5.0],
            SpawnTemplate::Builder => [80.0, 100.0, 10.0],
        }
    }

    /// Skill weights for this template
    pub fn skill_weights(&self) -> Vec<(&'static str, f32)> {
        match self {
            SpawnTemplate::Gatherer => vec![
                ("foraging", 0.8),
                ("navigation", 0.5),
                ("endurance", 0.6),
            ],
            SpawnTemplate::Warrior => vec![
                ("combat", 0.8),
                ("strength", 0.7),
                ("endurance", 0.6),
            ],
            SpawnTemplate::Trader => vec![
                ("negotiation", 0.8),
                ("memory", 0.6),
                ("perception", 0.5),
            ],
            SpawnTemplate::Explorer => vec![
                ("navigation", 0.8),
                ("perception", 0.7),
                ("endurance", 0.5),
            ],
            SpawnTemplate::Builder => vec![
                ("crafting", 0.8),
                ("planning", 0.6),
                ("endurance", 0.5),
            ],
        }
    }

    /// Personality ranges for this template (min, max) per trait
    pub fn personality_range(&self) -> PersonalityRange {
        match self {
            SpawnTemplate::Gatherer => PersonalityRange {
                openness: (0.3, 0.6),
                sociability: (0.4, 0.7),
                aggression: (0.1, 0.3),
                cooperativeness: (0.5, 0.8),
                curiosity: (0.4, 0.7),
            },
            SpawnTemplate::Warrior => PersonalityRange {
                openness: (0.3, 0.6),
                sociability: (0.3, 0.6),
                aggression: (0.6, 0.9),
                cooperativeness: (0.2, 0.5),
                curiosity: (0.3, 0.5),
            },
            SpawnTemplate::Trader => PersonalityRange {
                openness: (0.5, 0.8),
                sociability: (0.6, 0.9),
                aggression: (0.1, 0.3),
                cooperativeness: (0.6, 0.9),
                curiosity: (0.4, 0.7),
            },
            SpawnTemplate::Explorer => PersonalityRange {
                openness: (0.7, 1.0),
                sociability: (0.3, 0.6),
                aggression: (0.2, 0.4),
                cooperativeness: (0.3, 0.6),
                curiosity: (0.7, 1.0),
            },
            SpawnTemplate::Builder => PersonalityRange {
                openness: (0.4, 0.7),
                sociability: (0.4, 0.7),
                aggression: (0.1, 0.3),
                cooperativeness: (0.5, 0.8),
                curiosity: (0.5, 0.8),
            },
        }
    }

    /// Weight for random selection (higher = more common)
    pub fn spawn_weight(&self) -> f32 {
        match self {
            SpawnTemplate::Gatherer => 30.0,
            SpawnTemplate::Warrior => 15.0,
            SpawnTemplate::Trader => 20.0,
            SpawnTemplate::Explorer => 25.0,
            SpawnTemplate::Builder => 10.0,
        }
    }

    /// Skills this template starts with
    pub fn initial_skills(&self) -> Vec<String> {
        match self {
            SpawnTemplate::Gatherer => vec!["foraging".into(), "herbalism".into()],
            SpawnTemplate::Warrior => vec!["melee".into(), "defense".into()],
            SpawnTemplate::Trader => vec!["barter".into(), "appraisal".into()],
            SpawnTemplate::Explorer => vec!["scouting".into(), "mapping".into()],
            SpawnTemplate::Builder => vec!["construction".into(), "crafting".into()],
        }
    }
}

/// Personality trait ranges for template-based generation
#[derive(Debug, Clone)]
pub struct PersonalityRange {
    pub openness: (f32, f32),
    pub sociability: (f32, f32),
    pub aggression: (f32, f32),
    pub cooperativeness: (f32, f32),
    pub curiosity: (f32, f32),
}

/// Simple PRNG for spawn generation
struct SpawnRng {
    state: u32,
}

impl SpawnRng {
    fn new(seed: u64) -> Self {
        Self {
            state: (seed.max(1) as u32).wrapping_mul(1103515245).wrapping_add(12345),
        }
    }

    fn next_f32(&mut self) -> f32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        (self.state as f32) / (u32::MAX as f32)
    }

    fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

/// Create an agent from a template at a given position
pub fn create_agent(template: SpawnTemplate, position: Vec2, id: u64, seed: u64) -> SimAgent {
    let mut rng = SpawnRng::new(seed.wrapping_add(id));

    let resources = template.initial_resources();
    let pr = template.personality_range();
    let skills = template.initial_skills();
    let skill_weights = template.skill_weights();

    let personality = Personality {
        openness: rng.range(pr.openness.0, pr.openness.1),
        sociability: rng.range(pr.sociability.0, pr.sociability.1),
        aggression: rng.range(pr.aggression.0, pr.aggression.1),
        cooperativeness: rng.range(pr.cooperativeness.0, pr.cooperativeness.1),
        curiosity: rng.range(pr.curiosity.0, pr.curiosity.1),
    };

    let mut agent = SimAgent::new(id, position);
    agent.core.energy = resources[0];
    agent.core.health = resources[1];
    agent.core.hunger = resources[2];
    agent.personality = personality;
    agent.skills = skills;

    // Store skill weights in recent_actions as metadata (lightweight approach)
    for (skill, weight) in skill_weights {
        let entry = format!("skill:{}:{}", skill, (weight * 100.0) as u32);
        agent.recent_actions.push(entry);
    }

    agent
}

/// Select a random template using weighted distribution
pub fn random_template(seed: u64) -> SpawnTemplate {
    let templates = [
        SpawnTemplate::Gatherer,
        SpawnTemplate::Warrior,
        SpawnTemplate::Trader,
        SpawnTemplate::Explorer,
        SpawnTemplate::Builder,
    ];

    let total_weight: f32 = templates.iter().map(|t| t.spawn_weight()).sum();
    let mut rng = SpawnRng::new(seed);
    let roll = rng.next_f32() * total_weight;

    let mut cumulative = 0.0;
    for template in &templates {
        cumulative += template.spawn_weight();
        if roll <= cumulative {
            return *template;
        }
    }

    SpawnTemplate::Gatherer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gatherer_resources() {
        let res = SpawnTemplate::Gatherer.initial_resources();
        assert_eq!(res.len(), 3);
        assert!(res[0] > 0.0);
    }

    #[test]
    fn warrior_skills() {
        let skills = SpawnTemplate::Warrior.initial_skills();
        assert!(skills.contains(&"melee".to_string()));
    }

    #[test]
    fn personality_range_valid() {
        let pr = SpawnTemplate::Trader.personality_range();
        assert!(pr.openness.0 <= pr.openness.1);
        assert!(pr.sociability.0 <= pr.sociability.1);
    }

    #[test]
    fn create_agent_produces_valid_agent() {
        let agent = create_agent(SpawnTemplate::Explorer, Vec2::new(10.0, 20.0), 42, 99);
        assert_eq!(agent.id, 42);
        assert_eq!(agent.core.position, Vec2::new(10.0, 20.0));
        assert!(agent.is_alive());
        assert!(!agent.skills.is_empty());
    }

    #[test]
    fn create_agent_personality_in_range() {
        let agent = create_agent(SpawnTemplate::Warrior, Vec2::zero(), 1, 100);
        let pr = SpawnTemplate::Warrior.personality_range();
        assert!(agent.personality.aggression >= pr.aggression.0 - 0.01);
        assert!(agent.personality.aggression <= pr.aggression.1 + 0.01);
    }

    #[test]
    fn random_template_returns_valid() {
        let t = random_template(42);
        // Just ensure it doesn't panic and returns something
        let _ = t.spawn_weight();
    }

    #[test]
    fn random_template_varies_with_seed() {
        let mut templates = std::collections::HashSet::new();
        for seed in 0..100 {
            templates.insert(format!("{:?}", random_template(seed)));
        }
        // With 100 different seeds, we should get at least 2 different templates
        assert!(templates.len() >= 2);
    }

    #[test]
    fn spawn_weights_positive() {
        assert!(SpawnTemplate::Gatherer.spawn_weight() > 0.0);
        assert!(SpawnTemplate::Warrior.spawn_weight() > 0.0);
        assert!(SpawnTemplate::Trader.spawn_weight() > 0.0);
        assert!(SpawnTemplate::Explorer.spawn_weight() > 0.0);
        assert!(SpawnTemplate::Builder.spawn_weight() > 0.0);
    }
}
