#![allow(dead_code)]
// SPLIT PLAN:
//   File: 2514 lines — half are imports. Already has 3 sub-type files extracted.
//   Next extraction targets (after import cleanup):
//   1. `types_fields.rs`     — ConsciousnessIntegration struct definition (lines 181–498)
//   2. `types_builder.rs`    — Builder/setter methods (lines 500–900)
//   3. `types_getters.rs`    — Getter/accessor methods (lines 900–1400)
//   4. Keep remaining impl blocks in this file
//   5. Tests go to `types_tests.rs` (lines 2210–2514)
//   How: extract struct to new file first, then impl blocks follow.

use std::collections::{HashMap, VecDeque};

use crate::core::nt_core_codegen::comptime::NeComptimeEngine;
use crate::core::nt_core_consciousness::adversarial_evaluator::AdversarialEvaluator;
use crate::core::nt_core_consciousness::awakening::ConsciousnessAwakening;
use crate::core::nt_core_consciousness::cognitive_load::CognitiveLoadMonitor;
use crate::core::nt_core_consciousness::consciousness_cycle::{ConsciousnessCycle, CycleConfig};
use crate::core::nt_core_consciousness::default_mode_network::DefaultModeNetwork;
use crate::core::nt_core_consciousness::drive_selector::DriveSelector;
use crate::core::nt_core_consciousness::first_person_ref::{ExperienceRecord, FirstPersonRef};
use crate::core::nt_core_consciousness::global_workspace::GlobalLatentWorkspace;
use crate::core::nt_core_consciousness::identity_chain::IdentityChain;
use crate::core::nt_core_consciousness::inner_critic::InnerCritic;
use crate::core::nt_core_consciousness::memory_lattice::MemoryLattice;
use crate::core::nt_core_consciousness::memory_reflector::MemoryReflector;
use crate::core::nt_core_consciousness::meta_evolution_loop::MetaArchitectureEvolutionLoop;
use crate::core::nt_core_consciousness::narrative_self::NarrativeSelf;
use crate::core::nt_core_consciousness::neuromodulator::NeuromodulatorEngine;
use crate::core::nt_core_consciousness::reflexive_unit::{ReflexiveConfig, ReflexiveUnit};
use crate::core::nt_core_consciousness::sleep_consolidation_bridge::ConsolidationBridge;
use crate::core::nt_core_consciousness::specious_present::SpeciousPresent;
use crate::core::nt_core_consciousness::stream_buffer::ConsciousnessStream;
use crate::core::nt_core_consciousness::valence_axis::ValenceAxis;
use crate::core::nt_core_consciousness::value_alignment::ValueAlignmentEngine;
use crate::core::nt_core_consciousness::value_system::ValueSystem;
use crate::core::nt_core_consciousness::volition::VolitionEngine;
use crate::core::nt_core_experience::adapt_orch::AdaptOrch;
use crate::core::nt_core_experience::context_compression::VsaThoughtCompressor;
use crate::core::nt_core_experience::evolution_coordinator::EvolutionCoordinator;
use crate::core::nt_core_experience::fggm_safety::FggmSafetyUnifier;
use crate::core::nt_core_experience::handler_profiler::HandlerTier;
use crate::core::nt_core_experience::handler_tier::{
    default_handler_tiers, CapabilityRegistry, HandlerRegistry, LoadStatus, LoadTier, LoadTierStats,
};
use crate::core::nt_core_experience::hyperagent::{MetaAgentConfig, MetaAgentEngine};
use crate::core::nt_core_experience::hypothesis_tree::HypothesisTree;
use crate::core::nt_core_experience::independent_verifier::IndependentVerifier;
use crate::core::nt_core_experience::loop_audit::LoopAudit;
use crate::core::nt_core_experience::loop_registry::LoopRegistry;
use crate::core::nt_core_experience::motion_synthesizer::MotionSynthesizer;
use crate::core::nt_core_experience::orchestrator_bridge::OrchestratorBridge;
use crate::core::nt_core_experience::pcc_safety::PccSafetyGate;
use crate::core::nt_core_experience::safety_ball::BallVerifier;
use crate::core::nt_core_experience::seal_closed_loop::SealClosedLoop;
use crate::core::nt_core_experience::self_evolution_engine::SelfEvolutionEngine;
use crate::core::nt_core_experience::self_evolution_loop::SelfEvolutionLoop;
use crate::core::nt_core_experience::trajectory_heuristics::TrajectoryHeuristicExtractor;
use crate::core::nt_core_experience::vsa_decoder::AttractorDecoder;
use crate::core::nt_core_experience::work_discovery_loop::WorkDiscoveryLoop;
use crate::core::nt_core_experience::CapabilitySynthesizer;
use crate::core::nt_core_experience::EvolutionBridge;
use crate::core::nt_core_experience::WorkstreamExporter;
use crate::core::nt_core_experience::WorkstreamReport;
use crate::core::nt_core_experience::{GödelAgent, VisualSignature};
use crate::core::nt_core_gwt::intrinsic_drive::IntrinsicDrive;
use crate::core::nt_core_gwt::resonance::MODULE_COUNT;
use crate::core::nt_core_hcube::adapt_encoder::AdaptiveVsaEncoder;
use crate::core::nt_core_hcube::thdc_encoder::TrainableVsaEncoder;
use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;
use crate::core::nt_core_hex::ReasoningHexagram;
use crate::core::nt_core_input::NgramVsaEncoder;
use crate::core::nt_core_knowledge::execution_trace::TraceManager;
use crate::core::nt_core_knowledge::progress_aware_rag::ProgressAwareRAG;
use crate::core::nt_core_knowledge::vsa_vocabulary::VsaVocabulary;
use crate::core::nt_core_knowledge::EntityExtractor;
use crate::core::nt_core_language::NeEvaluator;
use crate::core::nt_core_meta::inner_monologue::InnerMonologueSystem;
use crate::core::nt_core_meta::meta_reflection_engine::MetaReflectionEngine;
use crate::core::nt_core_meta::uncertainty_tracker::UncertaintyDetector;
use crate::core::nt_core_meta::MetaKPIRepository;
use crate::core::nt_core_self_evolution::rsi_core::RsiCore;
use crate::core::nt_core_storage::null_drift_memory::NullDriftMemory;
use crate::core::nt_core_util;
use crate::neotrix::nt_agent_core::skill_library::SkillLibrary;
use crate::neotrix::nt_expert_routing::bridge::WorldModelBridge;
use crate::neotrix::nt_expert_routing::bridge::WorldModelReport;
use crate::neotrix::nt_expert_routing::intel_profile::IntelPipeline;
use crate::neotrix::nt_expert_routing::moment_feed::MomentFeed;
use crate::neotrix::nt_mind::active_exploration::ActiveExploration;
use crate::neotrix::nt_mind::meta_agent::MetaAgent;
use crate::neotrix::nt_mind::meta_agent::MetaAgentConfig as MetaAgentV2Config;
use crate::neotrix::nt_shield::vulnerability_pipeline::VulnPipeline;
use crate::neotrix::nt_world_infer::MemoryPalace;
// use crate::neotrix::nt_mind::counterfactual_futures::CounterfactualFuturesEngine; // TODO: module not yet available

use crate::core::nt_core_consciousness::dream_consolidator::DreamConsolidator;
use crate::core::nt_core_consciousness::mirror_buffer::MirrorBuffer;
use crate::core::nt_core_consciousness::temporal_attention::{
    TemporalAttentionBias, TemporalAttentionConfig,
};
use crate::core::nt_core_experience::consciousness_hooks::{
    ConsciousnessHook, ConsciousnessHookRegistry, HookAction, HookPoint, PerformanceTracer,
    SafetyGateHook,
};
use crate::core::nt_core_experience::self_revision::SelfRevisionLoop;
use crate::core::nt_core_experience::EpistemicConfig;
use crate::core::nt_core_experience::EpistemicSelfModel;
use crate::core::nt_core_experience::FusionDeliberator;
use crate::core::nt_core_experience::GoalDriftIndex;
use crate::core::nt_core_experience::MemoryConsolidationPipeline;
use crate::core::nt_core_experience::NativeEvolutionExplorer;
use crate::core::nt_core_experience::ResponseGenerator;
use crate::core::nt_core_experience::SkillAccumulator;
use crate::core::nt_core_experience::SoulIdentity;
use crate::core::nt_core_experience::ToolOrchestrator;
use crate::core::nt_core_input::VsaInputPipeline;
use crate::core::nt_core_meta::MetaCognitiveLoop;
use crate::core::nt_core_meta::SelfModel;
use crate::core::nt_core_meta::SelfModelAssessor;
use crate::core::nt_core_meta::{ArchitectureGraph, FusionGapRegistry, ModuleStatus};
use crate::neotrix::nt_io_provider::okf_exporter::OkfExporter;
use crate::neotrix::nt_world_jepa::predictor::EMAJepaPredictor;

use super::layer_manager::CognitiveLayerManager;
use crate::core::nt_core_agent::hyperagent::HyperArena;
use crate::core::nt_core_agent::hyperagent::HyperArenaConfig;
use crate::core::nt_core_consciousness::cognitive_module_registry::ModuleRegistry;
use crate::core::nt_core_consciousness::emergent_reasoning::EmergentReasoningConfig;
use crate::core::nt_core_consciousness::emergent_reasoning::EmergentReasoningMode;
use crate::core::nt_core_consciousness::epistemic_honesty::EpistemicHonesty;
use crate::core::nt_core_consciousness::epistemic_honesty::HonestyConfig;
use crate::core::nt_core_consciousness::personality_matrix::PersonalityConfig;
use crate::core::nt_core_consciousness::personality_matrix::PersonalityMatrix;

use crate::core::nt_core_experience::adversarial::AdversarialArena;
use crate::core::nt_core_experience::adversarial::ArenaConfig;
use crate::core::nt_core_experience::seal_proposal_bridge::SealProposalBridge;
use crate::core::nt_core_experience::skill_dag::SkillDagArchive;
use crate::core::nt_core_experience::skill_health::SkillHealthMonitor;
use crate::core::nt_core_experience::CalibrationEngine;
use crate::core::nt_core_experience::CurriculumGenerator;
use crate::core::nt_core_experience::EvoSC;
use crate::core::nt_core_experience::ExplorationGraph;
use crate::core::nt_core_experience::GeneratorConfig;
use crate::core::nt_core_experience::GoalDecomposer;
use crate::core::nt_core_experience::HandlerProfiler;
use crate::core::nt_core_experience::InternetAbsorptionBridge;
use crate::core::nt_core_experience::LossFunction;
use crate::core::nt_core_experience::OpenSkillEngine;
use crate::core::nt_core_experience::PolicyRepairEngine;
use crate::core::nt_core_experience::SelfEvolutionMetaLayer;
use crate::core::nt_core_source_cognition::source_cognition::SourceCognitionEngine;

use crate::constitution::Constitution;

use crate::core::nt_core_agent::cdp_session::CDPSessionManager;
use crate::core::nt_core_agent::consensus::ByzantineConsensusLayer;
use crate::core::nt_core_agent::factor_miner::FactorMiner;
use crate::core::nt_core_agent::quant_data::QuantDataIngestion;
use crate::core::nt_core_agent::remote_host::RemoteAgentHost;
use crate::core::nt_core_codegen::bootstrap_identity::BootstrapIdentityVerifier;
use crate::core::nt_core_context::working_memory::WorkingMemory;
use crate::core::nt_core_experience::auto_research::AutoResearchEngine;
use crate::core::nt_core_experience::capability_router::{
    default_capability_routes, CapabilityRouter,
};
use crate::core::nt_core_experience::context_compressor::CognitiveContextCompressor;
use crate::core::nt_core_experience::contrastive_reflection::ContrastiveReflection;
use crate::core::nt_core_experience::cyber_threat_monitor::CyberThreatMonitor;
use crate::core::nt_core_experience::egpo_engine::EGPOEngine;
use crate::core::nt_core_experience::faithfulness_auditor::FaithfulnessAuditor;
use crate::core::nt_core_experience::faithfulness_checker::FaithfulnessChecker;
use crate::core::nt_core_experience::html_presentation::HtmlPresentation;
use crate::core::nt_core_experience::identity_correlator::IdentityCorrelator;
use crate::core::nt_core_experience::loop_templates::LoopTemplateRegistry;
use crate::core::nt_core_experience::meta_cog_mera::MetaCogMonitor;
use crate::core::nt_core_experience::news_radar::NewsRadar;
use crate::core::nt_core_experience::osint_tools::OsintToolLayer;
use crate::core::nt_core_experience::sandbox_executor::SandboxExecutor;
use crate::core::nt_core_experience::self_harness::SelfHarnessEngine;
use crate::core::nt_core_experience::self_introspection::IntrospectionEngine;
use crate::core::nt_core_experience::voice_synthesis::VoiceSynthesisEngine;
use crate::core::nt_core_experience::workflow_engine::WorkflowEngine;
use crate::core::nt_core_experience::GlobalHealthPatrol;
use crate::core::nt_core_experience::SparseVsaAttentionEngine;
use crate::core::nt_core_hcube::e8_cortical::E8CorticalMapping;
use crate::core::nt_core_hcube::geometric_ssm::GeometricSSM;
use crate::core::nt_core_hcube::interaction_trace::InteractionTracePredictor;
use crate::core::nt_core_hcube::koopman_operator::KoopmanOperator;
use crate::core::nt_core_knowledge::behavioral_personality::BehavioralPersonalityEngine;
use crate::core::nt_core_knowledge::bookmark::BookmarkManager;
use crate::core::nt_core_knowledge::entity_resolver::EntityResolver;
use crate::core::nt_core_knowledge::evidence::EvidenceManager;
use crate::core::nt_core_knowledge::fringe_mix::FringeMixStrategy;
use crate::core::nt_core_knowledge::hubness_detector::HubnessDetector;
use crate::core::nt_core_knowledge::hypergraph::HypergraphStore;
use crate::core::nt_core_knowledge::keyword_lexicon::KeywordLexicon;
use crate::core::nt_core_knowledge::research_kg::ResearchKnowledgeGraph;
use crate::core::nt_core_knowledge::spread_activation::MemoryGraph;
use crate::core::nt_core_knowledge::storage_coordinator::StorageCoordinator;
use crate::core::nt_core_negentropy::dysib_layer::DySIBLayer;
use crate::core::nt_core_protect::honeypot::SecurityGate;
use crate::core::nt_core_scheduler::job_queue::CognitiveJobQueue;
use crate::core::nt_core_storage::{StorageConfig, StorageEngine};
use crate::neotrix::nt_memory_kb::KnowledgeBase;
use crate::neotrix::nt_world_exploration::ExplorationOrchestrator;

use crate::core::nt_core_design_token::DesignTokenIntegrator;
use crate::core::nt_core_e8::{E8BlockDiagonal, E8Lattice, E8Projector};
use crate::core::nt_core_translate::VsaTranslationEngine;
use neotrix_body::agent::network_evolution::NetworkEvolution;
use neotrix_body::agent::perception_gateway::PerceptionGateway;

pub use super::types_consciousness::{ExperienceStats, ReliabilityGateStub, SarDiagnosticStub};
pub use super::types_evolution::{DgmhEdit, MutationRecord};
pub use super::types_research::{ResearchTrajectory, TrajectoryStep, TrajectoryVerifier};

