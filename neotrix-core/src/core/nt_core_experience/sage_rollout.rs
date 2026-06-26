use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use serde::{Deserialize, Serialize};

/// Difficulty level for a task in a sequential rollout chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Master,
}

impl Difficulty {
    /// Returns the next difficulty level in the progression.
    /// Easy → Medium → Hard → Master → None (chain complete).
    pub fn next(&self) -> Option<Difficulty> {
        match self {
            Difficulty::Easy => Some(Difficulty::Medium),
            Difficulty::Medium => Some(Difficulty::Hard),
            Difficulty::Hard => Some(Difficulty::Master),
            Difficulty::Master => None,
        }
    }

    /// Order index for comparison (0 = Easy, 3 = Master).
    pub fn ordinal(&self) -> usize {
        match self {
            Difficulty::Easy => 0,
            Difficulty::Medium => 1,
            Difficulty::Hard => 2,
            Difficulty::Master => 3,
        }
    }
}

/// Semantic fingerprint of a SEAL task, used to find similar tasks
/// and assign them to the same sequential rollout chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSignature {
    pub domain: String,
    pub operation: String,
    pub difficulty: Difficulty,
    pub embedding: Vec<f64>,
}

impl TaskSignature {
    /// Create a new task signature with a deterministic embedding
    /// derived from domain + operation bytes.
    pub fn new(domain: &str, operation: &str, difficulty: Difficulty) -> Self {
        let embedding = compute_embedding(domain, operation);
        Self {
            domain: domain.to_string(),
            operation: operation.to_string(),
            difficulty,
            embedding,
        }
    }

    /// Cosine similarity with another TaskSignature's embedding.
    pub fn similarity(&self, other: &Self) -> f64 {
        cosine_similarity(&self.embedding, &other.embedding)
    }
}

/// A chain of tasks at progressively increasing difficulty.
///
/// Maps to the SAGE Sequential Rollout concept: a sequence of training
/// tasks that share similar semantics and form a difficulty ladder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolloutChain {
    pub task_ids: Vec<String>,
    pub difficulties: Vec<Difficulty>,
    pub scores: Vec<f64>,
    pub active: bool,
}

impl RolloutChain {
    pub fn new(task_id: &str, difficulty: Difficulty) -> Self {
        Self {
            task_ids: vec![task_id.to_string()],
            difficulties: vec![difficulty],
            scores: vec![],
            active: true,
        }
    }
}

/// SAGE Sequential Rollout tracker.
///
/// Maintains multiple independent rollout chains keyed by `domain:operation`.
/// Each chain tracks a sequence of tasks at increasing difficulty levels,
/// analogous to SAGE's curriculum-based sequential training across
/// a difficulty chain (arXiv 2512.17102).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequentialRollout {
    /// Rollout chains grouped by `domain:operation` key.
    pub rollout_chains: HashMap<String, Vec<RolloutChain>>,
    /// Cosine similarity threshold for matching task signatures to chains (default 0.75).
    pub similarity_threshold: f64,
    /// Maximum chain depth (default 4: Easy → Medium → Hard → Master).
    pub max_chain_depth: usize,
}

impl SequentialRollout {
    pub fn new() -> Self {
        Self {
            rollout_chains: HashMap::new(),
            similarity_threshold: 0.75,
            max_chain_depth: 4,
        }
    }

    /// Find an existing chain matching the task signature, or create a new one.
    /// Returns the index of the chain within its group.
    pub fn find_or_create_chain(&mut self, sig: &TaskSignature, task_id: &str) -> usize {
        let key = format!("{}:{}", sig.domain, sig.operation);
        let chains = self.rollout_chains.entry(key.clone()).or_default();

        // Try to find an existing chain with similar embedding and compatible difficulty
        for (idx, chain) in chains.iter().enumerate() {
            if !chain.active {
                continue;
            }
            // Check difficulty compatibility: the chain's current top difficulty
            // should be adjacent to or same as the task's difficulty
            let chain_top = chain.difficulties.last().copied().unwrap_or(sig.difficulty);
            let diff_dist =
                (chain_top.ordinal() as isize - sig.difficulty.ordinal() as isize).unsigned_abs();
            if diff_dist > 1 {
                continue;
            }
            // Check embedding similarity
            let chain_sig = TaskSignature::new(&sig.domain, &sig.operation, chain_top);
            if chain_sig.similarity(sig) >= self.similarity_threshold {
                return idx;
            }
        }

        // No matching chain found — create a new one
        let new_idx = chains.len();
        chains.push(RolloutChain::new(task_id, sig.difficulty));
        new_idx
    }

