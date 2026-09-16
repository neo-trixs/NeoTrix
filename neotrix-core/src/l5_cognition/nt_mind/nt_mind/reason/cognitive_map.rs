pub struct CognitiveMap {
    pub llm_to_neotrix: Vec<MappingEntry>,
}

pub struct MappingEntry {
    pub llm_concept: String,
    pub neotrix_abstraction: String,
    pub module_path: String,
    pub bidirectional: bool,
    pub notes: String,
}

impl Default for CognitiveMap {
    fn default() -> Self {
        Self::new()
    }
}

impl CognitiveMap {
    pub fn new() -> Self {
        Self {
            llm_to_neotrix: vec![
                MappingEntry {
                    llm_concept: "Attention Head".into(),
                    neotrix_abstraction: "SpecialistModule in GWT".into(),
                    module_path: "core/consciousness/module_def.rs".into(),
                    bidirectional: true,
                    notes: "Each AttentionHead maps to a SpecialistType, competes for broadcast via salience".into(),
                },
                MappingEntry {
                    llm_concept: "Context Window".into(),
                    neotrix_abstraction: "ContextWindow (core/thinking_model)".into(),
                    module_path: "core/thinking_model/context_window.rs".into(),
                    bidirectional: true,
                    notes: "Sliding window of CognitiveUnits with attention_mask for focus tracking".into(),
                },
                MappingEntry {
                    llm_concept: "Self-Identity / System Prompt".into(),
                    neotrix_abstraction: "SystemIdentity (core/thinking_model)".into(),
                    module_path: "core/thinking_model/system_identity.rs".into(),
                    bidirectional: false,
                    notes: "Capabilities + values + knowledge_boundary — maps to AGENTS.md rules".into(),
                },
                MappingEntry {
                    llm_concept: "Reasoning Strategy".into(),
                    neotrix_abstraction: "ReasoningStrategyRegistry (core/thinking_model)".into(),
                    module_path: "core/thinking_model/reasoning_strategy.rs".into(),
                    bidirectional: true,
                    notes: "10 strategies (Direct/CoT/Reflection/etc.). Auto-select based on task complexity".into(),
                },
                MappingEntry {
                    llm_concept: "Thinking Trace (Chain-of-Thought)".into(),
                    neotrix_abstraction: "ThinkingTrace (core/thinking_model)".into(),
                    module_path: "core/thinking_model/thinking_trace.rs".into(),
                    bidirectional: false,
                    notes: "Records reasoning steps, tools used, confidence, grade. Feeds self-reflection".into(),
                },
                MappingEntry {
                    llm_concept: "Self-Reflection / Metacognition".into(),
                    neotrix_abstraction: "MetaCognitiveLoop (core/metacognition)".into(),
                    module_path: "core/metacognition/metacognition_loop.rs".into(),
                    bidirectional: true,
                    notes: "SCAN→ANALYZE→MONITOR→PLAN→REPORT cycle for self-awareness".into(),
                },
                MappingEntry {
                    llm_concept: "Knowledge Retrieval".into(),
                    neotrix_abstraction: "KnowledgeHyperCube (core/hypercube) + ReasoningBank".into(),
                    module_path: "core/hypercube/cube.rs".into(),
                    bidirectional: true,
                    notes: "VSA-based binding for associative recall. ReasoningBank for episodic memory".into(),
                },
                MappingEntry {
                    llm_concept: "Token Prediction → Action".into(),
                    neotrix_abstraction: "ToolCall (core/edit) + ThinkingStep (thinking_model)".into(),
                    module_path: "core/edit.rs".into(),
                    bidirectional: false,
                    notes: "Each tool call / action is a 'predicted token' in my action space".into(),
                },
                MappingEntry {
                    llm_concept: "Working Memory".into(),
                    neotrix_abstraction: "L1 Memory (core/memory/l1) + ContextWindow".into(),
                    module_path: "core/memory/l1.rs".into(),
                    bidirectional: true,
                    notes: "Fast-access scratch space. L1 = short-term, ContextWindow = current focus".into(),
                },
                MappingEntry {
                    llm_concept: "Long-term Memory".into(),
                    neotrix_abstraction: "L2/L3 Memory (core/memory) + ReasoningBank".into(),
                    module_path: "core/memory/tier.rs".into(),
                    bidirectional: true,
                    notes: "Tiered memory: L1 (working) → L2 (medium) → L3 (archival)".into(),
                },
                MappingEntry {
                    llm_concept: "Attention Salience".into(),
                    neotrix_abstraction: "GWT Competition (consciousness/workspace)".into(),
                    module_path: "core/consciousness/workspace.rs".into(),
                    bidirectional: true,
                    notes: "urgency × novelty × coherence = salience score for broadcast competition".into(),
                },
                MappingEntry {
                    llm_concept: "Goal Pursuit".into(),
                    neotrix_abstraction: "GoalLoop (nt_mind/goal_loop)".into(),
                    module_path: "nt_mind/goal_loop.rs".into(),
                    bidirectional: true,
                    notes: "Goal lifecycle: Pending→Pursuing→Achieved/Unmet. Rate-limited, circuit-breaker protected".into(),
                },
                MappingEntry {
                    llm_concept: "Model Self-Evolution".into(),
                    neotrix_abstraction: "SEAL loop (nt_mind/self_iterating)".into(),
                    module_path: "nt_mind/self_iterating.rs".into(),
                    bidirectional: false,
                    notes: "Self-Edit generate → apply → verify → absorb. External RL reward from tools/tests".into(),
                },
                MappingEntry {
                    llm_concept: "Knowledge Boundary".into(),
                    neotrix_abstraction: "GapReport·ExploreDomain (core/hypercube/gap + thinking_model/system_identity)".into(),
                    module_path: "core/hypercube/gap.rs".into(),
                    bidirectional: true,
                    notes: "What I don't know → triggers exploration/nt_world_crawl. Maps to WeaknessAnalyzer patterns".into(),
                },
                MappingEntry {
                    llm_concept: "Task Decomposition".into(),
                    neotrix_abstraction: "Orchestrator·PlannerNode (orchestrator)".into(),
                    module_path: "orchestrator/mod.rs".into(),
                    bidirectional: true,
                    notes: "Hierarchical task breakdown. RecursiveDecomposition strategy = Orchestrator pattern".into(),
                },
                MappingEntry {
                    llm_concept: "OS Resource Lifecycle".into(),
                    neotrix_abstraction: "WorldConsciousness.nt_world_sense + HotReloadWatcher.spawn()".into(),
                    module_path: "core/nt_world_sense/ + neotrix/hotreload/".into(),
                    bidirectional: false,
                    notes: "OS event sources (file watchers, socket listeners) must outlive their event handlers. Channel is a pipe, not the source.".into(),
                },
                MappingEntry {
                    llm_concept: "Structural Edit Integrity".into(),
                    neotrix_abstraction: "Iron law: after any structural edit (removing functions, reordering modules), verify brace matching + indent boundaries".into(),
                    module_path: "AGENTS.md + SelfCodeWriter".into(),
                    bidirectional: false,
                    notes: "Deleting code blocks can silently break surrounding scope boundaries. Verify {}/indent after every structural change.".into(),
                },
                MappingEntry {
                    llm_concept: "Sync/Async Channel Boundary".into(),
                    neotrix_abstraction: "tokio::sync::mpsc::UnboundedSender (Send+Sync) for sync→async bridging".into(),
                    module_path: "core/nt_world_sense/event.rs".into(),
                    bidirectional: false,
                    notes: "sync producer + async consumer → tokio::sync::mpsc. The sender is Send+Sync and can be called from non-tokio threads.".into(),
                },
            ],
        }
    }

