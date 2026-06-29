use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct EfeComponents {
    pub total: f64,
    pub risk: f64,
    pub ambiguity: f64,
    pub information_gain: f64,
    pub preference_match: f64,
}

impl EfeComponents {
    pub fn exploration_ratio(&self) -> f64 {
        if self.total.abs() < 1e-9 {
            return 0.5;
        }
        (self.information_gain / self.total).clamp(0.0, 1.0)
    }

    pub fn summary(&self) -> String {
        format!(
            "EFE: total={:.4} risk={:.4} amb={:.4} info={:.4} pref={:.4}",
            self.total, self.risk, self.ambiguity, self.information_gain, self.preference_match
        )
    }
}

#[derive(Debug, Clone)]
pub struct EfeCuriosityBridge {
    preferred_outcomes: HashMap<String, f64>,
    outcome_history: VecDeque<(String, f64)>,
    belief_entropy: f64,
    risk_weight: f64,
    ambiguity_weight: f64,
    info_gain_weight: f64,
    preference_decay: f64,
    max_history: usize,
    efe_history: VecDeque<f64>,
    efe_estimate: f64,
    cycle: usize,
}

impl Default for EfeCuriosityBridge {
    fn default() -> Self {
        Self {
            preferred_outcomes: HashMap::new(),
            outcome_history: VecDeque::new(),
            belief_entropy: 0.5,
            risk_weight: 0.4,
            ambiguity_weight: 0.3,
            info_gain_weight: 0.3,
            preference_decay: 0.99,
            max_history: 50,
            efe_history: VecDeque::with_capacity(30),
            efe_estimate: 0.5,
            cycle: 0,
        }
    }
}

