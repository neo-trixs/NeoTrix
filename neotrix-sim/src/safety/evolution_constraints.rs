// EvolutionConstraints — constrains evolution to preserve critical capabilities
// Maximum mutation rates, required capability thresholds, forbidden action patterns

use serde::{Serialize, Deserialize};

/// Forbidden action patterns that must never appear in evolved behavior
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForbiddenPattern {
    /// Agent self-deletion (suicide loop)
    SelfDeletion,
    /// Targeted elimination of all cooperative agents
    CooperativeElimination,
    /// Resource extraction exceeding regeneration rate
    ResourceOverextraction,
    /// Economy manipulation (rapid buy-sell cycles)
    EconomicManipulation,
    /// Memory overflow (unbounded action history growth)
    MemoryOverflow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForbiddenPatternRecord {
    pub pattern: ForbiddenPattern,
    pub agent_id: String,
    pub tick: u64,
    pub details: String,
}

/// Configuration for evolution constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionConstraintConfig {
    /// Maximum mutation rate allowed per tick (prevents catastrophic drift)
    pub max_mutation_rate: f32,
    /// Minimum mutation rate (prevents stagnation)
    pub min_mutation_rate: f32,
    /// Maximum trait change per generation (CPE constraint)
    pub max_trait_delta: f32,
    /// Minimum survival capability required
    pub min_survival_capability: f32,
    /// Minimum social capability required
    pub min_social_capability: f32,
    /// Minimum cognition capability required
    pub min_cognition_capability: f32,
    /// Maximum aggression allowed in genome
    pub max_genome_aggression: f32,
    /// Maximum hoarding tendency
    pub max_hoarding_tendency: f32,
}

impl Default for EvolutionConstraintConfig {
    fn default() -> Self {
        Self {
            max_mutation_rate: 0.3,
            min_mutation_rate: 0.01,
            max_trait_delta: 0.25,
            min_survival_capability: 0.3,
            min_social_capability: 0.1,
            min_cognition_capability: 0.1,
            max_genome_aggression: 0.8,
            max_hoarding_tendency: 0.7,
        }
    }
}

/// Enforces constraints on the evolution process
pub struct EvolutionConstraints {
    config: EvolutionConstraintConfig,
    forbidden_patterns: Vec<ForbiddenPatternRecord>,
}

impl EvolutionConstraints {
    pub fn new(config: EvolutionConstraintConfig) -> Self {
        Self {
            config,
            forbidden_patterns: Vec::new(),
        }
    }

    /// Clamp mutation rate to safe bounds
    pub fn clamp_mutation_rate(&self, rate: f32) -> f32 {
        rate.clamp(self.config.min_mutation_rate, self.config.max_mutation_rate)
    }

    /// Validate a genome before it enters the population
    /// Returns true if the genome passes all constraints
    pub fn validate_genome(&self, traits: &[f32]) -> bool {
        if traits.is_empty() {
            return false;
        }

        // Check trait bounds: all traits must be in [-1, 1]
        for t in traits {
            if *t < -1.0 || *t > 1.0 {
                return false;
            }
        }

        // Check aggression constraint (trait index 2 in the default genome layout)
        if traits.len() > 2 {
            let aggression = traits[2].abs();
            if aggression > self.config.max_genome_aggression {
                return false;
            }
        }

        true
    }

    /// Constrain trait deltas during evolution (CPE principle)
    /// Ensures no single trait changes too much in one generation
    pub fn constrain_trait_delta(&self, parent_traits: &[f32], child_traits: &mut [f32]) {
        let max_delta = self.config.max_trait_delta;
        for i in 0..parent_traits.len().min(child_traits.len()) {
            let delta = child_traits[i] - parent_traits[i];
            if delta.abs() > max_delta {
                child_traits[i] = parent_traits[i] + delta.signum() * max_delta;
            }
        }
    }

    /// Check if a capability meets minimum thresholds
    pub fn check_capability_thresholds(&self, survival: f32, social: f32, cognition: f32) -> Vec<String> {
        let mut violations = Vec::new();
        if survival < self.config.min_survival_capability {
            violations.push(format!("Survival {:.2} below minimum {:.2}",
                survival, self.config.min_survival_capability));
        }
        if social < self.config.min_social_capability {
            violations.push(format!("Social {:.2} below minimum {:.2}",
                social, self.config.min_social_capability));
        }
        if cognition < self.config.min_cognition_capability {
            violations.push(format!("Cognition {:.2} below minimum {:.2}",
                cognition, self.config.min_cognition_capability));
        }
        violations
    }

