/// Internet Absorption Bridge — seeds evolution tasks from discovered patterns.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPattern {
    pub name: String,
    pub source: String,
    pub description: String,
    pub key_insight: String,
    pub relevance_to_neotrix: String,
    pub priority: u8, // 1-5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternetAbsorptionBridge {
    pub discovered_patterns: Vec<DiscoveredPattern>,
    pub absorbed_names: Vec<String>,
    pub patterns_to_task_counter: u64,
}

impl InternetAbsorptionBridge {
    pub fn new() -> Self {
        Self {
            discovered_patterns: Vec::new(),
            absorbed_names: Vec::new(),
            patterns_to_task_counter: 0,
        }
    }

    /// Seed with patterns discovered from internet search.
    /// Each pattern can later be converted into an evolution task.
    pub fn seed_discovered_patterns(&mut self, patterns: Vec<DiscoveredPattern>) {
        for p in patterns {
            if !self.discovered_patterns.iter().any(|e| e.name == p.name) {
                self.discovered_patterns.push(p);
            }
        }
    }

    /// Seed with known 2026 self-evolving architecture patterns.
    pub fn seed_known_2026_patterns(&mut self) {
        let patterns = vec![
            DiscoveredPattern {
                name: "SEPL_5Step".into(),
                source: "arXiv 2604.15034 — AGP/AGS".into(),
                description: "Self-Evolution Protocol Layer: reflect→select→improve→evaluate→commit".into(),
                key_insight: "Evolution should be traceable, reversible, and safe-by-construction via typed operator algebra".into(),
                relevance_to_neotrix: "NeoTrix has SelfEvolutionMetaLayer but not organized as SEPL 5-step with typed operators".into(),
                priority: 3,
            },
            DiscoveredPattern {
                name: "APEX_3Layer".into(),
                source: "arXiv 2606.15363 — APEX".into(),
                description: "Three-layer co-evolution: harness patching (L1), principle distillation (L2), workflow topology (L3)".into(),
                key_insight: "Multi-dimensional co-evolution (+90% health score) outperforms single-axis harness optimization".into(),
                relevance_to_neotrix: "NeoTrix only evolves harness (SelfEvolutionLoop). Missing principle distillation and workflow topology evolution.".into(),
                priority: 4,
            },
            DiscoveredPattern {
                name: "AgentFactory_3Phase".into(),
                source: "arXiv 2603.18000 — AgentFactory".into(),
                description: "Three-phase lifecycle: Install→Self-Evolve→Deploy. Subagent accumulation and reuse.".into(),
                key_insight: "Task decomposition into reusable subagents with automated refinement from execution feedback".into(),
                relevance_to_neotrix: "NeoTrix has SkillAccumulator but no systematic subagent accumulation/refinement cycle.".into(),
                priority: 3,
            },
            DiscoveredPattern {
                name: "GoedelAgent".into(),
                source: "arXiv 2410.04444 — Gödel Agent".into(),
                description: "Self-referential agent: analyzes and modifies own code including self-modification logic".into(),
                key_insight: "Monkey-patching at runtime enables full agent design space search via recursive self-improvement".into(),
                relevance_to_neotrix: "NeoTrix SEAL loop is close but lacks true self-referential meta-modification".into(),
                priority: 4,
            },
            DiscoveredPattern {
                name: "DGM_HyperAgents".into(),
                source: "arXiv 2603.19461 — DGM-H".into(),
                description: "Task agent + meta agent in same editable program. Archive-based open-ended exploration.".into(),
                key_insight: "Meta agent can rewrite itself — not just how tasks are done but how future self-improvements are generated".into(),
                relevance_to_neotrix: "NeoTrix evolution loop is fixed (not self-modifiable). DGM-H pattern would unlock true RSI.".into(),
                priority: 5,
            },
            DiscoveredPattern {
                name: "Yunjue_ISE".into(),
                source: "arXiv 2601.18226 — Yunjue Agent".into(),
                description: "In-situ Self-Evolving: zero-start, parallel batch evolution, tool synthesis on-the-fly".into(),
                key_insight: "Parallel batch evolution with LLM-based tool merging for convergence acceleration".into(),
                relevance_to_neotrix: "NeoTrix has parallel agents but no batch consolidation or tool merging".into(),
                priority: 2,
            },
            DiscoveredPattern {
                name: "Live_SWE_Agent".into(),
                source: "github.com/OpenAutoCoder/Live-SWE-agent".into(),
                description: "Runtime self-evolving SWE agent, 79.2% SWE-bench Verified. First OSS scaffold to beat proprietary scaffolds.".into(),
                key_insight: "Runtime scaffold evolution: agent behavior improves mid-execution without retraining".into(),
                relevance_to_neotrix: "NeoTrix has offline SEAL but no runtime self-evolution during active consciousness cycles.".into(),
                priority: 4,
            },
            DiscoveredPattern {
                name: "Socratic_SWE".into(),
                source: "arXiv 2026 — Socratic-SWE".into(),
                description: "Co-evolutionary self-play: agents challenge each other, skill registry accumulates winning strategies".into(),
                key_insight: "Self-play co-evolution +7.8 points on SWE-bench through adversarial skill acquisition".into(),
                relevance_to_neotrix: "NeoTrix has AdversarialArena but no skill registry from arena outcomes.".into(),
                priority: 3,
            },
            DiscoveredPattern {
                name: "SelfEvolve".into(),
                source: "arXiv 2026 — LADDER/SelfEvolve".into(),
                description: "92.7% Pass@1 for runtime code generation via TDD pipeline: generate→test→repair→evolve".into(),
                key_insight: "TDD-style pipeline is central to self-evolution: failing tests are evolution signals, not errors".into(),
                relevance_to_neotrix: "NeoTrix has self_evolution_loop but no test-as-signal paradigm. Tests are separate from evolution.".into(),
                priority: 3,
            },
            DiscoveredPattern {
                name: "Skill_RSI".into(),
                source: "arXiv 2606.xxxxx — Skill-RSI".into(),
                description: "Recursive self-improvement with ontology-driven skill evolution: skill→trace→reflect→refine→store".into(),
                key_insight: "Skills evolve their own improvement strategies via meta-skill ontology — meta layer is itself a skill".into(),
                relevance_to_neotrix: "NeoTrix has SkillAccumulator but skills don't self-evolve their improvement strategies.".into(),
                priority: 5,
            },
            DiscoveredPattern {
                name: "SE_Agent".into(),
                source: "arXiv 2026 — SE-Agent".into(),
                description: "+55% improvement via 3-stage evolution: revision→recombination→refinement with fitness-based selection".into(),
                key_insight: "Three distinct evolutionary operators (revision/recombination/refinement) outperform single-operator evolution 3x".into(),
                relevance_to_neotrix: "NeoTrix evolution is single-operator (tune mutations only). Missing recombination and refinement operators.".into(),
                priority: 4,
            },
            DiscoveredPattern {
                name: "leOS".into(),
                source: "github.com — leOS VSA-native OS".into(),
                description: "VSA-native operating system: everything is a hypervector. Includes dreaming engine, void detection, skill assimilation.".into(),
                key_insight: "VSA-native architecture eliminates all format conversion overhead. Dreaming engine runs periodic consolidation autonomously.".into(),
                relevance_to_neotrix: "NeoTrix uses VSA but not as native OS. DreamCycle exists but runs on schedule, not event-driven.".into(),
                priority: 3,
            },
            DiscoveredPattern {
                name: "RCK_ResonantCognitiveKernel".into(),
                source: "github.com/beyond-repair/RCK".into(),
                description: "Resonant Cognitive Kernel: 714 tests, auditable reasoning on VSA, resonator network for binding, cognitive microservice architecture.".into(),
                key_insight: "Resonator networks for VSA binding achieve auditable reasoning with proven correctness. 714 tests verify the entire cognitive stack.".into(),
                relevance_to_neotrix: "NeoTrix has VSA but no resonator binding. RCK's auditable reasoning pattern could replace NeoTrix's test-free VSA operations.".into(),
                priority: 4,
            },
            DiscoveredPattern {
                name: "SEEM_CognitiveMicroservice".into(),
                source: "github.com/beyond-repair/SEEM".into(),
                description: "Offline-first symbolic AI kernel: Resonator VSA (ℂ^16384), BaNEL negative learning, Dream consolidation, SHACL governance.".into(),
                key_insight: "Offline-first design with BaNEL (negative learning from failures) and SHACL governance for safe self-modification. Closest architectural cousin to NeoTrix.".into(),
                relevance_to_neotrix: "NeoTrix lacks negative learning (BaNEL) and governance layer (SHACL). These are critical for safe self-evolution.".into(),
                priority: 5,
            },
        ];
        self.seed_discovered_patterns(patterns);
    }

