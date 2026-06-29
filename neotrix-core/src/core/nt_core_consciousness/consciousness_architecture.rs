use std::collections::HashMap;

/// Current architecture version — bumped on each evolution round.
pub const ARCHITECTURE_VERSION: &str = "0.15.0"; // 2026-06-23: 13 new reasoning/meta/GWT modules wired
pub const ARCHITECTURE_BUILD: &str = "2026-06-23"; // freeze date
pub const ARCHITECTURE_MODULE_COUNT: usize = 150; // handlers in self_inspect
pub const ARCHITECTURE_LAYER_COUNT: usize = 6;

/// Human-readable changelog of architecture evolution.
pub fn architecture_changelog() -> Vec<(&'static str, &'static str)> {
    vec![
        ("0.15.0", "13 reasoning/meta/GWT modules + SEAL proposal bridge + full pipeline integration"),
        ("0.14.0", "OSINT evolution: real API sources, D-S fusion, identity correlation"),
        ("0.13.0", "SelfEvolutionMetaLayer: 5 feedback loops closed, 12-step cycle"),
        ("0.12.0", "Wave A-C: MCTS, PRM, selector, pruner, curiosity, self-interrupt"),
        ("0.11.0", "Architecture gap analysis v3: 25 gaps, 20-dimension cross-domain table"),
        ("0.10.0", "NTSSEG storage engine + MMLU/GSM8K/HumanEval benchmarks + image pipeline"),
        ("0.9.0", "Competitive landscape: hypergraph RAG, BFT consensus, EFE minimizer"),
        ("0.8.0", "Evidence tracking: 6-dimension scoring, provenance, Dempster-Shafer fusion"),
        ("0.7.0", "Ne language bootstrap: SelfInspectable → SystemCard → CodegenBridge"),
        ("0.6.0", "CapabilitySynthesizer + PDF pipeline + tool contracts"),
        ("0.5.0", "Phase 4 wiring: consciousness cycle, background loop, A2A bridge"),
        ("0.4.0", "Phase 2: cross-modal alignment, sleep cycle, theory of mind, value system"),
        ("0.3.0", "Phase 1: negentropy metric, curiosity drive, stagnation detector"),
        ("0.2.0", "Phase 0: VsaTag, FirstPersonRef, SpeciousPresent, 8 E8 state modules"),
        ("0.1.0", "Initial consciousness architecture: E8 64-state inference kernel + HyperCube VSA"),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArchitectureLayer {
    Substrate,
    Perception,
    Cognition,
    MetaCognition,
    SelfEvolution,
    MetaArchitecture,
}

impl ArchitectureLayer {
    pub fn name(&self) -> &'static str {
        match self {
            ArchitectureLayer::Substrate => "substrate",
            ArchitectureLayer::Perception => "perception",
            ArchitectureLayer::Cognition => "cognition",
            ArchitectureLayer::MetaCognition => "metacognition",
            ArchitectureLayer::SelfEvolution => "self_evolution",
            ArchitectureLayer::MetaArchitecture => "meta_architecture",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ArchitectureLayer::Substrate => "VSA vectors, E8 states, basic operations",
            ArchitectureLayer::Perception => "Sensory input: text, visual, audio, sensor",
            ArchitectureLayer::Cognition => {
                "E8 reasoning, MCTS planning, causal inference, analogy"
            }
            ArchitectureLayer::MetaCognition => {
                "Confidence calibration, self-critique, dead-end detection"
            }
            ArchitectureLayer::SelfEvolution => {
                "SEAL closed loop, experience distillation, architecture mutation"
            }
            ArchitectureLayer::MetaArchitecture => {
                "Self-model, gap analysis, evolution planning, roadmap generation"
            }
        }
    }

    pub fn all() -> &'static [ArchitectureLayer] {
        &[
            ArchitectureLayer::Substrate,
            ArchitectureLayer::Perception,
            ArchitectureLayer::Cognition,
            ArchitectureLayer::MetaCognition,
            ArchitectureLayer::SelfEvolution,
            ArchitectureLayer::MetaArchitecture,
        ]
    }

    pub fn priority(&self) -> u8 {
        match self {
            ArchitectureLayer::Substrate => 0,
            ArchitectureLayer::Perception => 1,
            ArchitectureLayer::Cognition => 2,
            ArchitectureLayer::MetaCognition => 3,
            ArchitectureLayer::SelfEvolution => 4,
            ArchitectureLayer::MetaArchitecture => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityStatus {
    Missing,
    Partial,
    Complete,
    Maturing,
}

impl CapabilityStatus {
    pub fn name(&self) -> &'static str {
        match self {
            CapabilityStatus::Missing => "missing",
            CapabilityStatus::Partial => "partial",
            CapabilityStatus::Complete => "complete",
            CapabilityStatus::Maturing => "maturing",
        }
    }

    pub fn score(&self) -> f64 {
        match self {
            CapabilityStatus::Missing => 0.0,
            CapabilityStatus::Partial => 0.33,
            CapabilityStatus::Complete => 0.66,
            CapabilityStatus::Maturing => 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CapabilityDescriptor {
    pub id: String,
    pub name: String,
    pub layer: ArchitectureLayer,
    pub status: CapabilityStatus,
    pub gap_ids: Vec<String>,
    pub module_path: Option<String>,
    pub estimated_lines: usize,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ArchitectureHealth {
    pub layer_scores: HashMap<ArchitectureLayer, f64>,
    pub overall_health: f64,
    pub total_capabilities: usize,
    pub completed_capabilities: usize,
    pub missing_capabilities: Vec<String>,
    pub critical_gaps: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ConsciousnessArchitecture {
    capabilities: HashMap<String, CapabilityDescriptor>,
    known_gaps: HashMap<String, GapInfo>,
}

#[derive(Debug, Clone)]
pub struct GapInfo {
    pub id: String,
    pub description: String,
    pub severity: GapSeverity,
    pub layer: ArchitectureLayer,
    pub target_capability: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapSeverity {
    Survival,
    Evolution,
    Enhancement,
}

impl GapSeverity {
    pub fn name(&self) -> &'static str {
        match self {
            GapSeverity::Survival => "survival",
            GapSeverity::Evolution => "evolution",
            GapSeverity::Enhancement => "enhancement",
        }
    }

    pub fn priority(&self) -> u8 {
        match self {
            GapSeverity::Survival => 0,
            GapSeverity::Evolution => 1,
            GapSeverity::Enhancement => 2,
        }
    }
}

impl Default for ConsciousnessArchitecture {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsciousnessArchitecture {
    pub fn new() -> Self {
        let mut arch = Self {
            capabilities: HashMap::new(),
            known_gaps: HashMap::new(),
        };
        arch.register_builtin_capabilities();
        arch.register_roadmap_gaps();
        arch
    }

    fn register_builtin_capabilities(&mut self) {
        let builtins = vec![
            // Layer 0: Substrate
            CapabilityDescriptor {
                id: "vsa_vectors".into(),
                name: "VSA Vector Operations".into(),
                layer: ArchitectureLayer::Substrate,
                status: CapabilityStatus::Maturing,
                gap_ids: vec![],
                module_path: Some("nt_core_hcube/vsa_vector.rs".into()),
                estimated_lines: 1200,
                dependencies: vec![],
            },
            CapabilityDescriptor {
                id: "e8_reasoning".into(),
                name: "E8 64-State Reasoning Kernel".into(),
                layer: ArchitectureLayer::Substrate,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_e8.rs".into()),
                estimated_lines: 700,
                dependencies: vec!["vsa_vectors".into()],
            },
            CapabilityDescriptor {
                id: "cross_modal_aligner".into(),
                name: "Cross-Modal VSA Alignment".into(),
                layer: ArchitectureLayer::Substrate,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_hcube/cross_modal.rs".into()),
                estimated_lines: 240,
                dependencies: vec!["vsa_vectors".into()],
            },
            CapabilityDescriptor {
                id: "spatial_scene".into(),
                name: "Spatial Scene Understanding".into(),
                layer: ArchitectureLayer::Substrate,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_hcube/spatial_scene.rs".into()),
                estimated_lines: 600,
                dependencies: vec!["vsa_vectors".into()],
            },
            // Layer 1: Perception
            CapabilityDescriptor {
                id: "text_perception".into(),
                name: "Text Input Perception".into(),
                layer: ArchitectureLayer::Perception,
                status: CapabilityStatus::Maturing,
                gap_ids: vec!["G303".into(), "G309".into()],
                module_path: Some("nt_core_consciousness/sensor_grounding.rs".into()),
                estimated_lines: 800,
                dependencies: vec!["vsa_vectors".into()],
            },
            CapabilityDescriptor {
                id: "pixel_perception".into(),
                name: "Pixel-Native Visual Perception".into(),
                layer: ArchitectureLayer::Perception,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G406".into()],
                module_path: Some("nt_core_consciousness/pixel_perception.rs".into()),
                estimated_lines: 250,
                dependencies: vec!["cross_modal_aligner".into(), "vsa_vectors".into()],
            },
            CapabilityDescriptor {
                id: "screenshot_pipeline".into(),
                name: "Chromium CDP Screenshot Capture".into(),
                layer: ArchitectureLayer::Perception,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G394".into()],
                module_path: Some("nt_core_consciousness/screenshot_pipeline.rs".into()),
                estimated_lines: 220,
                dependencies: vec!["pixel_perception".into()],
            },
            CapabilityDescriptor {
                id: "visual_embedding_frontend".into(),
                name: "Visual Embedding Model Frontend".into(),
                layer: ArchitectureLayer::Perception,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G395".into()],
                module_path: Some("nt_core_hcube/visual_embedding_frontend.rs".into()),
                estimated_lines: 340,
                dependencies: vec!["cross_modal_aligner".into(), "vsa_vectors".into()],
            },
            CapabilityDescriptor {
                id: "visual_rag_index".into(),
                name: "Visual RAG Index (FAISS+VSA)".into(),
                layer: ArchitectureLayer::Perception,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G397".into()],
                module_path: Some("nt_core_hcube/visual_rag_index.rs".into()),
                estimated_lines: 200,
                dependencies: vec!["vsa_vectors".into()],
            },
            // Layer 2: Cognition
            CapabilityDescriptor {
                id: "gwt_attention".into(),
                name: "GWT Global Workspace Attention".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_gwt/".into()),
                estimated_lines: 3000,
                dependencies: vec!["vsa_vectors".into()],
            },
            CapabilityDescriptor {
                id: "manar_attention".into(),
                name: "MANAR Concept Bottleneck Attention".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_gwt/manar_attention.rs".into()),
                estimated_lines: 500,
                dependencies: vec!["vsa_vectors".into()],
            },
            CapabilityDescriptor {
                id: "world_model".into(),
                name: "Hierarchical World Model".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G357".into(), "G370".into(), "G371".into()],
                module_path: Some("nt_core_consciousness/hierarchical_world_model.rs".into()),
                estimated_lines: 830,
                dependencies: vec!["vsa_vectors".into()],
            },
            CapabilityDescriptor {
                id: "counterfactual".into(),
                name: "Counterfactual Reasoning".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G358".into()],
                module_path: Some("nt_core_consciousness/counterfactual.rs".into()),
                estimated_lines: 920,
                dependencies: vec!["world_model".into()],
            },
            CapabilityDescriptor {
                id: "mcts_reasoning".into(),
                name: "MCTS Tree Search Reasoning".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G348".into()],
                module_path: Some("nt_core_reasoning/mcts_reasoner.rs".into()),
                estimated_lines: 1500,
                dependencies: vec!["world_model".into()],
            },
            CapabilityDescriptor {
                id: "causal_reasoning".into(),
                name: "Causal Reasoning (do-calculus)".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G360".into()],
                module_path: Some("nt_core_consciousness/causal_reasoning.rs".into()),
                estimated_lines: 1200,
                dependencies: vec!["world_model".into()],
            },
            CapabilityDescriptor {
                id: "consciousness_cycle".into(),
                name: "Consciousness Cycle v2 — Full Pipeline Integration".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G394".into()],
                module_path: Some("nt_core_consciousness/consciousness_cycle.rs".into()),
                estimated_lines: 750,
                dependencies: vec![
                    "gwt_attention".into(),
                    "world_model".into(),
                    "mcts_reasoning".into(),
                    "analogy_reasoning".into(),
                    "multi_modal_gate".into(),
                    "pixel_perception".into(),
                    "screenshot_pipeline".into(),
                    "visual_embedding_frontend".into(),
                    "mcts_reasoner".into(),
                    "parallel_hypothesis".into(),
                    "dead_end_detector".into(),
                    "strategy_selector".into(),
                    "bidirectional_pruner".into(),
                    "process_reward_model".into(),
                ],
            },
            CapabilityDescriptor {
                id: "spectrum_signal".into(),
                name: "Spectrum→Signal Diversity Pipeline".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G396".into()],
                module_path: Some("nt_core_consciousness/spectrum_signal.rs".into()),
                estimated_lines: 300,
                dependencies: vec!["consciousness_cycle".into()],
            },
            CapabilityDescriptor {
                id: "analogy_reasoning".into(),
                name: "Analogical Reasoning (Structure Mapping)".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec![],
                module_path: Some("nt_core_consciousness/analogical_reasoning.rs".into()),
                estimated_lines: 750,
                dependencies: vec!["vsa_vectors".into()],
            },
            CapabilityDescriptor {
                id: "multi_modal_gate".into(),
                name: "Multi-Modal GWT Attention Gating".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G407".into()],
                module_path: Some("nt_core_consciousness/multi_modal_gate.rs".into()),
                estimated_lines: 370,
                dependencies: vec!["gwt_attention".into(), "vsa_tag".into()],
            },
            CapabilityDescriptor {
                id: "mcts_gwt_bridge".into(),
                name: "MCTS↔GWT Integration Bridge".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec![],
                module_path: Some("nt_core_consciousness/mcts_gwt_bridge.rs".into()),
                estimated_lines: 220,
                dependencies: vec!["mcts_reasoning".into(), "gwt_attention".into()],
            },
            CapabilityDescriptor {
                id: "recurrent_world_model".into(),
                name: "Recurrent Latent State Refinement".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G370".into()],
                module_path: Some("nt_core_consciousness/recurrent_world_model.rs".into()),
                estimated_lines: 300,
                dependencies: vec!["world_model".into()],
            },
            CapabilityDescriptor {
                id: "causal_counterfactual_bridge".into(),
                name: "Causal↔Counterfactual Integration Bridge".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec![],
                module_path: Some("nt_core_consciousness/causal_counterfactual_bridge.rs".into()),
                estimated_lines: 300,
                dependencies: vec!["causal_reasoning".into(), "counterfactual".into()],
            },
            // Layer 3: Meta-Cognition
            CapabilityDescriptor {
                id: "parallel_hypothesis_evaluator".into(),
                name: "Parallel Hypothesis Evaluation".into(),
                layer: ArchitectureLayer::MetaCognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G347".into()],
                module_path: Some("nt_core_consciousness/parallel_hypothesis_evaluator.rs".into()),
                estimated_lines: 430,
                dependencies: vec!["vsa_vectors".into()],
            },
            CapabilityDescriptor {
                id: "dead_end_detector".into(),
                name: "Reasoning Dead-End Detection".into(),
                layer: ArchitectureLayer::MetaCognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G352".into()],
                module_path: Some("nt_core_reasoning/dead_end_detector.rs".into()),
                estimated_lines: 424,
                dependencies: vec!["vsa_vectors".into()],
            },
            CapabilityDescriptor {
                id: "confidence_calibrator".into(),
                name: "Confidence Calibration".into(),
                layer: ArchitectureLayer::MetaCognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec![],
                module_path: Some("nt_core_experience/calibration_engine.rs".into()),
                estimated_lines: 400,
                dependencies: vec![],
            },
            CapabilityDescriptor {
                id: "epistemic_honesty".into(),
                name: "Epistemic Honesty Engine".into(),
                layer: ArchitectureLayer::MetaCognition,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_consciousness/epistemic_honesty.rs".into()),
                estimated_lines: 350,
                dependencies: vec![],
            },
            CapabilityDescriptor {
                id: "metacognition_loop".into(),
                name: "Metacognition Loop (SCAN→ANALYZE→PLAN→ACT)".into(),
                layer: ArchitectureLayer::MetaCognition,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_meta/metacognition_loop.rs".into()),
                estimated_lines: 800,
                dependencies: vec![],
            },
            CapabilityDescriptor {
                id: "parallel_hypothesis".into(),
                name: "Parallel Hypothesis Evaluation".into(),
                layer: ArchitectureLayer::MetaCognition,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_reasoning/parallel_hypothesis.rs".into()),
                estimated_lines: 600,
                dependencies: vec!["mcts_reasoning".into()],
            },
            CapabilityDescriptor {
                id: "epistemic_humility".into(),
                name: "Epistemic Humility (Calibrated Don't-Know)".into(),
                layer: ArchitectureLayer::MetaCognition,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_consciousness/epistemic_humility.rs".into()),
                estimated_lines: 700,
                dependencies: vec!["epistemic_honesty".into()],
            },
            CapabilityDescriptor {
                id: "process_reward_model".into(),
                name: "Hierarchical Process Reward Model".into(),
                layer: ArchitectureLayer::MetaCognition,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_reasoning/process_reward_model.rs".into()),
                estimated_lines: 700,
                dependencies: vec!["vsa_vectors".into()],
            },
            CapabilityDescriptor {
                id: "bidirectional_pruner".into(),
                name: "Bidirectional Reasoning Path Pruner".into(),
                layer: ArchitectureLayer::MetaCognition,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_reasoning/bidirectional_pruner.rs".into()),
                estimated_lines: 500,
                dependencies: vec!["mcts_reasoning".into(), "dead_end_detector".into()],
            },
            CapabilityDescriptor {
                id: "strategy_selector".into(),
                name: "Self-Healing Reasoning Strategy Selector".into(),
                layer: ArchitectureLayer::MetaCognition,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_reasoning/strategy_selector.rs".into()),
                estimated_lines: 500,
                dependencies: vec!["dead_end_detector".into()],
            },
            CapabilityDescriptor {
                id: "counterfactual_simulator".into(),
                name: "Counterfactual What-If Simulator".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_reasoning/counterfactual_simulator.rs".into()),
                estimated_lines: 600,
                dependencies: vec!["vsa_vectors".into()],
            },
            CapabilityDescriptor {
                id: "gwt_self_interrupt".into(),
                name: "GWT Self-Interruption (Metacognitive Interrupt)".into(),
                layer: ArchitectureLayer::MetaCognition,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_gwt/self_interrupt.rs".into()),
                estimated_lines: 500,
                dependencies: vec!["gwt_attention".into(), "dead_end_detector".into()],
            },
            CapabilityDescriptor {
                id: "curiosity_exploration".into(),
                name: "Curiosity-Driven Exploration".into(),
                layer: ArchitectureLayer::MetaCognition,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_gwt/curiosity_exploration.rs".into()),
                estimated_lines: 600,
                dependencies: vec!["gwt_attention".into(), "intrinsic_drive".into()],
            },
            CapabilityDescriptor {
                id: "resource_allocator".into(),
                name: "Conscious Resource Allocator — Dynamic Cognitive Budget".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G416".into()],
                module_path: Some("nt_core_consciousness/resource_allocator.rs".into()),
                estimated_lines: 350,
                dependencies: vec!["consciousness_cycle".into()],
            },
            CapabilityDescriptor {
                id: "episodic_buffer".into(),
                name: "Episodic Consciousness Buffer — Short-Term State Memory".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G417".into()],
                module_path: Some("nt_core_consciousness/episodic_buffer.rs".into()),
                estimated_lines: 320,
                dependencies: vec!["consciousness_cycle".into()],
            },
            CapabilityDescriptor {
                id: "cognitive_blackboard".into(),
                name: "Cognitive Blackboard — Cross-Engine Shared Workspace".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G411".into()],
                module_path: Some("nt_core_consciousness/cognitive_blackboard.rs".into()),
                estimated_lines: 340,
                dependencies: vec!["consciousness_cycle".into()],
            },
            CapabilityDescriptor {
                id: "consciousness_refinery".into(),
                name: "Consciousness Refinery Inner Loop".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G413".into()],
                module_path: Some("nt_core_consciousness/consciousness_refinery.rs".into()),
                estimated_lines: 320,
                dependencies: vec!["consciousness_cycle".into()],
            },
            CapabilityDescriptor {
                id: "dual_path_inference".into(),
                name: "Dual-Path Inference (Constraint+Generative)".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G382".into()],
                module_path: Some("nt_core_consciousness/dual_path_inference.rs".into()),
                estimated_lines: 280,
                dependencies: vec!["consciousness_cycle".into()],
            },
            CapabilityDescriptor {
                id: "executable_belief".into(),
                name: "Executable Belief Verification (Inspector×CLR×Stability)".into(),
                layer: ArchitectureLayer::MetaCognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G390".into()],
                module_path: Some("nt_core_consciousness/executable_belief.rs".into()),
                estimated_lines: 300,
                dependencies: vec!["consciousness_cycle".into()],
            },
            CapabilityDescriptor {
                id: "analogy_reasoning".into(),
                name: "Analogical Reasoning".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Missing,
                gap_ids: vec!["G361".into()],
                module_path: None,
                estimated_lines: 800,
                dependencies: vec![],
            },
            // Layer 4: Self-Evolution
            CapabilityDescriptor {
                id: "seal_loop".into(),
                name: "SEAL Closed Evolution Loop".into(),
                layer: ArchitectureLayer::SelfEvolution,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_experience/seal_closed_loop.rs".into()),
                estimated_lines: 600,
                dependencies: vec!["metacognition_loop".into()],
            },
            CapabilityDescriptor {
                id: "experience_tree".into(),
                name: "Experience Tree Distillation".into(),
                layer: ArchitectureLayer::SelfEvolution,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("skills/neotrix-experience/SKILL.md".into()),
                estimated_lines: 0,
                dependencies: vec!["seal_loop".into()],
            },
            CapabilityDescriptor {
                id: "synthetic_data_factory".into(),
                name: "Synthetic Data Factory".into(),
                layer: ArchitectureLayer::SelfEvolution,
                status: CapabilityStatus::Missing,
                gap_ids: vec!["G402".into()],
                module_path: None,
                estimated_lines: 1200,
                dependencies: vec!["seal_loop".into()],
            },
            // Layer 5: Meta-Architecture
            CapabilityDescriptor {
                id: "meta_evolution_loop".into(),
                name: "Meta-Architecture Evolution Loop".into(),
                layer: ArchitectureLayer::MetaArchitecture,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G412".into()],
                module_path: Some("nt_core_consciousness/meta_evolution_loop.rs".into()),
                estimated_lines: 360,
                dependencies: vec!["architecture_self_model".into()],
            },
            CapabilityDescriptor {
                id: "self_model_code".into(),
                name: "Code-Level Self Model".into(),
                layer: ArchitectureLayer::MetaArchitecture,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_meta/self_model.rs".into()),
                estimated_lines: 670,
                dependencies: vec![],
            },
            CapabilityDescriptor {
                id: "architecture_self_model".into(),
                name: "Capability-Level Architecture Self-Model".into(),
                layer: ArchitectureLayer::MetaArchitecture,
                status: CapabilityStatus::Partial,
                gap_ids: vec![],
                module_path: Some("nt_core_consciousness/consciousness_architecture.rs".into()),
                estimated_lines: 300,
                dependencies: vec!["self_model_code".into()],
            },
            CapabilityDescriptor {
                id: "architecture_designer".into(),
                name: "Architecture Designer Agent".into(),
                layer: ArchitectureLayer::MetaArchitecture,
                status: CapabilityStatus::Complete,
                gap_ids: vec![],
                module_path: Some("nt_core_arch/designer.rs".into()),
                estimated_lines: 400,
                dependencies: vec!["self_model_code".into()],
            },
            CapabilityDescriptor {
                id: "roadmap_generator".into(),
                name: "Roadmap Generation + Gap Analysis".into(),
                layer: ArchitectureLayer::MetaArchitecture,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G347".into(), "G348".into()],
                module_path: Some("EVOLUTION_ROADMAP_v14.md".into()),
                estimated_lines: 0,
                dependencies: vec!["architecture_self_model".into()],
            },
            CapabilityDescriptor {
                id: "consciousness_pipeline".into(),
                name: "Consciousness Pipeline — Unified Integration of All 10 Architecture Modules"
                    .into(),
                layer: ArchitectureLayer::MetaArchitecture,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G418".into()],
                module_path: Some("nt_core_consciousness/consciousness_pipeline.rs".into()),
                estimated_lines: 350,
                dependencies: vec![
                    "consciousness_cycle".into(),
                    "consciousness_refinery".into(),
                    "dual_path_inference".into(),
                    "executable_belief".into(),
                    "cognitive_blackboard".into(),
                    "resource_allocator".into(),
                    "episodic_buffer".into(),
                    "meta_evolution_loop".into(),
                    "spectrum_signal".into(),
                ],
            },
            CapabilityDescriptor {
                id: "performance_oracle".into(),
                name: "Performance Oracle — Self-Learning Pipeline Optimizer with Health Dashboard"
                    .into(),
                layer: ArchitectureLayer::MetaArchitecture,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G419".into()],
                module_path: Some("nt_core_consciousness/performance_oracle.rs".into()),
                estimated_lines: 350,
                dependencies: vec!["consciousness_pipeline".into(), "resource_allocator".into()],
            },
            CapabilityDescriptor {
                id: "adaptive_controller".into(),
                name: "Adaptive Controller — closes Oracle→Pipeline automatic feedback loop".into(),
                layer: ArchitectureLayer::MetaArchitecture,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G420".into()],
                module_path: Some("nt_core_consciousness/adaptive_controller.rs".into()),
                estimated_lines: 250,
                dependencies: vec![
                    "consciousness_pipeline".into(),
                    "performance_oracle".into(),
                    "resource_allocator".into(),
                ],
            },
            // ═══════════════════════════════════════════════
            // Layer 6: Economic Agency (G500-G509 series)
            // ═══════════════════════════════════════════════
            CapabilityDescriptor {
                id: "key_vault".into(),
                name: "Encrypted Credential & Key Vault".into(),
                layer: ArchitectureLayer::Substrate,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G500".into()],
                module_path: Some("nt_core_economic/key_vault.rs".into()),
                estimated_lines: 150,
                dependencies: vec![],
            },
            CapabilityDescriptor {
                id: "data_feed".into(),
                name: "Unified Market Data Feed Abstraction".into(),
                layer: ArchitectureLayer::Perception,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G501".into()],
                module_path: Some("nt_core_economic/data_feed.rs".into()),
                estimated_lines: 200,
                dependencies: vec!["key_vault".into()],
            },
            CapabilityDescriptor {
                id: "economic_agent".into(),
                name: "Economic Agency — Autonomous Income Strategy Execution".into(),
                layer: ArchitectureLayer::Cognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G502".into()],
                module_path: Some("nt_core_economic/economic_agent.rs".into()),
                estimated_lines: 350,
                dependencies: vec!["key_vault".into(), "data_feed".into()],
            },
            CapabilityDescriptor {
                id: "risk_manager".into(),
                name: "Risk Management — VaR, Position Sizing, Kill-Switch".into(),
                layer: ArchitectureLayer::MetaCognition,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G503".into()],
                module_path: Some("nt_core_economic/risk_metrics.rs".into()),
                estimated_lines: 200,
                dependencies: vec![],
            },
            CapabilityDescriptor {
                id: "economic_world_model".into(),
                name: "Economic World Model — Macro Variables & Regime Prediction".into(),
                layer: ArchitectureLayer::SelfEvolution,
                status: CapabilityStatus::Partial,
                gap_ids: vec!["G504".into()],
                module_path: Some("nt_core_economic/economic_world_model.rs".into()),
                estimated_lines: 180,
                dependencies: vec!["data_feed".into()],
            },
        ];

        for cap in builtins {
            self.capabilities.insert(cap.id.clone(), cap);
        }
    }

    fn register_roadmap_gaps(&mut self) {
        let gaps = vec![
            // Survival gaps
            GapInfo {
                id: "G348".into(),
                description: "MCTS Tree Search Reasoning".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::Cognition,
                target_capability: "mcts_reasoning".into(),
            },
            GapInfo {
                id: "G351".into(),
                description: "Confidence Calibration".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::MetaCognition,
                target_capability: "confidence_calibrator".into(),
            },
            GapInfo {
                id: "G357".into(),
                description: "World Model Consequences Prediction".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::Cognition,
                target_capability: "world_model".into(),
            },
            GapInfo {
                id: "G358".into(),
                description: "Counterfactual Simulation".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::Cognition,
                target_capability: "counterfactual".into(),
            },
            GapInfo {
                id: "G360".into(),
                description: "Causal Reasoning (do-calculus)".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::Cognition,
                target_capability: "causal_reasoning".into(),
            },
            GapInfo {
                id: "G394".into(),
                description: "Pixel-Native Rendering Pipeline".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::Perception,
                target_capability: "pixel_perception".into(),
            },
            GapInfo {
                id: "G395".into(),
                description: "Visual Embedding Model Integration".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::Perception,
                target_capability: "visual_embedding_frontend".into(),
            },
            GapInfo {
                id: "G407".into(),
                description: "Multi-Modal GWT Attention Gating".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::Cognition,
                target_capability: "multi_modal_gate".into(),
            },
            GapInfo {
                id: "G406".into(),
                description: "Visual Perception → VSA Binding".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::Perception,
                target_capability: "pixel_perception".into(),
            },
            GapInfo {
                id: "G394".into(),
                description: "Consciousness Cycle 12-Step Orchestrator".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::Cognition,
                target_capability: "consciousness_cycle".into(),
            },
            GapInfo {
                id: "G395".into(),
                description: "VerificationGate Wired Into Consciousness Cycle".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::Cognition,
                target_capability: "consciousness_cycle".into(),
            },
            GapInfo {
                id: "G396".into(),
                description: "Spectrum→Signal Diversity Pipeline".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::Cognition,
                target_capability: "spectrum_signal".into(),
            },
            GapInfo {
                id: "G413".into(),
                description: "Consciousness Refinery Inner Loop — LoopWM-style iterative refinement".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::Cognition,
                target_capability: "consciousness_refinery".into(),
            },
            GapInfo {
                id: "G382".into(),
                description: "Dual-Path Inference — constraint+generative dual model architecture".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::Cognition,
                target_capability: "dual_path_inference".into(),
            },
            GapInfo {
                id: "G390".into(),
                description: "Executable Belief Verification — Inspector×CLR×Stability".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::MetaCognition,
                target_capability: "executable_belief".into(),
            },
            GapInfo {
                id: "G370".into(),
                description: "Recurrent Latent State Refinement".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::Cognition,
                target_capability: "world_model".into(),
            },
            // Evolution gaps
            GapInfo {
                id: "G347".into(),
                description: "Parallel Hypothesis Evaluation".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::MetaCognition,
                target_capability: "parallel_hypothesis_evaluator".into(),
            },
            GapInfo {
                id: "G352".into(),
                description: "Reasoning Dead-End Detection".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::MetaCognition,
                target_capability: "dead_end_detector".into(),
            },
            GapInfo {
                id: "G361".into(),
                description: "Analogical Reasoning (Structure Mapping)".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::Cognition,
                target_capability: "analogy_reasoning".into(),
            },
            GapInfo {
                id: "G396".into(),
                description: "Layout-Aware Visual Chunking".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::Perception,
                target_capability: "pixel_perception".into(),
            },
            GapInfo {
                id: "G397".into(),
                description: "Multi-Modal Index Architecture".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::Perception,
                target_capability: "visual_rag_index".into(),
            },
            GapInfo {
                id: "G398".into(),
                description: "Multi-Backend Inference Abstraction".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::Perception,
                target_capability: "pixel_perception".into(),
            },
            GapInfo {
                id: "G402".into(),
                description: "Synthetic Data Factory".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::SelfEvolution,
                target_capability: "synthetic_data_factory".into(),
            },
            GapInfo {
                id: "G416".into(),
                description: "Conscious Resource Allocator — dynamic cognitive budget based on internal state".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::Cognition,
                target_capability: "resource_allocator".into(),
            },
            GapInfo {
                id: "G417".into(),
                description: "Episodic Consciousness Buffer — short-term high-resolution state ring buffer".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::Cognition,
                target_capability: "episodic_buffer".into(),
            },
            GapInfo {
                id: "G400".into(),
                description: "GradCache Training".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::SelfEvolution,
                target_capability: "synthetic_data_factory".into(),
            },
            GapInfo {
                id: "G414".into(),
                description: "Cognitive Blackboard — cross-engine shared workspace for claims and contradictions".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::Cognition,
                target_capability: "cognitive_blackboard".into(),
            },
            GapInfo {
                id: "G415".into(),
                description: "Meta-Architecture Evolution Loop — autonomous assess→recommend→track→measure".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::MetaArchitecture,
                target_capability: "meta_evolution_loop".into(),
            },
            // Enhancement gaps
            GapInfo {
                id: "G401".into(),
                description: "LoRA Adapter Management".into(),
                severity: GapSeverity::Enhancement,
                layer: ArchitectureLayer::Perception,
                target_capability: "pixel_perception".into(),
            },
            GapInfo {
                id: "G403".into(),
                description: "Pixel-Level Evidence Attribution".into(),
                severity: GapSeverity::Enhancement,
                layer: ArchitectureLayer::Perception,
                target_capability: "pixel_perception".into(),
            },
            GapInfo {
                id: "G404".into(),
                description: "Pre-built Knowledge Indexes".into(),
                severity: GapSeverity::Enhancement,
                layer: ArchitectureLayer::Perception,
                target_capability: "visual_rag_index".into(),
            },
            GapInfo {
                id: "G409".into(),
                description: "Visual RAG Benchmark".into(),
                severity: GapSeverity::Enhancement,
                layer: ArchitectureLayer::Perception,
                target_capability: "visual_rag_index".into(),
            },
            // ═══════════════════════════════════════
            // Economic Agency gaps (G500 series)
            // ═══════════════════════════════════════
            GapInfo {
                id: "G500".into(),
                description: "Encrypted Credential & Key Vault — store API keys/wallet seeds".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::Substrate,
                target_capability: "key_vault".into(),
            },
            GapInfo {
                id: "G501".into(),
                description: "Unified Market Data Feed — exchange, news, ad API abstraction".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::Perception,
                target_capability: "data_feed".into(),
            },
            GapInfo {
                id: "G502".into(),
                description: "Economic Agency — autonomous opportunity analysis, strategy selection, execution, P&L tracking".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::Cognition,
                target_capability: "economic_agent".into(),
            },
            GapInfo {
                id: "G503".into(),
                description: "Risk Management — position sizing, VaR 95%, kill-switch, drawdown monitor".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::MetaCognition,
                target_capability: "risk_manager".into(),
            },
            GapInfo {
                id: "G504".into(),
                description: "Economic World Model — GDP/inflation/rate/sentiment variables, regime prediction".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::SelfEvolution,
                target_capability: "economic_world_model".into(),
            },
            GapInfo {
                id: "G505".into(),
                description: "Multi-Exchange Trading Connector — real exchange API integration".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::Perception,
                target_capability: "data_feed".into(),
            },
            GapInfo {
                id: "G506".into(),
                description: "Content Monetization Pipeline — blog/video/social media auto-publishing".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::Cognition,
                target_capability: "economic_agent".into(),
            },
            GapInfo {
                id: "G507".into(),
                description: "Ad Network Optimization Engine — Google/Meta/TikTok ad platform API".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::Cognition,
                target_capability: "economic_agent".into(),
            },
            GapInfo {
                id: "G508".into(),
                description: "SaaS API Monetization — consciousness capabilities as paid API service".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::MetaArchitecture,
                target_capability: "economic_agent".into(),
            },
            GapInfo {
                id: "G509".into(),
                description: "Autonomous Arbitrage Detection — cross-exchange, cross-platform arbitrage".into(),
                severity: GapSeverity::Enhancement,
                layer: ArchitectureLayer::Cognition,
                target_capability: "economic_agent".into(),
            },
            GapInfo {
                id: "G418".into(),
                description: "Consciousness Pipeline — unified integration of 10 architecture modules into a single run() call".into(),
                severity: GapSeverity::Survival,
                layer: ArchitectureLayer::MetaArchitecture,
                target_capability: "consciousness_pipeline".into(),
            },
            GapInfo {
                id: "G419".into(),
                description: "Performance Oracle — self-learning pipeline optimizer with sliding window metrics, bottleneck detection, adaptive config, and health dashboard".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::MetaArchitecture,
                target_capability: "performance_oracle".into(),
            },
            GapInfo {
                id: "G420".into(),
                description: "Adaptive Controller — closes the Oracle→Pipeline automatic feedback loop for self-adaptive pipeline operation".into(),
                severity: GapSeverity::Evolution,
                layer: ArchitectureLayer::MetaArchitecture,
                target_capability: "adaptive_controller".into(),
            },
        ];

        for gap in gaps {
            self.known_gaps.insert(gap.id.clone(), gap);
        }
    }

    pub fn get_capability(&self, id: &str) -> Option<&CapabilityDescriptor> {
        self.capabilities.get(id)
    }

    pub fn capabilities_by_layer(&self, layer: ArchitectureLayer) -> Vec<&CapabilityDescriptor> {
        self.capabilities
            .values()
            .filter(|c| c.layer == layer)
            .collect()
    }

    pub fn gaps_by_layer(&self, layer: ArchitectureLayer) -> Vec<&GapInfo> {
        self.known_gaps
            .values()
            .filter(|g| g.layer == layer)
            .collect()
    }

    pub fn assess_health(&self) -> ArchitectureHealth {
        let mut layer_scores = HashMap::new();

        for layer in ArchitectureLayer::all() {
            let caps = self.capabilities_by_layer(*layer);
            let total = caps.len();
            if total == 0 {
                layer_scores.insert(*layer, 0.0);
                continue;
            }
            let sum: f64 = caps.iter().map(|c| c.status.score()).sum();
            layer_scores.insert(*layer, sum / total as f64);
        }

        let overall: f64 = layer_scores.values().sum::<f64>() / layer_scores.len() as f64;
        let total_caps = self.capabilities.len();
        let completed = self
            .capabilities
            .values()
            .filter(|c| c.status.score() >= 0.66)
            .count();
        let missing: Vec<String> = self
            .capabilities
            .values()
            .filter(|c| c.status == CapabilityStatus::Missing)
            .map(|c| c.id.clone())
            .collect();

        let critical: Vec<String> = self
            .known_gaps
            .values()
            .filter(|g| g.severity == GapSeverity::Survival)
            .map(|g| g.id.clone())
            .collect();

        ArchitectureHealth {
            layer_scores,
            overall_health: overall,
            total_capabilities: total_caps,
            completed_capabilities: completed,
            missing_capabilities: missing,
            critical_gaps: critical,
        }
    }

    pub fn generate_evolution_plan(&self) -> EvolutionPlan {
        let health = self.assess_health();
        let mut steps = Vec::new();

        for layer in ArchitectureLayer::all() {
            let missing_caps = self
                .capabilities_by_layer(*layer)
                .into_iter()
                .filter(|c| c.status == CapabilityStatus::Missing)
                .collect::<Vec<_>>();

            for cap in &missing_caps {
                let layer_gaps: Vec<&GapInfo> = self
                    .known_gaps
                    .values()
                    .filter(|g| g.target_capability == cap.id)
                    .collect();

                let max_severity = layer_gaps
                    .iter()
                    .map(|g| g.severity.priority())
                    .min()
                    .unwrap_or(2);

                let gap_ids: Vec<String> = layer_gaps.iter().map(|g| g.id.clone()).collect();

                steps.push(EvolutionStep {
                    layer: *layer,
                    capability_id: cap.id.clone(),
                    capability_name: cap.name.clone(),
                    gap_ids,
                    severity_priority: max_severity,
                    estimated_lines: cap.estimated_lines,
                    has_module: cap.module_path.is_some(),
                    layer_health: *health.layer_scores.get(layer).unwrap_or(&0.0),
                });
            }
        }

        steps.sort_by(|a, b| {
            a.severity_priority
                .cmp(&b.severity_priority)
                .then_with(|| a.layer.priority().cmp(&b.layer.priority()))
                .then_with(|| b.estimated_lines.cmp(&a.estimated_lines))
        });

        EvolutionPlan {
            steps,
            health,
            generated_at: crate::core::nt_core_time::unix_now_ms(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvolutionStep {
    pub layer: ArchitectureLayer,
    pub capability_id: String,
    pub capability_name: String,
    pub gap_ids: Vec<String>,
    pub severity_priority: u8,
    pub estimated_lines: usize,
    pub has_module: bool,
    pub layer_health: f64,
}

#[derive(Debug, Clone)]
pub struct EvolutionPlan {
    pub steps: Vec<EvolutionStep>,
    pub health: ArchitectureHealth,
    pub generated_at: u64,
}

impl EvolutionPlan {
    pub fn total_estimated_lines(&self) -> usize {
        self.steps.iter().map(|s| s.estimated_lines).sum()
    }

    pub fn survival_steps(&self) -> Vec<&EvolutionStep> {
        self.steps
            .iter()
            .filter(|s| s.severity_priority == 0)
            .collect()
    }

    pub fn steps_by_layer(&self, layer: ArchitectureLayer) -> Vec<&EvolutionStep> {
        self.steps.iter().filter(|s| s.layer == layer).collect()
    }

    pub fn summary(&self) -> String {
        let surv = self.survival_steps().len();
        let total = self.steps.len();
        let lines = self.total_estimated_lines();
        format!(
            "进化计划: {} 步骤 ({} 生存级), ~{} 行代码. 架构健康: {:.1}%",
            total,
            surv,
            lines,
            self.health.overall_health * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architecture_layer_all() {
        let layers = ArchitectureLayer::all();
        assert_eq!(layers.len(), 6);
    }

    #[test]
    fn test_capability_registration() {
        let arch = ConsciousnessArchitecture::new();
        let cap = arch.get_capability("vsa_vectors");
        assert!(cap.is_some());
        assert_eq!(cap.unwrap().layer, ArchitectureLayer::Substrate);
    }

    #[test]
    fn test_capabilities_by_layer() {
        let arch = ConsciousnessArchitecture::new();
        let substrate = arch.capabilities_by_layer(ArchitectureLayer::Substrate);
        assert!(!substrate.is_empty());
        let meta = arch.capabilities_by_layer(ArchitectureLayer::MetaArchitecture);
        assert!(!meta.is_empty());
    }

    #[test]
    fn test_gaps_by_layer() {
        let arch = ConsciousnessArchitecture::new();
        let cog_gaps = arch.gaps_by_layer(ArchitectureLayer::Cognition);
        assert!(!cog_gaps.is_empty());
    }

    #[test]
    fn test_assess_health() {
        let arch = ConsciousnessArchitecture::new();
        let health = arch.assess_health();
        assert!(health.overall_health > 0.0);
        assert!(health.overall_health < 1.0);
        assert!(health.total_capabilities > 0);
    }

    #[test]
    fn test_evolution_plan_generation() {
        let arch = ConsciousnessArchitecture::new();
        let plan = arch.generate_evolution_plan();
        assert!(!plan.steps.is_empty());
        assert!(plan.total_estimated_lines() > 0);
    }

    #[test]
    fn test_evolution_plan_survival_steps() {
        let arch = ConsciousnessArchitecture::new();
        let plan = arch.generate_evolution_plan();
        let survival = plan.survival_steps();
        assert!(!survival.is_empty());
    }

    #[test]
    fn test_evolution_plan_summary() {
        let arch = ConsciousnessArchitecture::new();
        let plan = arch.generate_evolution_plan();
        let summary = plan.summary();
        assert!(summary.contains("进化计划"));
        assert!(summary.contains("生存级"));
    }

    #[test]
    fn test_layer_priority_order() {
        assert!(ArchitectureLayer::Substrate.priority() < ArchitectureLayer::Perception.priority());
        assert!(ArchitectureLayer::Perception.priority() < ArchitectureLayer::Cognition.priority());
        assert!(
            ArchitectureLayer::MetaArchitecture.priority()
                > ArchitectureLayer::SelfEvolution.priority()
        );
    }

    #[test]
    fn test_capability_status_scores() {
        assert!((CapabilityStatus::Missing.score() - 0.0).abs() < 0.01);
        assert!((CapabilityStatus::Partial.score() - 0.33).abs() < 0.01);
        assert!((CapabilityStatus::Complete.score() - 0.66).abs() < 0.01);
        assert!((CapabilityStatus::Maturing.score() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_mcts_reasoning_partial() {
        let arch = ConsciousnessArchitecture::new();
        let cap = arch.get_capability("mcts_reasoning");
        assert!(cap.is_some());
        assert_eq!(cap.unwrap().status, CapabilityStatus::Partial);
    }

    #[test]
    fn test_evolution_plan_contains_missing_or_partial() {
        let arch = ConsciousnessArchitecture::new();
        let plan = arch.generate_evolution_plan();
        for step in &plan.steps {
            let cap = arch.get_capability(&step.capability_id);
            assert!(cap.is_some());
            assert!(cap.unwrap().status == CapabilityStatus::Missing);
        }
    }
}