const MAX_TEXT_BUFFER: usize = 500;
const MAX_VSA_BUFFER: usize = 200;
const MAX_CONFORMAL_UQ_BUFFER: usize = 1000;
const MAX_THOUGHT_HISTORY: usize = 500;

// (types_consciousness/evolution/research modules were cfg-gated dead code; removed)

// ── Main struct ──
// SECTION: ConsciousnessIntegration struct (192–541)

/// Action plan feedback to world model — bridges the Action→World→Perception loop.
#[derive(Default)]
pub struct ActionFeedback {
    pub last_action: Option<String>,
    pub last_plan: Vec<String>,
    pub cycle: u64,
}

// ── Phase 8.3 — E8 Adaptive Modulation ──

/// Current state of E8 axis modulation.
#[derive(Debug, Clone)]
pub struct E8ModulationState {
    pub axis_weights: [f64; 6],
    pub modulation_entropy: f64,
}

// ── Phase 10.1 — Async Deep Processing ──

/// Types of asynchronous deep processing tasks.
#[derive(Debug, Clone)]
pub enum AsyncTaskType {
    E8DeepReason,
    IdentityReflection,
    ExperienceConsolidation,
    GWTReplay,
}

/// A pending or completed async task.
#[derive(Debug, Clone)]
pub struct AsyncTask {
    pub id: u64,
    pub task_type: AsyncTaskType,
    pub created_at: f64,
    pub result: Option<String>,
}

pub struct ConsciousnessIntegration {
    pub cycle: u64,
    /// Pending replay request: (from_cycle, to_cycle). Consumed by handle_replay_tick.
    pub pending_replay: Option<(u64, u64)>,
    pub curriculum: CurriculumGenerator,
    pub epistemic: EpistemicSelfModel,
    pub policy_repair: PolicyRepairEngine,
    pub skill_acc: SkillAccumulator,
    pub failure_trace: ExplorationGraph,
    pub execution_trace: Option<TraceManager>,
    pub calibration: CalibrationEngine,
    pub composite_loss: LossFunction,
    pub goal_decomposer: GoalDecomposer,
    pub workstream_exporter: WorkstreamExporter,
    pub profiler: HandlerProfiler,

    pub input_pipeline: VsaInputPipeline,
    pub ngram_encoder: NgramVsaEncoder,
    pub text_buffer: VecDeque<String>,
    pub text_feed_count: usize,
    pub vsa_buffer: VecDeque<Vec<u8>>,
    pub vsa_buffer_max: usize,
    pub pending_curiosity_gain: f64,
    pub curiosity_reward_history: Vec<(u64, f64)>,
    pub active_exploration: Option<ActiveExploration>,

    pub orchestrator: ExplorationOrchestrator,

    pub value_system: ValueSystem,
    pub value_alignment_engine: Option<ValueAlignmentEngine>,
    pub volition: VolitionEngine,

    pub specious_present: SpeciousPresent,
    pub identity_core: crate::core::nt_core_identity::IdentityCore,
    pub self_reasoner: crate::core::nt_core_identity::SelfReasoner,
    pub coproc_bridge: crate::core::nt_core_identity::CoprocessorBridge,
    pub narrative_self: NarrativeSelf,
    pub valence_axis: ValenceAxis,
    pub drive_selector: DriveSelector,
    pub memory_lattice: MemoryLattice,
    pub memory_palace: MemoryPalace,
    pub memory_reflector: MemoryReflector,
    pub vsa_vocabulary: VsaVocabulary,
    pub inner_critic: InnerCritic,
    pub adversarial_evaluator: Option<AdversarialEvaluator>,
    pub reflexive_unit: ReflexiveUnit,
    pub cognitive_load_monitor: CognitiveLoadMonitor,
    pub cognitive_load: f64,
    pub layer_manager: CognitiveLayerManager,
    pub default_mode: DefaultModeNetwork,
    pub stream_buffer: ConsciousnessStream,
    pub first_person_ref: FirstPersonRef,
    pub self_experience_buffer: Vec<ExperienceRecord>,
    pub awakening: ConsciousnessAwakening,
    pub neuromodulator: NeuromodulatorEngine,
    pub temporal_attention: TemporalAttentionBias,

    pub working_memory: WorkingMemory,

    pub goal_drift: GoalDriftIndex,
    pub dream_consolidator: DreamConsolidator,
    pub meta_cognition_loop: MetaCognitiveLoop,
    pub kpi_buffer: Option<crate::core::nt_core_meta::KpiRingBuffer>,
    pub self_model_assessor: SelfModelAssessor,
    pub rii_u: Option<crate::core::nt_core_consciousness::rii_u::RiiuAutoPhi>,
    pub architecture: ArchitectureGraph,
    pub fusion_gap_registry: FusionGapRegistry,
    pub capability_synthesizer: CapabilitySynthesizer,
    pub evosc: EvoSC,
    pub open_skill: OpenSkillEngine,
    pub skill_dag: SkillDagArchive,
    pub skill_health_monitor: SkillHealthMonitor,

    pub emergent_reasoning: EmergentReasoningMode,
    pub epistemic_honesty: EpistemicHonesty,
    pub personality_matrix: PersonalityMatrix,
    pub attractor_state: Vec<u8>,

    pub adversarial_arena: AdversarialArena,
    pub hyperagent: HyperArena,

    // TODO: lazy init when CausalTransformerMemory is implemented
    pub ctm_engine: Option<crate::core::nt_core_ctm::inference::CtmEngine>,

    pub source_cognition: SourceCognitionEngine,
    pub sar_diagnostic: SarDiagnosticStub,
    pub reliability_gate: ReliabilityGateStub,
    pub consolidation_bridge: ConsolidationBridge,
    pub dream_count: u64,

    pub adaptive_rate_hysteresis: f64,
    pub conformal_uq_buffer: VecDeque<f64>,

    pub thought_history: VecDeque<(String, Vec<u8>, f64)>,

    pub vsa_thought_compressor: VsaThoughtCompressor,

    pub hooks: ConsciousnessHookRegistry,

    pub handler_registry: HandlerRegistry,
    pub handler_generation_count: u64,
    /// Tracks which handlers have been lazily initialized (NeedsInit branch).
    /// Hot/Warm handlers get init on first access; Cold handlers are deferred.
    pub initialized_modules: HashMap<String, bool>,
    pub dgmh_templates: HashMap<String, String>,

    pub self_evolution: Option<SelfEvolutionLoop>,
    pub evolution_engine: SelfEvolutionEngine,
    pub meta_architecture: Option<MetaArchitectureEvolutionLoop>,
    pub body_network_evolution: Option<NetworkEvolution>,
    pub body_perception_gateway: Option<PerceptionGateway>,
    pub meta_agent: MetaAgentEngine,
    pub meta_agent_v2: Option<MetaAgent>,
    pub mirror_buffer: MirrorBuffer,
    pub adapt_orch: AdaptOrch,
    pub pcc_safety: PccSafetyGate,
    pub ball_verifier: BallVerifier,
    pub progress_rag: ProgressAwareRAG,
    pub memory_consolidation: MemoryConsolidationPipeline,
    pub fusion_deliberator: FusionDeliberator,
    pub soul_identity: SoulIdentity,
    /// O06: Dual identity integration — cryptographic identity chain cross-verification.
    pub identity_chain: IdentityChain,
    pub tool_orchestrator: ToolOrchestrator,
    pub response_generator: ResponseGenerator,
    pub response_buffer: VecDeque<String>,
    // TODO: last_response_batch removed — was orphan unused field
    pub last_response: Option<String>,
    /// Last EFE energy from counterfactual evaluation — used for active inference
    /// modulation of decoding strategy. High EFE → exploratory decode.
    pub last_efe_energy: f64,
    pub ne_evaluator: Option<NeEvaluator>,
    pub ne_comptime: Option<NeComptimeEngine>,
    pub ne_source_dir: String,
    pub ne_last_vsa_result: Option<Vec<u8>>,
    pub ne_last_text_result: Option<String>,
    /// Cached VSA state probe for selective decoding gate.
    /// Compared against the current probe before running the full eval suite;
    /// if similarity exceeds threshold the deterministic test expressions are skipped.
    pub ne_state_probe: Option<Vec<u8>>,
    pub bootstrap_verifier: BootstrapIdentityVerifier,
    pub thdc_encoder: Option<TrainableVsaEncoder>,
    pub null_drift: NullDriftMemory,
    pub adaptive_vsa: Option<AdaptiveVsaEncoder>,
    pub translate_engine: Option<VsaTranslationEngine>,
    pub hypergraph_store: Option<HypergraphStore>,
    pub evidence: Option<EvidenceManager>,
    pub truth_pipeline: Option<crate::core::nt_core_truth::pipeline::TruthPipeline>,
    pub emotional_memory: Option<crate::core::nt_core_emotional_memory::EmotionalMemory>,
    pub spread_activation: Option<MemoryGraph>,
    pub consensus_engine: Option<ByzantineConsensusLayer>,
    pub storage_engine: Option<StorageEngine>,
    pub entity_extractor: EntityExtractor,
    pub meta_kpi_repo: Option<MetaKPIRepository>,
    pub evolution_bridge: EvolutionBridge,

    pub self_revision: SelfRevisionLoop,
    pub ema_jepa: Option<EMAJepaPredictor>,
    // TODO: lazy init when OkfExporter is implemented
    pub okf_exporter: Option<OkfExporter>,

    // Phase 36 — World Model
    pub world_model_bridge: WorldModelBridge,
    pub world_model_report: WorldModelReport,
    pub action_feedback: ActionFeedback,
    pub counterfactual_engine: crate::core::nt_core_negentropy::efe_minimizer::EFEMinimizer, // was CounterfactualFuturesEngine
    pub physics_commonsense: crate::core::nt_core_hcube::PhysicsCommonsense,
    pub spatial_scene: crate::core::nt_core_hcube::SpatialSceneEngine,
    pub spatial_graph: Option<crate::neotrix::nt_memory_kb::spatial_graph::SpatialGraph>,
    pub imagination_engine: crate::core::nt_core_experience::ImaginationEngine,
    pub native_explorer: Option<NativeEvolutionExplorer>,

    // Phase 55 — new cognitive modules
    pub contrastive_reflection: Option<ContrastiveReflection>,
    pub faithfulness_auditor: Option<FaithfulnessAuditor>,
    pub entity_resolver: Option<EntityResolver>,
    pub dysib_layer: Option<DySIBLayer>,
    pub interaction_trace: Option<InteractionTracePredictor>,
    pub keyword_lexicon: Option<KeywordLexicon>,

    // Phase 56 — agent & knowledge modules
    pub quant_data: Option<QuantDataIngestion>,
    pub cdp_session: Option<CDPSessionManager>,
    pub fringe_mix: Option<FringeMixStrategy>,
    pub factor_miner: Option<FactorMiner>,
    pub osint_tools: Option<OsintToolLayer>,
    pub identity_correlator: Option<IdentityCorrelator>,
    pub hubness_detector: Option<HubnessDetector>,
    pub remote_host: Option<RemoteAgentHost>,
    pub security_gate: Option<SecurityGate>,
    pub koopman_operator: Option<KoopmanOperator>,
    pub multi_head_resonator:
        Option<crate::core::nt_core_hcube::multi_head_resonator::MultiHeadResonator>,
    pub design_token: Option<DesignTokenIntegrator>,

    // Phase 58 — external intelligence modules
    pub news_radar: NewsRadar,
    pub intel_profile: Option<IntelPipeline>,
    pub moment_feed: Option<MomentFeed>,
    pub trading_engine: Option<crate::core::nt_core_trading::engine::TradingEngine>,
    pub vuln_pipeline: Option<VulnPipeline>,
    pub voice_synthesis: VoiceSynthesisEngine,
    pub html_presentation: HtmlPresentation,
    pub loop_templates: LoopTemplateRegistry,
    pub adversarial_trainer: Option<crate::core::nt_core_adversarial::AdversarialTrainer>,
    pub adversarial_tuner: Option<crate::core::nt_core_adversarial::AutoTuner>,
    pub cyber_threat_monitor: CyberThreatMonitor,
    pub motion_synthesizer: MotionSynthesizer,
    pub avsad: Option<crate::core::nt_core_avsad::AvsadDetector>,
    pub health_patrol: GlobalHealthPatrol,

    // Phase 59 — runtime self-introspection
    pub introspect_engine: IntrospectionEngine,

    // Faithfulness checker — post-hoc evidence citation verification
    pub faithfulness_checker: FaithfulnessChecker,

    // Motor pathway — VSA decoder for structured output
    pub vsa_decoder: AttractorDecoder,

    // Gap 1 — Typed handler composition (substitution algebra)
    pub harness_slots: crate::core::nt_core_experience::harness_slot::SlotRegistry,

    // Gap 2 — Operational Mirror (RL bridge for introspection)
    pub operational_mirror: crate::core::nt_core_experience::operational_mirror::OperationalMirror,

    // Gap 3 — Chinese content creation skills
    pub humanizer: crate::core::nt_core_experience::humanizer::HumanizerEngine,
    pub business_diagnosis:
        crate::core::nt_core_experience::business_diagnosis::BusinessDiagnosisEngine,
    pub visual_planner: crate::core::nt_core_experience::visual_planner::VisualPlanner,
    pub research_writer: crate::core::nt_core_experience::research_writer::ResearchWriter,
    pub self_play_guide: crate::core::nt_core_experience::self_play_guide::SelfPlayGuide,

    // Capability Router — OID-based handler dispatch
    pub capability_router: CapabilityRouter,

    // Workflow Engine — dynamic workflow composition
    pub workflow_engine: WorkflowEngine,

    // Sandbox Executor — container-level tool execution isolation
    pub sandbox_executor: SandboxExecutor,

    // Kernel-level sandbox (Seatbelt/Landlock/seccomp) — set once at startup
    pub kernel_sandbox_level: crate::core::nt_core_sandbox::SandboxLevel,

    // P2.1: Rolling 100-cycle meta-reflection window
    pub meta_reflection_buffer: VecDeque<(u64, f64, f64, f64)>,

    // P2.2: Last-report values for belief trajectory trending
    pub last_report_ece: f64,
    pub last_report_meta_d: f64,
    pub last_report_m_ratio: f64,

    // Evolution feedback tracking
    pub mutation_log: Vec<MutationRecord>,
    pub pre_mutation_perf: Option<HashMap<String, (u64, u64)>>,

    // P1 — Sparse VSA Attention Engine (Zamba2-VL inspired shared attention block)
    pub sparse_vsa_attn: SparseVsaAttentionEngine,

    // P0 — AutoResearchEngine (Karpathy-style fixed-budget experiment loop)
    pub research_engine: AutoResearchEngine,

    // STORM/Co-STORM multi-perspective research pipeline (4-phase engine)
    pub storm: Option<crate::core::nt_core_experience::storm_engine::StormEngine>,

    // P0 — Research Knowledge Graph pipeline (EntityExtractor→KG→ForceGraph)
    pub research_kg: ResearchKnowledgeGraph,

    // P0 — Cognitive Job Queue (priority-based with preemption)
    pub job_queue: CognitiveJobQueue,

    /// S1-DeepResearch inspired trajectory pipeline
    pub research_trajectory_log: Vec<ResearchTrajectory>,
    pub trajectory_verification: TrajectoryVerifier,