    /// Convert discovered patterns into evolution tasks.
    /// Each unabsorbed pattern generates one TaskType::AbsorbPattern.
    pub fn generate_absorption_tasks(
        &mut self,
    ) -> Vec<crate::core::nt_core_experience::self_evolution_task_engine::EngineEvolutionTask> {
        use crate::core::nt_core_experience::self_evolution_task_engine::{
            EngineEvolutionTask, EngineTaskStatus, EngineTaskType,
        };
        let mut tasks = Vec::new();
        for pattern in &self.discovered_patterns {
            if self.absorbed_names.contains(&pattern.name) {
                continue;
            }
            self.patterns_to_task_counter += 1;
            tasks.push(EngineEvolutionTask {
                id: 9000 + self.patterns_to_task_counter,
                target_gap: format!("absorb:{}", pattern.name),
                description: format!(
                    "Absorb pattern '{}' from {}: {}",
                    pattern.name, pattern.source, pattern.description
                ),
                task_type: EngineTaskType::AbsorbPattern {
                    repo_url: pattern.source.clone(),
                    pattern_name: pattern.name.clone(),
                },
                status: EngineTaskStatus::Proposed,
                created_cycle: 0,
                completed_cycle: None,
                prerequisite_ids: Vec::new(),
            });
        }
        tasks
    }

