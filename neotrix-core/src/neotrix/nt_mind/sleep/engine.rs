use crate::core::nt_core_cap::CapabilityVector;
use crate::core::nt_core_bank::ReasoningBank;
use crate::neotrix::signal::core::SelectiveState;
use crate::neotrix::signal::select::SelectableOperator;
use crate::neotrix::error::NeoTrixResult;

use super::hebbian::HebbianUpdater;
use super::consolidation::{MemoryConsolidation, ConsolidationConfig};

#[derive(Debug, Clone)]
pub struct SleepStats {
    pub total_delta: f64,
    pub passes_done: usize,
    pub total_memories: usize,
    pub avg_similarity: f64,
    pub delta_per_pass: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct SleepConfig {
    pub passes: usize,
    pub consolidation_rate: f64,
    pub transition_noise: f64,
    pub consolidation: ConsolidationConfig,
}

impl Default for SleepConfig {
    fn default() -> Self {
        Self {
            passes: 3,
            consolidation_rate: 0.05,
            transition_noise: 0.01,
            consolidation: ConsolidationConfig::default(),
        }
    }
}

impl SleepConfig {
    pub fn with_passes(passes: usize) -> Self {
        Self { passes, ..Default::default() }
    }

    pub fn light_sleep() -> Self {
        Self { passes: 1, consolidation_rate: 0.02, transition_noise: 0.005, ..Default::default() }
    }

    pub fn deep_sleep() -> Self {
        Self { passes: 6, consolidation_rate: 0.08, transition_noise: 0.02, ..Default::default() }
    }
}

#[derive(Debug, Clone)]
pub struct SleepResult {
    pub stats: SleepStats,
    pub config: SleepConfig,
}

pub struct SleepEngine {
    pub config: SleepConfig,
    updater: HebbianUpdater,
    consolidator: MemoryConsolidation,
}

impl SleepEngine {
    pub fn new(config: SleepConfig) -> Self {
        let updater = HebbianUpdater {
            consolidation_rate: config.consolidation_rate,
            ..Default::default()
        };
        let consolidator = MemoryConsolidation::new(config.consolidation.clone());
        Self { config, updater, consolidator }
    }

    pub fn with_passes(passes: usize) -> Self {
        Self::new(SleepConfig::with_passes(passes))
    }

    pub fn sleep(
        &mut self,
        brain: &mut CapabilityVector,
        bank: &mut ReasoningBank,
        operator: &SelectableOperator,
        state: &mut SelectiveState,
    ) -> NeoTrixResult<SleepResult> {
        let passes = self.config.passes;
        if passes == 0 {
            return Ok(SleepResult {
                stats: SleepStats {
                    total_delta: 0.0,
                    passes_done: 0,
                    total_memories: 0,
                    avg_similarity: 0.0,
                    delta_per_pass: vec![],
                },
                config: self.config.clone(),
            });
        }

        let mut total_delta = 0.0;
        let mut delta_per_pass = Vec::with_capacity(passes);
        let mut total_memories = 0;
        let mut total_sim = 0.0;

        for pass_idx in 0..passes {
            let result = self.consolidator.run_consolidation_pass(
                bank, brain, state, operator, &self.updater,
            );
            total_delta += result.total_delta;
            delta_per_pass.push(result.total_delta);
            total_memories += result.memories_processed;
            total_sim += result.avg_similarity * result.memories_processed as f64;

            if pass_idx < passes - 1 {
                self.updater.add_transition_noise(state, self.config.transition_noise);
            }
        }

        let avg_sim = if total_memories > 0 { total_sim / total_memories as f64 } else { 0.0 };

        Ok(SleepResult {
            stats: SleepStats {
                total_delta,
                passes_done: passes,
                total_memories,
                avg_similarity: avg_sim,
                delta_per_pass,
            },
            config: self.config.clone(),
        })
    }

    pub fn should_sleep(&self, bank: &ReasoningBank) -> bool {
        let mems = bank.memories();
        let recent_count = mems.iter()
            .filter(|m| m.reward >= self.config.consolidation.min_reward)
            .count();
        recent_count >= 3
    }

    pub fn updater(&self) -> &HebbianUpdater {
        &self.updater
    }

