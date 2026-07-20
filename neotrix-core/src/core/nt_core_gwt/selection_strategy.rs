/// Unified trait for all specialist selection/competition strategies.
///
/// Consolidates 5 redundant mechanisms:
/// 1. `resonance.rs` — ResonanceMatrix::resonate_cycle() (Hamming dist / VSA cosine)
/// 2. `physics_attention.rs` — AdaptiveSlicer (Transolver-inspired)
/// 3. `competition_gate.rs` — CompetitionGate::compete() (WTA ignition)
/// 4. `moe_router.rs` — MoERouter::forward() (learned routing matrix)
/// 5. `workspace.rs` — e8_attention_weights (E8 bias)

/// Runtime state of a single specialist available for selection.
#[derive(Debug, Clone)]
pub struct SpecialistState {
    pub id: usize,
    pub activation: f64,
    pub salience: f64,
    pub resonance: f64,
}

/// Contextual signals that influence the selection decision.
#[derive(Debug, Clone)]
pub struct SelectionContext {
    /// Per-specialist E8 hexagram attention bias (one per specialist index).
    pub e8_attention_bias: Vec<f64>,
    /// Optional high-dimensional task embedding for content-aware routing.
    pub task_embedding: Option<Vec<f64>>,
    /// Minimum combined score required for ignition.
    pub threshold: f64,
}

/// Outcome of a single selection round.
#[derive(Debug, Clone)]
pub struct SelectionResult {
    /// Index (into the specialist slice) of the winning specialist.
    pub winner_id: usize,
    /// Combined ignition strength of the winner.
    pub ignition_strength: f64,
    /// Index of the second-place specialist, if any.
    pub runner_up_id: Option<usize>,
    /// Which strategy produced this result.
    pub strategy_used: &'static str,
}

/// Unified trait for all specialist selection/competition strategies.
pub trait SelectionStrategy: Send + Sync {
    fn name(&self) -> &'static str;
    fn select(&self, specialists: &[SpecialistState], context: &SelectionContext) -> SelectionResult;
}

/// Resonance-based selection strategy.
///
/// Computes a combined score for each specialist as:
/// `activation * (1.0 + resonance) * salience`, optionally modulated by
/// E8 attention bias from the context. The highest-scoring specialist
/// wins the competition (soft WTA).
///
/// This is a simplified version of the logic in `resonance.rs::resonate_cycle()`
/// that works with the generic `SpecialistState` slice instead of fixed arrays
/// of hexagram assignments.
pub struct ResonanceStrategy;

impl ResonanceStrategy {
    pub const fn new() -> Self {
        Self
    }
}

impl Default for ResonanceStrategy {
    fn default() -> Self {
        Self
    }
}