    /// Mark a pattern as absorbed.
    pub fn mark_absorbed(&mut self, name: &str) {
        if !self.absorbed_names.contains(&name.to_string()) {
            self.absorbed_names.push(name.to_string());
        }
    }

    pub fn unabsorbed_count(&self) -> usize {
        self.discovered_patterns.len() - self.absorbed_names.len()
    }

    pub fn summary(&self) -> String {
        format!(
            "InternetAbsorptionBridge: {} patterns ({} absorbed, {} pending)",
            self.discovered_patterns.len(),
            self.absorbed_names.len(),
            self.unabsorbed_count()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_has_no_patterns() {
        let bridge = InternetAbsorptionBridge::new();
        assert_eq!(bridge.discovered_patterns.len(), 0);
        assert_eq!(bridge.unabsorbed_count(), 0);
    }

    #[test]
    fn test_seed_adds_patterns() {
        let mut bridge = InternetAbsorptionBridge::new();
        bridge.seed_known_2026_patterns();
        assert_eq!(bridge.discovered_patterns.len(), 14);
        assert_eq!(bridge.unabsorbed_count(), 14);
    }

    #[test]
    fn test_generate_tasks_creates_one_per_unabsorbed() {
        use crate::core::nt_core_experience::self_evolution_task_engine::EngineTaskType;
        let mut bridge = InternetAbsorptionBridge::new();
        bridge.seed_known_2026_patterns();
        let tasks = bridge.generate_absorption_tasks();
        assert_eq!(tasks.len(), 14);
        assert!(tasks
            .iter()
            .all(|t| matches!(t.task_type, EngineTaskType::AbsorbPattern { .. })));
    }

    #[test]
    fn test_no_duplicate_tasks() {
        let mut bridge = InternetAbsorptionBridge::new();
        bridge.seed_known_2026_patterns();
        let first = bridge.generate_absorption_tasks();
        let second = bridge.generate_absorption_tasks();
        assert_eq!(first.len(), 14);
        assert_eq!(second.len(), 14); // Not yet absorbed
    }

    #[test]
    fn test_mark_absorbed_removes_from_pending() {
        let mut bridge = InternetAbsorptionBridge::new();
        bridge.seed_known_2026_patterns();
        bridge.mark_absorbed("SEPL_5Step");
        bridge.mark_absorbed("GoedelAgent");
        assert_eq!(bridge.unabsorbed_count(), 12);
    }

    #[test]
    fn test_tasks_only_for_unabsorbed() {
        let mut bridge = InternetAbsorptionBridge::new();
        bridge.seed_known_2026_patterns();
        bridge.mark_absorbed("APEX_3Layer");
        bridge.mark_absorbed("DGM_HyperAgents");
        let tasks = bridge.generate_absorption_tasks();
        let names: Vec<&str> = tasks.iter().map(|t| t.target_gap.as_str()).collect();
        assert!(names.contains(&"absorb:SEPL_5Step"));
        assert!(names.contains(&"absorb:GoedelAgent"));
        assert!(!names.contains(&"absorb:APEX_3Layer"));
        assert!(!names.contains(&"absorb:DGM_HyperAgents"));
    }

    #[test]
    fn test_summary_format() {
        let mut bridge = InternetAbsorptionBridge::new();
        bridge.seed_known_2026_patterns();
        bridge.mark_absorbed("SEPL_5Step");
        let s = bridge.summary();
        assert!(s.contains("14 patterns"));
        assert!(s.contains("1 absorbed"));
        assert!(s.contains("13 pending"));
    }
}
