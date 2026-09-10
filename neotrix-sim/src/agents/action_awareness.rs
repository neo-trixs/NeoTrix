use serde::{Deserialize, Serialize};

use super::sim_agent::{AgentAction, SimAgent};
use crate::foundation::math_bridge::Vec2;

/// What the agent expects will happen after executing an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedOutcome {
    pub action: AgentAction,
    pub expected_position: Option<Vec2>,
    pub expected_energy_delta: f32,
    pub expected_health_delta: f32,
    pub expected_resource_change: Option<(String, f32)>,
}

/// What actually happened after executing an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservedOutcome {
    pub actual_position: Vec2,
    pub actual_energy: f32,
    pub actual_health: f32,
    pub success: bool,
}

/// A record of a mismatch between expected and observed outcomes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discrepancy {
    pub action: AgentAction,
    pub expected: ExpectedOutcome,
    pub observed: ObservedOutcome,
    pub magnitude: f64,
    pub tick: u64,
}

/// Per-agent action awareness: compares expected vs observed outcomes
/// to detect hallucination loops and enable learning from mistakes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionAwareness {
    pub last_expected: Option<ExpectedOutcome>,
    pub last_observed: Option<ObservedOutcome>,
    pub discrepancy_history: Vec<Discrepancy>,
    pub world_model_confidence: f64,
}

impl Default for ActionAwareness {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionAwareness {
    pub fn new() -> Self {
        Self {
            last_expected: None,
            last_observed: None,
            discrepancy_history: Vec::new(),
            world_model_confidence: 0.8,
        }
    }

    /// Predict expected outcome before executing an action (rule-based).
    pub fn predict(&mut self, action: &AgentAction, agent: &SimAgent) {
        let expected = match action {
            AgentAction::Move { target } => {
                let dir = (*target - agent.core.position).normalize();
                let speed = 5.0;
                let new_pos = agent.core.position + dir * speed;
                ExpectedOutcome {
                    action: action.clone(),
                    expected_position: Some(Vec2::new(
                        new_pos.x.clamp(0.0, 1000.0),
                        new_pos.y.clamp(0.0, 1000.0),
                    )),
                    expected_energy_delta: -0.5,
                    expected_health_delta: 0.0,
                    expected_resource_change: None,
                }
            }
            AgentAction::Eat { resource_id } => ExpectedOutcome {
                action: action.clone(),
                expected_position: None,
                expected_energy_delta: 10.0,
                expected_health_delta: 0.0,
                expected_resource_change: Some((resource_id.clone(), -20.0)),
            },
            AgentAction::Rest => ExpectedOutcome {
                action: action.clone(),
                expected_position: None,
                expected_energy_delta: 5.0,
                expected_health_delta: 1.0,
                expected_resource_change: None,
            },
            AgentAction::Explore { direction } => {
                let dir = direction.normalize() * 10.0;
                let new_pos = agent.core.position + dir;
                ExpectedOutcome {
                    action: action.clone(),
                    expected_position: Some(Vec2::new(
                        new_pos.x.clamp(0.0, 1000.0),
                        new_pos.y.clamp(0.0, 1000.0),
                    )),
                    expected_energy_delta: -0.5,
                    expected_health_delta: 0.0,
                    expected_resource_change: None,
                }
            }
            AgentAction::Talk { .. } => ExpectedOutcome {
                action: action.clone(),
                expected_position: None,
                expected_energy_delta: 0.0,
                expected_health_delta: 0.0,
                expected_resource_change: None,
            },
            AgentAction::Attack { .. } => ExpectedOutcome {
                action: action.clone(),
                expected_position: None,
                expected_energy_delta: -2.0,
                expected_health_delta: -1.0,
                expected_resource_change: None,
            },
            AgentAction::Harvest { .. } => ExpectedOutcome {
                action: action.clone(),
                expected_position: None,
                expected_energy_delta: -1.0,
                expected_health_delta: 0.0,
                expected_resource_change: None,
            },
            AgentAction::Trade { .. } => ExpectedOutcome {
                action: action.clone(),
                expected_position: None,
                expected_energy_delta: 0.0,
                expected_health_delta: 0.0,
                expected_resource_change: None,
            },
            AgentAction::Build { .. } => ExpectedOutcome {
                action: action.clone(),
                expected_position: None,
                expected_energy_delta: -3.0,
                expected_health_delta: 0.0,
                expected_resource_change: None,
            },
            AgentAction::Think => ExpectedOutcome {
                action: action.clone(),
                expected_position: None,
                expected_energy_delta: -0.2,
                expected_health_delta: 0.0,
                expected_resource_change: None,
            },
        };

        self.last_expected = Some(expected);
    }

