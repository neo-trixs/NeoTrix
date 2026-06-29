pub mod absorption_quality_tracker;
pub mod adapt_orch;
pub mod archive_manager;
pub mod adversarial;
pub mod agent0_dual_loop;
pub mod agent_supervisor;
pub mod agent_team;
// pub mod anti_spiral; — DEPRECATED: superseded by anti_spiral_monitor.rs (857 lines dead)
pub mod auto_experiment_loop;
pub mod anti_spiral_monitor;
pub mod ast_mutation_engine;
pub mod background_evolution_scheduler;
pub mod self_source_reader;
pub mod gepa_asi_evaluator;
pub mod auto_commit_gate;
pub mod auto_review_classifier;
pub mod auto_deploy;
pub mod auto_mlir_generator;
pub mod auto_research;
pub mod business_diagnosis;
pub mod calibration_engine;
pub mod code_mutation_engine;
pub mod capability_router;
pub mod capability_synthesizer;
pub mod capacity_monitor;
pub mod co_evolution;
pub mod consciousness_hooks;
pub mod consolidation_bridge;
pub mod constraint_registry;
pub mod containment;
pub mod context_compression;
pub mod context_compressor;
pub mod context_manager;
pub mod retrieval_evolution_engine;
pub mod context_memory;
pub mod continuous_learning;
pub mod contrastive_reflection;
pub mod cross_repo_mapper;
pub mod cross_domain;
pub mod cog_dashboard;
pub mod competition_orch;
pub mod compound_knowledge;
pub mod cues;
pub mod curiosity;
pub mod curriculum;
pub mod cyber_threat_monitor;
pub mod data_quality_pipeline;
pub mod decent_mem;
pub mod decision_chain;
pub mod design_token;