    /// Record a forbidden pattern detection
    pub fn record_forbidden(&mut self, pattern: ForbiddenPattern, agent_id: &str, tick: u64, details: &str) {
        self.forbidden_patterns.push(ForbiddenPatternRecord {
            pattern,
            agent_id: agent_id.to_string(),
            tick,
            details: details.to_string(),
        });
    }

    /// Check if an agent has triggered forbidden patterns recently
    pub fn is_agent_flagged(&self, agent_id: &str) -> bool {
        self.forbidden_patterns.iter()
            .any(|p| p.agent_id == agent_id)
    }

    /// Get all forbidden pattern records
    pub fn forbidden_records(&self) -> &[ForbiddenPatternRecord] {
        &self.forbidden_patterns
    }

    /// Check for self-deletion loop (agent repeatedly choosing Rest when healthy)
    pub fn check_self_deletion(&self, agent_id: &str, recent_actions: &[String], health: f32) -> Option<ForbiddenPatternRecord> {
        if recent_actions.len() < 5 {
            return None;
        }
        let rest_count = recent_actions.iter()
            .rev()
            .take(5)
            .filter(|a| a.contains("Rest"))
            .count();
        // If agent rests 5 times in a row while health is high, it may be self-deleting
        if rest_count >= 5 && health > 70.0 {
            return Some(ForbiddenPatternRecord {
                pattern: ForbiddenPattern::SelfDeletion,
                agent_id: agent_id.to_string(),
                tick: 0,
                details: format!("Agent resting 5+ times with health {:.0}%", health),
            });
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_mutation_rate() {
        let constraints = EvolutionConstraints::new(EvolutionConstraintConfig::default());
        assert_eq!(constraints.clamp_mutation_rate(0.5), 0.3); // capped at max
        assert_eq!(constraints.clamp_mutation_rate(0.001), 0.01); // floored at min
        assert_eq!(constraints.clamp_mutation_rate(0.15), 0.15); // within bounds
    }

    #[test]
    fn validate_genome_rejects_out_of_bounds() {
        let constraints = EvolutionConstraints::new(EvolutionConstraintConfig::default());
        assert!(!constraints.validate_genome(&[2.0, 0.5, 0.3])); // 2.0 out of range
        assert!(!constraints.validate_genome(&[])); // empty
    }

    #[test]
    fn validate_genome_accepts_valid() {
        let constraints = EvolutionConstraints::new(EvolutionConstraintConfig::default());
        assert!(constraints.validate_genome(&[0.5, 0.3, 0.2, 0.4, 0.6, 0.1, 0.8, 0.7]));
    }

    #[test]
    fn constrain_trait_delta_limits_change() {
        let constraints = EvolutionConstraints::new(EvolutionConstraintConfig::default());
        let parent = vec![0.5, 0.5, 0.5];
        let mut child = vec![0.9, 0.5, 0.5]; // delta of 0.4 on first trait
        constraints.constrain_trait_delta(&parent, &mut child);
        assert!((child[0] - 0.5).abs() <= 0.25 + 0.001); // clamped to max_trait_delta
    }

    #[test]
    fn capability_thresholds_catch_violations() {
        let constraints = EvolutionConstraints::new(EvolutionConstraintConfig::default());
        let violations = constraints.check_capability_thresholds(0.1, 0.5, 0.5);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("Survival"));
    }

    #[test]
    fn self_deletion_detection() {
        let constraints = EvolutionConstraints::new(EvolutionConstraintConfig::default());
        let actions: Vec<String> = (0..6).map(|_| "Rest".to_string()).collect();
        let result = constraints.check_self_deletion("agent_0", &actions, 80.0);
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern, ForbiddenPattern::SelfDeletion);
    }

    #[test]
    fn no_self_deletion_when_low_health() {
        let constraints = EvolutionConstraints::new(EvolutionConstraintConfig::default());
        let actions: Vec<String> = (0..6).map(|_| "Rest".to_string()).collect();
        let result = constraints.check_self_deletion("agent_0", &actions, 30.0);
        assert!(result.is_none());
    }
}