    pub fn consolidator(&self) -> &MemoryConsolidation {
        &self.consolidator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_bank::{ReasoningMemory, MemoryTier, MemoryLifecycle, T3Views};
    use crate::core::{RewardSource, TaskType};

    fn make_memory(reward: f64, success: bool, id: &str) -> ReasoningMemory {
        ReasoningMemory {
            id: id.to_string(),
            task_description: format!("task_{}", id),
            task_type: TaskType::General,
            micro_edits: vec![],
            reward,
            reward_source: RewardSource::Internal,
            success,
            timestamp: 0,
            embedding: Some(vec![0.5; 128]),
            tier: MemoryTier::Episodic,
            lifecycle: MemoryLifecycle {
                importance: reward,
                confidence: 0.5,
                access_count: 0,
                created_at: 0,
                last_accessed: 0,
                ttl_seconds: None,
            },
            t3_views: T3Views::new(),
        }
    }

    fn setup_engine_and_state() -> (SleepEngine, CapabilityVector, ReasoningBank, SelectableOperator, SelectiveState) {
        let engine = SleepEngine::with_passes(3);
        let brain = CapabilityVector::default();
        let bank = {
            let mut b = ReasoningBank::new(100);
            for i in 0..5 {
                b.store(make_memory(0.5 + (i as f64 * 0.1), true, &format!("m{}", i)));
            }
            b
        };
        let operator = SelectableOperator::new(23, 64);
        let state = SelectiveState::new(23, 64);
        (engine, brain, bank, operator, state)
    }

    #[test]
    fn test_sleep_returns_result() {
        let (mut engine, mut brain, mut bank, operator, mut state) = setup_engine_and_state();
        let result = engine.sleep(&mut brain, &mut bank, &operator, &mut state).expect("value should be ok in test");
        assert_eq!(result.stats.passes_done, 3, "should execute 3 passes");
        assert!(result.stats.total_memories > 0, "should process memories");
    }

    #[test]
    fn test_sleep_all_passes_produce_delta() {
        let (mut engine, mut brain, mut bank, operator, mut state) = setup_engine_and_state();
        let result = engine.sleep(&mut brain, &mut bank, &operator, &mut state).expect("value should be ok in test");
        assert_eq!(result.stats.delta_per_pass.len(), 3, "should have 3 deltas");
        assert!(result.stats.delta_per_pass.iter().all(|d| *d >= 0.0),
            "all pass deltas should be non-negative");
    }

    #[test]
    fn test_sleep_zero_passes() {
        let mut engine = SleepEngine::new(SleepConfig { passes: 0, ..Default::default() });
        let mut brain = CapabilityVector::default();
        let mut bank = ReasoningBank::new(10);
        let operator = SelectableOperator::new(23, 64);
        let mut state = SelectiveState::new(23, 64);
        let result = engine.sleep(&mut brain, &mut bank, &operator, &mut state).expect("value should be ok in test");
        assert_eq!(result.stats.passes_done, 0, "zero passes returns immediately");
    }

    #[test]
    fn test_should_sleep_with_enough_memories() {
        let engine = SleepEngine::with_passes(3);
        let bank = {
            let mut b = ReasoningBank::new(100);
            for i in 0..5 {
                b.store(make_memory(0.5, true, &format!("m{}", i)));
            }
            b
        };
        assert!(engine.should_sleep(&bank), "should sleep with 5 qualifying memories");
    }

    #[test]
    fn test_should_sleep_with_few_memories() {
        let engine = SleepEngine::with_passes(3);
        let bank = ReasoningBank::new(100);
        assert!(!engine.should_sleep(&bank), "should not sleep with empty bank");
    }

    #[test]
    fn test_light_sleep_config() {
        let config = SleepConfig::light_sleep();
        assert_eq!(config.passes, 1, "light sleep has 1 pass");
        assert!(config.transition_noise < 0.01, "light sleep has less noise");
    }

    #[test]
    fn test_deep_sleep_config() {
        let config = SleepConfig::deep_sleep();
        assert_eq!(config.passes, 6, "deep sleep has 6 passes");
        assert!(config.consolidation_rate > 0.05, "deep sleep has higher consolidation rate");
    }

    #[test]
    fn test_capability_changes_after_sleep() {
        let (mut engine, mut brain, mut bank, operator, mut state) = setup_engine_and_state();
        let before = brain.arr.clone();
        engine.sleep(&mut brain, &mut bank, &operator, &mut state).expect("value should be ok in test");
        let changed = brain.arr.iter().zip(before.iter()).any(|(a, b)| (a - b).abs() > 1e-6);
        assert!(changed, "capability should change after sleep");
    }
}