impl SelectionStrategy for ResonanceStrategy {
    fn name(&self) -> &'static str {
        "resonance"
    }

    fn select(&self, specialists: &[SpecialistState], context: &SelectionContext) -> SelectionResult {
        if specialists.is_empty() {
            return SelectionResult {
                winner_id: 0,
                ignition_strength: 0.0,
                runner_up_id: None,
                strategy_used: self.name(),
            };
        }

        // Step 1: compute combined score for each specialist
        // Combined = activation * (1.0 + resonance) * salience
        // The (1.0 + resonance) factor ensures that resonance is a multiplicative boost
        // rather than a requirement — a low-resonance specialist can still win on activation.
        let mut scores: Vec<f64> = specialists
            .iter()
            .map(|s| s.activation * (1.0 + s.resonance) * s.salience)
            .collect();

        // Step 2: apply E8 attention bias from context
        // Each specialist gets a multiplicative boost: score *= (1.0 + bias)
        // Bias is indexed by position in the slice, same as e8_attention_weights
        // in workspace.rs::resonant_broadcast().
        if !context.e8_attention_bias.is_empty() {
            for (i, score) in scores.iter_mut().enumerate() {
                if let Some(&bias) = context.e8_attention_bias.get(i) {
                    *score = (*score * (1.0 + bias)).max(0.0);
                }
            }
        }

        // Step 3: find winner (highest combined score)
        let winner_id = scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);

        let ignition_strength = scores[winner_id];

        // Step 4: find runner-up (second highest score)
        let runner_up_id = scores
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != winner_id)
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i);

        SelectionResult {
            winner_id,
            ignition_strength,
            runner_up_id,
            strategy_used: self.name(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_specialists() -> Vec<SpecialistState> {
        vec![
            SpecialistState { id: 0, activation: 0.8, salience: 0.7, resonance: 0.3 },
            SpecialistState { id: 1, activation: 0.6, salience: 0.9, resonance: 0.5 },
            SpecialistState { id: 2, activation: 0.9, salience: 0.4, resonance: 0.1 },
            SpecialistState { id: 3, activation: 0.2, salience: 0.3, resonance: 0.0 },
        ]
    }

    #[test]
    fn test_resonance_strategy_selects_highest_score() {
        let strategy = ResonanceStrategy::new();
        let specialists = sample_specialists();
        let context = SelectionContext {
            e8_attention_bias: vec![],
            task_embedding: None,
            threshold: 0.5,
        };

        let result = strategy.select(&specialists, &context);

        // specialist[0]: 0.8 * (1+0.3) * 0.7 = 0.728
        // specialist[1]: 0.6 * (1+0.5) * 0.9 = 0.810 ← winner
        // specialist[2]: 0.9 * (1+0.1) * 0.4 = 0.396
        // specialist[3]: 0.2 * (1+0.0) * 0.3 = 0.060
        assert_eq!(result.winner_id, 1);
        assert!((result.ignition_strength - 0.81).abs() < 1e-9);
    }

    #[test]
    fn test_resonance_strategy_runner_up() {
        let strategy = ResonanceStrategy::new();
        let specialists = sample_specialists();
        let context = SelectionContext {
            e8_attention_bias: vec![],
            task_embedding: None,
            threshold: 0.5,
        };

        let result = strategy.select(&specialists, &context);

        // Winner is index 1 (0.81), runner-up should be index 0 (0.728)
        assert_eq!(result.runner_up_id, Some(0));
    }

    #[test]
    fn test_empty_specialists_returns_default() {
        let strategy = ResonanceStrategy::new();
        let specialists = vec![];
        let context = SelectionContext {
            e8_attention_bias: vec![],
            task_embedding: None,
            threshold: 0.0,
        };

        let result = strategy.select(&specialists, &context);

        assert_eq!(result.winner_id, 0);
        assert_eq!(result.ignition_strength, 0.0);
        assert_eq!(result.runner_up_id, None);
    }

    #[test]
    fn test_e8_attention_bias_flips_winner() {
        let strategy = ResonanceStrategy::new();
        let specialists = vec![
            SpecialistState { id: 0, activation: 0.5, salience: 0.5, resonance: 0.0 },
            SpecialistState { id: 1, activation: 0.6, salience: 0.5, resonance: 0.0 },
        ];

        // Without bias: index 1 wins (0.6 * 1.0 * 0.5 = 0.3 > 0.25)
        let context_neutral = SelectionContext {
            e8_attention_bias: vec![],
            task_embedding: None,
            threshold: 0.0,
        };
        let result = strategy.select(&specialists, &context_neutral);
        assert_eq!(result.winner_id, 1);

        // With bias favoring index 0: 0.25 * (1+0.5) = 0.375 > 0.3
        let context_biased = SelectionContext {
            e8_attention_bias: vec![0.5, 0.0],
            task_embedding: None,
            threshold: 0.0,
        };
        let result = strategy.select(&specialists, &context_biased);
        assert_eq!(result.winner_id, 0);
    }

    #[test]
    fn test_strategy_name_is_resonance() {
        assert_eq!(ResonanceStrategy::new().name(), "resonance");
    }

    #[test]
    fn test_selection_result_strategy_used() {
        let strategy = ResonanceStrategy::new();
        let specialists = sample_specialists();
        let context = SelectionContext {
            e8_attention_bias: vec![],
            task_embedding: None,
            threshold: 0.5,
        };

        let result = strategy.select(&specialists, &context);
        assert_eq!(result.strategy_used, "resonance");
    }

    #[test]
    fn test_selection_context_default_threshold() {
        let context = SelectionContext {
            e8_attention_bias: vec![],
            task_embedding: None,
            threshold: 0.0,
        };
        assert_eq!(context.threshold, 0.0);
        assert!(context.task_embedding.is_none());
        assert!(context.e8_attention_bias.is_empty());
    }

    #[test]
    fn test_strategy_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<ResonanceStrategy>();
        assert_sync::<ResonanceStrategy>();
        fn assert_strategy_send<T: SelectionStrategy>() {}
        assert_strategy_send::<ResonanceStrategy>();
    }
}
