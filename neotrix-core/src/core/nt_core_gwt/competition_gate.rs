//! Competition Gate — Winner-Take-All ignition mechanism
//!
//! Implements Dehaene's Global Neuronal Workspace (GNW) ignition:
//! - All specialists compute resonance scores simultaneously
//! - The highest-scoring specialist wins the competition
//! - Winner's content is broadcast globally
//! - Non-winners are suppressed proportionally to their distance from the winner

use super::resonance::{ResonanceMatrix, MODULE_COUNT};
use serde::{Deserialize, Serialize};

/// Result of a single competition round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionResult {
    /// Index of the winning specialist
    pub winner_index: usize,
    /// Winner's resonance score
    pub winner_score: f64,
    /// All specialist scores after competition (winner boosted, others suppressed)
    pub final_scores: [f64; MODULE_COUNT],
    /// Whether ignition occurred (winner_score > threshold)
    pub ignition: bool,
    /// Suppression factor applied to non-winners (0.0 = total, 1.0 = none)
    pub suppression_factor: f64,
}

/// WTA Competition Gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionGate {
    /// Ignition threshold — winner must exceed this to broadcast
    pub ignition_threshold: f64,
    /// Suppression strength applied to non-winners (0.0–1.0)
    pub suppression_strength: f64,
    /// Whether to use softmax instead of hard WTA
    pub softmax_mode: bool,
}

impl Default for CompetitionGate {
    fn default() -> Self {
        Self {
            ignition_threshold: 0.5,
            suppression_strength: 0.7,
            softmax_mode: false,
        }
    }
}

impl CompetitionGate {
    pub fn new(ignition_threshold: f64, suppression_strength: f64) -> Self {
        Self {
            ignition_threshold,
            suppression_strength: suppression_strength.clamp(0.0, 1.0),
            softmax_mode: false,
        }
    }

    /// Run one competition round.
    ///
    /// 1. Compute effective salience via resonance matrix
    /// 2. Find winner (highest score)
    /// 3. Apply suppression to non-winners
    /// 4. Check if ignition threshold was met
    pub fn compete(
        &self,
        raw_saliences: &[f64; MODULE_COUNT],
        resonance: &ResonanceMatrix,
    ) -> CompetitionResult {
        let effective = resonance.effective_salience(raw_saliences);

        if self.softmax_mode {
            self.softmax_competition(&effective)
        } else {
            self.hard_wta_competition(&effective)
        }
    }

    /// Hard WTA: winner takes all, others suppressed.
    fn hard_wta_competition(&self, effective: &[f64; MODULE_COUNT]) -> CompetitionResult {
        let winner_index = effective
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);

        let winner_score = effective[winner_index];
        let ignition = winner_score >= self.ignition_threshold;

        let mut final_scores = *effective;
        for (i, score) in final_scores.iter_mut().enumerate() {
            if i != winner_index {
                *score *= 1.0 - self.suppression_strength;
            }
        }