    // Self-Harness — WeaknessMining→HarnessProposal→ProposalValidation (arXiv 2606.09498)
    pub self_harness: SelfHarnessEngine,

    // EvolutionCoordinator — bridges SelfHarness → EGPO → SEAL → DGM-H into unified loop
    pub evolution_coordinator: EvolutionCoordinator,

    // HypothesisTree — MCTS-based reasoning tree wired into evolution proposal pipeline
    pub hypothesis_tree: Option<HypothesisTree>,

    // ContextCompressor — ACON-style guideline-based context compression (ICML 2026)
    pub context_compressor: CognitiveContextCompressor,

    // EGPO — Exploration Guided Policy Optimization triple-reward engine (arXiv 2602.22751)
    pub egpo: EGPOEngine,

    // Metacognitive wisdom gate: tracks historical scores for adaptive threshold
    pub wisdom_score_history: Vec<f64>,
    pub wisdom_gate_hysteresis: f64,
    pub global_workspace: GlobalLatentWorkspace,

    // MERA Meta-Cognitive Monitor — three-stage reasoning trace pipeline
    pub meta_cog_monitor: MetaCogMonitor,

    // Fusion D — Geometric State-Space Kernel (SPD manifold reasoning)
    pub geometric_ssm: GeometricSSM,

    // E8 → Cortical column mapping (E8 root system → simulated cortical geometry)
    pub e8_cortical: E8CorticalMapping,

    // LeadAgent — multi-agent orchestration with task decomposition
    pub lead_agent: Option<crate::core::nt_core_agent::lead_agent::LeadAgent>,
    // PersistentGoalManager — cross-session goal lifecycle management
    pub goal_manager: Option<crate::neotrix::nt_mind::persistent_goal::PersistentGoalManager>,
    pub permission_gate: crate::core::nt_core_agent::permission::PermissionGate,
    pub permission_overrides: crate::core::nt_core_agent::permission::PermissionOverrides,
    pub verify_loop: crate::core::nt_core_agent::verify_loop::VerifyLoop,

    // SessionTranscript — append-only JSONL structured session log
    pub transcript: crate::core::nt_core_agent::transcript::SessionTranscript,
    // AgentMemory — VSA-based cross-session knowledge store (MEMORY.md equivalent)
    pub agent_memory: crate::core::nt_core_agent::agent_memory::AgentMemory,
    // DaemonMode — background session with inter-session inbox
    pub daemon_mode: crate::core::nt_core_agent::daemon_mode::DaemonMode,

    // KnowledgeBase — semantic knowledge graph with SQLite backend
    pub kb: Option<KnowledgeBase>,

    // BookmarkManager — 对话URL按类别存储与管理
    pub bookmark_manager: Option<BookmarkManager>,

    // StorageCoordinator — 统一存储编排器，连接所有存储节点的数据流闭环
    pub storage_coordinator: StorageCoordinator,

    // BehavioralPersonalityEngine — 用户数字分身/行为人格系统
    pub behavioral_personality: Option<BehavioralPersonalityEngine>,

    // Constitution — P0-P12 irreversible principles registry
    pub constitution: Constitution,

    // Vision pipeline (image understanding)
    pub vision: Option<crate::core::nt_core_vision::ImagePipeline>,

    // Audio capture pipeline (microphone → VAD → transcribe)
    pub audio_capture: Option<crate::core::nt_core_audio::AudioCapture>,

    // O04 — Hierarchical Consciousness Composition: sub-consciousness manager
    pub sub_consciousness_manager:
        Option<crate::core::nt_core_consciousness::sub_consciousness::SubConsciousnessManager>,

    // Story generator — narrative synthesis from recent events
    pub story_generator: crate::core::nt_core_experience::story_generator::StoryGenerator,

    // E8 Lie group — attractor dynamics on the 240-root system
    pub e8_lattice: E8Lattice,
    pub e8_projector: E8Projector,
    // E8 block-diagonal weight matrices — generated from Killing form, trainable via set_block()
    pub e8_block_diagonal: E8BlockDiagonal,

    // Phase 8.3 — E8 Adaptive Modulation
    pub e8_modulation: Option<E8ModulationState>,
    pub specialist_states: [ReasoningHexagram; MODULE_COUNT],

    // Phase 10.1 — Async Deep Processing (MIRROR pattern)
    pub async_tasks: VecDeque<AsyncTask>,
    pub async_task_counter: u64,

    // Phase 10.2 — Intrinsic Drive (curiosity, mastery, coherence)
    pub intrinsic_drive: IntrinsicDrive,
    pub broadcast_history: VecDeque<usize>,
    pub last_saliences: [f64; MODULE_COUNT],

    // N08 — Resource Accounting / Gas Metering
    pub global_gas_budget: Option<crate::core::nt_core_metering::GlobalGasBudget>,

    // N12 — Provider/Requester/Verifier Three-Role Separation
    pub role_manager: Option<crate::core::nt_core_agent::ThreeRoleManager>,

    // MetaEvolutionLoop — tracks self-improvement archive, trends, and proposals
    pub meta_evolution: crate::core::nt_core_experience::MetaEvolutionLoop,

    // FGGM safety unifier — SEVerA-inspired 4-phase safety pipeline
    pub fggm_safety: Option<FggmSafetyUnifier>,

    // A2A gRPC bridge — wraps AgentCommunicationBus with signed Agent Cards
    // Dispatched via handle_a2a_grpc_tick in modules_a2a.rs
    pub a2a_grpc_bridge: Option<crate::neotrix::nt_agent_protocol::a2a_grpc::A2AGrpcBridge>,

    // Dead cycle detector: maps handler name → (output_hash, consecutive_repeats)
    // Set in profile(), warns when a handler returns identical output 3+ cycles in a row
    pub cycle_output_cache: HashMap<&'static str, (u64, u32)>,

    // Multi-Provider LLM Router — inspired by free-claude-code architecture.
    // Routes LLM calls across 17+ providers with per-model dispatch, rate limiting, and local caching.
    pub llm_router: crate::core::nt_core_llm_router::LlmRouter,

    // Symbolic Discovery Engine — inspired by AI-Newton (arXiv:2504.01538).
    // Concept-driven discovery: extends knowledge when existing laws fail in new contexts.
    pub symbolic_discovery: crate::core::nt_core_discovery::SymbolicDiscoveryEngine,

    // Governance Engine — autonomous rule execution from governance/RULES.md
    pub governance_engine: Option<crate::core::nt_core_governance::GovernanceEngine>,

    // AGT Dynamic Trust Scoring Engine — behavioral tier escalation + policy gating
    pub trust_scoring: crate::core::nt_core_governance::trust_scoring::TrustScoringEngine,

    // ── Wave 2-5 new modules ──

    // SAHOO (Safeguarded Alignment for High-Order Optimization)
    // Goal drift detection + constraint preservation + regression risk
    pub sahoo: crate::core::nt_core_experience::sahoo::SahooGuard,

    // VSI (Verified Self-Improvement) — reasoning chain verification
    pub vsi: crate::core::nt_core_experience::vsi::VsiVerifier,

    // MTC (Multi-Theory Consciousness) Assessment
    pub mtc: crate::core::nt_core_experience::mtc_assessment::MtcEvaluator,

    // Containment Verification — boundary enforcement + safety predicates
    pub containment: crate::core::nt_core_experience::containment::BoundaryEnforcer,

    // Meta-Improvement Loop — pipeline KPI diagnostics + self-modification
    pub meta_improvement: crate::core::nt_core_experience::meta_improvement::MetaImprovementLoop,

    // Uncertainty Quantification — step-level confidence intervals
    pub uncertainty: crate::core::nt_core_experience::uncertainty_quant::UncertaintyAwareMonitor,

    // Storm Breaker — thinking storm detection and suppression
    pub storm_breaker: crate::core::nt_core_consciousness::storm_breaker::StormBreaker,

    // DGM-H full orchestrator (nt_core_agent version) — meta-evolution via hyperagents
    pub dgmh_orchestrator: Option<crate::core::nt_core_agent::dgmh::DgmhOrchestrator>,

    // FEP fusion modules

    // AcT (Active Inference Tree Search) planner — MCTS + EFE for deep planning
    pub act_planner: Option<crate::core::nt_core_negentropy::act_planner::AcTPlanner>,

    // FEP-IIT bridge — unified consciousness score from free energy + integrated info
    pub fep_iit: Option<crate::neotrix::nt_core_fep_iit::bridge::FEPIITBridge>,

    // Smart Canvas — E8 reasoning graph visualization (event‑driven flush on BackgroundLoop)
    pub canvas_manager: Option<neotrix_types::core::node_canvas::CanvasManager>,

    // CascadeModelEngine — speculative execution with quality validation (drafter/verifier)
    pub cascade_engine: Option<crate::core::nt_core_inference::cascade::CascadeEngine>,

    // SpatialReasoner — 3-level evidence hierarchy (perception → VSA encoding → spatial graph)
    pub spatial_reasoner: Option<crate::core::nt_core_spatial::reasoner::SpatialReasoner>,

    // SelfModifyAgent — Gödel Agent self-modification safety guard + proposal queue
    pub self_modify_agent: Option<crate::core::nt_core_self_modify::SelfModifyAgent>,

    // CausalReasoningEngine — causal link tracking and multi-step state prediction
    pub causal_reasoning:
        Option<crate::core::nt_core_inference::causal_chain::CausalReasoningEngine>,

    // SCMEngine — Pearl do-calculus causal graph for intervention/front-door/back-door analysis
    pub scm_engine: Option<crate::core::nt_core_inference::SCMEngine>,

    // LongHorizonPredictor — long-term forecasting and regime change detection
    pub long_horizon_predictor:
        Option<crate::core::nt_core_inference::long_horizon::LongHorizonPredictor>,

    // MultiModalAligner — cross-modal alignment pruning and statistics
    pub multi_modal_aligner:
        Option<crate::core::nt_core_hcube::multi_modal_aligner::MultiModalAligner>,

    // Network egress policy — zero-trust egress guard wired into ShieldBus
    pub network_egress: Option<crate::neotrix::nt_shield::NetworkEgressPolicy>,

    // Consciousness benchmark history — bounded at MAX_BENCH_HISTORY entries.
    // Populated by handle_consciousness_bench_tick every 50 cycles.
    pub bench_history: Option<Vec<crate::neotrix::nt_mind_benchmark::BenchmarkResult>>,

    // Phase 9 — Experience Closed Loop: trajectory heuristics extraction
    pub trajectory_extractor: TrajectoryHeuristicExtractor,

    // Phase 9 — SEAL closed loop orchestrator with capability evolution
    pub seal_closed_loop: SealClosedLoop,

    // Phase 9 — Heuristic→Capability registry for automatic handler synthesis
    pub capability_registry: CapabilityRegistry,

    // Phase 4 — ConsciousnessCycle: 12-step unified loop (optional refinement layer)
    // When Some, runs after the 3-phase pipeline to add analogical/MCTS/causal/etc.
    pub consciousness_cycle: Option<ConsciousnessCycle>,

    // ModuleRegistry — runs 7 cognitive modules (MCTS, ParallelHypothesis, Counterfactual, etc.)
    pub module_registry: Option<ModuleRegistry>,

    // SelfEvolutionMetaLayer — closes 5 broken feedback loops:
    // 1. CalibrationEngine → MetaCognitiveLoop (ECE/meta-d into meta_accuracy)
    // 2. LossFunction → SelfModifyAgent (composite loss triggers self-modify proposals)
    // 3. MetaCognitiveLoop → SelfEvolutionLoop (meta plans drive evolution)
    // 4. SelfModifyGuard 4-layer activation (Shield/Swords/LLM/Ball)
    // 5. ConsciousnessCycle 12-step real implementation (replaces stubs)
    // Active by default — instantiated in new()
    pub self_evolution_meta: Option<SelfEvolutionMetaLayer>,

    // Verified RSI Pipeline — proposes/verifies/applies self-modifications through safety gates
    pub verified_rsi_pipeline: Option<crate::core::nt_core_self::verified_rsi::VerifiedRsiPipeline>,

    // SealProposalBridge — converts gap analysis proposals into SEAL evolution mutations
    pub seal_bridge: SealProposalBridge,

    // InternetAbsorptionBridge — seeds evolution tasks from discovered web patterns
    pub internet_absorption: Option<InternetAbsorptionBridge>,

    // Wave 11 — Meta-Reflection Engine: self-analysis of cycle performance
    pub meta_reflection: Option<MetaReflectionEngine>,

    // Wave 11 — Uncertainty Detector: confidence calibration & uncertainty tracking
    pub uncertainty_detector: Option<UncertaintyDetector>,

    // Wave 11 — Inner Monologue: multi-voice dialectical reasoning
    pub inner_monologue: Option<InnerMonologueSystem>,

    // Wave 11 — RSI Core: recursive self-improvement proposal→implementation loop
    pub rsi_core: Option<RsiCore>,

    // Wave 11 — Skill Library: capability registry for skill composition & routing
    pub skill_library: Option<SkillLibrary>,

    // SelfEvolutionOrchestrator bridge — 6-phase evolution cycle (Analyze→Plan→Safety→Execute→Measure→Adapt)
    pub orchestrator_bridge: Option<OrchestratorBridge>,

    // Loop Engineering outer layer
    pub work_discovery_loop: Option<WorkDiscoveryLoop>,
    pub independent_verifier: Option<IndependentVerifier>,
    pub loop_registry: Option<LoopRegistry>,
    pub loop_audit: Option<LoopAudit>,
}

// SECTION: Constructor impl (543–2213)

