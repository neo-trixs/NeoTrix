// AUTO-GENERATED MODULE DECLARATIONS
pub mod active_inference;
pub mod adversarial_evaluator;
pub mod affective_circumplex;
pub mod affective_forecast;
pub mod analogical_reasoning;
pub mod appraisal_engine;
pub mod attention_schema;
pub mod authority;
pub mod awakening;
pub mod backpressure;
pub mod belief_revision;
pub mod bio_memory;
pub mod boredom_accumulator;
pub mod caa_steering;
pub mod caa_validation;
pub mod canonical_sort;
pub mod causal_counterfactual_bridge;
pub mod causal_model;
pub mod causal_reasoning;
pub mod claim_calibrator;
pub mod cognitive_flexibility;
pub mod cognitive_load;
pub mod cognitive_state;
pub mod confidence_calibrator;
pub mod conformal_uq;
pub mod consciousness_architecture;
pub mod consciousness_assessment;
pub mod consciousness_checkpoint;
pub mod cognitive_module_registry;
pub mod continuous_ode;
pub mod counterfactual;
pub mod dead_end_detector;
pub mod default_mode_network;
pub mod dream_consolidator;
pub mod drive_selector;
pub mod earned_autonomy;
pub mod embodied_grounding;
pub mod emergent_reasoning;
pub mod emotion_regulation;
pub mod emotional_steering;
pub mod epistemic_calibrator;
pub mod epistemic_honesty;
pub mod epistemic_humility;
pub mod error;
pub mod existential_monitor;
pub mod ethics_compliance;
pub mod executive_controller;
pub mod first_person_ref;
pub mod gea_archive;
pub mod global_workspace;
pub mod hebbian_associative_memory;
pub mod hierarchical_memory;
pub mod hierarchical_phi;
pub mod hierarchical_world_model;
pub mod human_emotion_detector;
pub mod identity_chain;
pub mod identity_defense;
pub mod identity_fragments;
pub mod iit_phi;
pub mod integration_bus;
pub mod inner_critic;
pub mod kapro_assessor;
pub mod link_formation;
pub mod interior_monologue;
pub mod free_energy_curiosity;
pub mod intrinsic_motivation;
pub mod iron_laws;
pub mod joint_attention;
pub mod kv_cache_consolidation;
pub mod log_linear_attention;
pub mod long_horizon_ocr;
pub mod master_equation;
pub mod mcts_gwt_bridge;
pub mod mcts_tree_search;
pub mod memory_lattice;
pub mod memory_lattice_seed;
pub mod memory_palace;
pub mod memory_reflector;
pub mod mental_time_travel;
pub mod evolution_efficiency_tracker;
pub mod meta_accuracy_tracker;
pub mod meta_cognition_bridge;
pub mod metacognitive_controller;
pub mod minimal_self;
pub mod mirror_buffer;
pub mod mtc_safety;
pub mod multi_modal_gate;
pub mod narrative_journal;
pub mod narrative_self;
pub mod neuromodulator;
pub mod p2p_consensus;
pub mod parallel_hypothesis_evaluator;
pub mod personality_matrix;
pub mod phi_integration;
pub mod pixel_perception;
pub mod predictive_gate;
pub mod proof_search;
pub mod qualia_generator;
pub mod qualia_layer;
pub mod quality_gate;
pub mod reasoning_federation;
pub mod reconstructive_narrative;
pub mod recurrent_world_model;
pub mod reflexive_unit;
pub mod resource_pool;
pub mod rii_u;
pub mod salience_detector;
pub mod semantic_entropy;
pub mod scar_formation;
pub mod screenshot_pipeline;
pub mod sensor_grounding;
pub mod sleep_consolidation_bridge;
pub mod sleep_gate;
pub mod source_hierarchy;
pub mod specious_present;
pub mod spreading_activation;
pub mod storm_breaker;
pub mod stream_buffer;
pub mod sub_consciousness;
pub mod substrate_first_gen;
pub mod system1;
pub mod temporal_attention;
pub mod temporal_attention_stack;
pub mod temporal_prediction;
pub mod unified_will;
pub mod valence_axis;
pub mod value_alignment;
pub mod value_system;
pub mod volition;
pub mod vsa_prefix_fingerprint;
pub mod vsa_tag;
pub mod worldview_stack;

