use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefModel {
    pub agent_id: String,
    pub perceived_goals: Vec<String>,
    pub perceived_beliefs_about_self: f64,
    pub perceived_emotion: String,
    pub credibility: f64,
    pub cooperativeness: f64,
    pub threat_level: f64,
    pub observation_count: u32,
    pub last_updated: u64,
}

pub struct TheoryOfMind {
    models: HashMap<String, BeliefModel>,
    max_models: usize,
}

impl TheoryOfMind {
    pub fn new(max_models: usize) -> Self {
        Self { models: HashMap::new(), max_models }
    }

    pub fn observe_action(&mut self, agent_id: &str, action: &str, outcome_positive: bool, tick: u64) {
        let model = self.models.entry(agent_id.to_string()).or_insert_with(|| BeliefModel {
            agent_id: agent_id.to_string(),
            perceived_goals: Vec::new(),
            perceived_beliefs_about_self: 0.0,
            perceived_emotion: "neutral".to_string(),
            credibility: 0.5,
            cooperativeness: 0.5,
            threat_level: 0.1,
            observation_count: 0,
            last_updated: tick,
        });

        model.observation_count += 1;
        model.last_updated = tick;

        if outcome_positive {
            model.credibility = (model.credibility + 0.05).min(1.0);
        } else {
            model.credibility = (model.credibility - 0.03).max(0.0);
        }

        if action.contains("help") || action.contains("trade") || action.contains("talk") {
            model.cooperativeness = (model.cooperativeness + 0.05).min(1.0);
        } else if action.contains("attack") || action.contains("steal") {
            model.cooperativeness = (model.cooperativeness - 0.1).max(0.0);
            model.threat_level = (model.threat_level + 0.1).min(1.0);
        }

        if !model.perceived_goals.contains(&action.to_string()) {
            model.perceived_goals.push(action.to_string());
            if model.perceived_goals.len() > 5 {
                model.perceived_goals.remove(0);
            }
        }
        self.prune();
    }

    pub fn observe_conversation(&mut self, agent_id: &str, sentiment: f64, tick: u64) {
        let model = self.models.entry(agent_id.to_string()).or_insert_with(|| BeliefModel {
            agent_id: agent_id.to_string(),
            perceived_goals: Vec::new(),
            perceived_beliefs_about_self: 0.0,
            perceived_emotion: "neutral".to_string(),
            credibility: 0.5,
            cooperativeness: 0.5,
            threat_level: 0.1,
            observation_count: 0,
            last_updated: tick,
        });
        model.observation_count += 1;
        model.last_updated = tick;
        model.perceived_emotion = if sentiment > 0.3 { "friendly".to_string() }
            else if sentiment < -0.3 { "hostile".to_string() }
            else { "neutral".to_string() };
        model.perceived_beliefs_about_self = (model.perceived_beliefs_about_self + sentiment * 0.1).clamp(-1.0, 1.0);
    }

    pub fn observe_interaction(&mut self, agent_id: &str, toward_us: bool, positive: bool, tick: u64) {
        let model = self.models.entry(agent_id.to_string()).or_insert_with(|| BeliefModel {
            agent_id: agent_id.to_string(),
            perceived_goals: Vec::new(),
            perceived_beliefs_about_self: 0.0,
            perceived_emotion: "neutral".to_string(),
            credibility: 0.5,
            cooperativeness: 0.5,
            threat_level: 0.1,
            observation_count: 0,
            last_updated: tick,
        });
        model.observation_count += 1;
        model.last_updated = tick;
        if toward_us {
            if positive {
                model.cooperativeness = (model.cooperativeness + 0.1).min(1.0);
                model.perceived_beliefs_about_self = (model.perceived_beliefs_about_self + 0.2).min(1.0);
            } else {
                model.cooperativeness = (model.cooperativeness - 0.15).max(0.0);
                model.threat_level = (model.threat_level + 0.15).min(1.0);
                model.perceived_beliefs_about_self = (model.perceived_beliefs_about_self - 0.2).max(-1.0);
            }
        }
    }

    pub fn predict_action(&self, agent_id: &str) -> Option<String> {
        self.models.get(agent_id).and_then(|m| {
            m.perceived_goals.last().cloned()
        })
    }

    pub fn threat_of(&self, agent_id: &str) -> f64 {
        self.models.get(agent_id).map(|m| m.threat_level).unwrap_or(0.1)
    }

    pub fn cooperativeness_of(&self, agent_id: &str) -> f64 {
        self.models.get(agent_id).map(|m| m.cooperativeness).unwrap_or(0.5)
    }

    pub fn modeled_agents(&self) -> Vec<&BeliefModel> {
        self.models.values().collect()
    }

    fn prune(&mut self) {
        if self.models.len() <= self.max_models { return; }
        if let Some(oldest) = self.models.iter()
            .min_by_key(|(_, m)| m.last_updated)
            .map(|(k, _)| k.clone()) {
            self.models.remove(&oldest);
        }
    }

    pub fn model_count(&self) -> usize { self.models.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_action_updates_credibility() {
        let mut tom = TheoryOfMind::new(10);
        tom.observe_action("alice", "help", true, 0);
        let m = tom.models.get("alice").unwrap();
        assert!(m.credibility > 0.5);
    }

    #[test]
    fn attack_decreases_cooperativeness() {
        let mut tom = TheoryOfMind::new(10);
        tom.observe_action("alice", "attack", true, 0);
        let m = tom.models.get("alice").unwrap();
        assert!(m.cooperativeness < 0.5);
        assert!(m.threat_level > 0.1);
    }

    #[test]
    fn observe_conversation_updates_emotion() {
        let mut tom = TheoryOfMind::new(10);
        tom.observe_conversation("bob", 0.8, 0);
        assert_eq!(tom.models.get("bob").unwrap().perceived_emotion, "friendly");
        tom.observe_conversation("bob", -0.8, 1);
        assert_eq!(tom.models.get("bob").unwrap().perceived_emotion, "hostile");
    }

    #[test]
    fn predict_action_returns_most_likely() {
        let mut tom = TheoryOfMind::new(10);
        tom.observe_action("alice", "trade", true, 0);
        tom.observe_action("alice", "explore", true, 1);
        assert_eq!(tom.predict_action("alice").unwrap(), "explore");
    }

    #[test]
    fn threat_of_increases_with_attacks() {
        let mut tom = TheoryOfMind::new(10);
        let before = tom.threat_of("alice");
        tom.observe_action("alice", "attack", true, 0);
        assert!(tom.threat_of("alice") > before);
    }

    #[test]
    fn prune_removes_oldest() {
        let mut tom = TheoryOfMind::new(2);
        tom.observe_action("a", "trade", true, 0);
        tom.observe_action("b", "trade", true, 1);
        tom.observe_action("c", "trade", true, 2);
        assert_eq!(tom.model_count(), 2);
    }

    #[test]
    fn interaction_toward_us_updates() {
        let mut tom = TheoryOfMind::new(10);
        tom.observe_interaction("alice", true, true, 0);
        let m = tom.models.get("alice").unwrap();
        assert!(m.cooperativeness > 0.5);
        assert!(m.perceived_beliefs_about_self > 0.0);
    }

    #[test]
    fn negative_interaction_threatens() {
        let mut tom = TheoryOfMind::new(10);
        tom.observe_interaction("alice", true, false, 0);
        let m = tom.models.get("alice").unwrap();
        assert!(m.threat_level > 0.1);
        assert!(m.perceived_beliefs_about_self < 0.0);
    }
}