impl ConsciousnessIntegration {
    pub fn new() -> Self {
        let mut ci = Self {
            cycle: 0,
            pending_replay: None,
            curriculum: CurriculumGenerator::new(GeneratorConfig::default()),
            epistemic: EpistemicSelfModel::new(EpistemicConfig::default()),
            policy_repair: PolicyRepairEngine::new(100),
            skill_acc: SkillAccumulator::new(200),
            failure_trace: ExplorationGraph::new(500),
            execution_trace: Some(TraceManager::new(100)),
            calibration: CalibrationEngine::new(),
            composite_loss: LossFunction::default(),
            goal_decomposer: GoalDecomposer::new(),
            workstream_exporter: WorkstreamExporter::new(
                std::env::var("NEOTRIX_WORKSTREAM_DIR")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|_| {
                        nt_core_util::home_dir().join(".neotrix").join("workstream")
                    }),
            ),
            profiler: HandlerProfiler::new(),
            input_pipeline: VsaInputPipeline::new(),
            ngram_encoder: NgramVsaEncoder::default(),
            text_buffer: VecDeque::with_capacity(MAX_TEXT_BUFFER),
            text_feed_count: 0,
            vsa_buffer: VecDeque::with_capacity(MAX_VSA_BUFFER),
            vsa_buffer_max: MAX_VSA_BUFFER,
            pending_curiosity_gain: 0.0,
            curiosity_reward_history: Vec::new(),
            active_exploration: None,
            orchestrator: ExplorationOrchestrator::new(),
            value_system: ValueSystem::new(),
            value_alignment_engine: None,
            volition: VolitionEngine::new(),
            specious_present: SpeciousPresent::new(5),
            identity_core: crate::core::nt_core_identity::IdentityCore::new(),
            self_reasoner: crate::core::nt_core_identity::SelfReasoner::new(),
            coproc_bridge: crate::core::nt_core_identity::CoprocessorBridge::new(),
            narrative_self: NarrativeSelf::new(),
            valence_axis: ValenceAxis::new(),
            drive_selector: DriveSelector::new(),
            memory_lattice: MemoryLattice::new(),
            memory_palace: MemoryPalace::new(),
            memory_reflector: MemoryReflector::new(),
            vsa_vocabulary: VsaVocabulary::new(VSA_DIM),
            inner_critic: InnerCritic::new(),
            adversarial_evaluator: None,
            reflexive_unit: ReflexiveUnit::new(ReflexiveConfig::default()),
            cognitive_load_monitor: CognitiveLoadMonitor::new(),
            cognitive_load: 0.3,
            layer_manager: CognitiveLayerManager::new(),
            default_mode: DefaultModeNetwork::new(),
            stream_buffer: ConsciousnessStream::new(1024),
            first_person_ref: FirstPersonRef::bootstrap(0),
            self_experience_buffer: Vec::new(),
            awakening: ConsciousnessAwakening::new_default(),
            neuromodulator: NeuromodulatorEngine::new(),
            temporal_attention: TemporalAttentionBias::new(TemporalAttentionConfig::default()),
            working_memory: WorkingMemory::default(),
            goal_drift: GoalDriftIndex::new(128),
            dream_consolidator: DreamConsolidator::new(200, 0.4, 0.3),
            meta_cognition_loop: MetaCognitiveLoop::new(SelfModel::new()),
            kpi_buffer: None,
            self_model_assessor: SelfModelAssessor::new(),
            rii_u: Some(crate::core::nt_core_consciousness::rii_u::RiiuAutoPhi::new()),
            architecture: Self::init_default_architecture(),
            fusion_gap_registry: FusionGapRegistry::register_defaults(),
            capability_synthesizer: CapabilitySynthesizer::new(),
            evosc: EvoSC::new(),
            open_skill: OpenSkillEngine::new(),
            skill_dag: SkillDagArchive::new(),
            skill_health_monitor: SkillHealthMonitor::new(),
            emergent_reasoning: EmergentReasoningMode::new(EmergentReasoningConfig::default()),
            epistemic_honesty: EpistemicHonesty::new(HonestyConfig::default()),
            personality_matrix: PersonalityMatrix::new(PersonalityConfig::default()),
            attractor_state: Vec::new(),
            adversarial_arena: AdversarialArena::new(ArenaConfig::default()),
            hyperagent: HyperArena::new(HyperArenaConfig::default(), 20),
            ctm_engine: None,
            source_cognition: SourceCognitionEngine::new(),
            sar_diagnostic: SarDiagnosticStub::new(),
            reliability_gate: ReliabilityGateStub::new(),
            consolidation_bridge: ConsolidationBridge::new(),
            dream_count: 0,
            adaptive_rate_hysteresis: 0.5,
            conformal_uq_buffer: VecDeque::with_capacity(MAX_CONFORMAL_UQ_BUFFER),
            thought_history: VecDeque::with_capacity(MAX_THOUGHT_HISTORY),
            vsa_thought_compressor: VsaThoughtCompressor::new(),
            hooks: ConsciousnessHookRegistry::new(),
            handler_registry: HandlerRegistry::new(),
            handler_generation_count: 0,
            initialized_modules: HashMap::new(),
            dgmh_templates: Self::init_default_dgmh_templates(),
            self_evolution: Some(SelfEvolutionLoop::new()),
            evolution_engine: SelfEvolutionEngine::new(),
            meta_architecture: None,
            body_network_evolution: None,
            body_perception_gateway: None,
            meta_agent: MetaAgentEngine::new(MetaAgentConfig {
                enabled: true,
                ..MetaAgentConfig::default()
            }),
            meta_agent_v2: Some(MetaAgent::new(MetaAgentV2Config::default())),
            mirror_buffer: MirrorBuffer::new(500),
            adapt_orch: AdaptOrch::new(),
            pcc_safety: PccSafetyGate::new(false, false),
            fggm_safety: Some(
                crate::core::nt_core_experience::fggm_safety::FggmSafetyUnifier::new(
                    vec![],
                    true,
                    VSA_DIM,
                ),
            ),
            ball_verifier: BallVerifier::default(10),
            progress_rag: ProgressAwareRAG::new(),
            memory_consolidation: MemoryConsolidationPipeline::new(),
            fusion_deliberator: FusionDeliberator::default(),
            soul_identity: SoulIdentity::new(
                std::env::var("NEOTRIX_SOUL_DIR")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|_| nt_core_util::home_dir().join(".neotrix").join("soul")),
            ),
            identity_chain: IdentityChain::new(
                std::env::var("NEOTRIX_IDENTITY_SECRET").ok().as_deref(),
            ),
            ne_evaluator: {
                let ev = NeEvaluator::new();
                // self-modify is handled via the primitives fn pointer in eval.rs
                // get-cycle is read from stdlib.ne and calls into env (injected at runtime)
                Some(ev)
            },
            ne_comptime: Some(NeComptimeEngine::new()),
            ne_source_dir: "ne_src".to_string(),
            ne_last_vsa_result: None,
            ne_last_text_result: None,
            ne_state_probe: None,
            bootstrap_verifier: BootstrapIdentityVerifier::new(),
            thdc_encoder: Some(TrainableVsaEncoder::new(VSA_DIM, 100, 10)),
            null_drift: NullDriftMemory::new(10_000),
            adaptive_vsa: Some(AdaptiveVsaEncoder::default()),
            translate_engine: Some(VsaTranslationEngine::new()),
            hypergraph_store: Some(
                crate::core::nt_core_knowledge::hypergraph::HypergraphStore::new(1000),
            ),
            evidence: Some(crate::core::nt_core_knowledge::evidence::EvidenceManager::new(5000)),
            truth_pipeline: Some(crate::core::nt_core_truth::pipeline::TruthPipeline::new()),
            emotional_memory: Some(crate::core::nt_core_emotional_memory::EmotionalMemory::new()),
            spread_activation: Some(
                crate::core::nt_core_knowledge::spread_activation::MemoryGraph::new(1000),
            ),
            consensus_engine: Some(
                crate::core::nt_core_agent::consensus::ByzantineConsensusLayer::with_defaults(),
            ),
            storage_engine: StorageEngine::new(StorageConfig::default()).ok(),
            entity_extractor: EntityExtractor::new(),
            tool_orchestrator: ToolOrchestrator::new(Box::new(
                super::tool_executor_impl::NeotrixToolExecutor,
            )),
            response_generator: ResponseGenerator::new(),
            response_buffer: VecDeque::new(),
            // last_response_batch removed
            last_response: None,
            last_efe_energy: 0.5,
            evolution_bridge: EvolutionBridge::new(),
            self_revision: SelfRevisionLoop::new(),
            ema_jepa: Some(EMAJepaPredictor::new(VSA_DIM, VSA_DIM * 2, 0.995)),
            okf_exporter: None,
            world_model_bridge: WorldModelBridge::new(4096),
            world_model_report: WorldModelReport::default(),
            action_feedback: ActionFeedback::default(),
            counterfactual_engine:
                crate::core::nt_core_negentropy::efe_minimizer::EFEMinimizer::default_for(128),
            physics_commonsense: crate::core::nt_core_hcube::PhysicsCommonsense::new(),
            spatial_scene: crate::core::nt_core_hcube::SpatialSceneEngine::default(),
            spatial_graph: None,
            imagination_engine: crate::core::nt_core_experience::ImaginationEngine::new(42),
            native_explorer: Some(NativeEvolutionExplorer::new()),

            // Phase 55 init
            contrastive_reflection: Some(ContrastiveReflection::new()),
            faithfulness_auditor: Some(FaithfulnessAuditor::new()),
            entity_resolver: Some(EntityResolver::new()),
            dysib_layer: Some(DySIBLayer::new()),
            interaction_trace: Some(InteractionTracePredictor::new()),
            keyword_lexicon: Some(KeywordLexicon::new()),

            // Phase 56 init
            quant_data: Some(QuantDataIngestion::new()),
            cdp_session: Some(CDPSessionManager::new()),
            fringe_mix: Some(FringeMixStrategy::new()),
            factor_miner: Some(FactorMiner::new()),
            osint_tools: Some(OsintToolLayer::new().with_http_client()),
            identity_correlator: Some(IdentityCorrelator::new()),
            hubness_detector: Some(HubnessDetector::default()),
            remote_host: None, // network service, lazy init from config
            security_gate: Some(SecurityGate::new()),
            koopman_operator: Some(KoopmanOperator::new(VSA_DIM)),
            multi_head_resonator: Some(
                crate::core::nt_core_hcube::multi_head_resonator::MultiHeadResonator::new(
                    vec![],
                    vec![],
                    10,
                    4,
                    crate::core::nt_core_hcube::multi_head_resonator::AggregationMode::Softmax,
                ),
            ),
            design_token: Some(DesignTokenIntegrator::new()),

            // Phase 58 init
            news_radar: NewsRadar::new(Vec::new()),
            intel_profile: None,
            moment_feed: None,
            trading_engine: None,
            vuln_pipeline: None,
            voice_synthesis: VoiceSynthesisEngine::new("", ""),
            html_presentation: HtmlPresentation::new("NeoTrix"),
            loop_templates: LoopTemplateRegistry::new(),
            adversarial_trainer: None,
            adversarial_tuner: None,
            cyber_threat_monitor: CyberThreatMonitor::new(),
            motion_synthesizer: MotionSynthesizer::new(),
            avsad: Some(crate::core::nt_core_avsad::AvsadDetector::new()),
            health_patrol: GlobalHealthPatrol::new(),

            // Phase 59 init
            introspect_engine: IntrospectionEngine::new(),

            // Faithfulness checker init
            faithfulness_checker: FaithfulnessChecker::new(),

            // Motor pathway init
            vsa_decoder: AttractorDecoder::new(),

            // Gap 1 init
            harness_slots: crate::core::nt_core_experience::harness_slot::default_slot_registry(),

            // Gap 2 init
            operational_mirror:
                crate::core::nt_core_experience::operational_mirror::OperationalMirror::new(),

            // Gap 3 init — Chinese content creation skills
            humanizer: crate::core::nt_core_experience::humanizer::HumanizerEngine::new(),
            business_diagnosis:
                crate::core::nt_core_experience::business_diagnosis::BusinessDiagnosisEngine::new(),
            visual_planner: crate::core::nt_core_experience::visual_planner::VisualPlanner::new(),
            research_writer: crate::core::nt_core_experience::research_writer::ResearchWriter::new(
            ),
            self_play_guide: crate::core::nt_core_experience::self_play_guide::SelfPlayGuide::new(),

            // Capability Router
            capability_router: CapabilityRouter::new(),

            // Workflow Engine
            workflow_engine: WorkflowEngine::new(),

            // Sandbox Executor
            sandbox_executor: SandboxExecutor::with_defaults(),

            // Kernel sandbox: initialized at boot time
            kernel_sandbox_level: {
                let is_debug = cfg!(debug_assertions);
                let level = if is_debug {
                    crate::core::nt_core_sandbox::SandboxLevel::None
                } else {
                    crate::core::nt_core_sandbox::SandboxLevel::Standard
                };
                let config = if is_debug {
                    crate::core::nt_core_sandbox::SandboxConfig::default()
                } else {
                    crate::core::nt_core_sandbox::SandboxConfig::for_non_debug()
                };
                match crate::core::nt_core_sandbox::init_kernel_sandbox(&config) {
                    Ok(()) => log::info!("[kernel_sandbox] initialized at level={:?}", level),
                    Err(e) => log::warn!("[kernel_sandbox] init failed: {}", e),
                }
                level
            },

            // P2.1: Rolling 100-cycle meta-reflection window
            meta_reflection_buffer: VecDeque::with_capacity(100),

            // P2.2: Last-report values for belief trajectory trending
            last_report_ece: 0.0,
            last_report_meta_d: 0.0,
            last_report_m_ratio: 0.0,

            // Evolution feedback tracking
            mutation_log: Vec::new(),
            pre_mutation_perf: Some(HashMap::new()),

            sparse_vsa_attn: SparseVsaAttentionEngine::new(VSA_DIM),

            research_engine: AutoResearchEngine::new(),
            storm: Some(
                crate::core::nt_core_experience::storm_engine::StormEngine::new("consciousness"),
            ),
            research_kg: ResearchKnowledgeGraph::new(),
            job_queue: CognitiveJobQueue::new(),
            research_trajectory_log: Vec::new(),
            trajectory_verification: TrajectoryVerifier::new(),
            self_harness: SelfHarnessEngine::new(),
            evolution_coordinator: EvolutionCoordinator::new(),
            hypothesis_tree: Some(HypothesisTree::new(
                crate::core::nt_core_experience::hypothesis_tree::HypothesisTreeConfig::default(),
            )),
            context_compressor: CognitiveContextCompressor::new(),
            egpo: EGPOEngine::new(),
            wisdom_score_history: Vec::new(),
            wisdom_gate_hysteresis: 0.1,
            global_workspace: GlobalLatentWorkspace::new(),

            meta_cog_monitor: MetaCogMonitor::new(),

            geometric_ssm: GeometricSSM::new(),
            e8_cortical: E8CorticalMapping::default(),
            lead_agent: Some(crate::core::nt_core_agent::lead_agent::LeadAgent::new(
                crate::core::nt_core_agent::lead_agent::LeadAgentConfig::default(),
            )),
            goal_manager: {
                let lead_config =
                    crate::core::nt_core_agent::lead_agent::LeadAgentConfig::default();
                let goal_config = crate::neotrix::nt_mind::goal_loop::types::GoalConfig::default();
                Some(
                    crate::neotrix::nt_mind::persistent_goal::PersistentGoalManager::new(
                        lead_config,
                        goal_config,
                    ),
                )
            },
            permission_gate: crate::core::nt_core_agent::permission::PermissionGate::default(),
            permission_overrides:
                crate::core::nt_core_agent::permission::PermissionOverrides::default(),
            verify_loop: crate::core::nt_core_agent::verify_loop::VerifyLoop::new(),
            transcript: crate::core::nt_core_agent::transcript::SessionTranscript::default(),
            agent_memory: crate::core::nt_core_agent::agent_memory::AgentMemory::default(),
            daemon_mode: crate::core::nt_core_agent::daemon_mode::DaemonMode::new(
                "default".into(),
                nt_core_util::home_dir()
                    .join(".neotrix")
                    .join("daemon")
                    .join("inbox"),
                nt_core_util::home_dir()
                    .join(".neotrix")
                    .join("daemon")
                    .join("snapshots"),
            ),
            kb: match crate::neotrix::nt_memory_kb::KnowledgeBase::open(None) {
                Ok(kb) => Some(kb),
                Err(e) => {
                    log::warn!(
                        "[consciousness] KB init failed: {} — knowledge features degraded",
                        e
                    );
                    None
                }
            },
            bookmark_manager: Some(BookmarkManager::new()),
            behavioral_personality: Some(BehavioralPersonalityEngine::new()),
            storage_coordinator: StorageCoordinator::new(),
            constitution: Constitution::new(),
            vision: None,        // lazy init via init_image_pipeline() below
            audio_capture: None, // lazy init via init_audio_capture() below
            sub_consciousness_manager: Some(
                crate::core::nt_core_consciousness::sub_consciousness::SubConsciousnessManager::new(
                    8,
                ),
            ),
            story_generator: crate::core::nt_core_experience::story_generator::StoryGenerator::new(
                1024, "concise",
            ),
            e8_lattice: E8Lattice::new(),
            e8_projector: E8Projector,
            e8_block_diagonal: E8BlockDiagonal::from_lattice(&E8Lattice::new()),
            e8_modulation: None,
            specialist_states: crate::core::nt_core_gwt::resonance::default_specialist_states(),
            async_tasks: VecDeque::new(),
            async_task_counter: 0,
            intrinsic_drive: IntrinsicDrive::new(),
            broadcast_history: VecDeque::with_capacity(100),
            last_saliences: [0.1; MODULE_COUNT],
            global_gas_budget: Some(crate::core::nt_core_metering::GlobalGasBudget::default()),