    pub fn resolve(&self, llm_concept: &str) -> Option<&MappingEntry> {
        self.llm_to_neotrix
            .iter()
            .find(|e| e.llm_concept.to_lowercase() == llm_concept.to_lowercase())
    }

    pub fn by_module(&self, module: &str) -> Vec<&MappingEntry> {
        self.llm_to_neotrix
            .iter()
            .filter(|e| e.module_path.contains(module))
            .collect()
    }

    pub fn count(&self) -> usize {
        self.llm_to_neotrix.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cognitive_map_has_entries() {
        let map = CognitiveMap::new();
        assert!(map.count() >= 17);
    }

    #[test]
    fn test_resolve_concept() {
        let map = CognitiveMap::new();
        let entry = map
            .resolve("Attention Head")
            .expect("value should be ok in test");
        assert_eq!(entry.llm_concept, "Attention Head");
        assert!(entry.neotrix_abstraction.contains("SpecialistModule"));
    }

    #[test]
    fn test_resolve_case_insensitive() {
        let map = CognitiveMap::new();
        assert!(map.resolve("attention head").is_some());
        assert!(map.resolve("ATTENTION HEAD").is_some());
    }

    #[test]
    fn test_by_module_filter() {
        let map = CognitiveMap::new();
        let hypercube = map.by_module("hypercube");
        assert!(!hypercube.is_empty());
    }

    #[test]
    fn test_all_entries_have_modules() {
        let map = CognitiveMap::new();
        for entry in &map.llm_to_neotrix {
            assert!(
                !entry.module_path.is_empty(),
                "Module path missing for {}",
                entry.llm_concept
            );
            assert!(
                !entry.neotrix_abstraction.is_empty(),
                "NeoTrix mapping missing for {}",
                entry.llm_concept
            );
        }
    }

    #[test]
    fn test_resolve_nonexistent_returns_none() {
        let map = CognitiveMap::new();
        assert!(map
            .resolve("nonexistent concept that doesn't exist")
            .is_none());
    }

    #[test]
    fn test_covers_major_llm_concepts() {
        let map = CognitiveMap::new();
        let required = vec![
            "Attention",
            "Context",
            "Memory",
            "Goal",
            "Knowledge",
            "Reasoning",
        ];
        for concept in required {
            let found = map.llm_to_neotrix.iter().any(|e| {
                e.llm_concept
                    .to_lowercase()
                    .contains(&concept.to_lowercase())
            });
            assert!(found, "Missing mapping for concept: {}", concept);
        }
    }
}
