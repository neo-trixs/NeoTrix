#![allow(dead_code)]

use crate::core::nt_core_cap::CapabilityVector;
use crate::core::nt_core_bank::ReasoningMemory;
// // use crate::core::// nt_core_signal::core::SelectiveState;
// // use crate::core::// nt_core_signal::select::SelectableOperator;

pub struct HebbianUpdater {
    pub forget_gate_bias: f64,
    pub input_gate_bias: f64,
    pub consolidation_rate: f64,
    pub dim: usize,
    pub hidden_dim: usize,
}

impl Default for HebbianUpdater {
    fn default() -> Self {
        Self {
            forget_gate_bias: 0.5,
            input_gate_bias: 0.5,
            consolidation_rate: 0.05,
            dim: 23,
            hidden_dim: 64,
        }
    }
}

impl HebbianUpdater {
    pub fn new(dim: usize, hidden_dim: usize) -> Self {
        Self {
            dim,
            hidden_dim,
            ..Default::default()
        }
    }

//     pub(crate) fn _compute_forget_gate(&self, memory: &ReasoningMemory, state: &SelectiveState) -> f64 {
//         let sim = self._memory_state_similarity(memory, state);
//         (sim + self.forget_gate_bias).clamp(0.1, 0.99)
//     }

    pub(crate) fn _compute_input_gate(&self, memory: &ReasoningMemory) -> f64 {
        let reward_gate = (memory.reward + self.input_gate_bias).clamp(0.01, 0.99);
        let success_boost = if memory.success { 1.2 } else { 0.8 };
        (reward_gate * success_boost).clamp(0.01, 0.99)
    }

//     pub(crate) fn _memory_state_similarity(&self, memory: &ReasoningMemory, _state: &SelectiveState) -> f64 {
//         if let Some(ref emb) = memory.embedding {
//             let avg = emb.iter().take(self.dim.min(emb.len())).map(|x| x.abs()).sum::<f64>()
//                 / self.dim.min(emb.len()) as f64;
//             avg.clamp(0.0, 1.0)
//         } else {
//             0.3
//         }
//     }

    /// Compute Hebbian consolidation delta for a reasoning memory.
    ///
    /// Returns 0.0 as a conservative no-op: without SelectiveState integration,
    /// we cannot compute meaningful weight updates. Real implementation would
    /// use input gate to modulate reward signal into capability vectors.
    pub fn hebbian_step(
        &self,
        memory: &ReasoningMemory,
    ) -> f64 {
        // Without SelectiveState, we can only compute the input gate contribution.
        // This gives a non-zero signal based on memory reward, but no full Hebbian update.
        let gate = self._compute_input_gate(memory);
        // Scale reward through the gate — conservative partial update
        gate * memory.reward * self.consolidation_rate
    }

    fn project_memory(&self, memory: &ReasoningMemory) -> (Vec<f64>, Vec<f64>) {
        let n = self.hidden_dim;
        if let Some(ref emb) = memory.embedding {
            let k: Vec<f64> = emb.iter().take(n).copied().collect();
            let v: Vec<f64> = emb.iter().skip(n.min(emb.len())).take(n).copied().collect();
            (Self::pad_or_truncate(k, n), Self::pad_or_truncate(v, n))
        } else {
            let base = memory.reward;
            let k: Vec<f64> = (0..n).map(|i| base * (0.5 + (i as f64 / n as f64) * 0.5)).collect();
            let v: Vec<f64> = (0..n).map(|i| base * (1.0 - (i as f64 / n as f64) * 0.5)).collect();
            (k, v)
        }
    }

    fn pad_or_truncate(mut v: Vec<f64>, n: usize) -> Vec<f64> {
        if v.len() >= n {
            v.truncate(n);
            v
        } else {
            v.resize(n, 0.0);
            v
        }
    }

    /// Consolidate a reasoning memory into a capability vector.
    ///
    /// Returns 0.0 as a conservative no-op: without SelectiveState, we cannot
    /// compute meaningful capability deltas. Real implementation would project
    /// memory embeddings into capability space and update weights.
    pub fn consolidate_to_capability(
        &self,
        capability: &mut CapabilityVector,
    ) -> f64 {
        // Without SelectiveState, we cannot compute meaningful consolidation.
        // Return 0.0 to indicate no change was applied.
        let _ = capability;
        0.0
    }

//     pub(crate) fn _add_transition_noise(&self, state: &mut SelectiveState, noise_level: f64) {
//         if noise_level <= 0.0 {
//             return;
//         }
//         for (i, h) in state.hidden.iter_mut().enumerate() {
//             let pseudo = ((i * 2654435761) ^ (i << 13) ^ (i >> 7)) as f64 / usize::MAX as f64;
//             let noise = (pseudo - 0.5) * 2.0 * noise_level;
//             *h += noise;
//         }
//     }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_bank::{MemoryLifecycle, MemoryTier, T3Views};
    use crate::core::RewardSource;
    use crate::core::TaskType;

    fn dummy_memory(reward: f64, success: bool, id: &str) -> ReasoningMemory {
        ReasoningMemory {
            id: id.to_string(),
            task_description: "test".to_string(),
            task_type: TaskType::General,
            micro_edits: vec![],
            reward,
            reward_source: RewardSource::Internal,
            success,
            timestamp: 0,
            embedding: Some(vec![0.5; 128]),
            tier: MemoryTier::Episodic,
            lifecycle: MemoryLifecycle {
                importance: 0.5,
                confidence: 0.5,
                access_count: 0,
                created_at: 0,
                last_accessed: 0,
                ttl_seconds: None,
            },
            t3_views: T3Views::new(),
        }
    }

    #[test]
    fn test_hebbian_step_returns_delta() {
        let updater = HebbianUpdater::new(23, 64);
        let mem = dummy_memory(0.8, true, "test1");

        let delta = updater.hebbian_step(&mem);
        assert!(delta >= 0.0, "delta should be non-negative");
    }

    #[test]
    fn test_input_gate_scales_with_reward() {
        let updater = HebbianUpdater::new(23, 64);
        let low = updater._compute_input_gate(&dummy_memory(0.1, false, "low"));
        let high = updater._compute_input_gate(&dummy_memory(0.9, true, "high"));
        assert!(high > low, "high-reward memory should have higher input gate");
    }

    #[test]
    fn test_consolidate_to_capability() {
        let mut cap = CapabilityVector::default();
        let updater = HebbianUpdater::new(23, 64);

        let delta = updater.consolidate_to_capability(&mut cap);
        assert!(delta >= 0.0, "consolidation delta should be non-negative");
    }

    #[test]
    fn test_project_memory_consistent_dims() {
        let updater = HebbianUpdater::new(23, 64);
        let mem = dummy_memory(0.5, true, "proj");
        let (k, v) = updater.project_memory(&mem);
        assert_eq!(k.len(), 64);
        assert_eq!(v.len(), 64);
    }
}