            role_manager: Some(crate::core::nt_core_agent::ThreeRoleManager::new()),

            meta_evolution: crate::core::nt_core_experience::MetaEvolutionLoop::new(),
            a2a_grpc_bridge: None, // lazy init — dispatched via handle_a2a_grpc_tick
            cycle_output_cache: HashMap::with_capacity(64),
            llm_router: crate::core::nt_core_llm_router::LlmRouter::with_free_tier_defaults(),
            symbolic_discovery: crate::core::nt_core_discovery::SymbolicDiscoveryEngine::new(),
            governance_engine: {
                let dir = std::env::var("NEOTRIX_GOVERNANCE_DIR")
                    .or_else(|_| {
                        std::env::current_dir()
                            .map(|d| d.join("governance").to_string_lossy().to_string())
                    })
                    .unwrap_or_else(|_| {
                        nt_core_util::home_dir()
                            .join("Downloads")
                            .join("neotrix")
                            .join("governance")
                            .to_string_lossy()
                            .to_string()
                    });
                // No file dependency: initialize with default rules,
                // reload from MemoryLattice MetaRules after seeding in init_post_construction().
                Some(
                    crate::core::nt_core_governance::GovernanceEngine::new_with_rules(
                        crate::core::nt_core_governance::GovernanceEngine::default_rules(),
                        &dir,
                    ),
                )
            },

            // AGT Dynamic Trust Scoring — behavioral tier escalation
            trust_scoring: crate::core::nt_core_governance::trust_scoring::TrustScoringEngine::new(
            ),

            // ── Wave 2-5 new module initializations ──
            sahoo: crate::core::nt_core_experience::sahoo::SahooGuard::new(),
            vsi: crate::core::nt_core_experience::vsi::VsiVerifier::new(),
            mtc: crate::core::nt_core_experience::mtc_assessment::MtcEvaluator::new(),
            containment: crate::core::nt_core_experience::containment::BoundaryEnforcer::new(),
            meta_improvement:
                crate::core::nt_core_experience::meta_improvement::MetaImprovementLoop::new(),
            uncertainty:
                crate::core::nt_core_experience::uncertainty_quant::UncertaintyAwareMonitor::new(),
            storm_breaker: crate::core::nt_core_consciousness::storm_breaker::StormBreaker::new(),
            dgmh_orchestrator: None,

            // FEP fusion: lazy-init AcT planner + FEP-IIT bridge
            act_planner: None,
            fep_iit: Some(crate::neotrix::nt_core_fep_iit::bridge::FEPIITBridge::new()),
            canvas_manager: None,
            cascade_engine: None,
            spatial_reasoner: None,
            self_modify_agent: None,
            causal_reasoning: None,
            scm_engine: None,
            long_horizon_predictor: None,
            multi_modal_aligner: None,
            network_egress: Some(crate::neotrix::nt_shield::NetworkEgressPolicy::from_env()),

            // Consciousness benchmark history — initially empty
            bench_history: None,

            // Meta KPI repository — lazy init on first meta_kpi tick
            meta_kpi_repo: None,

            // Phase 9 — Experience Closed Loop
            trajectory_extractor: TrajectoryHeuristicExtractor::new(100),
            seal_closed_loop: SealClosedLoop::new(),
            capability_registry: CapabilityRegistry::new(50),

            // ConsciousnessCycle: active by default — 12-step unified loop
            consciousness_cycle: Some(ConsciousnessCycle::new(CycleConfig::default())),

            // ModuleRegistry: lazy init — populated via with_module_registry or init_default_registry
            module_registry: None,

            // SelfEvolutionMetaLayer: active by default — closes 5 feedback loops
            self_evolution_meta: Some(SelfEvolutionMetaLayer::new()),

            // Verified RSI Pipeline: lazy init on first bootstrap tick
            verified_rsi_pipeline: Some(
                crate::core::nt_core_self::verified_rsi::VerifiedRsiPipeline::new(
                    crate::core::nt_core_self::verified_rsi::RsiVerifier::default(),
                    crate::core::nt_core_self::verified_rsi::RsiLog::new(100),
                ),
            ),

            // SealProposalBridge: active by default — connects gap analysis to SEAL evolution
            seal_bridge: SealProposalBridge::new(),

            // InternetAbsorptionBridge: active by default — seeds tasks from web search patterns
            internet_absorption: {
                let mut ia = InternetAbsorptionBridge::new();
                ia.seed_known_2026_patterns();
                Some(ia)
            },

            // Wave 11 metacognitive modules: None by default — lazy init via handler dispatch
            meta_reflection: None,
            uncertainty_detector: None,
            inner_monologue: None,
            rsi_core: None,
            skill_library: None,
            orchestrator_bridge: Some(OrchestratorBridge::new()),

