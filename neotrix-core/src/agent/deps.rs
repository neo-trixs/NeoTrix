use crate::core::{BrainProvider, CapabilityVector, EngineProvider, RichMemoryProvider};
use std::sync::{Arc, Mutex};

/// Agent 依赖容器 — 隔离 agent/ 对 core/ 具体类型的直接依赖
pub struct AgentDeps {
    pub memory: Option<Box<dyn RichMemoryProvider + Send>>,
    pub brain: Option<Arc<Mutex<dyn BrainProvider + Send>>>,
    pub engine: Option<Arc<Mutex<dyn EngineProvider + Send>>>,
}

impl AgentDeps {
    pub fn new() -> Self {
        Self {
            memory: None,
            brain: None,
            engine: None,
        }
    }

    pub fn with_memory(mut self, m: Box<dyn RichMemoryProvider + Send>) -> Self {
        self.memory = Some(m);
        self
    }

    pub fn with_brain(mut self, b: Arc<Mutex<dyn BrainProvider + Send>>) -> Self {
        self.brain = Some(b);
        self
    }

    pub fn with_engine(mut self, e: Arc<Mutex<dyn EngineProvider + Send>>) -> Self {
        self.engine = Some(e);
        self
    }

    pub fn memory(&self) -> Option<&dyn RichMemoryProvider> {
        self.memory
            .as_ref()
            .map(|m| -> &dyn RichMemoryProvider { m.as_ref() })
    }

    pub fn brain(&self) -> Option<&Arc<Mutex<dyn BrainProvider + Send>>> {
        self.brain.as_ref()
    }

    pub fn engine(&self) -> Option<&Arc<Mutex<dyn EngineProvider + Send>>> {
        self.engine.as_ref()
    }
}

impl Default for AgentDeps {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_deps_new_all_none() {
        let deps = AgentDeps::new();
        assert!(deps.memory.is_none());
        assert!(deps.brain.is_none());
        assert!(deps.engine.is_none());
    }

    #[test]
    fn test_agent_deps_default() {
        let deps = AgentDeps::default();
        assert!(deps.memory.is_none());
    }

    #[test]
    fn test_agent_deps_builder_pattern() {
        let deps = AgentDeps::new()
            .with_memory(Box::new(MockMemoryProvider))
            .with_brain(Arc::new(Mutex::new(MockBrainProvider)))
            .with_engine(Arc::new(Mutex::new(MockEngineProvider)));
        assert!(deps.memory.is_some());
        assert!(deps.brain.is_some());
        assert!(deps.engine.is_some());
    }

    #[test]
    fn test_agent_deps_memory_accessor() {
        let deps = AgentDeps::new().with_memory(Box::new(MockMemoryProvider));
        let mem = deps.memory();
        assert!(mem.is_some());
    }

    #[test]
    fn test_agent_deps_memory_accessor_none() {
        let deps = AgentDeps::new();
        assert!(deps.memory().is_none());
    }

    #[test]
    fn test_agent_deps_brain_accessor() {
        let deps = AgentDeps::new().with_brain(Arc::new(Mutex::new(MockBrainProvider)));
        assert!(deps.brain().is_some());
    }

    #[test]
    fn test_agent_deps_engine_accessor() {
        let deps = AgentDeps::new().with_engine(Arc::new(Mutex::new(MockEngineProvider)));
        assert!(deps.engine().is_some());
    }

    use crate::core::nt_core_bank::ReasoningMemory;
    use crate::core::nt_core_knowledge::{AbsorptionRecord, KnowledgeSource};
    use crate::core::SealResult;

    struct MockMemoryProvider;
    impl RichMemoryProvider for MockMemoryProvider {
        fn store_memory(&mut self, _memory: ReasoningMemory) -> bool {
            true
        }
        fn recall_similar(&self, _query: &str, _limit: usize) -> Vec<ReasoningMemory> {
            vec![]
        }
        fn stats(&self) -> crate::core::nt_core_bank::ReasoningBankStats {
            crate::core::nt_core_bank::ReasoningBankStats {
                total_memories: 0,
                success_count: 0,
                success_rate: 0.0,
            }
        }
    }

    struct MockBrainProvider;
    impl BrainProvider for MockBrainProvider {
        fn capability_vector(&self) -> CapabilityVector {
            CapabilityVector::default()
        }
        fn absorb_knowledge(&mut self, source: KnowledgeSource) -> AbsorptionRecord {
            AbsorptionRecord {
                source,
                timestamp: 0,
                weight: 1.0,
            }
        }
        fn run_seal_iteration(&mut self) -> SealResult {
            SealResult {
                score_before: 0.5,
                score_after: 0.6,
                delta: 0.1,
                iterations: 1,
            }
        }
    }

    struct MockEngineProvider;
    impl EngineProvider for MockEngineProvider {
        fn reason(&mut self, _prompt: &str) -> Result<String, String> {
            Ok("ok".into())
        }
    }
}

/// 可吸收的知识来源 trait — 替代 agent/ 对 KnowledgeSource 枚举的直接依赖
pub trait AbsorbableSource {
    fn source_name(&self) -> String;
    fn capability_vector(&self) -> CapabilityVector;
    fn source_weight(&self) -> f64;
}