    /// Record a training result for the given chain.
    pub fn record_result(&mut self, chain_idx: usize, score: f64) {
        for chains in self.rollout_chains.values_mut() {
            if chain_idx < chains.len() {
                let chain = &mut chains[chain_idx];
                chain.scores.push(score);
                // Advance difficulty if score is good enough (> 0.5)
                if score > 0.5 {
                    if let Some(next) = chain.difficulties.last().and_then(|d| d.next()) {
                        if chain.difficulties.len() < self.max_chain_depth {
                            // Push a placeholder task ID for the next difficulty level
                            let next_id = format!(
                                "{}_{}",
                                chain.task_ids.last().cloned().unwrap_or_default(),
                                next.ordinal()
                            );
                            chain.task_ids.push(next_id);
                            chain.difficulties.push(next);
                        } else {
                            chain.active = false;
                        }
                    } else {
                        // Master completed — chain is done
                        chain.active = false;
                    }
                }
                return;
            }
        }
    }

    /// What difficulty should be trained next for this chain?
    pub fn next_difficulty(&self, chain_idx: usize) -> Option<Difficulty> {
        for chains in self.rollout_chains.values() {
            if chain_idx < chains.len() {
                let chain = &chains[chain_idx];
                if !chain.active {
                    return None;
                }
                return chain.difficulties.last().copied();
            }
        }
        None
    }

    /// Unified operation: find or create chain → record result → return next difficulty.
    pub fn chain_and_train(
        &mut self,
        sig: &TaskSignature,
        task_id: &str,
        score: f64,
    ) -> Option<Difficulty> {
        let idx = self.find_or_create_chain(sig, task_id);
        self.record_result(idx, score);
        self.next_difficulty(idx)
    }
}

impl Default for SequentialRollout {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Embedding helpers
// ---------------------------------------------------------------------------

/// Deterministic embedding derived from domain + operation bytes.
fn compute_embedding(domain: &str, operation: &str) -> Vec<f64> {
    let mut hasher = DefaultHasher::new();
    domain.hash(&mut hasher);
    operation.hash(&mut hasher);
    let seed = hasher.finish();

    // Generate D=64 pseudo-random f64 values in [-1, 1] as a lightweight VSA-style fingerprint
    let dim = 64;
    let mut embedding = Vec::with_capacity(dim);
    let mut h = seed;
    for _ in 0..dim {
        h = h
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let val = ((h >> 33) as i64) as f64 / (i64::MAX as f64);
        embedding.push(val);
    }
    embedding
}

/// Cosine similarity between two vectors.
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom < 1e-12 {
        return 0.0;
    }
    (dot / denom).clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_progression() {
        assert_eq!(Difficulty::Easy.next(), Some(Difficulty::Medium));
        assert_eq!(Difficulty::Medium.next(), Some(Difficulty::Hard));
        assert_eq!(Difficulty::Hard.next(), Some(Difficulty::Master));
        assert_eq!(Difficulty::Master.next(), None);
    }

    #[test]
    fn test_difficulty_ordinal() {
        assert_eq!(Difficulty::Easy.ordinal(), 0);
        assert_eq!(Difficulty::Medium.ordinal(), 1);
        assert_eq!(Difficulty::Hard.ordinal(), 2);
        assert_eq!(Difficulty::Master.ordinal(), 3);
    }

    #[test]
    fn test_task_signature_embedding_determinism() {
        let a = TaskSignature::new("codegen", "mutate", Difficulty::Easy);
        let b = TaskSignature::new("codegen", "mutate", Difficulty::Easy);
        assert_eq!(a.embedding, b.embedding);
    }

    #[test]
    fn test_task_signature_similarity_same() {
        let a = TaskSignature::new("codegen", "mutate", Difficulty::Easy);
        let b = TaskSignature::new("codegen", "mutate", Difficulty::Easy);
        let sim = a.similarity(&b);
        assert!((sim - 1.0).abs() < 1e-6, "expected ~1.0, got {}", sim);
    }

    #[test]
    fn test_task_signature_similarity_different() {
        let a = TaskSignature::new("codegen", "mutate", Difficulty::Easy);
        let b = TaskSignature::new("reasoning", "propose", Difficulty::Hard);
        let sim = a.similarity(&b);
        // Different domains+ops should produce different embeddings, thus not ~1.0
        assert!(sim < 0.99, "expected <0.99, got {}", sim);
    }

    #[test]
    fn test_chain_creation() {
        let mut rollout = SequentialRollout::new();
        let sig = TaskSignature::new("codegen", "mutate", Difficulty::Easy);
        let idx = rollout.find_or_create_chain(&sig, "task_001");
        assert_eq!(idx, 0);
        assert_eq!(rollout.rollout_chains.len(), 1);
    }

