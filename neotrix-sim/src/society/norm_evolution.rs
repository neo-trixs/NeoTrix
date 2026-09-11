use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::agents::sim_agent::SimAgent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Norm {
    pub id: String,
    pub description: String,
    pub compliance_score: f32,
    pub enforcement_strength: f32,
    pub violations: u32,
    pub total_observations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormViolation {
    pub agent_id: String,
    pub norm_id: String,
    pub tick: u64,
    pub severity: f32,
}

pub struct NormEvolution {
    pub norms: Vec<Norm>,
    pub violation_history: Vec<NormViolation>,
    pub compliance_window: HashMap<String, Vec<f32>>,
    pub window_size: usize,
}

impl NormEvolution {
    pub fn new() -> Self {
        let norms = vec![
            Norm {
                id: "no_harm".into(),
                description: "Do not harm others without cause".into(),
                compliance_score: 0.8,
                enforcement_strength: 0.7,
                violations: 0,
                total_observations: 0,
            },
            Norm {
                id: "share_resources".into(),
                description: "Share surplus resources with community".into(),
                compliance_score: 0.6,
                enforcement_strength: 0.4,
                violations: 0,
                total_observations: 0,
            },
            Norm {
                id: "respect_territory".into(),
                description: "Respect faction territory boundaries".into(),
                compliance_score: 0.7,
                enforcement_strength: 0.5,
                violations: 0,
                total_observations: 0,
            },
            Norm {
                id: "honesty".into(),
                description: "Be truthful in communications".into(),
                compliance_score: 0.65,
                enforcement_strength: 0.3,
                violations: 0,
                total_observations: 0,
            },
            Norm {
                id: "cooperation".into(),
                description: "Cooperate with allies in need".into(),
                compliance_score: 0.75,
                enforcement_strength: 0.6,
                violations: 0,
                total_observations: 0,
            },
        ];

        Self {
            norms,
            violation_history: Vec::new(),
            compliance_window: HashMap::new(),
            window_size: 50,
        }
    }

    pub fn update(&mut self, agents: &[SimAgent]) {
        for norm in &mut self.norms {
            let mut compliance_sum = 0.0;
            let mut observed = 0;

            for agent in agents {
                if !agent.core.alive { continue; }
                observed += 1;

                let agent_compliance = match norm.id.as_str() {
                    "no_harm" => {
                        if agent.personality.aggression > 0.7 { 0.3 }
                        else if agent.personality.cooperativeness > 0.5 { 0.9 }
                        else { 0.6 }
                    }
                    "share_resources" => agent.personality.cooperativeness,
                    "respect_territory" => {
                        if agent.personality.openness > 0.7 { 0.5 }
                        else { 0.8 }
                    }
                    "honesty" => 1.0 - agent.personality.aggression * 0.3,
                    "cooperation" => agent.personality.cooperativeness,
                    _ => 0.5,
                };

                compliance_sum += agent_compliance;
            }

            if observed > 0 {
                let new_compliance = compliance_sum / observed as f32;
                let window = self.compliance_window.entry(norm.id.clone()).or_default();
                window.push(new_compliance);
                if window.len() > self.window_size {
                    window.remove(0);
                }
                norm.compliance_score = window.iter().sum::<f32>() / window.len() as f32;
                norm.total_observations = observed;
            }
        }
    }

    pub fn enforce(&self, agent: &mut SimAgent) {
        for norm in &self.norms {
            if norm.compliance_score < 0.4 && norm.enforcement_strength > 0.5 {
                agent.core.health -= norm.enforcement_strength * 0.5;
                agent.core.energy -= norm.enforcement_strength * 0.3;
            }
        }
    }

    pub fn evolve(&mut self) {
        for norm in &mut self.norms {
            if norm.total_observations == 0 { continue; }

            let violation_ratio = norm.violations as f32 / norm.total_observations.max(1) as f32;

            if norm.compliance_score > 0.8 && violation_ratio < 0.1 {
                norm.enforcement_strength = (norm.enforcement_strength * 0.95).max(0.1);
            } else if norm.compliance_score < 0.5 || violation_ratio > 0.3 {
                norm.enforcement_strength = (norm.enforcement_strength * 1.1).min(1.0);
            }

            norm.violations = (norm.violations as f32 * 0.9) as u32;
        }
    }

    pub fn record_violation(&mut self, agent_id: &str, norm_id: &str, severity: f32, tick: u64) {
        self.violation_history.push(NormViolation {
            agent_id: agent_id.to_string(),
            norm_id: norm_id.to_string(),
            tick,
            severity,
        });

        if let Some(norm) = self.norms.iter_mut().find(|n| n.id == norm_id) {
            norm.violations += 1;
        }

        if self.violation_history.len() > 200 {
            self.violation_history.remove(0);
        }
    }

    pub fn add_norm(&mut self, norm: Norm) {
        self.norms.push(norm);
    }

    pub fn get_norm(&self, id: &str) -> Option<&Norm> {
        self.norms.iter().find(|n| n.id == id)
    }

    pub fn overall_compliance(&self) -> f32 {
        if self.norms.is_empty() { return 1.0; }
        let sum: f32 = self.norms.iter().map(|n| n.compliance_score).sum();
        sum / self.norms.len() as f32
    }
}

impl Default for NormEvolution {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::math_bridge::Vec2;

    fn make_agent(id: u64, aggression: f32, coop: f32) -> SimAgent {
        let mut agent = SimAgent::new(id, Vec2::new(0.0, 0.0));
        agent.personality.aggression = aggression;
        agent.personality.cooperativeness = coop;
        agent
    }

    #[test]
    fn test_initial_norms() {
        let ne = NormEvolution::new();
        assert_eq!(ne.norms.len(), 5);
        assert!(ne.get_norm("no_harm").is_some());
    }

    #[test]
    fn test_update_compliance() {
        let mut ne = NormEvolution::new();
        let agents = vec![make_agent(0, 0.3, 0.8), make_agent(1, 0.5, 0.6)];
        ne.update(&agents);

        let no_harm = ne.get_norm("no_harm").unwrap();
        assert!(no_harm.compliance_score > 0.5);
    }

    #[test]
    fn test_record_violation() {
        let mut ne = NormEvolution::new();
        ne.record_violation("agent_0", "no_harm", 0.8, 10);
        assert_eq!(ne.violation_history.len(), 1);
        let norm = ne.get_norm("no_harm").unwrap();
        assert!(norm.violations > 0);
    }

    #[test]
    fn test_evolve_strengthens_weak_norms() {
        let mut ne = NormEvolution::new();
        ne.norms[0].compliance_score = 0.3;
        ne.norms[0].violations = 10;
        ne.norms[0].total_observations = 20;
        let before = ne.norms[0].enforcement_strength;
        ne.evolve();
        assert!(ne.norms[0].enforcement_strength >= before);
    }

    #[test]
    fn test_overall_compliance() {
        let ne = NormEvolution::new();
        let overall = ne.overall_compliance();
        assert!(overall > 0.0 && overall <= 1.0);
    }
}