    /// Verify expected vs observed outcome after action execution.
    pub fn verify(&mut self, agent: &SimAgent, tick: u64) {
        let observed = ObservedOutcome {
            actual_position: agent.core.position,
            actual_energy: agent.core.energy,
            actual_health: agent.core.health,
            success: agent.core.alive,
        };
        self.last_observed = Some(observed.clone());

        if let Some(expected) = &self.last_expected {
            let magnitude = self.compute_magnitude(expected, &observed);
            if magnitude > 0.01 {
                let discrepancy = Discrepancy {
                    action: expected.action.clone(),
                    expected: expected.clone(),
                    observed,
                    magnitude,
                    tick,
                };
                self.discrepancy_history.push(discrepancy);
                // Keep history bounded
                if self.discrepancy_history.len() > 100 {
                    self.discrepancy_history.remove(0);
                }
            }
        }
    }

    /// Compute discrepancy magnitude (0.0 = perfect prediction, higher = worse).
    fn compute_magnitude(&self, expected: &ExpectedOutcome, observed: &ObservedOutcome) -> f64 {
        let mut error = 0.0_f64;

        // Position error
        if let Some(exp_pos) = expected.expected_position {
            let dx = (exp_pos.x - observed.actual_position.x) as f64;
            let dy = (exp_pos.y - observed.actual_position.y) as f64;
            error += (dx * dx + dy * dy).sqrt();
        }

        // Success mismatch (action failed but expected success)
        if !observed.success {
            error += 5.0;
        }

        error
    }

    /// World model confidence based on recent discrepancies.
    pub fn confidence(&self) -> f64 {
        self.world_model_confidence
    }

    /// Update confidence based on discrepancy history.
    /// More recent discrepancies have higher weight.
    pub fn learn(&mut self) {
        if self.discrepancy_history.is_empty() {
            self.world_model_confidence = (self.world_model_confidence + 0.01).min(1.0);
            return;
        }

        let now = self.discrepancy_history.last().map(|d| d.tick).unwrap_or(0);
        let mut weighted_error = 0.0_f64;
        let mut weight_sum = 0.0_f64;

        for disc in &self.discrepancy_history {
            let age = (now - disc.tick) as f64 + 1.0;
            let weight = 1.0 / age;
            weighted_error += disc.magnitude * weight;
            weight_sum += weight;
        }

        let avg_error = if weight_sum > 0.0 {
            weighted_error / weight_sum
        } else {
            0.0
        };

        // Map error to confidence: higher error → lower confidence
        let new_confidence = (1.0 - (avg_error / 10.0).min(1.0)).max(0.1);
        // Smooth update
        self.world_model_confidence = self.world_model_confidence * 0.9 + new_confidence * 0.1;
    }

    /// Returns true if the agent should explore more due to low confidence.
    pub fn should_explore(&self) -> bool {
        self.world_model_confidence < 0.5
    }