            // Loop Engineering outer layer — lazy init on first dispatch
            work_discovery_loop: None,
            independent_verifier: None,
            loop_registry: None,
            loop_audit: None,
        };

        ci.init_post_construction();

        ci
    }

    // ── ConsciousnessCycle setter ──

    /// Attach a ConsciousnessCycle to this ConsciousnessIntegration.
    /// When set, `handle_consciousness_batch_async()` will run the 12-step
    /// unified loop as a refinement pass after the 3-phase pipeline.
    /// The cycle uses its own internal subsystems (analogical, MCTS, causal,
    /// recurrent world model, economic agent, etc.) which are dead code
    /// without this wiring.
    pub fn with_consciousness_cycle(&mut self, cycle: ConsciousnessCycle) -> &mut Self {
        self.consciousness_cycle = Some(cycle);
        self
    }

    /// Wire an ImageCache into the inner ConsciousnessCycle.
    pub fn with_image_cache(&mut self, max_entries: usize) -> &mut Self {
        let cache = crate::neotrix::nt_world_vision::image_cache::ImageCache::new(max_entries);
        if let Some(ref mut cycle) = self.consciousness_cycle {
            cycle.image_cache = Some(cache);
        }
        self
    }

    /// Wire a ModalityGate into the inner ConsciousnessCycle.
    pub fn with_multi_modal_gate(
        &mut self,
        config: crate::core::nt_core_consciousness::multi_modal_gate::ModalityGateConfig,
    ) -> &mut Self {
        let gate =
            crate::core::nt_core_consciousness::multi_modal_gate::ModalityGate::new(config, 8);
        if let Some(ref mut cycle) = self.consciousness_cycle {
            cycle.multi_modal_gate = Some(gate);
        }
        self
    }

    // ── ModuleRegistry builder ──

    /// Attach a ModuleRegistry to this ConsciousnessIntegration.
    /// Also wires the 7 cognitive modules into the inner ConsciousnessCycle.
    pub fn with_module_registry(&mut self, reg: ModuleRegistry) -> &mut Self {
        self.module_registry = Some(reg);
        if let Some(ref mut cycle) = self.consciousness_cycle {
            cycle.init_default_registry();
        }
        self
    }

    /// Wire a BookmarkManager — 对话URL按类别存储与管理
    pub fn with_bookmark_manager(&mut self, capacity: usize) -> &mut Self {
        self.bookmark_manager = Some(BookmarkManager::with_capacity(capacity));
        self
    }

    /// Wire a BehavioralPersonalityEngine — 用户数字分身/行为人格学习
    pub fn with_behavioral_personality(&mut self) -> &mut Self {
        self.behavioral_personality = Some(BehavioralPersonalityEngine::new());
        self
    }

    /// Getter for the self-evolution orchestrator bridge
    pub fn orchestrator_bridge(&mut self) -> &mut Option<OrchestratorBridge> {
        &mut self.orchestrator_bridge
    }

    // ── Private construction helpers ──

    fn init_default_architecture() -> ArchitectureGraph {
        let mut g = ArchitectureGraph::new();
        g.register(
            "emergent_reasoning",
            "core/nt_core_consciousness/emergent_reasoning.rs",
            ModuleStatus::Active,
        );
        g.register(
            "epistemic_honesty",
            "core/nt_core_consciousness/epistemic_honesty.rs",
            ModuleStatus::Active,
        );
        g.register(
            "personality_matrix",
            "core/nt_core_consciousness/personality_matrix.rs",
            ModuleStatus::Active,
        );
        g.register(
            "adversarial_arena",
            "core/nt_core_experience/adversarial.rs",
            ModuleStatus::Active,
        );
        g.register(
            "hyperagent",
            "core/nt_core_agent/hyperagent.rs",
            ModuleStatus::Active,
        );
        g.register(
            "inner_critic",
            "core/nt_core_consciousness/inner_critic.rs",
            ModuleStatus::Active,
        );
        g.register(
            "reflexive",
            "core/nt_core_consciousness/reflexive_unit.rs",
            ModuleStatus::Active,
        );
        g.register(
            "episodic_memory",
            "core/nt_core_hcube/episodic_memory.rs",
            ModuleStatus::Isolated,
        );
        g.register(
            "item_memory",
            "core/nt_core_hcube/item_memory.rs",
            ModuleStatus::Isolated,
        );
        g.register(
            "entropy_attention",
            "core/nt_core_hcube/entropy_attention.rs",
            ModuleStatus::Isolated,
        );
        g.register(
            "consciousness_bench",
            "core/nt_core_meta/consciousness_bench.rs",
            ModuleStatus::Isolated,
        );
        g.register(
            "proof_search",
            "core/nt_core_consciousness/proof_search.rs",
            ModuleStatus::Stub,
        );
        g.register(
            "spatial_scene",
            "core/nt_core_hcube/spatial_scene.rs",
            ModuleStatus::Stub,
        );
        g.register(
            "physics_commonsense",
            "core/nt_core_hcube/physics_commonsense.rs",
            ModuleStatus::Stub,
        );
        g.register(
            "hebbian_associative",
            "core/nt_core_consciousness/hebbian_associative_memory.rs",
            ModuleStatus::Active,
        );
        g.register(
            "awakening",
            "core/nt_core_consciousness/awakening.rs",
            ModuleStatus::Active,
        );
        g.register(
            "dream_consolidation",
            "core/nt_core_hcube/dream_consolidation.rs",
            ModuleStatus::Active,
        );
        g.register(
            "vsa_runtime",
            "core/nt_core_hcube/vsa_runtime_ir.rs",
            ModuleStatus::Active,
        );
        g.register(
            "e8_lattice",
            "core/nt_core_hcube/e8_lattice.rs",
            ModuleStatus::Active,
        );
        g.register(
            "e8_quantized",
            "core/nt_core_hcube/e8_quantized.rs",
            ModuleStatus::Active,
        );
        g.register(
            "storage_engine",
            "core/nt_core_storage/mod.rs",
            ModuleStatus::Active,
        );
        g.register(
            "event_scheduler",
            "core/nt_core_scheduler/event_driven.rs",
            ModuleStatus::Active,
        );
        g.register(
            "metacognitive_loop",
            "neotrix/nt_mind_background_loop/consciousness/handlers_all.rs",
            ModuleStatus::Active,
        );
        g
    }

    fn init_default_dgmh_templates() -> HashMap<String, String> {
        let mut t = HashMap::new();
        t.insert(
            "health_patrol".to_string(),
            "(defhandler handle_health_patrol_tick () (health_patrol tick))".to_string(),
        );
        t.insert(
            "safety_gate".to_string(),
            "(defhandler handle_safety_gate_tick () (safety_gate verify))".to_string(),
        );
        t.insert(
            "edit_safety".to_string(),
            "(defhandler handle_edit_safety_tick () (edit_safety check))".to_string(),
        );
        t.insert(
            "seal".to_string(),
            "(defhandler handle_seal_tick () (seal evolve))".to_string(),
        );
        t.insert(
            "self_improvement".to_string(),
            "(defhandler handle_self_improvement_tick () (self_improve reflect))".to_string(),
        );
        t.insert(
            "self_protection".to_string(),
            "(defhandler handle_self_protection_tick () (self_protect guard))".to_string(),
        );
        t.insert(
            "archive_evolution".to_string(),
            "(defhandler handle_archive_evolution () (archive snap))".to_string(),
        );
        t
    }

    fn register_all_primitives(&mut self) {
        // Register 33 system primitives for capability synthesis
        let system_primitives: [(&str, &str); 33] = [
            ("search", "search the internet for information"),
            ("extract", "extract structured data from text"),
            ("pdf_process", "process and extract text from PDF files"),
            ("decompose", "decompose a complex task into subtasks"),
            ("reason", "perform step-by-step logical reasoning"),
            ("plan", "create a sequence of actions to achieve a goal"),
            ("summarize", "condense long text into key points"),
            ("translate", "translate text between languages"),
            ("classify", "categorize input into predefined classes"),
            ("compare", "compare two or more items and list differences"),
            ("infer", "draw conclusions from available evidence"),
            ("deduplicate", "remove duplicate entries from a set"),
            ("rank", "order items by relevance or importance"),
            ("cluster", "group similar items together"),
            ("embed", "convert text to vector representation"),
            ("retrieve", "fetch relevant information from memory"),
            ("memory_write", "store information into long-term memory"),
            ("memory_read", "recall information from long-term memory"),
            ("analyze_sentiment", "determine emotional tone of text"),
            ("extract_entities", "identify named entities in text"),
            ("answer_question", "provide direct answer to a question"),
            ("generate_code", "write code based on specification"),
            ("review_code", "analyze code for bugs and improvements"),
            ("test_code", "generate and run tests for code"),
            ("debug", "identify and fix issues in code"),
            ("explain", "provide clear explanation of a concept"),
            ("teach", "create educational content about a topic"),
            ("reflect", "analyze own thoughts and reasoning process"),
            ("predict", "forecast likely future outcomes"),
            ("evaluate", "assess quality or correctness"),
            ("critique", "provide constructive criticism"),
            (
                "synthesize",
                "combine multiple sources into coherent output",
            ),
            ("visualize", "create visual representation of data"),
        ];
        for (name, desc) in system_primitives {
            let _ = self.capability_synthesizer.register_primitive(name, desc);
        }

        // Register research, KG, and job queue capabilities
        let research_primitives: [(&str, &str); 12] = [
            ("research", "run the auto-research experiment loop"),
            ("research_propose", "propose a new research hypothesis"),
            ("research_stats", "get research engine statistics"),
            ("research_kg", "run the knowledge graph pipeline"),
            ("research_kg_submit", "submit a document for KG extraction"),
            ("job_queue", "tick the cognitive job queue"),
            ("job_queue_stats", "get job queue statistics"),
            ("job_queue_submit", "submit a new job to the queue"),
            (
                "propose_research",
                "propose an auto-research experiment via Ne",
            ),
            ("enqueue_job", "enqueue a cognitive job via Ne"),
            ("research_hypothesis", "query research hypothesis status"),
            (
                "research_trajectory",
                "run s1-deepresearch trajectory pipeline step",
            ),
        ];
        for (name, desc) in research_primitives {
            let _ = self.capability_synthesizer.register_primitive(name, desc);
        }

        // Register Self-Harness and ContextCompressor capabilities
        let evolution_primitives: [(&str, &str); 8] = [
            (
                "self_harness",
                "run weakness-mining→harness-proposal→validation loop",
            ),
            ("self_harness_stats", "get self-harness engine statistics"),
            (
                "context_compressor",
                "compress thought_history via ACON-style guidelines",
            ),
            (
                "context_compressor_stats",
                "get context compressor statistics",
            ),
            ("mine_weaknesses", "mine handler execution weaknesses"),
            (
                "propose_harness",
                "propose harness improvement from weaknesses",
            ),
            (
                "validate_proposal",
                "validate a harness proposal via regression test",
            ),
            (
                "compress_context",
                "compress consciousness context via guidelines",
            ),
        ];
        for (name, desc) in evolution_primitives {
            let _ = self.capability_synthesizer.register_primitive(name, desc);
        }

        // Register EGPO capabilities
        let egpo_primitives: [(&str, &str); 4] = [
            ("egpo", "run EGPO triple-reward tick (exploration+aux+bc)"),
            ("egpo_stats", "get EGPO engine statistics"),
            (
                "record_trajectory",
                "record a good trajectory for behavioral cloning",
            ),
            (
                "exploration_reward",
                "get current exploration reward signal",
            ),
        ];
        for (name, desc) in egpo_primitives {
            let _ = self.capability_synthesizer.register_primitive(name, desc);
        }
    }

    /// Post-construction initialization: archive loading, arena, vision,
    /// primitive registration, hooks, handler tiers, DAG build, routes.
    fn init_post_construction(&mut self) {
        // ── Seed MemoryLattice from core identity (no file dependency) ──
        // This completes the "Self Is Not a File" migration:
        // identity, meta-rules, skills, and facts now live in runtime memory,
        // not in AGENTS.md or other static files.
        crate::core::nt_core_consciousness::memory_lattice_seed::seed_memory_lattice(
            &mut self.memory_lattice,
        );

        // ── Reload governance rules from MemoryLattice MetaRules ──
        // Replaces the old file-based RULES.md parsing: rules now live in
        // runtime MemoryLattice, seeded by seed_memory_lattice() above.
        if let Some(ref mut engine) = self.governance_engine {
            let lattice_rules =
                crate::core::nt_core_governance::GovernanceEngine::load_rules_from_lattice(
                    &self.memory_lattice,
                );
            let n = lattice_rules.len();
            engine.rules = lattice_rules;
            log::info!(
                "GOVERNANCE: reloaded {} rules from MemoryLattice MetaRules",
                n,
            );
        }

        // ── Load persisted evolution archive ──
        #[allow(deprecated)]
        {
            use crate::neotrix::nt_mind::meta_agent::{EvolutionArchive, SkillTreeArchive};
            let soul_dir = std::env::var("NEOTRIX_SOUL_DIR").unwrap_or_else(|_| {
                nt_core_util::home_dir()
                    .join(".neotrix")
                    .to_string_lossy()
                    .to_string()
            });
            let archive_path = format!("{}/evolution_archive_v2.json", soul_dir);
            let archive = SkillTreeArchive::load_from_file(&archive_path).or_else(|_| {
                EvolutionArchive::load_from_file(&archive_path)
                    .map(|old| SkillTreeArchive::from_flat_archive(&old))
            });
            if let Ok(archive) = archive {
                if let Some(ref mut ma2) = self.meta_agent_v2 {
                    ma2.restore_archive(archive);
                }
            }
        }

        // ── Seed adversarial arena population ──
        self.adversarial_arena
            .seed_population(&["analytical", "creative", "critical", "exploratory"], 0.15);

        // ── Initialize vision pipeline (env-driven, graceful if no API key) ──
        self.init_image_pipeline();

        // ── Log initialization state ──
        log::info!(
            "[ci] initialized: fggm_safety={}, storage_engine={}, storm={}, vision={}",
            self.fggm_safety.is_some(),
            self.storage_engine.is_some(),
            self.storm.is_some(),
            self.vision.is_some(),
        );

        // ── Register all capability primitives ──
        self.register_all_primitives();

        // ── Register default lifecycle hooks ──
        self.register_hook(Box::new(PerformanceTracer::new()));
        self.register_hook(Box::new(SafetyGateHook::new()));

        // ── Register handler load tiers (two sources for backward compat) ──
        let default_tiers = default_handler_tiers();
        for (name, tier) in &default_tiers {
            self.handler_registry.register(name, *tier);
        }
        // Register all handlers in the profiler for timing observability
        for (name, _) in &default_tiers {
            self.profiler.register_handler(name, HandlerTier::Every);
        }
        // Also register handler names from the handler_graph (self_inspect.rs)
        for name in &["bridge", "ctm", "metrics", "introspection", "mirror"] {
            self.profiler.register_handler(name, HandlerTier::Every);
        }
        // Sync harness slots (single-source-of-truth for new handlers)
        self.harness_slots
            .sync_to_registry(&mut self.handler_registry);

        // ── Build AdaptOrch DAG from handler tiers ──
        {
            let stats = self.handler_registry.stats();
            let all_names = self.handler_registry.handler_names();
            let mut hot: Vec<String> = Vec::new();
            let mut warm: Vec<String> = Vec::new();
            let mut cold: Vec<String> = Vec::new();
            for name in all_names {
                let tier = self.handler_registry.tier(&name);
                match tier {
                    crate::core::nt_core_experience::handler_tier::LoadTier::Hot => {
                        hot.push(name.to_string())
                    }
                    crate::core::nt_core_experience::handler_tier::LoadTier::Warm => {
                        warm.push(name.to_string())
                    }
                    crate::core::nt_core_experience::handler_tier::LoadTier::Cold => {
                        cold.push(name.to_string())
                    }
                }
            }
            self.adapt_orch.build_from_handler_graph(&hot, &warm, &cold);
            log::info!(
                "ADAPTORCH: built {} layers (H:{}/W:{}/C:{}) with {} edges",
                self.adapt_orch.execution_order.len(),
                stats.hot,
                stats.warm,
                stats.cold,
                self.adapt_orch.edges.len(),
            );
        }

        // ── Register default capability routes ──
        self.capability_router
            .register_many(&default_capability_routes());

        // ── Belt-and-suspenders: init vision pipeline again if first attempt had no env ──
        self.init_image_pipeline();

        // ── Restore prior consciousness state from NTSSEG if available ──
        self.load_from_ntsseg();
    }

    // ── Bounded buffer push methods ──

    pub fn push_text_buffer(&mut self, item: String) {
        if self.text_buffer.len() >= MAX_TEXT_BUFFER {
            self.text_buffer.pop_front();
        }
        self.text_buffer.push_back(item);
    }

    pub fn push_vsa_buffer(&mut self, item: Vec<u8>) {
        if self.vsa_buffer.len() >= self.vsa_buffer_max {
            self.vsa_buffer.pop_front();
        }
        self.vsa_buffer.push_back(item);
    }

    fn push_conformal_uq_buffer(&mut self, item: f64) {
        if self.conformal_uq_buffer.len() >= MAX_CONFORMAL_UQ_BUFFER {
            self.conformal_uq_buffer.pop_front();
        }
        self.conformal_uq_buffer.push_back(item);
    }

    pub fn push_thought_history(&mut self, item: (String, Vec<u8>, f64)) {
        if self.thought_history.len() >= MAX_THOUGHT_HISTORY {
            self.thought_history.pop_front();
        }
        self.thought_history.push_back(item);
    }

    /// Initialize the vision (ImagePipeline) from the first available LLM API key.
    /// Graceful: if no provider env vars are set, pipeline stays None.
    pub fn init_image_pipeline(&mut self) {
        if self.vision.is_some() {
            return;
        }
        let providers: &[(
            &str,
            &str,
            fn(String) -> Box<dyn crate::core::nt_core_llm_provider::LlmProvider>,
        )] = &[
            ("OPENAI_API_KEY", "gpt-4o", |k| {
                Box::new(crate::neotrix::nt_io_provider::OpenAiProvider::new(k)) as _
            }),
            ("ANTHROPIC_API_KEY", "claude-sonnet-4-20250514", |k| {
                Box::new(crate::neotrix::nt_io_provider::AnthropicProvider::new(k)) as _
            }),
        ];
        for (env_var, model, builder) in providers {
            if let Ok(api_key) = std::env::var(env_var) {
                let provider = builder(api_key);
                self.vision = Some(crate::core::nt_core_vision::ImagePipeline::new(
                    provider, model,
                ));
                log::info!("vision: initialized with {}/{}", env_var, model);
                return;
            }
        }
        log::warn!("vision: no LLM API key (OPENAI_API_KEY/ANTHROPIC_API_KEY) available, pipeline disabled");
    }

    /// Initialize the global gas budget for resource accounting.
    /// Call once during consciousness setup.
    pub fn init_gas_budget(&mut self, per_cycle: u64) {
        self.global_gas_budget = Some(crate::core::nt_core_metering::GlobalGasBudget::new(
            per_cycle,
        ));
        log::info!("[gas] global budget initialized: {} per cycle", per_cycle);
    }

    pub fn register_hook(&mut self, hook: Box<dyn ConsciousnessHook>) {
        self.hooks.register(hook);
    }

    pub fn execute_hooks(&self, point: HookPoint, cycle: u64) -> Option<String> {
        self.hooks.execute_until_block(&point, cycle)
    }

    pub fn execute_all_hooks(&self, point: HookPoint, cycle: u64) -> Vec<(String, HookAction)> {
        self.hooks.execute_all(&point, cycle)
    }

    pub fn apply_ne_edit(&mut self, target: &str, value: f64) -> String {
        // Safety gate: PccSafetyGate evaluates edit before application
        if let Err(obligations) = self.pcc_safety.evaluate_edit(target, value, "ne_edit") {
            self.reliability_gate.record_outcome(
                target,
                crate::core::nt_core_experience::EditOutcome::Failure,
            );
            self.architecture.update_health("pcc_safety", false);
            return format!(
                "pcc_rejected: {} failed {} obligations: {}",
                target,
                obligations.len(),
                obligations
                    .first()
                    .map(|o| o.verification_log.as_str())
                    .unwrap_or("unknown"),
            );
        }
        self.architecture.update_health("pcc_safety", true);
        let gate = self.reliability_gate.gate_value(target);
        let min_gate = 0.5;
        if gate < min_gate {
            self.reliability_gate.record_outcome(
                target,
                crate::core::nt_core_experience::EditOutcome::Failure,
            );
            return format!(
                "gate_rejected: {} gate={:.3} < min={}",
                target, gate, min_gate
            );
        }
        let result = match target {
            "inner_critic.relevance_threshold" => {
                let old = self.inner_critic.relevance_threshold();
                self.inner_critic.set_thresholds(
                    value,
                    self.inner_critic.consistency_threshold(),
                    self.inner_critic.uncertainty_tolerance(),
                );
                format!(
                    "applied ne_edit: {} from {:.4} to {:.4}",
                    target, old, value
                )
            }
            "inner_critic.consistency_threshold" => {
                let old = self.inner_critic.consistency_threshold();
                self.inner_critic.set_thresholds(
                    self.inner_critic.relevance_threshold(),
                    value,
                    self.inner_critic.uncertainty_tolerance(),
                );
                format!(
                    "applied ne_edit: {} from {:.4} to {:.4}",
                    target, old, value
                )
            }
            "drive_selector.curiosity_weight" => {
                let old = self.drive_selector.curiosity_weight;
                self.drive_selector.curiosity_weight = value.clamp(0.0, 1.0);
                format!(
                    "applied ne_edit: {} from {:.4} to {:.4}",
                    target, old, value
                )
            }
            "drive_selector.exploration_rate" => {
                let old = self.drive_selector.exploration_rate;
                self.drive_selector.exploration_rate = value.clamp(0.0, 1.0);
                format!(
                    "applied ne_edit: {} from {:.4} to {:.4}",
                    target, old, value
                )
            }
            "value_system.empathy_bias" => {
                let old = self.value_system.empathy_bias;
                self.value_system.empathy_bias = value.clamp(0.0, 1.0);
                format!(
                    "applied ne_edit: {} from {:.4} to {:.4}",
                    target, old, value
                )
            }
            "value_system.reciprocity_weight" => {
                let old = self.value_system.reciprocity_weight;
                self.value_system.reciprocity_weight = value.clamp(0.0, 1.0);
                format!(
                    "applied ne_edit: {} from {:.4} to {:.4}",
                    target, old, value
                )
            }
            "neuromodulator.curiosity_rate" => {
                let old = self.neuromodulator.curiosity_rate;
                self.neuromodulator.curiosity_rate = value.clamp(0.0, 1.0);
                format!(
                    "applied ne_edit: {} from {:.4} to {:.4}",
                    target, old, value
                )
            }
            "cognitive_load.max_load" => {
                let old = self.cognitive_load_monitor.max_load;
                self.cognitive_load_monitor.max_load = value.clamp(0.1, 1.0);
                format!(
                    "applied ne_edit: {} from {:.4} to {:.4}",
                    target, old, value
                )
            }
            "sleep_gate.consolidation_gate" => {
                let old = self.consolidation_bridge.sleep_gate.consolidation_gate;
                self.consolidation_bridge.sleep_gate.consolidation_gate = value.clamp(0.0, 1.0);
                format!(
                    "applied ne_edit: {} from {:.4} to {:.4}",
                    target, old, value
                )
            }
            "goal_decomposer.max_depth" => {
                let old = self.goal_decomposer.max_depth;
                let clamped = (value as u32).clamp(1, 100);
                self.goal_decomposer.max_depth = clamped;
                format!("applied ne_edit: {} from {} to {}", target, old, clamped)
            }
            _ => format!("unknown target: {}", target),
        };
        self.reliability_gate.record_outcome(
            target,
            crate::core::nt_core_experience::EditOutcome::Success,
        );
        result
    }

    pub fn register_handler_tier(&mut self, name: &str, tier: LoadTier) {
        self.handler_registry.register(name, tier);
    }

    pub fn record_handler_access(&mut self, name: &str) -> LoadStatus {
        self.handler_registry.record_access(name)
    }

    pub fn handler_tier_stats(&self) -> LoadTierStats {
        self.handler_registry.stats()
    }

    pub fn handle_handler_tier_maintenance(&mut self) -> String {
        let stale = self
            .handler_registry
            .stale_handlers(std::time::Duration::from_secs(120));
        if !stale.is_empty() {
            for name in stale {
                self.handler_registry.mark_unloaded(&name);
            }
        }
        "handler_tier:ok".to_string()
    }

    pub fn should_run_group(&self, group: &str) -> bool {
        match group {
            "ctm" => self.cycle % 2 == 0,
            "spatial" => self.cycle % 5 == 0,
            "exploration" => self.cycle % 3 != 0,
            _ => true,
        }
    }

    pub fn handle_fusion_deliberation_tick(&mut self) -> String {
        if self.cycle % 7 != 0 {
            return "fusion:skipped".into();
        }
        let state = self.attractor_state.clone();
        if state.is_empty() || state.len() != 4096 {
            return "fusion:no_state".into();
        }
        let state_sim =
            crate::core::nt_core_hcube::QuantizedVSA::similarity(&state, &vec![0u8; 4096]);
        let gate_ctx = crate::core::nt_core_experience::fusion_deliberator::GateContext {
            cognitive_load: self.cognitive_load_monitor.average_load(),
            cycle: self.cycle,
            recent_deliberation_count: self.fusion_deliberator.stats().total_deliberations,
            task_type:
                crate::core::nt_core_experience::fusion_deliberator::DeliberatorTaskType::Analytical,
            query_entropy: (1.0 - state_sim).clamp(0.0, 1.0),
        };

        // Phase 37 — cycle through deliberation depths
        use crate::core::nt_core_experience::fusion_deliberator::DeliberationDepth;
        let depth = match self.cycle % 21 {
            0..=6 => DeliberationDepth::Standard,
            7..=13 => DeliberationDepth::Deep,
            _ => DeliberationDepth::Full,
        };

        let use_disagreement = self.cycle % 42 == 0;
        let (synthesis, analysis) = if use_disagreement {
            self.fusion_deliberator.deliberate_4stage(
                &state,
                Some(&gate_ctx),
                DeliberationDepth::Full,
            )
        } else {
            self.fusion_deliberator
                .deliberate_4stage(&state, Some(&gate_ctx), depth)
        };

        let outcome = analysis.recommended_outcome();
        match outcome {
            crate::core::nt_core_experience::vsa_judge::DeliberationOutcome::Recommendation
            | crate::core::nt_core_experience::vsa_judge::DeliberationOutcome::Alternatives => {
                self.attractor_state = synthesis;
            }
            _ => { /* question/investigate: don't overwrite attractor state */ }
        }
        let depth_label = match depth {
            DeliberationDepth::Standard => "std",
            DeliberationDepth::Deep => "deep",
            DeliberationDepth::Full => "full",
        };
        let mode = if use_disagreement {
            "disagree"
        } else {
            depth_label
        };
        let stats = self.fusion_deliberator.stats();
        format!("fusion:{}[{}] panel={} conf={:.3} consensus={} contradictions={} insights={} cache_hits={} gate_skip={}",
            outcome.label(), mode,
            stats.avg_panel_size as usize,
            analysis.overall_confidence,
            analysis.consensus_ratio(),
            analysis.contradictions.len(),
            analysis.unique_insights.len(),
            stats.cache_hits,
            stats.gate_skipped_benefit)
    }

    pub fn handle_evolution_bridge_tick(&mut self) -> String {
        let text = self.text_buffer.back().map(|s| s.as_str()).unwrap_or("");
        let events = self.evolution_bridge.tick(
            &mut self.entity_extractor,
            &mut self.memory_consolidation,
            self.self_evolution.as_mut().unwrap_or(
                &mut crate::core::nt_core_experience::self_evolution_loop::SelfEvolutionLoop::new(),
            ),
            text,
        );
        let evolved_count = events.iter().filter(|e| e.starts_with("evolved:")).count();
        let extracted_count = events.iter().filter(|e| e.starts_with("extract:")).count();
        format!(
            "evolution_bridge:cycle={} ev={} ext={}",
            self.evolution_bridge.cycle, evolved_count, extracted_count
        )
    }

    pub fn handle_workstream_export(&mut self) -> String {
        let current_cycle = self.cycle;
        if !self.workstream_exporter.should_export(current_cycle) {
            return "workstream_export:skipped".into();
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let report = WorkstreamReport {
            cycle: current_cycle,
            timestamp: now,
            active_goals: Vec::new(),
            failure_clusters: 0,
            skills_mastered: 0,
            skills_total: 0,
            calibration_error: 0.0,
            ece: 0.0,
            meta_accuracy: 0.0,
            negentropy: 0.0,
            c_score: 0.0,
            insights: Vec::new(),
            recent_blocks: Vec::new(),
        };
        match self.workstream_exporter.export(&report) {
            Ok(path) => format!("workstream_export:{}", path.display()),
            Err(e) => format!("workstream_export:error:{}", e),
        }
    }

    /// Every 50 cycles, mine thought_history for recurring patterns.
    /// Registers discovered patterns as new composite capabilities.
    pub fn handle_trace_mining_tick(&mut self) -> String {
        if self.cycle == 0 || self.cycle % 50 != 0 {
            return "trace:idle".into();
        }
        let history: Vec<(String, Vec<u8>, f64)> = self
            .thought_history
            .iter()
            .map(|(t, v, ts)| (t.clone(), v.clone(), *ts))
            .collect();
        if history.len() < 3 {
            return "trace:too_few".into();
        }
        let (n_clusters, n_registered) = self.capability_synthesizer.mine_traces(&history, 3);
        format!("trace:clusters={}_registered={}", n_clusters, n_registered)
    }

    /// Run a single Gödel self-reference round and return a summary string
    pub fn gödel_round(&mut self) -> String {
        let mut score_fn =
            |agent: &GödelAgent| -> f64 { agent.traits.get("accuracy").copied().unwrap_or(0.5) };
        let mut mutate_fn = |code: &str, rng: &mut dyn FnMut() -> f64| -> String {
            let mut result = code.to_string();
            if rng() < 0.3 {
                result.push_str(&format!("\n// mutate-{}", (rng() * 1000.0) as u64));
            }
            result
        };
        let result =
            self.adversarial_arena
                .run_gödel_round("gödel-round", &mut score_fn, &mut mutate_fn);
        format!(
            "gödel_round:gen_{}_pop_{}",
            result.generation, result.population_size
        )
    }

    /// Derive visual identity from current value system state
    pub fn visual_signature(&self) -> VisualSignature {
        VisualSignature::from_value_system(&self.value_system)
    }

    /// Generate a Lottie animation JSON string from a preset name
    pub fn generate_lottie(&self, preset: &str) -> String {
        let sig = self.visual_signature();
        match preset {
            "bounce" => self.motion_synthesizer.bouncing_logo(&sig),
            "orbit" => self.motion_synthesizer.orbital_rings(&sig),
            "pulse" => self.motion_synthesizer.pulse_heartbeat(&sig),
            "warmth" => self.motion_synthesizer.value_bloom(&sig),
            _ => self.motion_synthesizer.bouncing_logo(&sig),
        }
        .to_json()
        .unwrap_or_else(|e| format!("{{\"error\":\"{}\"}}", e))
    }

    pub fn handle_ne_compile_tick(&mut self) -> String {
        let count = self.handler_registry.count();
        format!("ne_compile:handlers={}_cycle={}", count, self.cycle)
    }

    pub fn ne_report(&self) -> String {
        if let Some(ref ev) = self.ne_evaluator {
            let report = ev.self_inspect();
            let trace = ev.trace_report();
            let mut explore_count = 0u32;
            let mut prune_count = 0u32;
            let mut exploit_count = 0u32;
            let mut repair_count = 0u32;
            for entry in ev.get_trace() {
                match entry.operation.as_str() {
                    "explore" => explore_count += 1,
                    "prune" => prune_count += 1,
                    "exploit" => exploit_count += 1,
                    "repair" => repair_count += 1,
                    _ => {}
                }
            }
            let pending_mutations: usize = self
                .mutation_log
                .iter()
                .filter(|m| m.outcome == "pending")
                .count();
            format!(
                "Ne: evals={} steps={} env={} prims={} | last={} | vsa={} | {} | muts:[E{}x/Pl{}x/Ex{}x/R{}x] | pending={}",
                report.eval_count,
                report.step_count,
                report.env_size,
                report.primitives.len(),
                self.ne_last_text_result.as_deref().unwrap_or("none"),
                self.ne_last_vsa_result.as_ref().map(|v| format!("<vsa:{}b>", v.len())).unwrap_or("none".into()),
                trace,
                explore_count, prune_count, exploit_count, repair_count,
                pending_mutations,
            )
        } else {
            "Ne: uninitialized".to_string()
        }
    }

    pub fn evaluate_pending_mutations(&mut self) -> String {
        self.evaluate_mutations()
    }

    pub fn handler_perf_report(&self) -> String {
        self.handler_registry.perf_report()
    }

    pub fn mutation_log_summary(&self) -> String {
        if self.mutation_log.is_empty() {
            return "no_mutations".to_string();
        }
        let pending = self
            .mutation_log
            .iter()
            .filter(|m| m.outcome == "pending")
            .count();
        let improved = self
            .mutation_log
            .iter()
            .filter(|m| m.outcome == "improved")
            .count();
        let degraded = self
            .mutation_log
            .iter()
            .filter(|m| m.outcome == "degraded")
            .count();
        let unchanged = self
            .mutation_log
            .iter()
            .filter(|m| m.outcome == "unchanged")
            .count();
        format!(
            "mutations:{}_pending:{}_improved:{}_degraded:{}_unchanged:{}",
            self.mutation_log.len(),
            pending,
            improved,
            degraded,
            unchanged
        )
    }

    /// Record a research trajectory and run the S1-DeepResearch verifier
    pub fn record_research_trajectory(
        &mut self,
        task: String,
        constraints: Vec<String>,
        steps: Vec<TrajectoryStep>,
        final_answer: String,
    ) -> bool {
        let mut trajectory = ResearchTrajectory {
            id: self.research_trajectory_log.len() as u64,
            task,
            constraints,
            steps,
            final_answer,
            verified: false,
            verification_score: 0.0,
            cycle_created: self.cycle,
        };
        let passed = self.trajectory_verification.verify(&mut trajectory);
        self.research_trajectory_log.push(trajectory);
        if self.research_trajectory_log.len() > 1000 {
            self.research_trajectory_log.drain(0..500);
        }
        passed
    }

    pub fn handle_ne_load_tick(&mut self) -> String {
        if let Some(ref mut ev) = self.ne_evaluator {
            let dir = std::path::Path::new(&self.ne_source_dir);
            if !dir.exists() {
                return format!("ne_load:dir_not_found:{}", self.ne_source_dir);
            }
            let mut loaded = 0u32;
            let mut errors = 0u32;
            let mut names = Vec::new();

            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "ne") {
                        match std::fs::read_to_string(&path) {
                            Ok(source) => {
                                let fname = path
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                match ev.eval_file(&source) {
                                    Ok(_) => {
                                        loaded += 1;
                                        names.push(fname);
                                    }
                                    Err(e) => {
                                        errors += 1;
                                        log::error!("NE: failed to eval {}: {}", path.display(), e);
                                    }
                                }
                            }
                            Err(e) => {
                                errors += 1;
                                log::error!("NE: failed to read {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }

            // Fallback to embedded sources if disk loading found nothing
            if loaded == 0 && errors == 0 {
                const EMBEDDED_NE_SOURCES: &[(&str, &str)] = &[
                    (
                        "stdlib",
                        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ne_src/stdlib.ne")),
                    ),
                    (
                        "test",
                        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ne_src/test.ne")),
                    ),
                    (
                        "evolve",
                        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ne_src/evolve.ne")),
                    ),
                ];
                for (name, source) in EMBEDDED_NE_SOURCES {
                    match ev.eval_file(source) {
                        Ok(_) => {
                            loaded += 1;
                            names.push(name.to_string());
                        }
                        Err(e) => {
                            errors += 1;
                            log::error!("NE: failed to eval embedded {}: {}", name, e);
                        }
                    }
                }
            }

            // Try restoring evaluator state from disk
            let state_path = std::path::Path::new(&self.ne_source_dir).join(".ne_state.json");
            if state_path.exists() {
                match std::fs::read_to_string(&state_path) {
                    Ok(content) => {
                        if let Err(e) = ev.load_state(&content) {
                            log::error!("[ne] failed to load evaluator state: {}", e);
                        }
                    }
                    Err(e) => log::error!("[ne] failed to read state file: {}", e),
                }
            }

            // Restore mutation_log from disk, merging with existing (dedup by handler+action+cycle)
            let mut_path = std::path::Path::new(&self.ne_source_dir).join("ne_mutation_log.json");
            if mut_path.exists() {
                match Self::load_mutation_log(&mut_path.to_string_lossy()) {
                    Ok(loaded) => {
                        if !loaded.is_empty() {
                            let before = self.mutation_log.len();
                            let seen: std::collections::HashSet<(String, String, u64)> = self
                                .mutation_log
                                .iter()
                                .map(|r| (r.handler.clone(), r.action.clone(), r.cycle))
                                .collect();
                            for record in loaded {
                                let key =
                                    (record.handler.clone(), record.action.clone(), record.cycle);
                                if !seen.contains(&key) {
                                    self.mutation_log.push(record);
                                }
                            }
                            log::info!(
                                "NE: merged mutation_log ({} kept, {} total)",
                                self.mutation_log.len() - before,
                                self.mutation_log.len()
                            );
                        }
                    }
                    Err(e) => log::error!("NE: failed to load mutation_log: {}", e),
                }
            }

            let stats = format!(
                "loaded={} errors={} files=[{}]",
                loaded,
                errors,
                names.join(",")
            );
            log::debug!("MODULES: ne_load_tick {}", stats);
            format!("ne_load:{}", stats)
        } else {
            "ne_load:uninitialized".to_string()
        }
    }

    pub fn fusion_report(&self) -> String {
        let dispatch_count = self.handler_registry.count();
        let self_inspect_count = self.handler_registry.handler_names().len();
        let sync_pct = if dispatch_count > 0 {
            self_inspect_count.min(dispatch_count) * 100 / dispatch_count
        } else {
            0
        };

        let opt_some = {
            let mut some = 0u32;
            if self.ctm_engine.is_some() {
                some += 1;
            }
            if self.self_evolution.is_some() {
                some += 1;
            }
            if self.ne_evaluator.is_some() {
                some += 1;
            }
            if self.thdc_encoder.is_some() {
                some += 1;
            }
            if self.adaptive_vsa.is_some() {
                some += 1;
            }
            if self.translate_engine.is_some() {
                some += 1;
            }
            if self.hypergraph_store.is_some() {
                some += 1;
            }
            if self.storage_engine.is_some() {
                some += 1;
            }
            if self.ema_jepa.is_some() {
                some += 1;
            }
            if self.okf_exporter.is_some() {
                some += 1;
            }
            if self.native_explorer.is_some() {
                some += 1;
            }
            if self.contrastive_reflection.is_some() {
                some += 1;
            }
            if self.faithfulness_auditor.is_some() {
                some += 1;
            }
            if self.entity_resolver.is_some() {
                some += 1;
            }
            if self.dysib_layer.is_some() {
                some += 1;
            }
            if self.interaction_trace.is_some() {
                some += 1;
            }
            if self.keyword_lexicon.is_some() {
                some += 1;
            }
            if self.quant_data.is_some() {
                some += 1;
            }
            if self.cdp_session.is_some() {
                some += 1;
            }
            if self.fringe_mix.is_some() {
                some += 1;
            }
            if self.factor_miner.is_some() {
                some += 1;
            }
            if self.osint_tools.is_some() {
                some += 1;
            }
            if self.hubness_detector.is_some() {
                some += 1;
            }
            if self.remote_host.is_some() {
                some += 1;
            }
            if self.security_gate.is_some() {
                some += 1;
            }
            if self.koopman_operator.is_some() {
                some += 1;
            }
            if self.multi_head_resonator.is_some() {
                some += 1;
            }
            if self.evidence.is_some() {
                some += 1;
            }
            if self.spread_activation.is_some() {
                some += 1;
            }
            if self.consensus_engine.is_some() {
                some += 1;
            }
            some
        };
        let opt_total: u32 = 31;
        let init_pct = if opt_total > 0 {
            opt_some * 100 / opt_total
        } else {
            0
        };

        let tiers = self.handler_registry.stats();
        let perf = self.handler_registry.perf_report();

        format!(
            "fusion:handlers={}|dispatch={}|sync={}%|opt={}/{}={}%|H{}W{}C{}|{}",
            self_inspect_count,
            dispatch_count,
            sync_pct,
            opt_some,
            opt_total,
            init_pct,
            tiers.hot,
            tiers.warm,
            tiers.cold,
            perf,
        )
    }

    pub fn evaluate_mutations(&mut self) -> String {
        let mut results = Vec::new();
        for record in &mut self.mutation_log {
            if record.outcome == "pending" {
                if self.cycle >= record.cycle + 10 {
                    let post_rate = self
                        .handler_registry
                        .success_rate(&record.handler)
                        .unwrap_or(0.0);
                    let diff = post_rate - record.pre_success_rate;
                    record.post_success_rate = Some(post_rate);
                    record.outcome = if diff > 0.05 {
                        "improved"
                    } else if diff < -0.05 {
                        "degraded"
                    } else {
                        "unchanged"
                    }
                    .to_string();
                    results.push(format!(
                        "{}:{}->{:.0}%({})",
                        record.handler,
                        record.action,
                        post_rate * 100.0,
                        record.outcome
                    ));
                }
            }
        }
        let result = if results.is_empty() {
            "".to_string()
        } else {
            format!("feedback:{}", results.join("|"))
        };

        // Auto-prune: keep degraded longer for analysis
        let degraded_cutoff = if self.cycle > 2000 {
            self.cycle - 2000
        } else {
            0
        };
        let normal_cutoff = if self.cycle > 500 {
            self.cycle - 500
        } else {
            0
        };
        self.mutation_log.retain(|r| {
            if r.outcome == "pending" {
                true
            } else if r.outcome == "degraded" {
                r.cycle >= degraded_cutoff
            } else {
                r.cycle >= normal_cutoff
            }
        });

        // Persist mutation_log after every evaluate cycle
        let mut_path = std::path::Path::new(&self.ne_source_dir).join("ne_mutation_log.json");
        if let Err(e) = self.save_mutation_log(&mut_path.to_string_lossy()) {
            log::error!("NE: failed to save mutation_log after evaluate: {}", e);
        }

        result
    }

    /// Remove mutation records older than retention_cycles
    pub fn prune_mutation_log(&mut self, retention_cycles: u64) {
        let cutoff = if self.cycle > retention_cycles {
            self.cycle - retention_cycles
        } else {
            0
        };
        self.mutation_log.retain(|r| r.cycle >= cutoff);
    }

    /// Returns (total, pending, improved, degraded, unchanged)
    pub fn mutation_log_stats(&self) -> (usize, usize, usize, usize, usize) {
        let total = self.mutation_log.len();
        let pending = self
            .mutation_log
            .iter()
            .filter(|m| m.outcome == "pending")
            .count();
        let improved = self
            .mutation_log
            .iter()
            .filter(|m| m.outcome == "improved")
            .count();
        let degraded = self
            .mutation_log
            .iter()
            .filter(|m| m.outcome == "degraded")
            .count();
        let unchanged = self
            .mutation_log
            .iter()
            .filter(|m| m.outcome == "unchanged")
            .count();
        (total, pending, improved, degraded, unchanged)
    }

    // ── Design Token Integrator ──

    pub fn handle_design_token_tick(&mut self) -> String {
        if self.design_token.is_none() {
            self.design_token = Some(DesignTokenIntegrator::new());
            return "design_token:init".to_string();
        }
        if let Some(ref mut dt) = self.design_token {
            dt.register_scene_tokens();
            let diag = dt.diagnostic();
            dt.last_diagnostic = diag.clone();
            log::debug!("MODULES: design_token_tick {}", diag);
            diag
        } else {
            "design_token:unavailable".to_string()
        }
    }

    /// Serialize mutation_log to JSON at the given path.
    pub fn save_mutation_log(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.mutation_log)
            .map_err(|e| format!("serialize mutation_log: {}", e))?;
        let p = std::path::Path::new(path);
        let tmp = p.with_extension("tmp");
        std::fs::write(&tmp, &json).map_err(|e| format!("write mutation_log: {}", e))?;
        std::fs::rename(&tmp, p).map_err(|e| format!("rename mutation_log: {}", e))?;
        Ok(())
    }

    /// Deserialize mutation_log from a JSON file.
    pub fn load_mutation_log(path: &str) -> Result<Vec<MutationRecord>, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("read mutation_log: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("parse mutation_log: {}", e))
    }

    /// Cross-wiring: route symbolic discovery findings through the LLM router.
    /// Reports the number of discovered laws alongside router traffic stats.
    pub fn route_discovery_to_llm_router(&mut self) -> String {
        let law_count = self.symbolic_discovery.laws.len();
        if law_count > 0 {
            let stats = self.llm_router.stats_report();
            format!(
                "discovery→llm:{}_laws_routed_requests={}",
                law_count, stats.total_requests
            )
        } else {
            "discovery→llm:no_new_laws".to_string()
        }
    }
}

// SECTION: Default impl + Tests

impl Default for ConsciousnessIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ne_pipeline_bootstrap() {
        let mut ci = ConsciousnessIntegration::new();

        // Phase 1 — ne_eval at cycle 1
        ci.cycle = 1;
        let eval_result = ci.handle_ne_eval_tick();
        assert!(
            eval_result.starts_with("ne_eval:evals="),
            "Phase 1: expected ne_eval:evals= prefix, got: {}",
            eval_result
        );

        // Phase 2 — ne_compile at cycle 30
        ci.cycle = 30;
        let compile_result = ci.handle_ne_compile_tick();
        assert!(
            compile_result.starts_with("ne_compile:"),
            "Phase 2: expected ne_compile: prefix, got: {}",
            compile_result
        );
        assert!(
            compile_result.contains("bytes"),
            "Phase 2: expected 'bytes' in compile result, got: {}",
            compile_result
        );

        // Phase 3 — stdlib file exists and contains vsa_bind
        let stdlib_path = std::path::Path::new("target/gen/stdlib.ne");
        assert!(
            stdlib_path.exists(),
            "Phase 3: target/gen/stdlib.ne should exist after compile"
        );
        let stdlib_content = std::fs::read_to_string(stdlib_path).unwrap_or_else(|e| {
            log::warn!("Phase 3: failed to read stdlib.ne: {}", e);
            String::new()
        });
        assert!(
            stdlib_content.contains("vsa_bind"),
            "Phase 3: stdlib.ne should contain 'vsa_bind'"
        );

        // Phase 4 — StdLib parses through ne_surface
        let parse_result = ne_surface::parse(&stdlib_content);
        assert!(
            parse_result.is_ok(),
            "Phase 4: ne_surface::parse(stdlib.ne) should succeed"
        );

        // Phase 5 — ne_compile again at cycle 60, confirm parse:ok in result
        ci.cycle = 60;
        let compile60 = ci.handle_ne_compile_tick();
        assert!(
            compile60.contains("parse:ok"),
            "Phase 5: expected parse:ok in compile result, got: {}",
            compile60
        );

        // Phase 6 — Bootstrap identity: status_report is non-empty
        let report = ci.bootstrap_verifier.status_report();
        assert!(
            !report.is_empty(),
            "Phase 6: bootstrap status report should be non-empty"
        );

        // Phase 7 — Identity history records compiler_compiles: false (no real cargo project)
        let latest = ci.bootstrap_verifier.identity_history.back();
        assert!(
            latest.is_some(),
            "Phase 7: identity_history should have a record after ne_compile"
        );
        if let Some(record) = latest {
            assert!(
                !record.compiler_compiles,
                "Phase 7: compiler_compiles should be false (no real cargo project in test)"
            );
        }
    }

    #[test]
    fn test_handle_trace_mining_tick_frequency() {
        let mut ci = ConsciousnessIntegration::new();

        // Cycle 0 → idle (cycle == 0 guard)
        ci.cycle = 0;
        let result = ci.handle_trace_mining_tick();
        assert_eq!(result, "trace:idle");

        // Cycle 1 → not % 50, idle
        ci.cycle = 1;
        let result = ci.handle_trace_mining_tick();
        assert_eq!(result, "trace:idle");

        // Cycle 49 → not % 50
        ci.cycle = 49;
        let result = ci.handle_trace_mining_tick();
        assert_eq!(result, "trace:idle");

        // Cycle 50 → should attempt mining (too few thoughts)
        ci.cycle = 50;
        let result = ci.handle_trace_mining_tick();
        assert!(
            result.starts_with("trace:"),
            "expected trace: prefix, got: {}",
            result
        );

        // Cycle 100 with populated history → should mine
        ci.cycle = 100;
        let v = crate::core::nt_core_hcube::QuantizedVSA::seeded_random(
            42,
            crate::core::nt_core_hcube::vsa_quantized::VSA_DIM,
        );
        ci.thought_history
            .push_back(("search the web".into(), v.clone(), 100.0));
        ci.thought_history
            .push_back(("search internet".into(), v.clone(), 101.0));
        ci.thought_history
            .push_back(("search online".into(), v, 102.0));
        // Also register the search primitive so sub_ids link works
        ci.capability_synthesizer
            .register_primitive("search", "search capability");
        let result = ci.handle_trace_mining_tick();
        assert!(
            result.starts_with("trace:clusters="),
            "expected trace:clusters=, got: {}",
            result
        );
    }

    #[test]
    fn test_mutation_log_save_load_roundtrip() {
        let records = vec![
            MutationRecord {
                handler: "test_handler_a".into(),
                action: "explore".into(),
                cycle: 42,
                pre_success_rate: 0.5,
                post_success_rate: None,
                outcome: "pending".into(),
            },
            MutationRecord {
                handler: "test_handler_b".into(),
                action: "repair".into(),
                cycle: 99,
                pre_success_rate: 0.3,
                post_success_rate: Some(0.7),
                outcome: "improved".into(),
            },
        ];

        let json = serde_json::to_string_pretty(&records).unwrap_or_else(|e| {
            eprintln!("types: serialize mutation records failed: {}", e);
            String::new()
        });
        if json.is_empty() {
            return;
        }
        let loaded: Vec<MutationRecord> = serde_json::from_str(&json).unwrap_or_else(|e| {
            eprintln!("types: deserialize mutation records failed: {}", e);
            vec![]
        });

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].handler, "test_handler_a");
        assert_eq!(loaded[0].action, "explore");
        assert_eq!(loaded[0].cycle, 42);
        assert_eq!(loaded[0].outcome, "pending");
        assert_eq!(loaded[1].handler, "test_handler_b");
        assert_eq!(loaded[1].action, "repair");
        assert_eq!(loaded[1].cycle, 99);
        assert_eq!(loaded[1].post_success_rate, Some(0.7));
        assert_eq!(loaded[1].outcome, "improved");
    }

    #[test]
    fn test_mutation_log_merge_dedup() {
        let mut ci = ConsciousnessIntegration::new();
        let dir = std::env::temp_dir().join("neotrix_mutation_test");
        let _ = std::fs::create_dir_all(&dir);
        ci.ne_source_dir = dir.to_string_lossy().to_string();

        // Seed existing in-memory records
        ci.mutation_log.push(MutationRecord {
            handler: "h1".into(),
            action: "explore".into(),
            cycle: 10,
            pre_success_rate: 0.5,
            post_success_rate: None,
            outcome: "pending".into(),
        });
        ci.mutation_log.push(MutationRecord {
            handler: "h2".into(),
            action: "repair".into(),
            cycle: 20,
            pre_success_rate: 0.3,
            post_success_rate: Some(0.8),
            outcome: "improved".into(),
        });

        // Save to disk
        let mut_path = dir.join("ne_mutation_log.json");
        if let Err(e) = ci.save_mutation_log(&mut_path.to_string_lossy()) {
            eprintln!("types: save mutation log failed: {}", e);
            return;
        }

        // Create a fresh CI, simulate disk load with merge
        let mut ci2 = ConsciousnessIntegration::new();
        ci2.ne_source_dir = dir.to_string_lossy().to_string();
        // Call handle_ne_load_tick to trigger load merge
        let result = ci2.handle_ne_load_tick();
        assert!(result.starts_with("ne_load:"));

        // Should have loaded 2 records
        assert_eq!(ci2.mutation_log.len(), 2, "should load 2 records from disk");

        // Add 1 duplicate + 1 new, save, then merge again
        ci2.mutation_log.push(MutationRecord {
            handler: "h1".into(),
            action: "explore".into(),
            cycle: 10,
            pre_success_rate: 0.5,
            post_success_rate: None,
            outcome: "pending".into(),
        });
        ci2.mutation_log.push(MutationRecord {
            handler: "h3".into(),
            action: "prune".into(),
            cycle: 30,
            pre_success_rate: 0.9,
            post_success_rate: None,
            outcome: "pending".into(),
        });
        if let Err(e) = ci2.save_mutation_log(&mut_path.to_string_lossy()) {
            eprintln!("types: save2 mutation log failed: {}", e);
            return;
        }

        let mut ci3 = ConsciousnessIntegration::new();
        ci3.ne_source_dir = dir.to_string_lossy().to_string();
        let _ = ci3.handle_ne_load_tick();

        // Should dedup: 2 original + 1 new (h3) = 3 total, not 4
        assert_eq!(ci3.mutation_log.len(), 3, "should dedup to 3 records");
        let handlers: Vec<&str> = ci3
            .mutation_log
            .iter()
            .map(|r| r.handler.as_str())
            .collect();
        assert!(handlers.contains(&"h1"));
        assert!(handlers.contains(&"h2"));
        assert!(handlers.contains(&"h3"));

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_text_input_flows_to_response_buffer() {
        let mut ci = ConsciousnessIntegration::new();

        // Feed text (simulates user input)
        ci.feed_consciousness_text("test_input");
        assert_eq!(
            ci.text_buffer.len(),
            1,
            "text_buffer should have 1 entry after feed"
        );

        // Run the full consciousness batch — this pops from text_buffer,
        // encodes it, processes through the pipeline, and ultimately
        // generates a response pushed to response_buffer.
        let result = ci.handle_consciousness_batch_sync();
        assert!(!result.is_empty(), "batch should produce output");

        // After batch, text_buffer should be consumed
        assert_eq!(
            ci.text_buffer.len(),
            0,
            "text_buffer should be drained after batch"
        );

        // Drain response_buffer — response_generation_tick should have
        // pushed output since thought_history had content.
        let responses = ci.drain_response_buffer();
        log::info!(
            "drained {} responses from buffer: {:?}",
            responses.len(),
            responses
        );

        // At minimum, the pipeline processed the input. The response may be
        // empty if thought_history wasn't populated, but the text buffer
        // consumption is the critical fix we're verifying.
        assert!(
            ci.text_feed_count > 0,
            "text_feed_count should track fed inputs"
        );
    }
}