pub mod dependency_strategy;
pub mod diff_impact;
pub mod dream;
pub mod dream_cycle_scheduler;
pub mod dual_lever_orchestrator;
pub mod edit_guard;
pub mod egpo_engine;
pub mod engineering_workflow;
pub mod escher_loop_engine;
pub mod epistemic;
pub mod evolution_bridge;
pub mod evolution_coordinator;
pub mod evolution_task_system;
pub mod evosc;
pub mod experience_tree;
pub mod extract_pipeline;
pub mod findings_aggregator;
pub mod erl_heuristic_pool;
pub mod failure_taxonomy;
pub mod failure_trace;
pub mod faithfulness_auditor;
pub mod faithfulness_checker;
pub mod fggm_safety;
pub mod frame_grounded_repair;
pub mod fusion_deliberator;
pub mod gap_detector_bridge;
pub mod gate;
pub mod goal_decomposer;
pub mod goal_drift_index;
pub mod godel_checker;
pub mod graceful;
pub mod gradient_seal_bridge;
pub mod grpo_trainer;
pub mod handler_profiler;
pub mod handler_tier;
pub mod harness_slot;
pub mod health_checkable;
pub mod health_patrol;
pub mod homeostatic_drive;
pub mod html_presentation;
pub mod humanizer;
pub mod hybrid_retrieval;
pub mod hyperagent;
pub mod hypothesis_tree;
pub mod identity_correlator;
pub mod identity_generator;
pub mod imagination_engine;
pub mod in_page_agent;
pub mod knowledge_node;
pub mod loop_engine;
pub mod loop_templates;
pub mod loop_registry;
pub mod loop_audit;
pub mod independent_verifier;
pub mod work_discovery_loop;
pub mod layered_mutability;
pub mod loss_function;
pub mod memory_consolidation;
pub mod memory_ops;
pub mod meta_cog_mera;
pub mod meta_evolution;
pub mod meta_improvement;
pub mod micro_reflective_loop;
pub mod mirror_threads;
pub mod motion_synthesizer;
pub mod mtc_assessment;
pub mod multi_signal_retrieval;
pub mod native_evolution_explorer;
pub mod news_radar;
pub mod open_skill;
pub mod operational_mirror;
pub mod orchestrator_bridge;
pub mod osint_tools;
pub mod outcome_tracker;
pub mod ouroboros_stage_manager;
pub mod pareto_front_selector;
pub mod parl;
pub mod pcc_safety;
pub mod phase2_memory;
pub mod phase3_meta;
pub mod policy_repair;
pub mod population_funnel;
pub mod principle_distiller;
pub mod reasoning_ke_bridge;
pub mod recovery_recipe;
pub mod recursive_delegation;
pub mod repo_understanding;
pub mod research_package;
pub mod reflective_analyzer;
pub mod reflector;
pub mod reliability_gate;
pub mod research_writer;
pub mod response_generator;
pub mod edit_journal;
pub mod cpe_regularizer;
pub mod ideal_state;
pub use ideal_state::{
    AIRating, Criterion, CriterionDomain, EFEGoal, EffortLevel, IdealState,
    IdealStateOutput, Prediction, PredictionConfidence, PredictionRegistry,
    PredictionStats, PredictionStatus, ReverseIntent, VerificationResult,
    bitter_lesson_check, process_with_ideal_state, reverse_intent,
};
pub mod rl_consolidation;
pub mod safety_ball;
pub mod safety_gate;
pub mod sage_rollout;
pub mod sahoo;
pub mod sahoo_embed;
pub mod sandbox_evaluator;
pub mod sandbox_executor;
pub mod persona_adapter;
pub mod sar_diagnostic;
pub mod scaffold;
pub mod schemas;
pub mod seal_closed_loop;
pub mod seal_governance;
pub mod seal_proposal_bridge;
pub use seal_proposal_bridge::{SealProposal, SealProposalBridge, ProposalPriority, ProposalStatus};
pub mod self_arch_audit;
pub mod self_evolution_engine;
pub mod self_evolution_loop;
pub mod sepl_operators;
pub mod search_keyword_optimizer;
pub use evolution_task_system::{EvolutionTask, EvolutionTaskSystem, TaskSystemStats, TaskType};
pub mod internet_absorption_bridge;
pub mod self_model_generator;
pub mod memory_archiver;
pub mod self_evolution_meta_layer;
pub mod self_evolution_orchestrator;
pub mod self_evolution_pipeline;
pub mod self_evolution_task_engine;
pub mod self_harness;
pub mod failure_evidence_batcher;
pub mod self_manifest;
pub mod self_introspection;
pub mod self_pacing_governor;
pub mod self_play_guide;
pub mod self_revision;
pub mod self_understanding;
pub mod skill_acc;
pub mod skill_crystal;
pub mod skill_dag;
pub mod skill_health;
pub mod skill_progressive;
pub mod skill_unified;
pub mod multi_timeline;
pub mod deep_absorption_pipeline;
pub mod deep_digestion;
pub mod constellation;
pub mod cross_timeline;
pub mod sub_agent_accumulator;
pub mod soul_identity;
pub mod sparse_vsa_attention;
pub mod stacked_validation;
pub use stacked_validation::{LayerResult, StackedValidationPipeline, ValidationReport};
pub mod storm_engine;
pub mod story_generator;
pub mod thought_flow_viz;
pub mod timem;
pub mod tool_orchestrator;
pub mod tool_safety;
pub mod tool_synthesizer;
pub mod trace_capture_engine;
pub mod trace_encoder;
pub mod trajectory_heuristics;
pub mod trial_worker;
pub mod two_phase_evolution;
pub mod uncertainty_quant;
pub mod verification_gate;
pub mod visual_planner;
pub mod voice_synthesis;
pub mod vsa_decoder;
pub mod vsa_judge;
pub mod vsi;
pub mod workflow_engine;
pub mod web_content_extractor;
pub mod workstream_exporter;
pub use calibration_engine::{CalibrationEngine, CalibrationStats};
pub use code_mutation_engine::{CodeMutationEngine, CodeMutation, MutationStrategy, EvaluatorFeedback};
pub use capability_synthesizer::{
    Capability, CapabilityStats, CapabilitySynthesizer, CapabilityType, SynthesisOutcome,
};
pub use consolidation_bridge::{
    BridgeConfig, BridgeV2Stats, ConsolidatedMemory, ConsolidationBridgeV2,
};
pub use cues::{CueConfig, CueEngine, CueStats, CueTask};
pub use curriculum::{CurriculumGenerator, DifficultyLevel, GeneratorConfig, TaskTemplate};
pub use data_quality_pipeline::{
    ActionType, DQConfig, DataQualityPipeline, DiagnosticReport, MonitorSnapshot, QualityDimension,
    QualityIssue, QualityTrend, RemediationAction, RuleType, Severity, TrendDirection,
};
pub use evosc::{
    ConsolidationStats, ContrastiveInsight, ContrastiveReflector, ContrastiveStats, EvoSC,
    EvoSCStats, SelfConsolidation,
};
pub use failure_taxonomy::{
    ClassifierConfig, FailureModeClassifier, FailureModeStats, FailureModeType,
};
pub use gate::{AttentionGate, GateStats, GatedItem, UtilitySignal};
pub use mirror_threads::{CognitiveThread, ThreadManager, ThreadManagerStats, ThreadType};
pub use open_skill::{
    KnowledgeAnchor, OpenSkillEngine, OpenSkillStats, SelfBuiltVerifier, SkillBlueprint,
    VerifierStats, VirtualTask,
};
pub use ouroboros_stage_manager::{
    CheckType, GateCheck, OuroborosConfig, OuroborosLoop, OuroborosStageManager, Stage, StageGate,
    StageStatus, StageTransition,
};
pub use safety_gate::{CheckResult, SafetyGate, SafetyReport};
pub use web_content_extractor::{
    ContentCategory, ExtractionMethod, WebContentConfig, WebContentExtractor, WebPageContent,
};
pub use anti_spiral_monitor::{AntiSpiralConfig, AntiSpiralMonitor, SpiralDetection, SpiralPattern};
pub use ast_mutation_engine::{
    AstMutation, AstMutationConfig, AstMutationEngine, AstMutationStats, MutationResult, MutationType,
};
pub use auto_commit_gate::{AutoCommitConfig, AutoCommitGate, GateResult, VerificationStep};
pub use background_evolution_scheduler::{
    AuditRecord, AuditType, BESConfig, BESStats, BackgroundEvolutionScheduler,
};
pub use self_source_reader::{ParsedSourceFile, SelfSourceReader, SourceReaderConfig, SourceReaderStats};
pub use workstream_exporter::*;