    /// Get the last discrepancy magnitude, if any.
    pub fn last_discrepancy_magnitude(&self) -> Option<f64> {
        self.discrepancy_history.last().map(|d| d.magnitude)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::math_bridge::Vec2;

    fn make_agent(x: f32, y: f32, energy: f32, health: f32) -> SimAgent {
        let mut agent = SimAgent::new(1, Vec2::new(x, y));
        agent.core.energy = energy;
        agent.core.health = health;
        agent
    }

    #[test]
    fn new_has_default_confidence() {
        let aa = ActionAwareness::new();
        assert_eq!(aa.confidence(), 0.8);
        assert!(aa.last_expected.is_none());
        assert!(aa.last_observed.is_none());
        assert!(aa.discrepancy_history.is_empty());
    }

    #[test]
    fn predict_rest_sets_expected() {
        let mut aa = ActionAwareness::new();
        let agent = make_agent(50.0, 50.0, 50.0, 80.0);
        aa.predict(&AgentAction::Rest, &agent);
        let exp = aa.last_expected.as_ref().unwrap();
        assert_eq!(exp.expected_energy_delta, 5.0);
        assert_eq!(exp.expected_health_delta, 1.0);
        assert!(exp.expected_position.is_none());
    }

    #[test]
    fn predict_move_sets_expected_position() {
        let mut aa = ActionAwareness::new();
        let agent = make_agent(50.0, 50.0, 80.0, 100.0);
        let target = Vec2::new(100.0, 100.0);
        aa.predict(&AgentAction::Move { target }, &agent);
        let exp = aa.last_expected.as_ref().unwrap();
        assert!(exp.expected_position.is_some());
    }

    #[test]
    fn verify_records_observed_and_creates_no_discrepancy_for_rest() {
        let mut aa = ActionAwareness::new();
        let mut agent = make_agent(50.0, 50.0, 50.0, 80.0);
        aa.predict(&AgentAction::Rest, &agent);
        // Simulate rest effect
        agent.core.rest(5.0);
        aa.verify(&agent, 100);
        assert!(aa.last_observed.is_some());
        // Rest is predictable (no position, success=true), so no discrepancy
        assert!(aa.discrepancy_history.is_empty());
    }

    #[test]
    fn verify_catches_discrepancy_on_death() {
        let mut aa = ActionAwareness::new();
        let mut agent = make_agent(50.0, 50.0, 50.0, 80.0);
        aa.predict(&AgentAction::Eat { resource_id: "r1".into() }, &agent);
        // Simulate agent dying during action
        agent.core.alive = false;
        aa.verify(&agent, 100);
        assert!(!aa.discrepancy_history.is_empty());
        assert!(aa.discrepancy_history[0].magnitude >= 5.0);
    }

    #[test]
    fn learn_increases_confidence_when_no_discrepancies() {
        let mut aa = ActionAwareness::new();
        aa.world_model_confidence = 0.5;
        aa.learn();
        assert!(aa.confidence() > 0.5);
    }

    #[test]
    fn learn_decreases_confidence_with_errors() {
        let mut aa = ActionAwareness::new();
        aa.world_model_confidence = 0.9;
        // Add a large discrepancy
        aa.discrepancy_history.push(Discrepancy {
            action: AgentAction::Rest,
            expected: ExpectedOutcome {
                action: AgentAction::Rest,
                expected_position: None,
                expected_energy_delta: 5.0,
                expected_health_delta: 1.0,
                expected_resource_change: None,
            },
            observed: ObservedOutcome {
                actual_position: Vec2::new(0.0, 0.0),
                actual_energy: 10.0,
                actual_health: 10.0,
                success: false,
            },
            magnitude: 8.0,
            tick: 1,
        });
        aa.learn();
        assert!(aa.confidence() < 0.9);
    }

    #[test]
    fn should_explore_when_low_confidence() {
        let mut aa = ActionAwareness::new();
        aa.world_model_confidence = 0.3;
        assert!(aa.should_explore());
        aa.world_model_confidence = 0.7;
        assert!(!aa.should_explore());
    }

    #[test]
    fn discrepancy_history_bounded() {
        let mut aa = ActionAwareness::new();
        let agent = make_agent(50.0, 50.0, 50.0, 80.0);
        for i in 0..120 {
            // Move creates position discrepancy when agent doesn't actually move
            aa.predict(&AgentAction::Move { target: Vec2::new(200.0, 200.0) }, &agent);
            // Agent position doesn't change → discrepancy
            aa.verify(&agent, i);
        }
        assert!(aa.discrepancy_history.len() <= 100);
    }

    #[test]
    fn last_discrepancy_magnitude_none_when_empty() {
        let aa = ActionAwareness::new();
        assert!(aa.last_discrepancy_magnitude().is_none());
    }

    #[test]
    fn compute_magnitude_zero_for_perfect_prediction() {
        let aa = ActionAwareness::new();
        let exp = ExpectedOutcome {
            action: AgentAction::Rest,
            expected_position: Some(Vec2::new(50.0, 50.0)),
            expected_energy_delta: 5.0,
            expected_health_delta: 1.0,
            expected_resource_change: None,
        };
        let obs = ObservedOutcome {
            actual_position: Vec2::new(50.0, 50.0),
            actual_energy: 55.0,
            actual_health: 81.0,
            success: true,
        };
        let mag = aa.compute_magnitude(&exp, &obs);
        assert!(mag < 0.01);
    }

    #[test]
    fn compute_magnitude_nonzero_for_position_mismatch() {
        let aa = ActionAwareness::new();
        let exp = ExpectedOutcome {
            action: AgentAction::Move { target: Vec2::new(100.0, 100.0) },
            expected_position: Some(Vec2::new(100.0, 100.0)),
            expected_energy_delta: -0.5,
            expected_health_delta: 0.0,
            expected_resource_change: None,
        };
        let obs = ObservedOutcome {
            actual_position: Vec2::new(50.0, 50.0),
            actual_energy: 99.5,
            actual_health: 100.0,
            success: true,
        };
        let mag = aa.compute_magnitude(&exp, &obs);
        assert!(mag > 50.0);
    }

    #[test]
    fn compute_magnitude_penalty_for_failure() {
        let aa = ActionAwareness::new();
        let exp = ExpectedOutcome {
            action: AgentAction::Rest,
            expected_position: None,
            expected_energy_delta: 5.0,
            expected_health_delta: 1.0,
            expected_resource_change: None,
        };
        let obs = ObservedOutcome {
            actual_position: Vec2::new(50.0, 50.0),
            actual_energy: 0.0,
            actual_health: 0.0,
            success: false,
        };
        let mag = aa.compute_magnitude(&exp, &obs);
        assert!(mag >= 5.0);
    }

    #[test]
    fn predict_all_action_variants() {
        let mut aa = ActionAwareness::new();
        let agent = make_agent(50.0, 50.0, 80.0, 100.0);

        let actions = vec![
            AgentAction::Move { target: Vec2::new(100.0, 100.0) },
            AgentAction::Eat { resource_id: "r1".into() },
            AgentAction::Rest,
            AgentAction::Explore { direction: Vec2::new(1.0, 0.0) },
            AgentAction::Talk { target_id: "a1".into(), message: "hi".into() },
            AgentAction::Attack { target_id: "a2".into() },
            AgentAction::Harvest { resource_id: "r2".into() },
            AgentAction::Trade { target_id: "a3".into(), item: "food".into(), amount: 1 },
            AgentAction::Build { position: Vec2::new(60.0, 60.0), structure_type: "wall".into() },
            AgentAction::Think,
        ];

        for action in &actions {
            aa.predict(action, &agent);
            assert!(aa.last_expected.is_some());
        }
    }
}