// Architecture-level evolution modules (Phase 1-6)
pub mod adaptive_controller;
pub mod cognitive_blackboard;
pub mod consciousness_cycle;
pub mod consciousness_pipeline;
pub mod cte_consolidation;
pub mod consciousness_refinery;
pub mod dual_path_inference;
pub mod episodic_buffer;
pub mod executable_belief;
pub mod meta_evolution_loop;
pub mod performance_oracle;
pub mod resource_allocator;
pub mod self_evolution_orchestrator;
pub mod spectrum_signal;

pub use authority::{
    AuthorityLevel, AuthorityResolver, AuthorityTag, ConflictResolution, Constitution,
};
pub use awakening::{AwakeningReport, ConsciousnessAwakening};
pub use cognitive_load::{CognitiveLoadMonitor, ThinkingMode};
pub use confidence_calibrator::ConfidenceCalibrator;
pub use conformal_uq::{ConformalSet, ConformalUQ};
pub use default_mode_network::{DMNActivity, DefaultModeNetwork};
pub use evolution_efficiency_tracker::{EfficiencyReport, EvolutionEfficiencyTracker};
pub use first_person_ref::FirstPersonRef;
pub use inner_critic::{CritiqueResult, InnerCritic};
pub use narrative_self::{NarrativeEvent, NarrativeSelf};
pub use resource_pool::{PoolTier, Resource, ResourcePool};
pub use sleep_gate::{SleepGate, SleepReport};
pub use specious_present::SpeciousPresent;
pub use stream_buffer::ConsciousnessStream;
pub use valence_axis::{NamedEmotion, ValenceAxis};
pub use value_alignment::{UserValueConflict, UserValueProfile, ValueAlignmentEngine};
pub use value_system::{CoreValue, ValueSystem};
pub use existential_monitor::{
    EiReport, ExistentialIndifferenceMonitor, UcipProbe,
};
pub use volition::{ActionCandidate, VolitionEngine};
pub use vsa_tag::{VsaOrigin, VsaSelfCategory, VsaTagged, VsaWorldCategory};

// Additional re-exports needed by neotrix/ modules
pub use claim_calibrator::global_claim_calibrator;
pub use continuous_ode::{LiquidODE, ODEConfig, ODEStateObserver, SolverMethod};
pub use hierarchical_phi::{HierarchicalPhi, PhiProfile, ScaledPhiResult};
pub use memory_lattice::LatticeLayer;
pub use vsa_tag::SenseModality;

// Re-exports for qualia_generator
pub use qualia_generator::{Qualia5, QualiaBinding, QualiaGenerator, QualiaTone, QualifiedVsa};

// Re-exports for missing types from appraisal_engine
pub use appraisal_engine::{AppraisalDimensions, AppraisalEngine};

// Re-exports for human_emotion_detector
pub use human_emotion_detector::{
    HumanEmotionDetector, HumanEmotionReading, LinguisticFeatureSet, QuestionType,
};

// Re-exports for emergent_reasoning
pub use emergent_reasoning::{
    EmergentReasoningConfig, EmergentReasoningMode, ModeTransition, ReasoningMode,
};

// Re-exports for reflexive_unit
pub use reflexive_unit::{ReflexiveConfig, ReflexiveUnit};

// Re-exports for epistemic_honesty
pub use epistemic_honesty::{CalibrationBin, EpistemicHonesty, EpistemicReport, HonestyConfig};

// Re-exports for epistemic_humility
pub use epistemic_humility::{
    EpistemicBoundary, EpistemicHumility, HumilityAssessment, HumilityConfig,
    HumilityRecommendation, HumilityStats, UncertaintyLevel,
};

// Re-exports for personality_matrix
pub use personality_matrix::{OCEANTrait, PersonalityConfig, PersonalityMatrix, TraitState};

// Re-exports for cognitive_state
pub use cognitive_state::{CognitiveState, CognitiveStateIngestion, DimSnapshot, IngestionConfig};

// Re-exports for master_equation
pub use master_equation::{
    ConsciousnessEvolution, ConsciousnessMetrics, MasterConsciousness, MasterConsciousnessConfig,
};

// Re-exports for reconstructive_narrative
pub use reconstructive_narrative::{
    NarrativeThread, ReconstructedNarrative, ReconstructiveConfig, ReconstructiveNarrative,
    ThreadType,
};

// Re-exports for proof_search
pub use proof_search::{
    ModificationProposal, ProofSearchConfig, ProofSearchSelfModification, SafetyLevel,
    SafetyVerificationResult, SelfModificationProof,
};