// Additional re-exports needed by neotrix/ modules
pub use handler_profiler::HandlerTier;
pub use imagination_engine::ImaginationEngine;
pub use meta_evolution::MetaEvolutionLoop;
pub use native_evolution_explorer::NativeEvolutionExplorer;
pub use reliability_gate::{EditOutcome, ReliabilityReport};
pub use sar_diagnostic::{ConsciousnessVitals, SarReport};
pub use scaffold::ScaffoldGenerator;
pub use seal_closed_loop::SealClosedLoop;
pub use self_evolution_engine::SelfEvolutionEngine;
pub use self_evolution_loop::MutationOp;
pub use self_evolution_meta_layer::{MetaProcedureConfig, SelfEvolutionMetaLayer};
pub use self_evolution_orchestrator::{
    GapScanResult, OrchestratorConfig, OrchestratorStats, SelfEvolutionTaskOrchestrator, WiringStatus,
    WiringTask,
};
pub use self_evolution_pipeline::{PipelinePhase, PipelineStats, SelfEvolutionPipeline};
pub use soul_identity::IdentityUpdateData;
// Re-exports for missing items (compiler-driven)
pub use adversarial::GödelAgent;
pub use context_manager::{
    BudgetSection, ContextBudget, ContextManager, ContextTier, LoadedSection, VsDedupPipeline,
};
pub use context_memory::{
    ContextCoherentMemory, EpisodicChunk, MemoryChunk, MemoryStats, SemanticChunk,
    SessionFingerprint,
};
pub use escher_loop_engine::{
    AgentType, ArchivedAgent, EscherLoopConfig, EscherLoopEngine, EscherLoopResult, EscherLoopStats,
    EvolvableAgent,
};
pub use epistemic::{
    ConceptNode, DomainConfidence, EpistemicConfig, EpistemicSelfModel, EpistemicState,
};
pub use evolution_bridge::EvolutionBridge;
pub use failure_trace::ExplorationGraph;
pub use fusion_deliberator::FusionDeliberator;
pub use goal_decomposer::GoalDecomposer;
pub use goal_drift_index::GoalDriftIndex;
pub use handler_profiler::HandlerProfiler;
pub use handler_tier::LoadTier;
pub use health_patrol::GlobalHealthPatrol;
pub use html_presentation::{Slide, SlideContent, SlideLayout};
pub use hypothesis_tree::HypothesisTreeConfig;
pub use identity_generator::VisualSignature;
pub use in_page_agent::{ActionPlan, DomAction, DomElement};
pub use internet_absorption_bridge::{DiscoveredPattern, InternetAbsorptionBridge};
pub use loss_function::LossFunction;
pub use memory_consolidation::MemoryConsolidationPipeline;
pub use policy_repair::{
    FailurePattern, FailureType, PolicyRepairEngine, RepairMode, RepairPolicy,
};
pub use reflector::{
    ExperienceReflector, Heuristic, HeuristicCategory, HeuristicFilter, ReflectorConfig,
};
pub use response_generator::{ConsciousnessSnapshot, ResponseGenerator};
pub use self_arch_audit::{ArchAuditReport, SelfArchAudit, WiringGap};
pub use repo_understanding::{
    ArchLayer, Capability as RepoCapability, CrossRepoComparison, DesignPrinciple, ExternalDep,
    KeyDecision, RepoUnderstanding, RepoUnderstandingEngine, UniqueInnovation,
};
pub use self_evolution_task_engine::{
    EngineEvolutionTask, EngineTaskStatus, EngineTaskType, SelfEvolutionTaskEngine, TaskResult,
};
pub use self_harness::{
    HarnessProposal, HarnessProposer, HarnessWeakness, ProposalValidator, SelfHarnessEngine,
    WeaknessMiner,
};
pub use self_manifest::SelfManifestGenerator;
pub use self_manifest::SelfManifest;
pub use knowledge_node::KnowledgeGraph;
pub use design_token::{TokenRegistry, PrimitiveToken, PrimitiveDomain, SemanticToken, ComponentToken};
pub use decision_chain::{DecisionChain, DecisionContext, DecisionEntry};
pub use principle_distiller::{PrincipleDistiller, DistillerConfig};
pub use skill_acc::{SkillAccumulator, SkillComposition, SkillFilter, VSASkill};
pub use sub_agent_accumulator::{AccumulatorStats, RetrievalStrategy, SubAgent, SubAgentAccumulator, SubAgentPhase};
pub use soul_identity::SoulIdentity;
pub use sparse_vsa_attention::SparseVsaAttentionEngine;
pub use thought_flow_viz::{
    ThoughtEdge, ThoughtFlow, ThoughtFlowViz, ThoughtNode, ThoughtNodeType,
};
pub use tool_orchestrator::{DetectedIntent, ToolOrchestrator};
pub use tool_safety::SafetyDecision;
pub use tool_synthesizer::{SynthesizedTool, ToolSynthesizer};
pub use trace_capture_engine::{
    AggregatedTrace, TraceCaptureEngine, TraceCaptureStats, TraceEvent, TraceSeverity, TraceSource,
};
pub use trace_encoder::{CalibrationSnapshot, InterventionEvent, StructuredTrace, TraceEncoder, TraceEncoderConfig};
pub use vsa_decoder::OutputQualityRecord;
pub use workflow_engine::{ExperienceWorkflowResult, OutputMapping, StepResult, WorkflowEngine};

pub use archive_manager::{ArchiveManager, ArchiveNode, ArchiveStats, ArchiveTree};
pub use sandbox_evaluator::{SandboxConfig, SandboxEvaluator, SandboxResult};
pub use auto_mlir_generator::{AutoMlirConfig, AutoMlirGenerator, MutationStep, StepStatus};
pub use sepl_operators::{
    CommitGate, EvaluationEngine, HypothesisEngine, ImprovementEngine, SeplCommit, SeplContext,
    SeplEvaluate, SeplHypothesis, SeplImprove, SeplPipeline, SeplProposal, SeplReflect, SeplReport,
    SeplScore, SeplSelect, SelectionEngine,
};
pub use constraint_registry::ConstraintRegistry;
pub use context_compressor::CognitiveContextCompressor;
pub use retrieval_evolution_engine::{
    DiagnosisCategory, FusionMode, RetrievalConfig, RetrievalDiagnosis, RetrievalEvolutionEngine,
    RetrievalTrace,
};
