// Re-export moved traits from neotrix-mind
pub use neotrix_mind::traits::{ConsciousnessHandle, SealResult, ToolExecutor};

// Remaining core-local traits (not yet migrated — depend on non-migrated types)

use crate::core::nt_core_bank::{ReasoningBankStats, ReasoningMemory};
use crate::core::nt_core_cap::CapabilityVector;
use crate::core::nt_core_knowledge::{AbsorptionRecord, KnowledgeSource};

pub trait MemoryProvider {
    fn store(&mut self, key: &str, value: &str) -> Result<String, String>;
    fn search(&self, query: &str, limit: usize) -> Result<Vec<(String, String)>, String>;
    fn delete(&mut self, key: &str) -> Result<(), String>;
}

pub trait RichMemoryProvider: Send + Sync {
    fn store_memory(&mut self, memory: ReasoningMemory) -> bool;
    fn recall_similar(&self, query: &str, limit: usize) -> Vec<ReasoningMemory>;
    fn stats(&self) -> ReasoningBankStats;
}

pub trait AgentExecutor {
    type Output;
    fn execute(&mut self, task: &str) -> Result<Self::Output, String>;
    fn interrupt(&mut self) -> Result<(), String>;
    fn status(&self) -> String;
    fn capability(&self) -> &CapabilityVector;
    fn capability_mut(&mut self) -> &mut CapabilityVector;
}

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

pub trait SessionProvider {
    type Session;
    fn create_session(&mut self, id: &str, name: &str) -> Self::Session;
    fn switch_session(&mut self, id: &str) -> bool;
    fn active_session(&self) -> Option<&Self::Session>;
    fn list_sessions(&self) -> Vec<&Self::Session>;
}

pub use super::nt_core_knowledge::KnowledgeProvider;

pub trait BrainProvider: Send + Sync {
    fn capability_vector(&self) -> CapabilityVector;
    fn absorb_knowledge(&mut self, source: KnowledgeSource) -> AbsorptionRecord;
    fn run_seal_iteration(&mut self) -> SealResult;
}

pub trait NeEditHandler {
    fn apply_ne_edit(&mut self, target: &str, value: f64) -> String;
}

pub trait EngineProvider: Send + Sync {
    fn reason(&mut self, prompt: &str) -> Result<String, String>;
}

/// No-op implementation of ConsciousnessHandle for contexts where
/// SEAL needs a CI handle but no actual consciousness cycle is running.
pub struct NoopCI;

impl ConsciousnessHandle for NoopCI {
    fn apply_ne_edit(&mut self, _target: &str, _value: f64) -> String { String::new() }
    fn stats_c_score(&self) -> f64 { 0.5 }
    fn cognitive_load(&self) -> f64 { 0.5 }
    fn self_evolution_best_score(&self) -> f64 { 0.5 }
    fn eval_ne_string(&mut self, _expr: &str) -> Result<String, String> { Ok(String::new()) }
    fn set_self_evolution_archive(&mut self, _best_score: f64) {}
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
        let o = ToolOutput {
            success: true,
            content: "done".into(),
        };
        assert!(o.success);
        assert_eq!(o.content, "done");
    }

    #[test]
    fn test_tool_output_failure() {
        let o = ToolOutput {
            success: false,
            content: "error".into(),
        };
        assert!(!o.success);
    }

    #[test]
    fn test_tool_def_display_trait() {
        let t = ToolDef {
            name: "calc".into(),
            description: "Calculator".into(),
            input_schema: serde_json::json!({}),
        };
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
