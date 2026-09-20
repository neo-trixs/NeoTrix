//! GWT Router configuration — single source of truth for attention routing parameters.
//!
//! A1 axiom: Cost-Aware Routing — not all tasks need the strongest model.
//! Default weights are calibrated against empirical salience distributions.

/// Default total attention budget (units = arbitrary salience-weighted shares).
pub const DEFAULT_BUDGET: f64 = 1000.0;

/// Default salience component weights (urgency, importance, novelty, cost_weight).
pub const DEFAULT_SALIENCE_WEIGHTS: SalienceWeights = SalienceWeights {
    urgency: 0.35,
    importance: 0.30,
    novelty: 0.20,
    cost_weight: 0.15,
};

/// Default cost weighting factor — higher => stronger preference for cheap models.
pub const DEFAULT_COST_WEIGHT_FACTOR: f64 = 1.0;

/// Salience component weights — sum must equal 1.0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SalienceWeights {
    pub urgency: f64,
    pub importance: f64,
    pub novelty: f64,
    pub cost_weight: f64,
}

impl SalienceWeights {
    /// Validate that all weights are in [0,1] and sum to 1.0 (within epsilon).
    pub fn validate(&self) -> bool {
        let sum = self.urgency + self.importance + self.novelty + self.cost_weight;
        (sum - 1.0).abs() < 1e-6
            && self.urgency >= 0.0
            && self.importance >= 0.0
            && self.novelty >= 0.0
            && self.cost_weight >= 0.0
    }
}

impl Default for SalienceWeights {
    fn default() -> Self {
        DEFAULT_SALIENCE_WEIGHTS
    }
}

/// GWT routing configuration.
#[derive(Debug, Clone)]
pub struct GwtConfig {
    /// Total attention budget available per routing cycle.
    pub default_budget: f64,
    /// Salience component weights.
    pub salience_weights: SalienceWeights,
    /// Cost weight factor — multiplied with cost_weight to amplify/attenuate
    /// the cost-aware component. Higher => stronger preference for cheap models.
    pub cost_weight_factor: f64,
}

impl Default for GwtConfig {
    fn default() -> Self {
        Self {
            default_budget: DEFAULT_BUDGET,
            salience_weights: DEFAULT_SALIENCE_WEIGHTS,
            cost_weight_factor: DEFAULT_COST_WEIGHT_FACTOR,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = GwtConfig::default();
        assert!(cfg.salience_weights.validate());
        assert!(cfg.default_budget > 0.0);
        assert!(cfg.cost_weight_factor > 0.0);
    }

    #[test]
    fn salience_weights_validate() {
        let w = SalienceWeights {
            urgency: 0.5,
            importance: 0.3,
            novelty: 0.1,
            cost_weight: 0.1,
        };
        assert!(w.validate());
    }

    #[test]
    fn salience_weights_reject_invalid_sum() {
        let w = SalienceWeights {
            urgency: 0.5,
            importance: 0.3,
            novelty: 0.1,
            cost_weight: 0.3,
        };
        assert!(!w.validate());
    }

    #[test]
    fn salience_weights_reject_negative() {
        let w = SalienceWeights {
            urgency: -0.1,
            importance: 0.5,
            novelty: 0.3,
            cost_weight: 0.3,
        };
        assert!(!w.validate());
    }
}
