use serde::{Serialize, Deserialize};
use crate::core::nt_core_e8_vsa::E8VsaEmbedding;

/// VSA-aware content scorer for GWT.
///
/// Bridges the E8 VSA embedding space into GWT resonance scoring,
/// allowing attention weights to be computed in continuous vector space
/// rather than discrete state space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsaContentScorer {
    embedding: E8VsaEmbedding,
}

impl VsaContentScorer {
    pub fn new(dim: usize) -> Self {
        Self {
            embedding: E8VsaEmbedding::new(dim),
        }
    }

    pub fn with_embedding(embedding: E8VsaEmbedding) -> Self {
        Self { embedding }
    }

    /// Score how well an E8 state matches a task embedding (for GWT attention).
    pub fn score_state_task(&self, e8_state: u8, task_embedding: &[f64]) -> f64 {
        let state_hv = self.embedding.embed(e8_state);
        if task_embedding.len() != self.embedding.dim {
            return 0.5;
        }
        self.embedding.similarity(state_hv, task_embedding).max(0.0).min(1.0)
    }

    /// Score a transition between two E8 states (for GWT resonance).
    pub fn score_transition(&self, from: u8, to: u8) -> f64 {
        self.embedding.similarity(self.embedding.embed(from), self.embedding.embed(to))
            .max(0.0).min(1.0)
    }

    pub fn embedding(&self) -> &E8VsaEmbedding {
        &self.embedding
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_state_task_self_similarity() {
        let scorer = VsaContentScorer::new(64);
        let score = scorer.score_state_task(31, scorer.embedding().embed(31));
        assert!((score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_score_transition() {
        let scorer = VsaContentScorer::new(64);
        let score = scorer.score_transition(0, 0);
        assert!((score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_different_states_lower_score() {
        let scorer = VsaContentScorer::new(64);
        let same = scorer.score_transition(5, 5);
        let diff = scorer.score_transition(5, 37);
        assert!(diff <= same, "different states should have lower or equal score");
    }
}