        CompetitionResult {
            winner_index,
            winner_score,
            final_scores,
            ignition,
            suppression_factor: if ignition {
                self.suppression_strength
            } else {
                0.0
            },
        }
    }

    /// Softmax competition: probabilistic winner selection, proportional suppression.
    fn softmax_competition(&self, effective: &[f64; MODULE_COUNT]) -> CompetitionResult {
        let max_val = effective.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let shifted: Vec<f64> = effective.iter().map(|x| (x - max_val).exp()).collect();
        let sum: f64 = shifted.iter().sum();
        let probabilities: Vec<f64> = if sum > 0.0 {
            shifted.iter().map(|x| x / sum).collect()
        } else {
            vec![1.0 / MODULE_COUNT as f64; MODULE_COUNT]
        };

        let winner_index = probabilities
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);

        let winner_score = effective[winner_index];
        let ignition = winner_score >= self.ignition_threshold;

        let mut final_scores = *effective;
        let suppression_base = 1.0 - self.suppression_strength;
        for (i, score) in final_scores.iter_mut().enumerate() {
            if i != winner_index {
                let prob = probabilities[i];
                let suppression = suppression_base + prob * self.suppression_strength;
                *score *= suppression.min(1.0);
            }
        }

        CompetitionResult {
            winner_index,
            winner_score,
            final_scores,
            ignition,
            suppression_factor: if ignition {
                self.suppression_strength
            } else {
                0.0
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::default_specialist_states;

    #[test]
    fn test_hard_wta_selects_highest() {
        let gate = CompetitionGate::new(0.3, 0.7);
        let states = default_specialist_states();
        let resonance = ResonanceMatrix::from_states(&states);

        let mut raw = [0.0; MODULE_COUNT];
        raw[5] = 0.9;
        raw[2] = 0.5;

        let result = gate.compete(&raw, &resonance);
        assert_eq!(result.winner_index, 5);
        assert!(result.ignition);
    }

    #[test]
    fn test_hard_wta_suppresses_others() {
        let gate = CompetitionGate::new(0.3, 0.7);
        let states = default_specialist_states();
        let resonance = ResonanceMatrix::from_states(&states);

        let mut raw = [0.0; MODULE_COUNT];
        raw[0] = 0.8;
        raw[1] = 0.7;

        let result = gate.compete(&raw, &resonance);
        assert_eq!(result.winner_index, 0);
        // winner retains score, non-winner suppressed
        assert!(result.final_scores[0] > result.final_scores[1]);
    }

    #[test]
    fn test_ignition_fails_below_threshold() {
        let gate = CompetitionGate::new(0.9, 0.7);
        let states = default_specialist_states();
        let resonance = ResonanceMatrix::from_states(&states);

        let mut raw = [0.0; MODULE_COUNT];
        raw[3] = 0.5;

        let result = gate.compete(&raw, &resonance);
        assert!(!result.ignition);
    }

    #[test]
    fn test_softmax_mode_supports_probabilistic_selection() {
        let mut gate = CompetitionGate::new(0.3, 0.5);
        gate.softmax_mode = true;
        let states = default_specialist_states();
        let resonance = ResonanceMatrix::from_states(&states);

        let mut raw = [0.0; MODULE_COUNT];
        raw[7] = 0.9;
        raw[3] = 0.85;

        let result = gate.compete(&raw, &resonance);
        // Either winner is valid in softmax
        assert!(result.winner_score > 0.0);
    }

    #[test]
    fn test_competition_with_resonance_boost() {
        let gate = CompetitionGate::new(0.4, 0.6);
        let states = default_specialist_states();
        let resonance = ResonanceMatrix::from_states(&states);

        let mut raw = [0.0; MODULE_COUNT];
        raw[0] = 0.5;

        let result = gate.compete(&raw, &resonance);
        // Resonance boost may increase effective score
        assert!(result.winner_score >= 0.5);
    }

    #[test]
    fn test_final_scores_sum_to_positive() {
        let gate = CompetitionGate::new(0.3, 0.5);
        let states = default_specialist_states();
        let resonance = ResonanceMatrix::from_states(&states);

        let mut raw = [0.1; MODULE_COUNT];
        raw[4] = 0.8;

        let result = gate.compete(&raw, &resonance);
        let sum: f64 = result.final_scores.iter().sum();
        assert!(sum > 0.0);
    }

    // ── Calibration tests ──────────────────────────────────────────
    #[test]
    fn test_ignition_threshold_calibration_mid_range() {
        let gate = CompetitionGate::default();
        let states = default_specialist_states();
        let resonance = ResonanceMatrix::from_states(&states);
        let mut raw = [0.0; MODULE_COUNT];
        for (i, v) in raw.iter_mut().enumerate() {
            *v = 0.4 + (i as f64 / MODULE_COUNT as f64) * 0.3;
        }
        raw[0] = 0.65;
        let result = gate.compete(&raw, &resonance);
        assert!(
            result.ignition,
            "0.65 salience should ignite with threshold"
        );
        assert!(
            result.winner_score >= 0.4,
            "winner score should be meaningful"
        );
    }

    #[test]
    fn test_ignition_threshold_no_false_positive_noise() {
        let gate = CompetitionGate::default();
        let states = default_specialist_states();
        let resonance = ResonanceMatrix::from_states(&states);
        let raw = [0.15; MODULE_COUNT];
        let result = gate.compete(&raw, &resonance);
        assert!(
            !result.ignition || result.winner_score < 0.85,
            "pure noise should not produce high confidence ignition: score={}",
            result.winner_score
        );
    }

    #[test]
    fn test_ignition_threshold_high_winner() {
        let gate = CompetitionGate::default();
        let states = default_specialist_states();
        let resonance = ResonanceMatrix::from_states(&states);
        let mut raw = [0.0; MODULE_COUNT];
        raw[3] = 0.9;
        let result = gate.compete(&raw, &resonance);
        assert!(result.ignition, "0.9 should definitely ignite");
        assert_eq!(result.winner_index, 3);
        assert!(result.winner_score >= 0.5);
    }

    #[test]
    fn test_all_equal_noise_handling() {
        let gate = CompetitionGate::new(0.1, 0.7);
        let states = default_specialist_states();
        let resonance = ResonanceMatrix::from_states(&states);

        let raw = [0.3; MODULE_COUNT];
        let result = gate.compete(&raw, &resonance);
        // Should still select a winner deterministically
        assert!(result.winner_index < MODULE_COUNT);
        assert!(result.winner_score > 0.0);
    }
}
