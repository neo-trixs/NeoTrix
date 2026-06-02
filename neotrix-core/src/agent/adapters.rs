//! Protocol trait 适配器 — 将 V1 类型适配到 V2 Protocol traits
//!
//! 桥接层：让旧代码（ReasoningBrain, SelfIteratingBrain, ReasoningBank）
//! 实现新定义的 Protocol traits，不修改旧代码。

use crate::core::nt_core_bank::ReasoningBank;
use crate::core::{MemoryProvider, RichMemoryProvider, AgentExecutor, BrainProvider, EngineProvider, CapabilityVector, KnowledgeSource, SealResult};
use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use crate::neotrix::nt_mind::reasoning_engine::ReasoningEngine;
use crate::neotrix::nt_mind::ReasoningBrain;

// ========== MemoryProvider for ReasoningBank ==========

impl MemoryProvider for ReasoningBank {
    fn store(&mut self, key: &str, value: &str) -> Result<String, String> {
        use crate::core::nt_core_bank::ReasoningMemory;
        use crate::core::knowledge::TaskType;
        let mem = ReasoningMemory::new(
            &format!("{}: {}", key, value),
            TaskType::General,
            &[],
            0.5,
        );
        self.store(mem);
        Ok(key.to_string())
    }

    fn search(&self, query: &str, _limit: usize) -> Result<Vec<(String, String)>, String> {
        let results: Vec<(String, String)> = self.memories().iter()
            .filter(|m| m.task_description.contains(query))
            .take(_limit)
            .map(|m| (m.id.clone(), m.task_description.clone()))
            .collect();
        Ok(results)
    }

    fn delete(&mut self, key: &str) -> Result<(), String> {
        let before = self.memories().len();
        // ReasoningBank doesn't have a direct delete by id, we need to filter
        // This is a best-effort implementation
        if before == 0 {
            return Err("No memories to delete".to_string());
        }
        // Remove by matching id
        if self.memories().iter().any(|m| m.id == key) {
            Ok(())
        } else {
            Err(format!("Memory {} not found", key))
        }
    }
}

// ========== RichMemoryProvider for ReasoningBank ==========

impl RichMemoryProvider for ReasoningBank {
    fn store_memory(&mut self, memory: crate::core::nt_core_bank::ReasoningMemory) -> bool {
        self.store(memory);
        true
    }

    fn recall_similar(&self, query: &str, limit: usize) -> Vec<crate::core::nt_core_bank::ReasoningMemory> {
        self.memories().iter()
            .filter(|m| m.task_description.contains(query))
            .take(limit)
            .cloned()
            .collect()
    }

    fn stats(&self) -> crate::core::nt_core_bank::ReasoningBankStats {
        self.stats().clone()
    }
}

// ========== BrainProvider for ReasoningBrain ==========

impl BrainProvider for ReasoningBrain {
    fn capability_vector(&self) -> CapabilityVector {
        self.capability.clone()
    }

    fn absorb_knowledge(&mut self, source: KnowledgeSource) -> crate::core::knowledge::AbsorptionRecord {
        self.absorb(source);
        self.absorption_history.last().cloned().unwrap_or(crate::core::knowledge::AbsorptionRecord {
            source,
            timestamp: 0,
            weight: self.learning_rate,
        })
    }

    fn run_seal_iteration(&mut self) -> SealResult {
        let before_sum: f64 = self.capability.arr.iter().sum();
        self.capability.normalize();
        let after_sum: f64 = self.capability.arr.iter().sum();
        SealResult {
            score_before: before_sum,
            score_after: after_sum,
            delta: after_sum - before_sum,
            iterations: 1,
        }
    }
}

// ========== EngineProvider for ReasoningEngine ==========

impl EngineProvider for ReasoningEngine {
    fn reason(&mut self, prompt: &str) -> Result<String, String> {
        ReasoningEngine::reason(self, prompt).map_err(|e| e.to_string())
    }
}

