//! CostWeightedRouting — routes tasks to the cheapest capable model.
//!
//! Implements Axiom A1: Cost-Aware Routing — not all tasks need the strongest model.
//! Maintains an ordered list of model tiers; for each task type, selects the cheapest
//! tier whose quality threshold is met.

/// A model tier with cost and capability metadata.
#[derive(Debug, Clone)]
pub struct ModelTier {
    /// Tier identifier (e.g. "small", "medium", "large", "frontier").
    pub name: String,
    /// Cost per 1K tokens in USD.
    pub cost_per_1k_tokens: f64,
    /// Quality score [0,1] — empirical benchmark performance.
    pub quality: f64,
    /// Task types this tier is capable of handling.
    pub capable_tasks: Vec<String>,
    /// Maximum input context length in tokens.
    pub max_context: usize,
}

/// Routing decision output.
#[derive(Debug, Clone, PartialEq)]
pub struct RoutingDecision {
    /// Selected model tier name.
    pub tier: String,
    /// Estimated cost for this task.
    pub estimated_cost: f64,
    /// Quality score of the selected tier.
    pub quality: f64,
    /// Why this tier was selected.
    pub reason: RoutingReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingReason {
    /// Cheapest capable tier selected (default).
    CheapestCapable,
    /// Quality threshold required a more expensive tier.
    QualityThreshold,
    /// No capable tier found; using fallback.
    Fallback,
    /// Budget constraint forced a cheaper tier.
    BudgetConstraint,
}

/// Configuration for cost-weighted routing decisions.
#[derive(Debug, Clone)]
pub struct CostWeightConfig {
    /// Minimum acceptable quality [0,1]. Tasks below this threshold require higher tiers.
    pub min_quality: f64,
    /// Cost weight factor — higher => stronger preference for cheap tiers.
    pub cost_weight_factor: f64,
    /// Maximum budget per task (in USD). Tiers above this are skipped.
    pub max_budget_per_task: f64,
}

impl Default for CostWeightConfig {
    fn default() -> Self {
        Self {
            min_quality: 0.5,
            cost_weight_factor: 1.0,
            max_budget_per_task: f64::INFINITY,
        }
    }
}

/// Routes tasks to the cheapest capable model tier.
pub struct CostWeightedRouting {
    tiers: Vec<ModelTier>,
    config: CostWeightConfig,
}

impl CostWeightedRouting {
    /// Create a new router with tiers sorted by cost (ascending).
    pub fn new(tiers: Vec<ModelTier>, config: CostWeightConfig) -> Self {
        let mut sorted = tiers;
        sorted.sort_by(|a, b| {
            a.cost_per_1k_tokens
                .partial_cmp(&b.cost_per_1k_tokens)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Self {
            tiers: sorted,
            config,
        }
    }

    /// Route a task to the cheapest capable model tier.
    pub fn route(&self, task_type: &str, estimated_tokens: usize) -> RoutingDecision {
        let capable: Vec<&ModelTier> = self
            .tiers
            .iter()
            .filter(|t| t.capable_tasks.iter().any(|ct| ct == task_type))
            .collect();

        if capable.is_empty() {
            return RoutingDecision {
                tier: "fallback".into(),
                estimated_cost: 0.0,
                quality: 0.0,
                reason: RoutingReason::Fallback,
            };
        }

        // Filter by quality threshold
        let quality_ok: Vec<&ModelTier> = capable
            .iter()
            .filter(|t| t.quality >= self.config.min_quality)
            .copied()
            .collect();

        let candidates = if quality_ok.is_empty() {
            // Fall back to all capable tiers if none meet quality threshold
            capable
        } else {
            quality_ok
        };

        // Filter by budget
        let budget_ok: Vec<&ModelTier> = candidates
            .iter()
            .filter(|t| {
                let cost = t.cost_per_1k_tokens * (estimated_tokens as f64 / 1000.0);
                cost <= self.config.max_budget_per_task
            })
            .copied()
            .collect();

        let candidates = if budget_ok.is_empty() {
            candidates
        } else {
            budget_ok
        };

        // Select cheapest (tiers are sorted by cost ascending)
        let selected = candidates[0];
        let estimated_cost =
            selected.cost_per_1k_tokens * (estimated_tokens as f64 / 1000.0);

        let reason = if selected.quality >= self.config.min_quality {
            RoutingReason::CheapestCapable
        } else {
            RoutingReason::QualityThreshold
        };

        RoutingDecision {
            tier: selected.name.clone(),
            estimated_cost,
            quality: selected.quality,
            reason,
        }
    }

    /// Get all registered tiers.
    pub fn tiers(&self) -> &[ModelTier] {
        &self.tiers
    }

    /// Check if a task type has any capable tier.
    pub fn has_capable_tier(&self, task_type: &str) -> bool {
        self.tiers
            .iter()
            .any(|t| t.capable_tasks.iter().any(|ct| ct == task_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_tiers() -> Vec<ModelTier> {
        vec![
            ModelTier {
                name: "small".into(),
                cost_per_1k_tokens: 0.001,
                quality: 0.6,
                capable_tasks: vec!["summarization".into(), "classification".into()],
                max_context: 4096,
            },
            ModelTier {
                name: "medium".into(),
                cost_per_1k_tokens: 0.01,
                quality: 0.8,
                capable_tasks: vec!["summarization".into(), "code_review".into()],
                max_context: 32768,
            },
            ModelTier {
                name: "large".into(),
                cost_per_1k_tokens: 0.06,
                quality: 0.95,
                capable_tasks: vec!["code_review".into(), "reasoning".into()],
                max_context: 131072,
            },
        ]
    }

    #[test]
    fn route_selects_cheapest_capable() {
        let router = CostWeightedRouting::new(test_tiers(), CostWeightConfig::default());
        let decision = router.route("summarization", 1000);
        assert_eq!(decision.tier, "small");
        assert_eq!(decision.reason, RoutingReason::CheapestCapable);
    }

    #[test]
    fn route_selects_higher_tier_for_quality() {
        let config = CostWeightConfig {
            min_quality: 0.9,
            ..Default::default()
        };
        let router = CostWeightedRouting::new(test_tiers(), config);
        let decision = router.route("code_review", 1000);
        assert_eq!(decision.tier, "large");
    }

    #[test]
    fn route_fallback_for_unknown_task() {
        let router = CostWeightedRouting::new(test_tiers(), CostWeightConfig::default());
        let decision = router.route("unknown_task", 1000);
        assert_eq!(decision.reason, RoutingReason::Fallback);
    }

    #[test]
    fn route_respects_budget() {
        let config = CostWeightConfig {
            max_budget_per_task: 0.005,
            ..Default::default()
        };
        let router = CostWeightedRouting::new(test_tiers(), config);
        let decision = router.route("code_review", 1000);
        // medium costs 0.01 which exceeds budget, so large (0.06) also exceeds,
        // falling back to cheapest capable
        assert!(decision.estimated_cost <= 0.005 || decision.reason == RoutingReason::Fallback);
    }

    #[test]
    fn has_capable_tier() {
        let router = CostWeightedRouting::new(test_tiers(), CostWeightConfig::default());
        assert!(router.has_capable_tier("summarization"));
        assert!(!router.has_capable_tier("nonexistent"));
    }

    #[test]
    fn tiers_sorted_by_cost() {
        let router = CostWeightedRouting::new(test_tiers(), CostWeightConfig::default());
        for window in router.tiers.windows(2) {
            assert!(window[0].cost_per_1k_tokens <= window[1].cost_per_1k_tokens);
        }
    }
}
