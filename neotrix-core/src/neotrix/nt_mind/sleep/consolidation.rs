use crate::core::nt_core_bank::ReasoningMemory;
use crate::core::nt_core_bank::ReasoningBank;
use crate::core::nt_core_cap::CapabilityVector;
use crate::neotrix::nt_core_signal::core::SelectiveState;
use crate::neotrix::nt_core_signal::select::SelectableOperator;
use chrono::Utc;
use super::hebbian::HebbianUpdater;

#[derive(Debug, Clone)]
pub struct ConsolidationConfig {
    pub min_reward: f64,
    pub max_memories_per_pass: usize,
    pub recent_weight: f64,
    pub high_value_weight: f64,
    pub diversity_weight: f64,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            min_reward: 0.2,
            max_memories_per_pass: 10,
            recent_weight: 0.4,
            high_value_weight: 0.4,
            diversity_weight: 0.2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConsolidationResult {
    pub memories_processed: usize,
    pub total_delta: f64,
    pub passes_done: usize,
    pub avg_similarity: f64,
}

pub struct MemoryConsolidation {
    pub config: ConsolidationConfig,
}

impl MemoryConsolidation {
    pub fn new(config: ConsolidationConfig) -> Self {
        Self { config }
    }

    pub fn select_memories_for_sleep(&self, bank: &ReasoningBank) -> Vec<ReasoningMemory> {
        let all_mems = bank.memories();
        let mut scored: Vec<(f64, &ReasoningMemory)> = all_mems.iter()
            .filter(|m| m.reward >= self.config.min_reward)
            .map(|m| {
                let age = (Utc::now().timestamp() - m.timestamp).max(0) as f64;
                let recency = 1.0 / (1.0 + age).sqrt();
                let value = m.reward;
                let diversity = 1.0 - (m.lifecycle.access_count as f64).tanh();
                let score = self.config.recent_weight * recency
                    + self.config.high_value_weight * value
                    + self.config.diversity_weight * diversity;
                (score, m)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(self.config.max_memories_per_pass);
        scored.into_iter().map(|(_, m)| m.clone()).collect()
    }

    pub fn run_consolidation_pass(
        &self,
        bank: &mut ReasoningBank,
        brain: &mut CapabilityVector,
        state: &mut SelectiveState,
        operator: &SelectableOperator,
        updater: &HebbianUpdater,
    ) -> ConsolidationResult {
        let memories = self.select_memories_for_sleep(bank);
        let count = memories.len();
        if count == 0 {
            return ConsolidationResult {
                memories_processed: 0,
                total_delta: 0.0,
                passes_done: 1,
                avg_similarity: 0.0,
            };
        }

        let mut total_delta = 0.0;
        let mut total_sim = 0.0;

        for mem in &memories {
            let delta = updater.hebbian_step(state, mem, operator);
            total_delta += delta;
            total_sim += updater.memory_state_similarity(mem, state);
        }

        let cap_delta = updater.consolidate_to_capability(state, brain);
        total_delta += cap_delta;

        ConsolidationResult {
            memories_processed: count,
            total_delta,
            passes_done: 1,
            avg_similarity: if count > 0 { total_sim / count as f64 } else { 0.0 },
        }
    }

    pub fn scoring_breakdown(&self, memory: &ReasoningMemory) -> (f64, f64, f64, f64) {
        let recency = 1.0 / (1.0 + memory.lifecycle.last_accessed as f64).sqrt();
        let value = memory.reward;
        let diversity = 1.0 - (memory.lifecycle.access_count as f64).tanh();
        let score = self.config.recent_weight * recency
            + self.config.high_value_weight * value
            + self.config.diversity_weight * diversity;
        (recency, value, diversity, score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::core::nt_core_bank::{MemoryTier, MemoryLifecycle, T3Views};
    use crate::core::{RewardSource, TaskType};
    use crate::core::nt_core_cap::CapabilityVector;

    fn make_memory(reward: f64, success: bool, id: &str, ts: i64) -> ReasoningMemory {
        ReasoningMemory {
            id: id.to_string(),
            task_description: format!("task_{}", id),
            task_type: TaskType::General,
            micro_edits: vec![],
            reward,
            reward_source: RewardSource::Internal,
            success,
            timestamp: ts,
            embedding: Some(vec![0.5; 128]),
            tier: MemoryTier::Episodic,
            lifecycle: MemoryLifecycle {
                importance: reward,
                confidence: 0.5,
                access_count: 0,
                created_at: ts,
                last_accessed: ts,
                ttl_seconds: None,
            },
            t3_views: T3Views::new(),
        }
    }

    fn build_test_bank(memories: Vec<ReasoningMemory>) -> ReasoningBank {
        let mut bank = ReasoningBank::new(100);
        for m in memories {
            bank.store(m);
        }
        bank
    }

    #[test]
    fn test_select_memories_filters_low_reward() {
        let mems = vec![
            make_memory(0.1, false, "low", 0),
            make_memory(0.5, true, "high", 0),
        ];
        let bank = build_test_bank(mems);
        let mc = MemoryConsolidation::new(ConsolidationConfig::default());
        let selected = mc.select_memories_for_sleep(&bank);
        assert!(selected.iter().any(|m| m.id == "high"), "high-reward memory should be selected");
    }

    #[test]
    fn test_select_memories_empty_bank() {
        let bank = ReasoningBank::new(10);
        let mc = MemoryConsolidation::new(ConsolidationConfig::default());
        let selected = mc.select_memories_for_sleep(&bank);
        assert!(selected.is_empty(), "empty bank should return empty selection");
    }

    #[test]
    fn test_select_memories_respects_max() {
        let mems: Vec<_> = (0..20).map(|i| {
            make_memory(0.5 + (i as f64) * 0.02, true, &format!("m{}", i), i as i64)
        }).collect();
        let bank = build_test_bank(mems);
        let mc = MemoryConsolidation::new(ConsolidationConfig { max_memories_per_pass: 5, ..Default::default() });
        let selected = mc.select_memories_for_sleep(&bank);
        assert!(selected.len() <= 5, "should limit to max_memories_per_pass");
    }

    #[test]
    fn test_select_memories_scores_higher_for_recent() {
        let now = Utc::now().timestamp();
        let mems = vec![
            make_memory(0.5, true, "old", now - 86400),
            make_memory(0.5, true, "recent", now),
        ];
        let bank = build_test_bank(mems);
        let mc = MemoryConsolidation::new(ConsolidationConfig::default());
        let selected = mc.select_memories_for_sleep(&bank);
        let recent = selected.iter().position(|m| m.id == "recent");
        let old = selected.iter().position(|m| m.id == "old");
        assert!(recent < Some(old.unwrap_or(usize::MAX)), "recent memory should rank higher");
    }

    #[test]
    fn test_consolidation_pass_no_crash() {
        let bank = ReasoningBank::new(10);
        let mut cap = CapabilityVector::default();
        let mut state = SelectiveState::new(23, 64);
        let operator = SelectableOperator::new(23, 64);
        let updater = HebbianUpdater::new(23, 64);
        let mc = MemoryConsolidation::new(ConsolidationConfig::default());

        let mut bank = bank;
        let result = mc.run_consolidation_pass(&mut bank, &mut cap, &mut state, &operator, &updater);
        assert_eq!(result.memories_processed, 0, "empty bank should process 0 memories");
    }

    #[test]
    fn test_scoring_breakdown_returns_valid() {
        let mem = make_memory(0.8, true, "score_test", 100);
        let mc = MemoryConsolidation::new(ConsolidationConfig::default());
        let (recency, value, diversity, score) = mc.scoring_breakdown(&mem);
        assert!(recency > 0.0, "recency should be positive");
        assert!(value == 0.8, "value should match reward");
        assert!(diversity > 0.0, "diversity should be positive");
        assert!(score > 0.0, "score should be positive");
    }
}