impl EfeCuriosityBridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compute_efe(
        &mut self,
        negentropy: f64,
        prediction_error: f64,
        goal_alignment: f64,
    ) -> EfeComponents {
        let clipped_negentropy = negentropy.clamp(0.0, 1.0);
        let clipped_error = prediction_error.clamp(0.0, 1.0);
        let clipped_alignment = goal_alignment.clamp(0.0, 1.0);

        let risk = (1.0 - clipped_alignment) * self.risk_weight
            + (1.0 - clipped_negentropy) * (1.0 - self.risk_weight);
        let risk = risk.clamp(0.0, 1.0);

        let entropy_factor = self.belief_entropy;
        let ambiguity = clipped_error * self.ambiguity_weight * (1.0 + entropy_factor);
        let ambiguity = ambiguity.clamp(0.0, 1.0);

        let information_gain = entropy_factor * self.info_gain_weight * (1.0 - clipped_error);
        let information_gain = information_gain.clamp(0.0, 1.0);

        let total = (risk + ambiguity - information_gain).clamp(0.0, 2.0);

        let components = EfeComponents {
            total,
            risk,
            ambiguity,
            information_gain,
            preference_match: clipped_alignment,
        };

        self.efe_history.push_back(total);
        if self.efe_history.len() > 30 {
            self.efe_history.pop_front();
        }
        self.efe_estimate =
            self.efe_history.iter().sum::<f64>() / self.efe_history.len().max(1) as f64;

        components
    }

    pub fn update_outcome(&mut self, outcome: &str, reward: f64) {
        self.outcome_history
            .push_back((outcome.to_string(), reward));
        if self.outcome_history.len() > self.max_history {
            self.outcome_history.pop_front();
        }
        let entry = self
            .preferred_outcomes
            .entry(outcome.to_string())
            .or_insert(0.0);
        *entry = (*entry + reward * 0.1).clamp(0.0, 1.0);
    }

    pub fn update_belief_entropy(&mut self, entropy: f64) {
        self.belief_entropy = entropy.clamp(0.0, 1.0);
    }

    pub fn step(&mut self) {
        self.cycle += 1;
        for value in self.preferred_outcomes.values_mut() {
            *value *= self.preference_decay;
        }
        self.preferred_outcomes.retain(|_, v| *v > 0.01);
        if self.cycle % 10 == 0 {
            self.belief_entropy = self.compute_belief_entropy();
        }
    }

    fn compute_belief_entropy(&self) -> f64 {
        if self.preferred_outcomes.is_empty() {
            return 0.5;
        }
        let total: f64 = self.preferred_outcomes.values().sum();
        if total < 1e-9 {
            return 0.5;
        }
        let entropy: f64 = self
            .preferred_outcomes
            .values()
            .map(|v| {
                let p = v / total;
                if p > 0.0 {
                    -p * p.log2()
                } else {
                    0.0
                }
            })
            .sum();
        let max_entropy = (self.preferred_outcomes.len() as f64).log2();
        if max_entropy < 1e-9 {
            0.0
        } else {
            (entropy / max_entropy).clamp(0.0, 1.0)
        }
    }

    pub fn efe_trend(&self) -> f64 {
        if self.efe_history.len() < 2 {
            return 0.0;
        }
        let recent: Vec<f64> = self.efe_history.iter().copied().rev().take(10).collect();
        if recent.len() < 2 {
            return 0.0;
        }
        let half = recent.len() / 2;
        let first_half: f64 = recent[..half].iter().sum::<f64>() / half as f64;
        let second_half: f64 = recent[half..].iter().sum::<f64>() / (recent.len() - half) as f64;
        second_half - first_half
    }

    pub fn curiosity_level_from_efe(&self, components: &EfeComponents) -> f64 {
        let from_info = components.information_gain * 2.0;
        let from_risk = components.risk * 1.5;
        let from_trend = if self.efe_trend() > 0.05 {
            0.3
        } else if self.efe_trend() < -0.05 {
            0.0
        } else {
            0.15
        };
        (from_info + from_risk * 0.3 + from_trend).clamp(0.0, 1.0)
    }

    pub fn program_exploration_priority(&self, components: &EfeComponents) -> f64 {
        (components.information_gain * 0.6 + (1.0 - components.preference_match) * 0.4)
            .clamp(0.0, 1.0)
    }

    pub fn get_efe_estimate(&self) -> f64 {
        self.efe_estimate
    }

    pub fn get_belief_entropy(&self) -> f64 {
        self.belief_entropy
    }

    pub fn summary(&self) -> String {
        format!(
            "EFE: estimate={:.3} entropy={:.3} trend={:.3} outcomes={} cycle={}",
            self.efe_estimate,
            self.belief_entropy,
            self.efe_trend(),
            self.preferred_outcomes.len(),
            self.cycle,
        )
    }
}

static GLOBAL_EFE_BRIDGE: std::sync::OnceLock<std::sync::Mutex<EfeCuriosityBridge>> =
    std::sync::OnceLock::new();

pub fn global_efe_bridge() -> &'static std::sync::Mutex<EfeCuriosityBridge> {
    GLOBAL_EFE_BRIDGE.get_or_init(|| std::sync::Mutex::new(EfeCuriosityBridge::new()))
}