    #[test]
    fn test_similarity_matching_reuses_chain() {
        let mut rollout = SequentialRollout::new();
        let sig1 = TaskSignature::new("codegen", "mutate", Difficulty::Easy);
        let sig2 = TaskSignature::new("codegen", "mutate", Difficulty::Medium);

        let idx1 = rollout.find_or_create_chain(&sig1, "task_001");
        let idx2 = rollout.find_or_create_chain(&sig2, "task_002");

        // Same domain+operation, difficulty adjacent → should reuse chain
        assert_eq!(idx1, idx2);
    }

    #[test]
    fn test_different_domain_creates_separate_chain() {
        let mut rollout = SequentialRollout::new();
        let sig1 = TaskSignature::new("codegen", "mutate", Difficulty::Easy);
        let sig2 = TaskSignature::new("reasoning", "evaluate", Difficulty::Easy);

        let _ = rollout.find_or_create_chain(&sig1, "task_001");
        let _ = rollout.find_or_create_chain(&sig2, "task_002");

        assert_eq!(rollout.rollout_chains.len(), 2);
    }

    #[test]
    fn test_full_chain_progression() {
        let mut rollout = SequentialRollout::new();
        let sig = TaskSignature::new("codegen", "mutate", Difficulty::Easy);

        // Start at Easy
        let next = rollout.chain_and_train(&sig, "task_easy", 0.8);
        assert_eq!(next, Some(Difficulty::Easy)); // Recorded, now at Easy (next task should be Easy->Medium)

        // Record again at Easy to push to Medium
        let next = rollout.chain_and_train(&sig, "task_easy2", 0.9);
        assert_eq!(next, Some(Difficulty::Medium));

        // Medium → Hard
        let sig_med = TaskSignature::new("codegen", "mutate", Difficulty::Medium);
        let next = rollout.chain_and_train(&sig_med, "task_medium", 0.85);
        assert_eq!(next, Some(Difficulty::Hard));

        // Hard → Master
        let sig_hard = TaskSignature::new("codegen", "mutate", Difficulty::Hard);
        let next = rollout.chain_and_train(&sig_hard, "task_hard", 0.9);
        assert_eq!(next, Some(Difficulty::Master));

        // Master → None (chain complete)
        let sig_master = TaskSignature::new("codegen", "mutate", Difficulty::Master);
        let next = rollout.chain_and_train(&sig_master, "task_master", 0.95);
        assert_eq!(next, None);
    }

    #[test]
    fn test_score_tracking_across_chain() {
        let mut rollout = SequentialRollout::new();
        let sig = TaskSignature::new("codegen", "mutate", Difficulty::Easy);
        let idx = rollout.find_or_create_chain(&sig, "task_001");

        rollout.record_result(idx, 0.75);
        rollout.record_result(idx, 0.85);
        rollout.record_result(idx, 0.95);

        for chains in rollout.rollout_chains.values() {
            if idx < chains.len() {
                assert_eq!(chains[idx].scores.len(), 3);
                assert!((chains[idx].scores[0] - 0.75).abs() < 1e-6);
                assert!((chains[idx].scores[1] - 0.85).abs() < 1e-6);
                assert!((chains[idx].scores[2] - 0.95).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn test_multiple_independent_chains() {
        let mut rollout = SequentialRollout::new();
        let sig_a = TaskSignature::new("codegen", "mutate", Difficulty::Easy);
        let sig_b = TaskSignature::new("reasoning", "evaluate", Difficulty::Easy);
        let sig_c = TaskSignature::new("planning", "propose", Difficulty::Hard);

        let idx_a = rollout.find_or_create_chain(&sig_a, "cg_001");
        let idx_b = rollout.find_or_create_chain(&sig_b, "re_001");
        let idx_c = rollout.find_or_create_chain(&sig_c, "pl_001");

        assert_eq!(rollout.rollout_chains.len(), 3);

        // Record results independently
        rollout.record_result(idx_a, 0.8);
        rollout.record_result(idx_b, 0.9);
        rollout.record_result(idx_c, 0.7);

        // Check independent score tracking
        for chains in rollout.rollout_chains.values() {
            for chain in chains {
                assert!(!chain.scores.is_empty());
            }
        }
    }

    #[test]
    fn test_low_score_does_not_advance_difficulty() {
        let mut rollout = SequentialRollout::new();
        let sig = TaskSignature::new("codegen", "mutate", Difficulty::Easy);
        let idx = rollout.find_or_create_chain(&sig, "task_001");

        // Low score → should not advance
        rollout.record_result(idx, 0.3);
        let chain_top = rollout
            .rollout_chains
            .values()
            .flat_map(|c| c.iter())
            .next()
            .unwrap()
            .difficulties
            .last()
            .copied();
        assert_eq!(chain_top, Some(Difficulty::Easy));
    }
}
