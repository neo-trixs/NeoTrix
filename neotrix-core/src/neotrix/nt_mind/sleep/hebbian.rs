use crate::core::nt_core_bank::ReasoningMemory;
use crate::core::nt_core_cap::CapabilityVector;
use crate::core::nt_core_ssm::SelectiveState;
use crate::neotrix::nt_core_signal::select::SelectableOperator;

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

    pub fn compute_forget_gate(&self, memory: &ReasoningMemory, state: &SelectiveState) -> f64 {
        let sim = self.memory_state_similarity(memory, state);
        (sim + self.forget_gate_bias).clamp(0.1, 0.99)
    }

    pub fn compute_input_gate(&self, memory: &ReasoningMemory) -> f64 {
        let reward_gate = (memory.reward + self.input_gate_bias).clamp(0.01, 0.99);
        let success_boost = if memory.success { 1.2 } else { 0.8 };
        (reward_gate * success_boost).clamp(0.01, 0.99)
    }

    pub fn memory_state_similarity(
        &self,
        memory: &ReasoningMemory,
        _state: &SelectiveState,
    ) -> f64 {
        if let Some(ref emb) = memory.embedding {
            let avg = emb
                .iter()
                .take(self.dim.min(emb.len()))
                .map(|x| x.abs())
                .sum::<f64>()
                / self.dim.min(emb.len()) as f64;
            avg.clamp(0.0, 1.0)
        } else {
            0.3
        }
    }

    pub fn hebbian_step(
        &self,
        state: &mut SelectiveState,
        memory: &ReasoningMemory,
        _operator: &SelectableOperator,
    ) -> f64 {
        let alpha = self.compute_forget_gate(memory, state);
        let beta = self.compute_input_gate(memory);

        let (k_proj, v_proj) = self.project_memory(memory);

        for i in 0..state.hidden.len().min(k_proj.len().min(v_proj.len())) {
            let outer = v_proj[i] * k_proj[i];
            state.hidden[i] = alpha * state.hidden[i] + beta * outer * self.consolidation_rate;
        }

        let delta = beta * memory.reward;
        delta * self.consolidation_rate
    }

    fn project_memory(&self, memory: &ReasoningMemory) -> (Vec<f64>, Vec<f64>) {
        let n = self.hidden_dim;
        if let Some(ref emb) = memory.embedding {
            let k: Vec<f64> = emb.iter().take(n).copied().collect();
            let v: Vec<f64> = emb.iter().skip(n.min(emb.len())).take(n).copied().collect();
            (Self::pad_or_truncate(k, n), Self::pad_or_truncate(v, n))
        } else {
            let base = memory.reward;
            let k: Vec<f64> = (0..n)
                .map(|i| base * (0.5 + (i as f64 / n as f64) * 0.5))
                .collect();
            let v: Vec<f64> = (0..n)
                .map(|i| base * (1.0 - (i as f64 / n as f64) * 0.5))
                .collect();
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

    pub fn consolidate_to_capability(
        &self,
        state: &SelectiveState,
        capability: &mut CapabilityVector,
    ) -> f64 {
        let hidden_avg = state.hidden.iter().sum::<f64>() / state.hidden.len().max(1) as f64;
        let delta = hidden_avg.abs();

        for i in 0..capability.arr.len().min(state.hidden.len()) {
            let hidden_val = state.hidden[i];
            let current = capability.arr[i];
            capability.arr[i] = current + (hidden_val - current) * self.consolidation_rate * 0.1;
        }

        capability.normalize();
        delta
    }

    pub fn add_transition_noise(&self, state: &mut SelectiveState, noise_level: f64) {
        if noise_level <= 0.0 {
            return;
        }
        for (i, h) in state.hidden.iter_mut().enumerate() {
            let pseudo = ((i * 2654435761) ^ (i << 13) ^ (i >> 7)) as f64 / usize::MAX as f64;
            let noise = (pseudo - 0.5) * 2.0 * noise_level;
            *h += noise;
        }
    }
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
    fn test_hebbian_step_updates_hidden() {
        let operator = SelectableOperator::new(23, 64);
        let mut state = SelectiveState::new(23, 64);
        let prev_hidden = state.hidden.clone();
        let updater = HebbianUpdater::new(23, 64);
        let mem = dummy_memory(0.8, true, "test1");

        let delta = updater.hebbian_step(&mut state, &mem, &operator);
        assert!(
            delta > 0.0,
            "delta should be positive for high-reward memory"
        );
        let changed = state
            .hidden
            .iter()
            .zip(prev_hidden.iter())
            .any(|(a, b)| (a - b).abs() > 1e-10);
        assert!(changed, "hidden state should change after hebbian step");
    }

    #[test]
    fn test_forget_gate_high_similarity() {
        let state = SelectiveState::new(23, 64);
        let updater = HebbianUpdater::new(23, 64);
        let mem = dummy_memory(0.9, true, "test2");

        let gate = updater.compute_forget_gate(&mem, &state);
        assert!(
            gate >= 0.1 && gate <= 0.99,
            "forget gate should be in [0.1, 0.99]"
        );
        assert!(
            gate >= 0.5,
            "high-reward memory should have high forget gate retention"
        );
    }

    #[test]
    fn test_input_gate_scales_with_reward() {
        let updater = HebbianUpdater::new(23, 64);
        let low = updater.compute_input_gate(&dummy_memory(0.1, false, "low"));
        let high = updater.compute_input_gate(&dummy_memory(0.9, true, "high"));
        assert!(
            high > low,
            "high-reward memory should have higher input gate"
        );
    }

    #[test]
    fn test_consolidate_to_capability() {
        let mut state = SelectiveState::new(23, 64);
        for i in 0..state.hidden.len() {
            state.hidden[i] = (i as f64) / 64.0;
        }
        let mut cap = CapabilityVector::default();
        let updater = HebbianUpdater::new(23, 64);

        let delta = updater.consolidate_to_capability(&state, &mut cap);
        assert!(delta > 0.0, "consolidation delta should be positive");
        let changed = cap.arr.iter().any(|&x| x > 0.0);
        assert!(changed, "capability should be updated after consolidation");
    }

    #[test]
    fn test_transition_noise() {
        let mut state = SelectiveState::new(23, 64);
        let original = state.hidden.clone();
        let updater = HebbianUpdater::new(23, 64);

        updater.add_transition_noise(&mut state, 0.01);
        let changed = state
            .hidden
            .iter()
            .zip(original.iter())
            .any(|(a, b)| (a - b).abs() > 1e-10);
        assert!(changed, "noise should alter hidden state");
    }

    #[test]
    fn test_zero_noise_no_change() {
        let mut state = SelectiveState::new(23, 64);
        let original = state.hidden.clone();
        let updater = HebbianUpdater::new(23, 64);

        updater.add_transition_noise(&mut state, 0.0);
        assert_eq!(
            state.hidden, original,
            "zero noise should not alter hidden state"
        );
    }

    #[test]
    fn test_memory_state_similarity_with_embedding() {
        let state = SelectiveState::new(23, 64);
        let updater = HebbianUpdater::new(23, 64);
        let mem = dummy_memory(0.5, true, "sim_test");
        let sim = updater.memory_state_similarity(&mem, &state);
        assert!(sim >= 0.0 && sim <= 1.0, "similarity should be in [0, 1]");
    }

    #[test]
    fn test_hebbian_step_delta_decreases_with_low_reward() {
        let operator = SelectableOperator::new(23, 64);
        let mut state = SelectiveState::new(23, 64);
        let updater = HebbianUpdater::new(23, 64);

        let high = updater.hebbian_step(&mut state, &dummy_memory(0.9, true, "h"), &operator);
        let low = updater.hebbian_step(&mut state, &dummy_memory(0.1, false, "l"), &operator);
        assert!(
            high > low,
            "high reward should produce larger delta than low reward"
        );
    }

    #[test]
    fn test_project_memory_consistent_dims() {
        let updater = HebbianUpdater::new(23, 64);
        let mem = dummy_memory(0.5, true, "proj");
        let (k, v) = updater.project_memory(&mem);
        assert_eq!(k.len(), 64, "key projection should match hidden_dim");
        assert_eq!(v.len(), 64, "value projection should match hidden_dim");
    }
}
