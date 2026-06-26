//! # NeoTrix Core Protocol Traits
//!
//! 核心接口定义，解耦各层之间的直接引用。
//! 所有实现都在各自层中，traits 本身在 core 层。

use crate::core::nt_core_cap::CapabilityVector;
use crate::core::nt_core_knowledge::{KnowledgeSource, AbsorptionRecord, TaskType};
use crate::core::nt_core_bank::{ReasoningMemory, ReasoningBankStats, ReasoningBank};

/// MemoryProvider — 记忆存储/检索抽象
pub trait MemoryProvider {
    fn store(&mut self, key: &str, value: &str) -> Result<String, String>;
    fn search(&self, query: &str, limit: usize) -> Result<Vec<(String, String)>, String>;
    fn delete(&mut self, key: &str) -> Result<(), String>;
}

/// RichMemoryProvider — 针对 ReasoningBank 的完整记忆抽象
pub trait RichMemoryProvider: Send + Sync {
    fn store_memory(&mut self, memory: ReasoningMemory) -> bool;
    fn recall_similar(&self, query: &str, limit: usize) -> Vec<ReasoningMemory>;
    fn stats(&self) -> ReasoningBankStats;
}

/// AgentExecutor — Agent 执行抽象
pub trait AgentExecutor {
    type Output;
    fn execute(&mut self, task: &str) -> Result<Self::Output, String>;
    fn interrupt(&mut self) -> Result<(), String>;
    fn status(&self) -> String;
    fn capability(&self) -> &CapabilityVector;
    fn capability_mut(&mut self) -> &mut CapabilityVector;
}

/// ToolProvider — 工具提供抽象
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

pub struct ToolOutput {
    pub success: bool,
    pub content: String,
}

pub trait ToolProvider {
    fn list_tools(&self) -> Vec<ToolDef>;
    fn call_tool(&self, name: &str, args: &serde_json::Value) -> Result<ToolOutput, String>;
}

/// SessionProvider — 会话管理抽象
pub trait SessionProvider {
    type Session;
    fn create_session(&mut self, id: &str, name: &str) -> Self::Session;
    fn switch_session(&mut self, id: &str) -> bool;
    fn active_session(&self) -> Option<&Self::Session>;
    fn list_sessions(&self) -> Vec<&Self::Session>;
}

/// KnowledgeProvider re-export (defined in knowledge.rs)
pub use super::nt_core_knowledge::KnowledgeProvider;

/// SealResult — SEAL 自迭代循环的结果
#[derive(Debug, Clone)]
pub struct SealResult {
    pub score_before: f64,
    pub score_after: f64,
    pub delta: f64,
    pub iterations: usize,
}

/// BrainProvider — 推理大脑抽象
pub trait BrainProvider: Send + Sync {
    fn capability_vector(&self) -> CapabilityVector;
    fn absorb_knowledge(&mut self, source: KnowledgeSource) -> AbsorptionRecord;
    fn run_seal_iteration(&mut self) -> SealResult;
    fn get_brain_report(&self) -> String;
}

/// EngineProvider — 推理引擎抽象
pub trait EngineProvider: Send + Sync {
    fn reason(&mut self, prompt: &str) -> Result<String, String>;
}

// ========== Orphan-rule-safe impls for ReasoningBank ==========
// These must live in the same crate as both the trait and the type.

impl MemoryProvider for ReasoningBank {
    fn store(&mut self, key: &str, value: &str) -> Result<String, String> {
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
        if self.memories().is_empty() {
            return Err("No memories to delete".to_string());
        }
        if self.memories().iter().any(|m| m.id == key) {
            Ok(())
        } else {
            Err(format!("Memory {} not found", key))
        }
    }
}

impl RichMemoryProvider for ReasoningBank {
    fn store_memory(&mut self, memory: ReasoningMemory) -> bool {
        self.store(memory);
        true
    }

    fn recall_similar(&self, query: &str, limit: usize) -> Vec<ReasoningMemory> {
        self.memories().iter()
            .filter(|m| m.task_description.contains(query))
            .take(limit)
            .cloned()
            .collect()
    }

    fn stats(&self) -> ReasoningBankStats {
        self.stats().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_def_construction() {
        let t = ToolDef {
            name: "test_tool".into(),
            description: "A test tool".into(),
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
        };
        assert_eq!(t.name, "test_tool");
        assert_eq!(t.description, "A test tool");
    }

    #[test]
    fn test_tool_output_success() {
        let o = ToolOutput { success: true, content: "done".into() };
        assert!(o.success);
        assert_eq!(o.content, "done");
    }

    #[test]
    fn test_tool_output_failure() {
        let o = ToolOutput { success: false, content: "error".into() };
        assert!(!o.success);
    }

    #[test]
    fn test_seal_result() {
        let r = SealResult { score_before: 0.5, score_after: 0.8, delta: 0.3, iterations: 5 };
        assert!((r.delta - 0.3).abs() < 1e-10);
        assert_eq!(r.iterations, 5);
    }

    #[test]
    fn test_seal_result_zero_delta() {
        let r = SealResult { score_before: 1.0, score_after: 1.0, delta: 0.0, iterations: 0 };
        assert!((r.delta).abs() < 1e-10);
    }

    #[test]
    fn test_tool_def_display_trait() {
        let t = ToolDef {
            name: "calc".into(),
            description: "Calculator".into(),
            input_schema: serde_json::json!({}),
        };
        let _debug = format!("{:?}", t.input_schema);
        assert!(serde_json::to_string(&t.input_schema).is_ok());
    }

    #[test]
    fn test_memory_provider_trait_object_safe() {
        fn _take_memory_provider(_: &dyn MemoryProvider) {}
        let _ = _take_memory_provider;
    }

    #[test]
    fn test_agent_executor_trait_object_safe() {
        fn _take_executor(_: &dyn AgentExecutor<Output = String>) {}
        let _ = _take_executor;
    }

    #[test]
    fn test_engine_provider_trait_object_safe() {
        fn _take_engine(_: &dyn EngineProvider) {}
        let _ = _take_engine;
    }
}