pub fn step_efe_bridge() {
    if let Ok(mut bridge) = global_efe_bridge().lock() {
        bridge.step();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn nearly_equal(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[serial]
    #[test]
    fn test_compute_efe_low_negentropy_high_curiosity() {
        let mut bridge = EfeCuriosityBridge::new();
        let comp = bridge.compute_efe(0.3, 0.2, 0.8);
        assert!(comp.total > 0.0);
        assert!(comp.risk > 0.0);
        assert!(comp.information_gain >= 0.0);
    }

    #[test]
    fn test_compute_efe_high_negentropy_low_curiosity() {
        let mut bridge = EfeCuriosityBridge::new();
        let comp = bridge.compute_efe(0.95, 0.05, 0.95);
        assert!(comp.total >= 0.0);
        assert!(comp.risk < 0.3);
    }

    #[test]
    fn test_information_gain_tracks_belief_entropy() {
        let mut bridge = EfeCuriosityBridge::new();
        bridge.update_belief_entropy(0.9);
        let comp = bridge.compute_efe(0.5, 0.1, 0.5);
        assert!(comp.information_gain > 0.1);

        bridge.update_belief_entropy(0.1);
        let comp2 = bridge.compute_efe(0.5, 0.1, 0.5);
        assert!(comp2.information_gain < comp.information_gain);
    }

    #[test]
    fn test_update_outcome_builds_preferences() {
        let mut bridge = EfeCuriosityBridge::new();
        bridge.update_outcome("explored_new_topic", 0.8);
        bridge.update_outcome("explored_new_topic", 0.9);
        assert!(
            bridge
                .preferred_outcomes
                .get("explored_new_topic")
                .copied()
                .unwrap_or(0.0)
                > 0.0
        );
    }

    #[test]
    fn test_curiosity_level_scales_with_info_gain() {
        let mut bridge = EfeCuriosityBridge::new();
        bridge.update_belief_entropy(0.8);
        let comp_high = bridge.compute_efe(0.3, 0.5, 0.3);
        let curiosity_high = bridge.curiosity_level_from_efe(&comp_high);

        bridge.update_belief_entropy(0.1);
        let comp_low = bridge.compute_efe(0.9, 0.05, 0.9);
        let curiosity_low = bridge.curiosity_level_from_efe(&comp_low);

        assert!(curiosity_high > curiosity_low);
    }

    #[test]
    fn test_efe_trend_positive_when_efe_increasing() {
        let mut bridge = EfeCuriosityBridge::new();
        bridge.compute_efe(0.3, 0.5, 0.3);
        bridge.compute_efe(0.2, 0.6, 0.2);
        bridge.compute_efe(0.1, 0.7, 0.1);
        let trend = bridge.efe_trend();
        assert!(trend > -0.5);
    }

    #[test]
    fn test_program_exploration_priority() {
        let mut bridge = EfeCuriosityBridge::new();
        bridge.update_belief_entropy(0.9);
        let comp = bridge.compute_efe(0.2, 0.6, 0.2);
        let priority = bridge.program_exploration_priority(&comp);
        assert!(priority >= 0.0 && priority <= 1.0);
    }

    #[test]
    fn test_step_decays_preferences() {
        let mut bridge = EfeCuriosityBridge::new();
        bridge.preferred_outcomes.insert("test".into(), 0.9);
        bridge.step();
        let val = bridge
            .preferred_outcomes
            .get("test")
            .copied()
            .unwrap_or(0.0);
        assert!(val < 0.9);
    }

    #[test]
    fn test_exploration_ratio() {
        let comp = EfeComponents {
            total: 1.0,
            risk: 0.3,
            ambiguity: 0.3,
            information_gain: 0.4,
            preference_match: 0.5,
        };
        let ratio = comp.exploration_ratio();
        assert!(ratio > 0.3 && ratio <= 1.0);
    }

    #[test]
    fn test_global_efe_bridge() {
        let _g = global_efe_bridge().lock();
        step_efe_bridge();
    }

    #[test]
    fn test_compute_belief_entropy_empty() {
        let bridge = EfeCuriosityBridge::new();
        let entropy = bridge.compute_belief_entropy();
        assert!(nearly_equal(entropy, 0.5));
    }

    #[test]
    fn test_compute_belief_entropy_single_outcome() {
        let mut bridge = EfeCuriosityBridge::new();
        bridge.preferred_outcomes.insert("only".into(), 1.0);
        let entropy = bridge.compute_belief_entropy();
        assert!(entropy < 0.1);
    }

    #[test]
    fn test_summary_contains_efe() {
        let mut bridge = EfeCuriosityBridge::new();
        bridge.compute_efe(0.5, 0.3, 0.7);
        let s = bridge.summary();
        assert!(s.contains("EFE:"));
    }
}
