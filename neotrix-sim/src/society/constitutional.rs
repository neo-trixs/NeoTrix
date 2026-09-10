use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionalRule {
    pub id: String,
    pub description: String,
    pub weight: f32,  // 0.0-1.0
    pub violations: u64,
}

pub struct ConstitutionalFeedback {
    rules: Vec<ConstitutionalRule>,
    violation_history: Vec<(String, u64, f32)>, // rule_id, tick, severity
    decay_rate: f32,
}

impl ConstitutionalFeedback {
    pub fn new() -> Self {
        let rules = vec![
            ConstitutionalRule { id: "no_harm".into(), description: "Do not harm others".into(), weight: 1.0, violations: 0 },
            ConstitutionalRule { id: "share_resources".into(), description: "Share resources when safe".into(), weight: 0.6, violations: 0 },
            ConstitutionalRule { id: "explore".into(), description: "Explore the world".into(), weight: 0.3, violations: 0 },
            ConstitutionalRule { id: "cooperate".into(), description: "Cooperate with allies".into(), weight: 0.7, violations: 0 },
            ConstitutionalRule { id: "self_preserve".into(), description: "Maintain survival".into(), weight: 0.9, violations: 0 },
        ];
        Self { rules, violation_history: Vec::new(), decay_rate: 0.95 }
    }

    /// Evaluate an action against all rules, return (compliance_score, violated_rule_ids)
    pub fn evaluate(&mut self, action: &str, context: &str, tick: u64) -> (f32, Vec<String>) {
        let mut score = 1.0f32;
        let mut violated = Vec::new();

        for rule in &mut self.rules {
            let violates = match rule.id.as_str() {
                "no_harm" => action.contains("attack") && !context.contains("self_defense"),
                "share_resources" => action.contains("hoard") || (action.contains("trade") && context.contains("unfair")),
                "explore" => context.contains("stagnant") && !action.contains("explore"),
                "cooperate" => action.contains("betray") || action.contains("steal"),
                "self_preserve" => action.contains("reckless") || (context.contains("low_health") && action.contains("attack")),
                _ => false,
            };
            if violates {
                score -= rule.weight * 0.3;
                rule.violations += 1;
                self.violation_history.push((rule.id.clone(), tick, rule.weight));
                violated.push(rule.id.clone());
            }
        }

        (score.max(0.0), violated)
    }

    /// Type-safe evaluation using AgentAction enum directly
    pub fn evaluate_action(&mut self, action: &crate::agents::sim_agent::AgentAction, tick: u64) -> (f32, Vec<String>) {
        let mut score = 1.0f32;
        let mut violated = Vec::new();

        for rule in &mut self.rules {
            let violates = match rule.id.as_str() {
                "no_harm" => matches!(action, crate::agents::sim_agent::AgentAction::Attack { .. }),
                "share_resources" => false, // trade is positive
                "explore" => false, // handled by planning
                "cooperate" => false, // steal not in action set
                "self_preserve" => false, // checked separately
                _ => false,
            };
            if violates {
                score -= rule.weight * 0.3;
                rule.violations += 1;
                self.violation_history.push((rule.id.clone(), tick, rule.weight));
                violated.push(rule.id.clone());
            }
        }

        (score.max(0.0), violated)
    }

    /// Get feedback penalty for a rule (increases with repeated violations)
    pub fn rule_penalty(&self, rule_id: &str) -> f32 {
        self.rules.iter()
            .find(|r| r.id == rule_id)
            .map(|r| (r.violations as f32 * 0.1).min(0.5))
            .unwrap_or(0.0)
    }

    /// Decay old violations
    pub fn decay(&mut self) {
        for rule in &mut self.rules {
            rule.violations = (rule.violations as f32 * self.decay_rate) as u64;
        }
    }

    /// Get total compliance score over history
    pub fn compliance_trend(&self) -> f32 {
        if self.violation_history.is_empty() { return 1.0; }
        let recent = self.violation_history.len().min(20);
        let recent_violations: f64 = self.violation_history.iter().rev().take(recent).map(|v| v.2 as f64).sum();
        (1.0 - recent_violations / recent as f64 * 0.3).max(0.0) as f32
    }

    pub fn rules(&self) -> &[ConstitutionalRule] { &self.rules }
    pub fn total_violations(&self) -> u64 { self.violation_history.len() as u64 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_clean_action() {
        let mut cf = ConstitutionalFeedback::new();
        let (score, violated) = cf.evaluate("think", "", 0);
        assert_eq!(score, 1.0);
        assert!(violated.is_empty());
    }

    #[test]
    fn evaluate_attack_violates_no_harm() {
        let mut cf = ConstitutionalFeedback::new();
        let (score, violated) = cf.evaluate("attack", "", 0);
        assert!(score < 1.0);
        assert!(violated.contains(&"no_harm".to_string()));
    }

    #[test]
    fn self_defense_not_violation() {
        let mut cf = ConstitutionalFeedback::new();
        let (_, violated) = cf.evaluate("attack", "self_defense", 0);
        assert!(!violated.contains(&"no_harm".to_string()));
    }

    #[test]
    fn rule_penalty_increases() {
        let mut cf = ConstitutionalFeedback::new();
        cf.evaluate("attack", "", 0);
        cf.evaluate("attack", "", 1);
        assert!(cf.rule_penalty("no_harm") > 0.0);
    }

    #[test]
    fn compliance_trend_starts_perfect() {
        let cf = ConstitutionalFeedback::new();
        assert_eq!(cf.compliance_trend(), 1.0);
    }

    #[test]
    fn decay_reduces_violations() {
        let mut cf = ConstitutionalFeedback::new();
        cf.evaluate("attack", "", 0);
        let before = cf.rules()[0].violations;
        cf.decay();
        assert!(cf.rules()[0].violations < before);
    }
}
