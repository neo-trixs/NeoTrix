#![forbid(unsafe_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::cognitive_engine::ContentItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    Perceptual,
    Motor,
    Associative,
    Reflective,
    Attentional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveAgent {
    pub id: usize,
    pub name: String,
    pub agent_type: AgentType,
    pub expertise: Vec<String>,
    pub salience_weight: f64,
    pub activation_threshold: f64,
    last_broadcast: String,
}

impl CognitiveAgent {
    pub fn new(id: usize, name: String, agent_type: AgentType) -> Self {
        Self {
            id,
            name,
            agent_type,
            expertise: Vec::new(),
            salience_weight: 0.5,
            activation_threshold: 0.3,
            last_broadcast: String::new(),
        }
    }

    pub fn generate_content(&self, tick_count: u64) -> ContentItem {
        let content = match self.agent_type {
            AgentType::Perceptual => {
                format!("Observation at tick {}: {} reports", tick_count, self.name)
            }
            AgentType::Motor => {
                format!("Action proposal from {}", self.name)
            }
            AgentType::Associative => {
                format!("Memory pattern: {} associates", self.name)
            }
            AgentType::Reflective => {
                format!("Meta-observation by {}", self.name)
            }
            AgentType::Attentional => {
                format!("Focus report: {} highlights", self.name)
            }
        };
        let pseudo_rand =
            ((tick_count.wrapping_mul(7).wrapping_add(self.id as u64).wrapping_mul(13)) as f64
                * 0.001)
                .fract();
        let salience = self.salience_weight * (0.5 + 0.5 * pseudo_rand);
        ContentItem {
            id: 0,
            content,
            salience: salience.max(0.0).min(1.0),
            source_agent: self.id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn receive_broadcast(&mut self, content: &ContentItem) {
        self.last_broadcast = content.content.clone();
    }

    pub fn activation_probability(&self, current_entropy: f64) -> f64 {
        let base = self.salience_weight;
        if current_entropy > self.activation_threshold {
            (base + 0.2).max(0.0).min(1.0)
        } else {
            base
        }
    }

    pub fn specialization_score(&self, task: &str) -> f64 {
        let matched = self.expertise.iter().filter(|e| task.contains(e.as_str())).count();
        if self.expertise.is_empty() {
            0.0
        } else {
            (matched as f64 / self.expertise.len() as f64).max(0.0).min(1.0)
        }
    }
}

pub fn create_default_agents() -> Vec<CognitiveAgent> {
    vec![
        CognitiveAgent {
            id: 1,
            name: "Sensor".into(),
            agent_type: AgentType::Perceptual,
            expertise: vec!["vision".into(), "audio".into(), "text".into()],
            salience_weight: 0.5,
            activation_threshold: 0.3,
            last_broadcast: String::new(),
        },
        CognitiveAgent {
            id: 2,
            name: "Executor".into(),
            agent_type: AgentType::Motor,
            expertise: vec!["code".into(), "actions".into(), "tools".into()],
            salience_weight: 0.5,
            activation_threshold: 0.3,
            last_broadcast: String::new(),
        },
        CognitiveAgent {
            id: 3,
            name: "PatternMatcher".into(),
            agent_type: AgentType::Associative,
            expertise: vec!["memory".into(), "patterns".into(), "analogies".into()],
            salience_weight: 0.5,
            activation_threshold: 0.3,
            last_broadcast: String::new(),
        },
        CognitiveAgent {
            id: 4,
            name: "Introspector".into(),
            agent_type: AgentType::Reflective,
            expertise: vec!["self-model".into(), "meta-cognition".into(), "ethics".into()],
            salience_weight: 0.5,
            activation_threshold: 0.3,
            last_broadcast: String::new(),
        },
        CognitiveAgent {
            id: 5,
            name: "Gatekeeper".into(),
            agent_type: AgentType::Attentional,
            expertise: vec!["focus".into(), "priority".into(), "filtering".into()],
            salience_weight: 0.5,
            activation_threshold: 0.3,
            last_broadcast: String::new(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = CognitiveAgent::new(1, "TestAgent".into(), AgentType::Perceptual);
        assert_eq!(agent.id, 1);
        assert_eq!(agent.name, "TestAgent");
        assert_eq!(agent.agent_type, AgentType::Perceptual);
        assert!((agent.salience_weight - 0.5).abs() < 1e-9);
        assert!((agent.activation_threshold - 0.3).abs() < 1e-9);
    }

    #[test]
    fn test_perceptual_content() {
        let agent = CognitiveAgent::new(1, "Sensor".into(), AgentType::Perceptual);
        let item = agent.generate_content(42);
        assert!(item.content.contains("Observation at tick 42"));
        assert!(item.content.contains("Sensor"));
    }

    #[test]
    fn test_motor_content() {
        let agent = CognitiveAgent::new(2, "Executor".into(), AgentType::Motor);
        let item = agent.generate_content(1);
        assert!(item.content.contains("Action proposal from"));
        assert!(item.content.contains("Executor"));
    }

    #[test]
    fn test_associative_content() {
        let agent = CognitiveAgent::new(3, "Matcher".into(), AgentType::Associative);
        let item = agent.generate_content(5);
        assert!(item.content.contains("Memory pattern"));
        assert!(item.content.contains("Matcher"));
    }

    #[test]
    fn test_reflective_content() {
        let agent = CognitiveAgent::new(4, "Self".into(), AgentType::Reflective);
        let item = agent.generate_content(10);
        assert!(item.content.contains("Meta-observation by"));
        assert!(item.content.contains("Self"));
    }

    #[test]
    fn test_attentional_content() {
        let agent = CognitiveAgent::new(5, "Focus".into(), AgentType::Attentional);
        let item = agent.generate_content(3);
        assert!(item.content.contains("Focus report"));
        assert!(item.content.contains("Focus"));
    }

    #[test]
    fn test_broadcast_reception() {
        let mut agent = CognitiveAgent::new(1, "Sensor".into(), AgentType::Perceptual);
        let item = ContentItem {
            id: 99,
            content: "broadcast message".into(),
            salience: 0.8,
            source_agent: 2,
            timestamp: 1000,
        };
        agent.receive_broadcast(&item);
        assert_eq!(agent.last_broadcast, "broadcast message");
    }

    #[test]
    fn test_activation_probability_increases_with_entropy() {
        let agent = CognitiveAgent::new(1, "A".into(), AgentType::Perceptual);
        let low = agent.activation_probability(0.1);
        let high = agent.activation_probability(0.5);
        assert!(high >= low);
    }

    #[test]
    fn test_specialization_score() {
        let mut agent = CognitiveAgent::new(1, "A".into(), AgentType::Perceptual);
        agent.expertise = vec!["vision".into(), "audio".into()];
        let score = agent.specialization_score("vision processing");
        assert!((score - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_specialization_score_empty_expertise() {
        let agent = CognitiveAgent::new(1, "A".into(), AgentType::Perceptual);
        let score = agent.specialization_score("anything");
        assert!((score - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_default_agents_creation() {
        let agents = create_default_agents();
        assert_eq!(agents.len(), 5);
        let types: Vec<AgentType> = agents.iter().map(|a| a.agent_type).collect();
        assert!(types.contains(&AgentType::Perceptual));
        assert!(types.contains(&AgentType::Motor));
        assert!(types.contains(&AgentType::Associative));
        assert!(types.contains(&AgentType::Reflective));
        assert!(types.contains(&AgentType::Attentional));
    }

    #[test]
    fn test_content_salience_range() {
        let agent = CognitiveAgent::new(1, "Sensor".into(), AgentType::Perceptual);
        for tick in 0..100 {
            let item = agent.generate_content(tick);
            assert!(item.salience >= 0.0);
            assert!(item.salience <= 1.0);
        }
    }

    #[test]
    fn test_content_source_agent() {
        let agent = CognitiveAgent::new(7, "Agent7".into(), AgentType::Motor);
        let item = agent.generate_content(0);
        assert_eq!(item.source_agent, 7);
    }
}
