pub mod always_on_daemon;
pub mod cross_binding_loop;
pub mod formal_introspect;
pub mod knowledge_gap_detector;
pub mod meta_learning;
pub mod metacognition_loop;
pub mod metacognitive_state;
pub mod monitor;
pub mod planner;
pub mod scanner;
pub mod self_model;
pub mod self_model_level;
pub mod timer;
pub mod weakness;
// consciousness_bench — was unstable module, removed

pub use knowledge_gap_detector::{
    GapCategory, GapCluster, GapReport, KnowledgeGap, KnowledgeGapDetector,
};
pub use metacognition_loop::{MetaCognitiveLoop, MetaCycleResult};
pub use monitor::{AlertSeverity, HealthCheck, HealthTrend, MetaAlert, MetaMonitor};
pub use planner::{
    weakness_to_goals, ActionStatus, EvolutionAction, EvolutionPlanner, ImpactEstimate, MetaGoal,
    MetaGoalBridge, PlannedEvolution, RiskLevel,
};
pub use scanner::CodeScanner;
pub use self_model::{
    ArchitectureGraph, ArchitectureNode, CompilationHealth, ComponentMap, ComponentNode,
    DebtSeverity, DepEdge, DepGraph, DepKind, EventKind, EvolutionEvent, FileInfo, ModuleInfo,
    ModuleStatus, SelfModel, TechDebtInventory, TechDebtItem, TechDebtKind, TestCoverage,
};
pub use self_model_level::{SelfModelAssessor, SelfModelLevel, SelfModelReport};
pub use timer::{TimerEntry, TimerRegistry, TimerStats};
pub use weakness::{Weakness, WeaknessAnalyzer, WeaknessReport, WeaknessSummary};
pub mod memory_evolution;
pub use memory_evolution::{
    global_memory_evolution, AnswerPolicy, FailureType, FusionStrategy, MemoryEvolutionEngine,
    RetrievalConfig, RetrievalFailure, ScoringFunction,
};
pub mod audit;
pub use audit::{AuditEngine, AuditFinding, AuditPhase, AuditReport, AuditStatus, FindingSeverity};
pub mod mirror_bench;
pub use mirror_bench::{
    metacognitive_scenario_default, metacognitive_scenario_overconfident,
    metacognitive_scenario_scaffolded, metacognitive_scenario_underconfident, MirrorBenchmark,
    MirrorEpisode, MirrorReport,
};
pub mod fusion_gap;
pub use fusion_gap::{FusionGapEntry, FusionGapRegistry};
pub mod kpi_persistence;
pub use kpi_persistence::{KpiRecord, KpiRingBuffer};
pub mod error_bounds;
pub use error_bounds::{BoundSource, ErrorBound, PredictionErrorTracker, VsaErrorModel};
pub mod embodiment_curriculum;
pub mod harness;
pub mod mcp_callback_bridge;
pub mod mission_hub;
pub mod mod_sandbox;
pub mod skill_evolution_modes;
pub mod uncertainty_tracker;
pub use uncertainty_tracker::{
    CalibratedConfidence, ConfidenceCalibrator, DecisionTracker, EvidencePiece, PredictionOutcome,
    UncertaintyDetector, UncertaintyReport, UncertaintyType,
};
pub mod a2a_router;
pub mod inner_monologue;
pub use inner_monologue::{
    DialoguePhase, DialogueResult, DialogueSummary, InnerMonologueSystem, InternalVoice,
    SynthesisEngine, SynthesisOutput, Utterance, VoiceProfile, VoiceProfiles,
};
pub mod meta_kpi_repo;
pub use meta_kpi_repo::{
    GoalStatus, MetaGapReport, MetaKPIRepository, MetaKPISnapshot, SelfDirectedGoal,
};
pub use metacognitive_state::MetacognitiveState;
pub mod skill_registry;
pub use meta_learning::{
    ConsolidationOutcome, KleosAdjustments, MetaLearning, MetaLearningParams, MetaLearningSignal,
};
pub use skill_registry::SkillRegistry;
pub mod meta_reflection_engine;
pub use meta_reflection_engine::{
    CycleOutcome, CycleReflectionData, Intervention, MetaCognitionController, MetaHealth,
    MetaReflectionEngine, ReasoningPathData, RecurringPattern, ReflectionDimension,
    ReflectionResult, ReflectionSeverity, ReflectionTrace,
};