// Re-exports for pixel perception pipeline
pub use pixel_perception::{
    EmbeddingBackend, PixelPerceptionPipeline, PixelRenderingConfig, VisualEmbedding,
    VisualEmbeddingConfig, VisualSceneBuffer, VisualTile, VisualToVSABridge,
};

// Re-exports for cognitive architecture engines
pub use attention_schema::AttentionSchemaEngine;
pub use belief_revision::BeliefRevisionEngine;
pub use cognitive_flexibility::CognitiveFlexibility;
pub use executive_controller::ExecutiveController;
pub use free_energy_curiosity::FreeEnergyCuriosityEngine;
pub use intrinsic_motivation::IntrinsicMotivationEngine;

// Re-exports for architecture-level evolution modules (Phase 1-6)
pub use adaptive_controller::{
    AdaptationEvent, AdaptiveController, AdaptiveResult, ControllerConfig,
};
pub use cognitive_blackboard::{
    BlackboardConfig, Claim, CognitiveBlackboard, Contradiction, EngineType, Synthesis, TopicState,
};
pub use consciousness_cycle::{
    ConsciousnessCycle, CycleConfig, CycleResult, CycleStep, StepHealth,
};
pub use temporal_prediction::TemporalPredictionTracker;
pub use cte_consolidation::{CteCycle, CteReport};
pub use consciousness_pipeline::{
    ConsciousnessPipeline, IntegratedResult, PipelineConfig, PipelineStepResult,
};
pub use consciousness_refinery::{
    ConsciousnessRefineryLoop, ConvergenceSignal, RefineryConfig, RefineryMetrics, RefineryResult,
};
pub use dual_path_inference::{
    CrossValidation, DualPathConfig, DualPathInference, DualPathResult, MergeStrategy, PathOutput,
    PathType,
};
pub use episodic_buffer::{BufferConfig, EpisodicConsciousnessBuffer, EpisodicEntry, RecallResult};
pub use executable_belief::{
    Belief, BeliefVerificationConfig, EvidenceAnchor, ExecutableBeliefVerifier, VerificationLevel,
    VerificationReport,
};
pub use meta_evolution_loop::{
    EvolutionAction, EvolutionAttempt, EvolutionOutcome, EvolutionRecommendation,
    MetaArchitectureEvolutionLoop, MetaEvolutionConfig, Trend,
};
pub use performance_oracle::{
    AdaptiveRecommendation, HealthDashboard, OracleConfig, PerformanceOracle, PipelineMetrics,
    StepMetrics, TrendDirection,
};
pub use resource_allocator::{
    AllocatorConfig, BudgetAllocation, CognitiveProcess, ConsciousResourceAllocator, InternalState,
};
pub use spectrum_signal::{Candidate, PipelinePhase, SpectrumConfig, SpectrumSignal};

// Re-exports for p2p_consensus
pub use p2p_consensus::{BanachConsensus, ConsensusResult, PeerState};

pub use cognitive_module_registry::{CognitiveModule, ModulePhase, ModuleRegistry};

// Re-export for SubsystemIntegrationBus
pub use integration_bus::{IntegrationSignal, ModulationCommand, SubsystemIntegrationBus};

// Re-export for IitPhi8Engine (8-way parallel IIT Φ)
pub use iit_phi::IitPhi8Engine;

// Re-export for HeLa-Mem Hebbian distillation agent
pub use hebbian_associative_memory::HebbianDistillationAgent;

// Re-export for KAPRO ActingDimensionAssessor
pub use kapro_assessor::{ActingDimensionAssessor, KaproReport};
// Re-export for CTM-AI LinkFormation
pub use link_formation::{LinkFormation, LinkFormationConfig, SubsystemLink};

// ── SecurityExecutive subsystems (Phase 32) ──
pub mod security_executive;
pub use security_executive::{
    AdversarialProbe, AdversarialReasoner, AttackSurface, AuditReport, DefenseAction,
    DefenseDecision, DependencyRecord, ProvenanceLevel, RedTeamReport, RiskLevel, RiskReport,
    RiskSensor, SelfDefense, SupplyChainGuard, ThreatAssessment, ThreatCategory, ThreatModeler,
};

// ── Mind Bridge (Phase 33) — connects neotrix-mind modules ──
pub mod mind_bridge;