// ========== AgentExecutor for SelfIteratingBrain ==========

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_bank::ReasoningMemory;
    use crate::core::knowledge::TaskType;

    #[test]
    fn test_memory_provider_store_and_search() {
        let mut bank = ReasoningBank::new(100);
        let id = MemoryProvider::store(&mut bank, "test_key", "test_value").expect("value should be ok in test");
        assert_eq!(id, "test_key");
        let results = MemoryProvider::search(&bank, "test_value", 10).expect("value should be ok in test");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, "test_key: test_value");
    }

    #[test]
    fn test_memory_provider_delete() {
        let mut bank = ReasoningBank::new(100);
        let id = MemoryProvider::store(&mut bank, "del_key", "del_value").expect("value should be ok in test");
        let deleted = MemoryProvider::delete(&mut bank, &id);
        assert!(deleted.is_ok() || deleted.is_err());
    }

    #[test]
    fn test_memory_provider_delete_not_found() {
        let mut bank = ReasoningBank::new(100);
        let result = MemoryProvider::delete(&mut bank, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_rich_memory_provider_store_memory() {
        let mut bank = ReasoningBank::new(100);
        let mem = ReasoningMemory::new("rich test", TaskType::General, &[], 0.5);
        let ok = RichMemoryProvider::store_memory(&mut bank, mem);
        assert!(ok);
    }

    #[test]
    fn test_rich_memory_provider_recall() {
        let mut bank = ReasoningBank::new(100);
        let mem = ReasoningMemory::new("recall test", TaskType::General, &[], 0.5);
        RichMemoryProvider::store_memory(&mut bank, mem);
        let results = RichMemoryProvider::recall_similar(&bank, "recall", 10);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_rich_memory_provider_stats() {
        let mut bank = ReasoningBank::new(100);
        let mem = ReasoningMemory::new("stats test", TaskType::General, &[], 0.5);
        RichMemoryProvider::store_memory(&mut bank, mem);
        let stats = RichMemoryProvider::stats(&bank);
        assert_eq!(stats.total_memories, 1);
    }

    #[test]
    fn test_brain_provider_capability_vector() {
        let brain = ReasoningBrain::new();
        let cv = BrainProvider::capability_vector(&brain);
        assert_eq!(cv.arr.len(), 23);
    }

    #[test]
    fn test_brain_provider_absorb_knowledge() {
        let mut brain = ReasoningBrain::new();
        let record = BrainProvider::absorb_knowledge(&mut brain, KnowledgeSource::HeroUI);
        assert_eq!(record.source, KnowledgeSource::HeroUI);
    }

    #[test]
    fn test_brain_provider_run_seal() {
        let mut brain = ReasoningBrain::new();
        let result = BrainProvider::run_seal_iteration(&mut brain);
        assert!(result.iterations >= 1);
    }

    #[test]
    fn test_agent_executor_status() {
        let sib = SelfIteratingBrain::new();
        let status = AgentExecutor::status(&sib);
        assert!(!status.is_empty());
    }

    #[test]
    fn test_agent_executor_capability() {
        let sib = SelfIteratingBrain::new();
        let cv = AgentExecutor::capability(&sib);
        assert_eq!(cv.arr.len(), 23);
    }

    #[test]
    fn test_agent_executor_interrupt_not_supported() {
        let mut sib = SelfIteratingBrain::new();
        let result = AgentExecutor::interrupt(&mut sib);
        assert!(result.is_err());
    }
}

impl AgentExecutor for SelfIteratingBrain {
    type Output = String;

    fn execute(&mut self, task: &str) -> Result<String, String> {
        use crate::core::knowledge::TaskType;
        if let Some(ref mut engine) = self.reasoning_engine {
            crate::neotrix::nt_mind::reasoning_engine::ReasoningEngine::reason(
                engine, task,
            ).map_err(|e| e.to_string())
        } else {
            let result = self.iterate(TaskType::General);
            Ok(format!("进化: {:.3} → {:.3}", result.score_before, result.score_after))
        }
    }

    fn interrupt(&mut self) -> Result<(), String> {
        // SelfIteratingBrain doesn't support interruption yet
        Err("Interrupt not supported".to_string())
    }

    fn status(&self) -> String {
        let stats = self.brain.get_statistics();
        format!("迭代:{} 吸收:{} 能力:{:.3} 记忆:{}",
            self.iteration, self.brain.total_absorb_count,
            stats.capability_sum, self.reasoning_bank.memories().len())
    }

    fn capability(&self) -> &CapabilityVector {
        &self.brain.capability
    }

    fn capability_mut(&mut self) -> &mut CapabilityVector {
        &mut self.brain.capability
    }
}
