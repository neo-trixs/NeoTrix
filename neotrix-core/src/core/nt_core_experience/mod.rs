//! # Phase 5 — Self-Evolving Experience Loop
//!
//! 闭合 Loop Engineering 的最后闭环:
//! Context OS → Decision Compression → **Experience Reflection → Skill Accumulation
//! → Curriculum Generation → Policy Repair → Epistemic Self-Knowledge**
//!
//! 五种子系统各有独立模块，共享 VSA 向量表征。

pub mod calibration_engine;
pub mod capability_synthesizer;
pub mod consolidation_bridge;
pub mod cues;
pub mod curriculum;
pub mod dream;
pub mod evosc;
pub mod epistemic;
pub mod failure_taxonomy;
pub mod failure_trace;
pub mod gate;
pub mod hyperagent;
pub mod mirror_threads;
pub mod open_skill;
pub mod policy_repair;
pub mod reflector;
pub mod skill_acc;

pub use calibration_engine::{CalibrationEngine, CalibrationStats};
pub use cues::{CueConfig, CueEngine, CueStats, CueTask};
pub use curriculum::{CurriculumGenerator, DifficultyLevel, GeneratorConfig, TaskTemplate};
pub use failure_taxonomy::{ClassifierConfig, FailureModeClassifier, FailureModeStats, FailureModeType};
pub use dream::{
    ConsolidationPriority, ConsolidationResult, DreamConsolidator, DreamEntry, DreamReport,
};
pub use epistemic::{
    ConceptNode, DomainConfidence, EpistemicConfig, EpistemicSelfModel, EpistemicState,
};
pub use consolidation_bridge::{ConsolidationBridgeV2, BridgeConfig, BridgeV2Stats, ConsolidatedMemory};
pub use evosc::{ContrastiveInsight, ContrastiveReflector, ContrastiveStats, ConsolidationStats, EvoSC, EvoSCStats, SelfConsolidation};
pub use gate::{AttentionGate, GatedItem, GateStats, UtilitySignal};
pub use hyperagent::{
    EditRecord, MetaAgentConfig, MetaAgentEngine, MetaAgentReport, MetaEdit, MetaTarget,
};
pub use failure_trace::{
    ExplorationGraph, FailureTrace, TraceNode, TraceNodeType, VsaFailureCluster,
};
pub use policy_repair::{
    FailurePattern, FailureType, PolicyRepairEngine, RepairMode, RepairPolicy,
};
pub use reflector::{
    ExperienceReflector, Heuristic, HeuristicCategory, HeuristicFilter, ReflectorConfig,
};
pub use mirror_threads::{CognitiveThread, ThreadManager, ThreadManagerStats, ThreadType};
pub use open_skill::{KnowledgeAnchor, OpenSkillEngine, OpenSkillStats, SelfBuiltVerifier, SkillBlueprint, VerifierStats, VirtualTask};
pub mod hypothesis_tree;
pub use hypothesis_tree::{HypothesisNode, HypothesisStatus, HypothesisTree, HypothesisTreeConfig, HypothesisTreeStats};
pub mod loss_function;
pub use loss_function::{CompositeLoss, LossFunction, LossSample, LossStats};
pub mod adversarial;
pub use adversarial::{AdversarialArena, ArenaConfig, AgentGenotype, GenerationResult, MatchResult};
pub mod skill_dag;
pub use skill_dag::{DagStats, SkillDagArchive, SkillNode};
pub use skill_acc::{SkillAccumulator, SkillComposition, SkillEvaluator, SkillFilter, SkillMemory, SkillRefinement, SkillTrace, VSASkill};
pub mod workstream_exporter;
pub use workstream_exporter::*;
pub use capability_synthesizer::{CapabilitySynthesizer, Capability, CapabilityStats, CapabilityType, SynthesisOutcome};
pub mod health_patrol;
pub use health_patrol::{
    GlobalHealthPatrol, PatrolReport, PatrolNode, DegradationLevel, IntegrityCheck,
    IntegritySeverity, AnomalyRecord, AnomalySeverity, HealingOutcome,
};
pub mod safety_gate;
pub use safety_gate::{SafetyGate, SafetyReport, CheckResult};
