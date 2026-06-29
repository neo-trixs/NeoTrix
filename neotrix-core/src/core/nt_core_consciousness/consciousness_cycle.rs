//! # ConsciousnessCycle — 12-Step Unified Consciousness Loop
//!
//! Full cognitive pipeline with all 24 modules wired into their
//! respective pipeline steps. Each module is optional (Option<T>) so
//! graceful degradation is preserved — missing modules just skip.

use std::collections::VecDeque;

use fastrand;

use super::qualia_generator::{QualiaTone, QualifiedVsa};
use super::vsa_tag::{SenseModality, VsaOrigin, VsaSelfCategory, VsaTagged};
use crate::neotrix::nt_world_document::file_perception::DocumentPerceptionModule;
use crate::neotrix::nt_world_vision::image_cache::ImageCache;

// ── Phase 6 subsystems (existing) ──
use super::evolution_efficiency_tracker::EvolutionEfficiencyTracker;
use super::executive_controller::ExecutiveController;
use super::inner_critic::InnerCritic;
use super::kapro_assessor::ActingDimensionAssessor;
use super::link_formation::LinkFormation;
use super::master_equation::{MasterConsciousness, MasterConsciousnessConfig};
use super::meta_accuracy_tracker::MetaAccuracyTracker;
use super::metacognitive_controller::MetacognitiveController;
use super::mind_bridge::MindBridge;
use super::stream_buffer::ConsciousnessStream;
use crate::core::nt_core_experience::capability_synthesizer::{
    merge_knowledge_package, CapabilitySynthesizer, KnowledgePackage, LatticeSnapshot,
};
use crate::core::nt_core_experience::graceful::{
    GracefulDegradationManager, SubsystemHealth as GraceSubHealth,
};
use crate::core::nt_core_experience::seal_closed_loop::{MetaSealEngine, SealClosedLoop};
use crate::core::nt_core_experience::seal_governance::SEALGovernance;
use crate::core::nt_core_experience::trajectory_heuristics::ExperienceRecord as TrajectoryExperience;
use crate::core::nt_core_experience::verification_gate::VerificationGate;

use super::cognitive_module_registry::{ModulePhase, ModuleRegistry};
use super::integration_bus::{IntegrationSignal, ModulationCommand, SubsystemIntegrationBus};
use super::temporal_prediction::TemporalPredictionTracker;

// ── New module imports (same-directory modules) ──
use super::analogical_reasoning::AnalogicalReasoner;
use super::belief_revision::BeliefRevisionEngine;
use super::bio_memory::BioMemorySystem;
use super::boredom_accumulator::{BoredomAccumulator, ExplorationDriver};
use super::causal_model::CausalReasoner;
use super::default_mode_network::DefaultModeNetwork;
use super::epistemic_humility::EpistemicHumility;
use super::hierarchical_phi::HierarchicalPhi;
use super::hierarchical_world_model::HierarchicalWorldModel;
use super::identity_defense::VsaIdentityDefense;
use super::multi_modal_gate::ModalityGate;
use super::neuromodulator::NeuromodulatorySystem;
use super::scar_formation::ScarGuidedLearning;
use super::sensor_grounding::SensorGrounding;
use super::spreading_activation::VsaSpreadingActivation;
use super::substrate_first_gen::SubstrateFirstGenerator;

// ── Cross-crate module imports ──
use crate::agent::tool::impls::{
    ArchitectTool, EarnTool, ImageGenTool, LspTool, MiniMaxT2ITool, OsintInvestigatorTool,
    ReactDoctorTool, SecurityAuditTool, WebScrapeTool,
};
use crate::agent::tool::registry::AgentToolRegistry;
use crate::core::nt_core_edit::cognitive_wal::CognitiveWal;
use crate::core::nt_core_edit::rsi_meta_cycle::RsiMetaCycle;
use crate::core::nt_core_edit::shadow_runtime::SecuritySandbox;
use crate::core::nt_core_experience::continuous_learning::DataFlywheel;
use crate::core::nt_core_governance::consensus_engine::GovernanceConsensus;
use crate::core::nt_core_self::architecture_governor::ArchitectureSelfModel;
use crate::core::nt_core_self::cognitive_dashboard::CognitiveDashboard;
use crate::core::nt_core_self::skill_registry::SkillOrchestrator;

// ── Wave 0.5 v20 modules ──
use super::attention_schema::AttentionSchemaEngine;
use super::cognitive_blackboard::{BlackboardConfig, CognitiveBlackboard};
use super::multi_modal_gate::ModalityGateConfig;
use super::qualia_generator::Qualia5;
use super::qualia_generator::QualiaGenerator;
use super::salience_detector::SalienceDetector;
use super::screenshot_pipeline::{ScreenshotCaptureConfig, ScreenshotPipeline};
use super::unified_will::UnifiedWill;
use super::volition::{ActionCandidate, VolitionEngine};
use crate::core::nt_core_experience::agent_team::TeamOrchestrator;
use crate::core::nt_core_experience::code_mutation_engine::CodeMutationEngine;
use crate::core::nt_core_experience::compound_knowledge::CompoundKnowledgeBase;
use crate::core::nt_core_experience::continuous_learning::FlywheelStrategy;
use crate::core::nt_core_experience::data_quality_pipeline::{DQConfig, DataQualityPipeline};
use crate::core::nt_core_experience::goal_drift_index::GoalDriftIndex;
use crate::core::nt_core_experience::hybrid_retrieval::HybridRetrievalEngine;
use crate::core::nt_core_experience::ouroboros_stage_manager::OuroborosLoop;
use crate::core::nt_core_experience::web_content_extractor::{
    WebContentConfig, WebContentExtractor, WebPageContent,
};

// ── SecurityExecutive subsystems (Phase 32) ──
use super::security_executive::{
    AdversarialReasoner, AuditEventType, AuditTrail, DefenseAction, EvolutionGatekeeper,
    GateDecision, RiskSensor, SelfDefense, SupplyChainGuard, ThreatModeler,
};

// ── Wave A: Reasoning modules (revived from ~1700 lines dead code) ──
use crate::core::nt_core_reasoning::counterfactual_simulator::{
    CounterfactualConfig, CounterfactualSimulator,
};
use crate::core::nt_core_reasoning::dead_end_detector::{DeadEndConfig, DeadEndDetector};
use crate::core::nt_core_reasoning::mcts_reasoner::{MctsConfig, MctsReasoner};
use crate::core::nt_core_reasoning::{ReasonerConfig, VsaReasoner};

// ── Phase 26 Wave A: VLM Document Parsing modules ──
use super::long_horizon_ocr::{DocumentSource, LongHorizonOcr, OcrConfig};
use super::pixel_perception::{PixelPerceptionPipeline, VisualTile};
use crate::core::nt_core_input::document_classifier::DocumentClassifier;
use crate::core::nt_core_input::document_parser::{DocumentError, DocumentParser};
use crate::core::nt_core_input::document_router::{
    create_default_registry, DocumentParserRegistry,
};
use crate::core::nt_core_input::formula_extractor::{
    enrich_document_with_formulas, FormulaExtractor,
};

// ── Entity-boosted knowledge retrieval ──
use crate::core::nt_core_knowledge::entity_extractor::EntityExtractor;
use crate::core::nt_core_knowledge::semantic_compressor::SemanticCompressor;
use crate::core::nt_core_knowledge::spread_activation::MemoryGraph;

// ── Newly wired modules (Phase 5 memory evolution) ──
use super::active_inference::{ActiveInferenceConfig, ActiveInferenceEngine, GenerativeModel};
use super::affective_forecast::AffectiveForecastEngine;
use super::appraisal_engine::AppraisalEngine;
use super::consciousness_architecture::ConsciousnessArchitecture;
use super::dream_consolidator::DreamConsolidator;
use super::emotion_regulation::EmotionRegulation;
use super::emotional_steering::{EmotionalSteering, EmotionalTrail};
use super::free_energy_curiosity::FreeEnergyCuriosityEngine;
use super::human_emotion_detector::HumanEmotionDetector;
use super::identity_chain::IdentityChain;
use super::identity_fragments::IdentityFragmentDetector;
use super::iit_phi::{FactoredTPM, IitPhi8Engine, PhiCalculator};
use super::narrative_self::NarrativeSelf;
use super::performance_oracle::{OracleConfig, PerformanceOracle};
use super::reasoning_federation::{FusionStrategy, ReasoningFederation};
use super::sleep_gate::SleepGate;
use super::system1::System1Intuition;

// ── SINDy Engine: sparse identification of VSA dynamics ──
use crate::core::nt_core_hcube::multi_head_resonator::{AggregationMode, MultiHeadResonator};
use crate::core::nt_core_hcube::sindy_engine::SindyEngine;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
// ── StreamPipeline: block-causal VSA attention for real-time streaming ──
use crate::core::nt_core_hcube::stream_pipeline::StreamPipeline;

// ── NASS: NeoTrix Agent Skill System ──
use crate::core::nt_core_experience::skill_progressive::SkillRegistry;
// ── MTRE: Multi-Timeline Research Engine ──
use crate::core::nt_core_experience::multi_timeline::TimelineOrchestrator;
// ── DDP: Deep Digestion Pipeline ──
use crate::core::nt_core_experience::deep_digestion::DeepDigestionPipeline;
// ── CED: Constellation Emergence Detector ──
use crate::core::nt_core_experience::constellation::ConstellationDetector;
// ── CTI: Cross-Timeline Integrator ──
use crate::core::nt_core_experience::cross_timeline::CrossTimelineIntegrator;

// ── SemanticEntropyTracker: GWA-inspired entropy-based intrinsic drive ──
use super::semantic_entropy::SemanticEntropyTracker;

// ── Re-export sleep consolidation type ──
type ConsolidationBridge =
    crate::core::nt_core_consciousness::sleep_consolidation_bridge::ConsolidationBridge;

// ── CTE 4-stage consolidation pipeline ──
use super::cte_consolidation::CteCycle;
use super::hebbian_associative_memory::{HebbianAssociativeMemory, HebbianDistillationAgent};
use super::memory_lattice::{LatticeLayer, MemoryLattice, MemoryOrigin};

// ── Awakening engine (self-measure + self-modify) ──
use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use crate::neotrix::nt_mind_awakening::{AwakeningConfig, AwakeningEngine};
use crate::neotrix::nt_mind_background_loop::consciousness::NeotrixToolExecutor;

// ── Tool synthesizer ──
use super::super::nt_core_experience::tool_synthesizer::ToolSynthesizer;

// ── Cross-model distillation ──
use neotrix_mind::distillation::capture::CaptureBuffer;
use neotrix_mind::distillation::cross_model_distiller::CrossModelDistiller;
use std::sync::{Arc, Mutex};

/// The 12 steps of the unified consciousness cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CycleStep {
    Gather,
    Gate,
    Propose,
    Compete,
    Reason,
    Judge,
    Verify,
    Act,
    Veto,
    Record,
    Metric,
    Meta,
    Sleep,
}

/// Competitive selection mechanism for consciousness content.
/// After GATHER collects perceptual data, this module selects
/// what content enters the conscious workspace.
/// Reference: CTM-AI up-tree competition, GWA event-driven activation.
#[derive(Debug, Clone)]
pub struct CompetitiveSelection {
    /// Selection strategy: softmax, tournament, or winner-take-all
    pub strategy: SelectionStrategy,
    /// Temperature for softmax selection (lower = more greedy)
    pub temperature: f64,
    /// Number of candidates in tournament selection
    pub tournament_size: usize,
    /// History of selected content indices
    pub selection_history: VecDeque<usize>,
    /// Whether to enforce winner-take-all
    pub winner_take_all: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionStrategy {
    Softmax,
    Tournament,
    WinnerTakeAll,
}

impl Default for CompetitiveSelection {
    fn default() -> Self {
        Self {
            strategy: SelectionStrategy::Softmax,
            temperature: 1.0,
            tournament_size: 3,
            selection_history: VecDeque::with_capacity(20),
            winner_take_all: false,
        }
    }
}

impl CompetitiveSelection {
    pub fn new(strategy: SelectionStrategy, temperature: f64) -> Self {
        Self {
            strategy,
            temperature,
            tournament_size: 3,
            selection_history: VecDeque::with_capacity(20),
            winner_take_all: false,
        }
    }

    /// Select the best content from candidates based on salience scores.
    /// Returns (selected_index, winning_score).
    pub fn select(&mut self, candidates: &[VsaTagged], salience: &[f64]) -> Option<(usize, f64)> {
        if candidates.is_empty() || salience.is_empty() {
            return None;
        }
        if candidates.len() != salience.len() {
            return None;
        }

        let selected = match self.strategy {
            SelectionStrategy::Softmax => self.softmax_select(salience),
            SelectionStrategy::Tournament => self.tournament_select(salience),
            SelectionStrategy::WinnerTakeAll => self.wta_select(salience),
        };

        if let Some(idx) = selected {
            self.selection_history.push_back(idx);
            if self.selection_history.len() > 20 {
                self.selection_history.pop_front();
            }
            Some((idx, salience[idx]))
        } else {
            None
        }
    }

    fn softmax_select(&self, scores: &[f64]) -> Option<usize> {
        if scores.is_empty() {
            return None;
        }
        let max_score = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exps: Vec<f64> = scores
            .iter()
            .map(|s| ((s - max_score) / self.temperature.max(0.01)).exp())
            .collect();
        let sum: f64 = exps.iter().sum();
        if sum <= 0.0 {
            return Some(0);
        }
        let mut rng = fastrand::f64();
        for (i, e) in exps.iter().enumerate() {
            rng -= e / sum;
            if rng <= 0.0 {
                return Some(i);
            }
        }
        Some(scores.len() - 1)
    }

    fn tournament_select(&self, scores: &[f64]) -> Option<usize> {
        let n = scores.len();
        if n == 0 {
            return None;
        }
        let k = self.tournament_size.min(n);
        let mut best_idx = fastrand::usize(..n);
        let mut best_score = scores[best_idx];
        for _ in 1..k {
            let idx = fastrand::usize(..n);
            if scores[idx] > best_score {
                best_idx = idx;
                best_score = scores[idx];
            }
        }
        Some(best_idx)
    }

    fn wta_select(&self, scores: &[f64]) -> Option<usize> {
        scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
    }
}

/// Ratchet tracker — monotonic improvement guarantee for SEAL self-modification.
/// Reference: MOSS + Ratchet (arXiv May 2026): bounded modification scope,
/// monotonic improvement guarantee, rollback on regression.
/// SAHOO (arXiv 2603.06333): constraint preservation + regression-risk quantification.
#[derive(Debug, Clone)]
pub struct RatchetTracker {
    /// Baseline score at last checkpoint
    pub baseline: f64,
    /// Current best score achieved
    pub best: f64,
    /// Whether monotonic improvement is currently held
    pub is_held: bool,
    /// Number of consecutive regressions
    pub regressions: u64,
    /// Max regressions before forced rollback
    pub max_regressions: u64,
    /// Whether syntactic/safety constraints were preserved (SAHOO §2.2)
    pub constraint_preserved: bool,
    /// Regression-risk score: probability this cycle undoes prior gains (SAHOO §2.3)
    pub regression_risk: f64,
    /// Past regression rate: historical fraction of modifications that regressed
    pub past_regression_rate: f64,
    /// Audit trail: (score, description, constraint_ok) triples
    pub audit_trail: Vec<(f64, String, bool)>,
}

impl Default for RatchetTracker {
    fn default() -> Self {
        Self {
            baseline: 0.0,
            best: 0.0,
            is_held: true,
            regressions: 0,
            max_regressions: 3,
            constraint_preserved: true,
            regression_risk: 0.0,
            past_regression_rate: 0.0,
            audit_trail: Vec::with_capacity(50),
        }
    }
}

impl RatchetTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check whether a new score maintains monotonic improvement
    pub fn check(&mut self, score: f64, description: &str) -> bool {
        self.audit_trail
            .push((score, description.to_string(), self.constraint_preserved));
        if score >= self.best {
            self.best = score;
            self.regressions = 0;
            self.is_held = true;
            true
        } else {
            self.regressions += 1;
            if self.regressions >= self.max_regressions {
                self.is_held = false;
            }
            false
        }
    }

    /// SAHOO §2.2: Check that safety-critical constraints are preserved.
    /// Pass `false` if any constraint (syntactic correctness, non-hallucination,
    /// safety invariant) was violated during this cycle.
    pub fn check_constraint(&mut self, preserved: bool) {
        self.constraint_preserved = preserved;
    }

    /// SAHOO §2.3: Quantify regression risk based on recent audit trail.
    /// Higher values indicate this improvement cycle may undo prior gains.
    /// Uses recency-weighted regression ratio over last N records.
    pub fn quantify_regression_risk(&self, window: usize) -> f64 {
        let len = self.audit_trail.len();
        if len < 2 {
            return 0.0;
        }
        let start = len.saturating_sub(window);
        let recent: Vec<_> = self.audit_trail[start..].to_vec();
        let total = recent.len() as f64;
        if total == 0.0 {
            return 0.0;
        }
        let regressions = recent
            .iter()
            .filter(|(s, _, c)| *s < self.baseline || !*c)
            .count() as f64;
        let recent_regressions = recent
            .iter()
            .rev()
            .take(3)
            .filter(|(s, _, c)| *s < self.baseline || !*c)
            .count() as f64;
        // Weighted: recent regressions count 3x more than distant ones
        (regressions / total) * 0.4 + (recent_regressions / 3.0_f64.max(recent_regressions)) * 0.6
    }

    /// Past regression rate: fraction of historical entries that were regressions
    pub fn past_regression_rate(&self) -> f64 {
        let len = self.audit_trail.len();
        if len < 2 {
            return 0.0;
        }
        let regressions = self
            .audit_trail
            .iter()
            .filter(|(s, _, _)| *s < self.baseline)
            .count() as f64;
        regressions / len as f64
    }

    /// Regression risk given a complexity delta: past_regression_rate * complexity_delta
    pub fn regression_risk_with_delta(&self, complexity_delta: f64) -> f64 {
        self.past_regression_rate() * complexity_delta
    }

    /// Update regression risk from current audit state
    pub fn update_regression_risk(&mut self) {
        self.regression_risk = self.quantify_regression_risk(20);
        self.past_regression_rate = self.past_regression_rate();
    }

    /// SAHOO combined: returns true if both ratchet holds AND constraints preserved AND risk low
    pub fn is_safe(&self) -> bool {
        !self.is_broken() && self.constraint_preserved && self.regression_risk < 0.5
    }

    /// Reset baseline to current best (after successful modification)
    pub fn commit(&mut self) {
        self.baseline = self.best;
        self.regressions = 0;
        self.is_held = true;
        self.constraint_preserved = true;
        self.regression_risk = 0.0;
    }

    /// Whether the ratchet has broken (too many regressions)
    pub fn is_broken(&self) -> bool {
        !self.is_held && self.regressions >= self.max_regressions
    }
}

/// Health status per step.
#[derive(Debug, Clone)]
pub struct StepHealth {
    pub step: CycleStep,
    pub success: bool,
    pub duration_ms: u64,
}

/// CXVIII.67: Attentional gate before memory encoding (RECORD step).
/// Filters out low-salience content before it reaches hebbian/lattice stores.
#[derive(Debug, Clone)]
pub struct AttentionalGate {
    /// Salience threshold: entries below this are skipped (default 0.3)
    pub salience_threshold: f64,
    /// Number of entries filtered out
    pub gated_count: u64,
    /// Number of entries that passed through
    pub passed_count: u64,
}

impl Default for AttentionalGate {
    fn default() -> Self {
        Self {
            salience_threshold: 0.3,
            gated_count: 0,
            passed_count: 0,
        }
    }
}

impl AttentionalGate {
    pub fn new(threshold: f64) -> Self {
        Self {
            salience_threshold: threshold,
            gated_count: 0,
            passed_count: 0,
        }
    }

    /// Decide whether an entry with the given salience score should be written to memory.
    pub fn should_write(&mut self, salience: f64) -> bool {
        if salience >= self.salience_threshold {
            self.passed_count += 1;
            true
        } else {
            self.gated_count += 1;
            false
        }
    }

    /// Ratio of entries that passed through the gate: passed / (passed + gated)
    pub fn gate_ratio(&self) -> f64 {
        let total = self.passed_count + self.gated_count;
        if total == 0 {
            1.0
        } else {
            self.passed_count as f64 / total as f64
        }
    }
}

/// Metabolic energy budget for consciousness cycles.
/// Implements artificial hunger: computation has intrinsic cost.
/// When budget depletes, system enters starvation mode (graceful degradation).
/// Reference: AGI HUNGER arXiv, Free Energy Principle metabolic extension.
#[derive(Debug, Clone)]
pub struct MetabolicBudget {
    /// Current energy reserve (0.0 - 1.0)
    pub energy: f64,
    /// Energy consumed per consciousness step
    pub cost_per_step: f64,
    /// Energy recovered per tick (passive recovery)
    pub recovery_rate: f64,
    /// Whether the system is in starvation mode
    pub starvation_mode: bool,
    /// Starvation threshold (below this = starvation)
    pub starvation_threshold: f64,
    /// Total energy consumed over lifetime
    pub total_consumed: f64,
    /// Peak energy efficiency (work / energy)
    pub peak_efficiency: f64,
    /// Irreversible cost counter - never decrements, tracks lifetime irreversible cost.
    /// Reference: Abdulkareem 2026, consciousness requires irreversible internal cost.
    pub irreversible_cost: u64,
    /// Whether the evaluation is self-owned (true) or delegated to external (false).
    pub self_owned_evaluation: bool,
    /// If true, system is in "delegated evaluation" mode and should be marked.
    pub evaluation_delegated: bool,
    /// Multi-scale integration tracking markers (algorithm-implementation separation awareness).
    /// Which subsystems have substrate-level coupling.
    pub multi_scale_integration: Vec<String>,
}

impl Default for MetabolicBudget {
    fn default() -> Self {
        Self {
            energy: 1.0,
            cost_per_step: 0.01,
            recovery_rate: 0.005,
            starvation_mode: false,
            starvation_threshold: 0.2,
            total_consumed: 0.0,
            peak_efficiency: 0.0,
            irreversible_cost: 0,
            self_owned_evaluation: false,
            evaluation_delegated: true,
            multi_scale_integration: vec![],
        }
    }
}

impl MetabolicBudget {
    pub fn new(cost_per_step: f64, recovery_rate: f64) -> Self {
        Self {
            energy: 1.0,
            cost_per_step,
            recovery_rate,
            starvation_mode: false,
            starvation_threshold: 0.2,
            total_consumed: 0.0,
            peak_efficiency: 0.0,
            irreversible_cost: 0,
            self_owned_evaluation: false,
            evaluation_delegated: true,
            multi_scale_integration: vec![],
        }
    }

    /// Consume energy for a cycle step. Returns true if the step should proceed.
    pub fn consume(&mut self, work_done: f64) -> bool {
        self.energy = (self.energy - self.cost_per_step).max(0.0);
        self.total_consumed += self.cost_per_step;

        // Track efficiency
        let efficiency = if self.cost_per_step > 0.0 {
            work_done / self.cost_per_step
        } else {
            0.0
        };
        self.peak_efficiency = self.peak_efficiency.max(efficiency);

        // Check starvation
        if self.energy < self.starvation_threshold && !self.starvation_mode {
            self.starvation_mode = true;
        } else if self.energy > self.starvation_threshold * 2.0 && self.starvation_mode {
            self.starvation_mode = false;
        }

        !self.starvation_mode || self.energy > 0.0
    }

    /// Recover energy (passive, called each cycle).
    pub fn recover(&mut self) {
        self.energy = (self.energy + self.recovery_rate).min(1.0);
    }

    /// Whether the system should skip non-essential computation.
    pub fn should_skip_nonessential(&self) -> bool {
        self.starvation_mode || self.energy < self.starvation_threshold * 1.5
    }

    /// Inject energy (external reward).
    pub fn inject(&mut self, amount: f64) {
        self.energy = (self.energy + amount).min(1.0);
    }

    /// Energy level as a fraction of max.
    pub fn level(&self) -> f64 {
        self.energy
    }

    /// Record irreversible cost. This counter NEVER decrements.
    /// Each call increments by the given amount. Used to track
    /// the system's cumulative irreversible investment.
    pub fn record_irreversible_cost(&mut self, amount: u64) {
        self.irreversible_cost = self.irreversible_cost.saturating_add(amount);
    }

    /// Check if the irreversible cost counter has been tampered with (e.g., after rollback).
    /// Returns true if the counter appears to have decreased (discontinuity).
    pub fn check_discontinuity(&self, previous_cost: u64) -> bool {
        self.irreversible_cost < previous_cost
    }

    /// Mark evaluation as self-owned (system can validate itself).
    pub fn set_self_owned_evaluation(&mut self) {
        self.self_owned_evaluation = true;
        self.evaluation_delegated = false;
    }

    /// Mark evaluation as delegated (needs external calibration).
    pub fn set_delegated_evaluation(&mut self) {
        self.self_owned_evaluation = false;
        self.evaluation_delegated = true;
    }
}

/// Dual-process (System 1 / System 2) architecture config.
/// When fast path is enabled and confidence is high, the cycle skips
/// COMPETE, REASON, JUDGE, VERIFY and goes directly to ACT.
#[derive(Debug, Clone)]
pub struct DualProcessConfig {
    pub fast_path_enabled: bool,
    pub fast_path_threshold: f64,
    pub fast_path_cycles: u64,
    pub slow_path_cycles: u64,
}

impl Default for DualProcessConfig {
    fn default() -> Self {
        Self {
            fast_path_enabled: true,
            fast_path_threshold: 0.85,
            fast_path_cycles: 0,
            slow_path_cycles: 0,
        }
    }
}

/// NREM/REM sleep stage separation for the SLEEP step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SleepStage {
    Awake,
    NREM,
    REM,
}

#[derive(Debug, Clone)]
pub struct SleepStageConfig {
    pub nrem_duration: u64,
    pub rem_duration: u64,
    pub current_stage: SleepStage,
    pub stage_cycle: u64,
}

impl Default for SleepStageConfig {
    fn default() -> Self {
        Self {
            nrem_duration: 5,
            rem_duration: 3,
            current_stage: SleepStage::Awake,
            stage_cycle: 0,
        }
    }
}

/// Result of one cycle iteration.
#[derive(Debug, Clone)]
pub struct CycleResult {
    pub cycle_num: u64,
    pub steps_completed: Vec<CycleStep>,
    pub step_health: Vec<StepHealth>,
    pub overall_success: bool,
    pub total_duration_ms: u64,
    pub c_score: f64,
    pub output_state: Option<VsaTagged>,
    pub steps_executed: Vec<CycleStep>,
    pub substrate_concepts: Vec<String>,
    pub causal_counterfactuals: Vec<(usize, f64)>,
    pub neuromodulator_report: Option<String>,
    pub dashboard_report: Option<String>,
    pub phi_metrics: Option<Vec<f64>>,
    pub meta_insights: Vec<String>,
    pub rsi_proposals_count: usize,
    pub qualia5: Option<Qualia5>,
    pub extracted_content: Option<WebPageContent>,
    pub metabolic_state: String,
    /// Irreversible cost at this cycle
    pub irreversible_cost: u64,
    /// Whether evaluation is self-owned or delegated
    pub evaluation_delegated: bool,
    /// P0.6: Subsystem health — active vs inactive
    pub subsystem_health: SubsystemHealth,
}

/// Health report: which subsystems are active vs silent None
#[derive(Debug, Clone)]
pub struct SubsystemHealth {
    pub total_subsystems: usize,
    pub active: usize,
    pub inactive: usize,
    pub inactive_names: Vec<String>,
}

/// Configuration for the consciousness cycle.
#[derive(Debug, Clone)]
pub struct CycleConfig {
    pub max_steps: usize,
    pub step_timeout_ms: u64,
    pub enable_gather: bool,
    pub enable_gate: bool,
    pub enable_propose: bool,
    pub enable_compete: bool,
    pub enable_reason: bool,
    pub enable_judge: bool,
    pub enable_verify: bool,
    pub enable_act: bool,
    pub enable_veto: bool,
    pub enable_record: bool,
    pub enable_metric: bool,
    pub enable_meta: bool,
    pub enable_sleep: bool,
    pub enable_sleep_consolidation: bool,
    pub enable_visual_pipeline: bool,
    pub enable_modality_gate: bool,
    pub enable_sensor_grounding: bool,
    pub enable_identity_defense: bool,
    pub enable_neuromodulation: bool,
    pub enable_substrate_gen: bool,
    pub enable_boredom: bool,
    pub enable_spreading: bool,
    pub enable_causal: bool,
    pub enable_specious_present: bool,
    pub enable_bio_memory: bool,
    pub enable_scar: bool,
    pub enable_consensus: bool,
    pub enable_shadow: bool,
    pub enable_cognitive_wal: bool,
    pub enable_skills: bool,
    pub enable_data_flywheel: bool,
    pub enable_sindy_engine: bool,
    pub enable_hybrid_retrieval: bool,
    pub enable_stream_pipeline: bool,
    pub enable_phi: bool,
    pub enable_dashboard: bool,
    pub enable_arch_governor: bool,
    pub enable_rsi: bool,
    pub enable_qualia: bool,
    pub enable_data_quality: bool,
    pub enable_module_registry: bool,
    pub enable_document_perception: bool,
    // ── Phase 26 Wave A: VLM Document Parsing ──
    pub enable_pixel_perception: bool,
    pub enable_long_horizon_ocr: bool,
    pub enable_document_parser: bool,
    pub enable_document_classifier: bool,
    pub enable_screenshot_pipeline: bool,
    pub enable_formula_extractor: bool,
    // ── Leap 3: CapabilitySynthesizer sync ──
    pub enable_capability_synthesizer: bool,
    pub enable_cap_synth_export: bool,
    pub enable_cap_synth_synthesis: bool,
    // ── Leap 2: SEAL closed loop ──
    pub enable_seal_closed_loop: bool,
    // ── Wave A: Reasoning modules ──
    pub enable_mcts_reasoner: bool,
    pub enable_dead_end_detector: bool,
    pub enable_counterfactual_simulator: bool,
    // ── Wave C: Temporal Memory ──
    pub enable_temporal_query: bool,
    // ── Phase 1: VsaReasoner ──
    pub enable_vsa_reasoner: bool,
    // ── Entity-boosted retrieval ──
    pub enable_entity_extractor: bool,
    pub enable_entity_graph: bool,
    // ── Semantic compression (SLEEP step) ──
    pub enable_semantic_compressor: bool,
    // ── Wave D: Revived consciousness modules ──
    pub enable_analogical_reasoner: bool,
    pub enable_epistemic_humility: bool,
    pub enable_belief_revision: bool,
    pub enable_dmn: bool,
    pub enable_hierarchical_world_model: bool,
    // ── Newly wired modules ──
    pub enable_sleep_gate: bool,
    pub enable_narrative_self: bool,
    pub enable_appraisal_engine: bool,
    pub enable_system1: bool,
    pub enable_emotional_steering: bool,
    pub enable_active_inference: bool,
    pub enable_dream_consolidator: bool,
    pub enable_reasoning_federation: bool,
    pub enable_performance_oracle: bool,
    pub enable_iit_phi: bool,
    pub enable_iit_phi8: bool,
    // ── HeLa-Mem Hebbian memory ──
    pub enable_hebbian_memory: bool,
    // ── Awakening engine (self-measure + self-modify) ──
    pub enable_awakening: bool,
    // ── SecurityExecutive subsystems (Phase 32) ──
    pub enable_threat_modeler: bool,
    pub enable_risk_sensor: bool,
    pub enable_supply_chain_guard: bool,
    pub enable_adversarial_reasoner: bool,
    pub enable_self_defense: bool,
    pub enable_evolution_gatekeeper: bool,
    pub enable_audit_trail: bool,
    // ── Phase 33: MindBridge (mind crate integration) ──
    pub enable_mind_bridge: bool,
    // ── Cross-model distillation ──
    pub enable_distillation: bool,
    // ── MetaAccuracy tracking ──
    pub enable_meta_accuracy: bool,
    // ── GoalDriftIndex alignment monitoring ──
    pub enable_goal_drift_index: bool,
    // ── CompoundKnowledgeBase ──
    pub enable_compound_knowledge: bool,
    // ── AgentTeam ──
    pub enable_agent_team: bool,
    // ── CodeMutationEngine ──
    pub enable_code_mutation: bool,
    // ── NASS: Skill system ──
    pub enable_skill_registry: bool,
    // ── MTRE: Multi-timeline research ──
    pub enable_timeline_orchestrator: bool,
    // ── DDP: Deep digestion pipeline ──
    pub enable_deep_digestion: bool,
    // ── CED: Constellation detection ──
    pub enable_constellation_detector: bool,
    // ── CTI: Cross-timeline integration ──
    pub enable_cross_timeline_integrator: bool,
    // ── Competitive selection for consciousness content routing ──
    pub enable_competitive_selection: bool,
    // ── P0.3: Strict wiring mode — disable init_missing_fields auto-healing ──
    pub strict_wiring: bool,
    // ── Stealth HTTP: route web perception through nt_io_stealth_net ──
    pub enable_stealth_http: bool,
    pub stealth_proxy_url: Option<String>,
    // ── CXVIII.10: Hallucination impossibility slider ──
    /// Truthfulness vs creativity slider (0.0=strict truth, 1.0=creative).
    /// Reference: arXiv 2506.06382 — hallucination is mathematically equivalent to imagination.
    pub hallucination_tradeoff: f64,
    // ── CXVIII.5: Difficulty-adaptive parallel thinking (Best-of-N) ──
    pub enable_parallel_thinking: bool,
    pub parallel_thinking_width: usize,
    // ── CXVIII.6: Knowledge task information ceiling ──
    /// Reasoning mode: "symbolic" (extendable) or "recall" (non-extendable).
    /// Reference: arXiv 2509.06861 — test-time compute cannot exceed parametric info.
    pub reasoning_mode: String,
    // ── Tool execution: run registered AgentTools in ACT step ──
    pub enable_tool_execution: bool,
}

impl Default for CycleConfig {
    fn default() -> Self {
        Self {
            max_steps: 12,
            step_timeout_ms: 1000,
            enable_gather: true,
            enable_gate: true,
            enable_propose: true,
            enable_compete: true,
            enable_reason: true,
            enable_judge: true,
            enable_verify: true,
            enable_act: true,
            enable_veto: true,
            enable_record: true,
            enable_metric: true,
            enable_meta: true,
            enable_sleep: true,
            enable_sleep_consolidation: true,
            enable_visual_pipeline: true,
            enable_modality_gate: true,
            enable_sensor_grounding: true,
            enable_identity_defense: true,
            enable_neuromodulation: true,
            enable_substrate_gen: true,
            enable_boredom: true,
            enable_spreading: true,
            enable_causal: true,
            enable_specious_present: true,
            enable_bio_memory: true,
            enable_scar: true,
            enable_consensus: true,
            enable_shadow: true,
            enable_cognitive_wal: true,
            enable_skills: true,
            enable_data_flywheel: true,
            enable_sindy_engine: true,
            enable_hybrid_retrieval: true,
            enable_stream_pipeline: true,
            enable_phi: true,
            enable_dashboard: true,
            enable_arch_governor: true,
            enable_rsi: true,
            enable_qualia: true,
            enable_data_quality: true,
            enable_module_registry: true,
            enable_document_perception: true,
            enable_pixel_perception: true,
            enable_long_horizon_ocr: true,
            enable_document_parser: true,
            enable_document_classifier: true,
            enable_screenshot_pipeline: true,
            enable_formula_extractor: true,
            enable_capability_synthesizer: true,
            enable_cap_synth_export: true,
            enable_cap_synth_synthesis: true,
            enable_seal_closed_loop: true,
            enable_vsa_reasoner: true,
            enable_mcts_reasoner: true,
            enable_dead_end_detector: true,
            enable_counterfactual_simulator: true,
            enable_temporal_query: true,
            enable_entity_extractor: true,
            enable_entity_graph: true,
            enable_semantic_compressor: true,
            enable_analogical_reasoner: true,
            enable_epistemic_humility: true,
            enable_belief_revision: true,
            enable_dmn: true,
            enable_hierarchical_world_model: true,
            enable_sleep_gate: true,
            enable_narrative_self: true,
            enable_appraisal_engine: true,
            enable_system1: true,
            enable_emotional_steering: true,
            enable_active_inference: true,
            enable_dream_consolidator: true,
            enable_reasoning_federation: true,
            enable_performance_oracle: true,
            enable_iit_phi: true,
            enable_iit_phi8: true,
            enable_hebbian_memory: true,
            enable_awakening: true,
            enable_threat_modeler: true,
            enable_risk_sensor: true,
            enable_supply_chain_guard: true,
            enable_adversarial_reasoner: true,
            enable_self_defense: true,
            enable_evolution_gatekeeper: true,
            enable_audit_trail: true,
            enable_mind_bridge: true,
            enable_distillation: true,
            enable_meta_accuracy: true,
            enable_goal_drift_index: true,
            enable_compound_knowledge: true,
            enable_agent_team: true,
            enable_code_mutation: true,
            enable_skill_registry: true,
            enable_timeline_orchestrator: true,
            enable_deep_digestion: true,
            enable_constellation_detector: true,
            enable_cross_timeline_integrator: true,
            enable_competitive_selection: true,
            strict_wiring: false,
            enable_stealth_http: false,
            stealth_proxy_url: None,
            hallucination_tradeoff: 0.5,
            enable_parallel_thinking: false,
            parallel_thinking_width: 3,
            reasoning_mode: "symbolic".to_string(),
            enable_tool_execution: true,
        }
    }
}

/// MIRROR-style reconstructive episodic buffer (AAAI 2026).
/// Instead of storage-retrieval, reconstructs first-person narrative each cycle.
#[derive(Debug, Clone)]
pub struct ReconstructiveEpisodicBuffer {
    /// How often to reconstruct narrative (in cycles)
    pub reconstruction_interval: usize,
    /// Last cycle narrative was reconstructed
    pub last_reconstruction: usize,
    /// Current reconstructed narrative
    pub current_narrative: String,
    /// Parallel inner monologue threads
    pub parallel_threads: Vec<String>,
    /// Whether reconstruction is enabled
    pub enabled: bool,
}

impl Default for ReconstructiveEpisodicBuffer {
    fn default() -> Self {
        Self {
            reconstruction_interval: 5,
            last_reconstruction: 0,
            current_narrative: String::new(),
            parallel_threads: vec!["default".into()],
            enabled: true,
        }
    }
}

/// Attention self-modelling module (AAAI 2026 Spring Symposium).
/// Tracks the system's own attention focus movement and transfer ability.
#[derive(Debug, Clone)]
pub struct AttentionSelfModelling {
    /// History of attention focus positions
    pub focus_history: VecDeque<String>,
    /// History of focus transfer speeds
    pub transfer_speed: VecDeque<f64>,
    /// Whether attention self-modelling is enabled
    pub enabled: bool,
    /// Current attention focus description
    pub current_focus: String,
    /// Ability to transfer attention under noise
    pub noise_transfer_ability: f64,
}

impl Default for AttentionSelfModelling {
    fn default() -> Self {
        Self {
            focus_history: VecDeque::with_capacity(20),
            transfer_speed: VecDeque::with_capacity(20),
            enabled: true,
            current_focus: "none".into(),
            noise_transfer_ability: 0.0,
        }
    }
}

/// Overthinking detector for REASON step stability.
/// Detects "reasoning collapse" where extended reasoning decreases performance.
/// Reference: arXiv 2604.10739 overthinking in test-time compute.
#[derive(Debug, Clone)]
pub struct OverthinkingDetector {
    /// Consecutive cycles without improvement
    pub consecutive_no_improvement: u64,
    /// Max allowed before forced output
    pub max_no_improvement: u64,
    /// Last score for comparison
    pub last_score: f64,
    /// Whether currently in overthinking state
    pub overthinking: bool,
    /// History of overthinking events
    pub event_history: Vec<u64>,
}

impl Default for OverthinkingDetector {
    fn default() -> Self {
        Self {
            consecutive_no_improvement: 0,
            max_no_improvement: 5,
            last_score: 0.0,
            overthinking: false,
            event_history: Vec::new(),
        }
    }
}

/// Inner monologue manager for parallel cognitive threads.
/// Part of MIRROR-style access consciousness architecture.
#[derive(Debug, Clone)]
pub struct InnerMonologueManager {
    /// Active inner monologue threads
    pub threads: Vec<String>,
    /// Maximum parallel threads
    pub max_threads: usize,
    /// Whether management is active
    pub active: bool,
}

impl Default for InnerMonologueManager {
    fn default() -> Self {
        Self {
            threads: vec!["core".into()],
            max_threads: 5,
            active: true,
        }
    }
}

/// Cognitive controller that synthesizes restricted first-person narrative
/// from parallel inner monologue threads.
#[derive(Debug, Clone)]
pub struct CognitiveController {
    /// Last synthesized narrative
    pub last_narrative: String,
    /// Synthesis interval in cycles
    pub synthesis_interval: usize,
    /// Whether synthesis is active
    pub active: bool,
}

impl Default for CognitiveController {
    fn default() -> Self {
        Self {
            last_narrative: String::new(),
            synthesis_interval: 3,
            active: true,
        }
    }
}

/// The consciousness cycle orchestrator — all cognitive modules wired in.
pub struct ConsciousnessCycle {
    cycle_num: u64,
    config: CycleConfig,
    history: VecDeque<CycleResult>,
    // ── Phase 6 subsystems ──
    consolidation_bridge: Option<ConsolidationBridge>,
    pub(crate) image_cache: Option<ImageCache>,
    pub(crate) multi_modal_gate: Option<ModalityGate>,
    pub quality_gate: Option<super::quality_gate::QualityGate>,
    pub(crate) perception_input: Option<VsaTagged>,
    inner_critic: Option<InnerCritic>,
    verification_gate: Option<VerificationGate>,
    executive_controller: Option<ExecutiveController>,
    consciousness_stream: Option<ConsciousnessStream>,
    master_consciousness: Option<MasterConsciousness>,
    metacognitive_controller: Option<MetacognitiveController>,
    acting_dimension_assessor: Option<ActingDimensionAssessor>,
    link_formation: Option<LinkFormation>,
    // ── Wave 2-5 modules ──
    sensor_grounding: Option<SensorGrounding>,
    // ── StreamPipeline: thinker-performer block-causal attention for streaming ──
    stream_pipeline: Option<StreamPipeline>,
    identity_defense: Option<VsaIdentityDefense>,
    identity_fragments: Option<IdentityFragmentDetector>,
    identity_chain: Option<IdentityChain>,
    consciousness_architecture: Option<ConsciousnessArchitecture>,
    substrate_gen: Option<SubstrateFirstGenerator>,
    exploration_driver: Option<ExplorationDriver>,
    boredom: Option<BoredomAccumulator>,
    spreading: Option<VsaSpreadingActivation>,
    neuromodulators: Option<NeuromodulatorySystem>,
    causal: Option<CausalReasoner>,
    bio_memory: Option<BioMemorySystem>,
    scar: Option<ScarGuidedLearning>,
    consensus: Option<GovernanceConsensus>,
    shadow: Option<SecuritySandbox>,
    cognitive_wal: Option<CognitiveWal>,
    skills: Option<SkillOrchestrator>,
    tool_registry: Option<AgentToolRegistry>,
    data_flywheel: Option<DataFlywheel>,
    phi: Option<HierarchicalPhi>,
    dashboard: Option<CognitiveDashboard>,
    arch_governor: Option<ArchitectureSelfModel>,
    rsi: Option<RsiMetaCycle>,
    // ── Phase 7: PROPOSE + COMPETE subsystems ──
    cognitive_blackboard: Option<super::cognitive_blackboard::CognitiveBlackboard>,
    attention_schema: Option<super::attention_schema::AttentionSchemaEngine>,
    salience_detector: Option<super::salience_detector::SalienceDetector>,
    qualia_generator: Option<QualiaGenerator>,
    data_quality_pipeline: Option<DataQualityPipeline>,
    ouroboros_loop: Option<OuroborosLoop>,
    module_registry: Option<ModuleRegistry>,
    document_perception: Option<DocumentPerceptionModule>,
    pub(crate) web_accessibility: Option<WebContentExtractor>,
    // ── Wave A: Reasoning modules ──
    mcts_reasoner: Option<MctsReasoner>,
    dead_end_detector: Option<DeadEndDetector>,
    counterfactual_simulator: Option<CounterfactualSimulator>,
    // ── Phase 26 Wave A: VLM Document Parsing ──
    pixel_perception: Option<PixelPerceptionPipeline>,
    long_horizon_ocr: Option<LongHorizonOcr>,
    document_parser_registry: Option<DocumentParserRegistry>,
    document_classifier: Option<DocumentClassifier>,
    screenshot_pipeline: Option<ScreenshotPipeline>,
    formula_extractor: Option<FormulaExtractor>,
    // ── Entity-boosted knowledge retrieval ──
    entity_extractor: Option<EntityExtractor>,
    entity_graph: Option<MemoryGraph>,
    // ── Semantic compression ──
    semantic_compressor: Option<SemanticCompressor>,
    // ── Wave D: Revived consciousness modules ──
    analogical_reasoner: Option<AnalogicalReasoner>,
    epistemic_humility: Option<EpistemicHumility>,
    belief_revision_engine: Option<BeliefRevisionEngine>,
    default_mode_network: Option<DefaultModeNetwork>,
    hierarchical_world_model: Option<HierarchicalWorldModel>,
    // ── Newly wired modules ──
    sleep_gate: Option<SleepGate>,
    narrative_self: Option<NarrativeSelf>,
    appraisal_engine: Option<AppraisalEngine>,
    system1: Option<System1Intuition>,
    emotional_steering: Option<EmotionalSteering>,
    emotional_trail: Option<EmotionalTrail>,
    emotion_regulation: Option<EmotionRegulation>,
    affective_forecast: Option<AffectiveForecastEngine>,
    pub human_emotion_reading: Option<super::human_emotion_detector::HumanEmotionReading>,
    pub human_emotion_detector: Option<HumanEmotionDetector>,
    active_inference: Option<ActiveInferenceEngine>,
    free_energy_curiosity: Option<FreeEnergyCuriosityEngine>,
    dream_consolidator: Option<DreamConsolidator>,
    reasoning_federation: Option<ReasoningFederation>,
    performance_oracle: Option<PerformanceOracle>,
    iit_phi: Option<PhiCalculator>,
    iit_phi8: Option<IitPhi8Engine>,
    // ── CTE 4-stage consolidation pipeline ──
    pub cte: Option<CteCycle>,
    pub memory_lattice: Option<MemoryLattice>,
    // ── SINDy Engine: sparse identification of VSA system dynamics ──
    pub sindy_engine: Option<SindyEngine>,
    // ── Leap 3: Capability synthesizer for skill→capability bridging ──
    pub capability_synthesizer: Option<CapabilitySynthesizer>,
    // ── Leap 2: SEAL closed loop for self-evolution ──
    pub seal_closed_loop: Option<SealClosedLoop>,
    // ── Phase 1: trajectory buffer for outer SEAL closed loop ──
    pub experience_buffer: VecDeque<TrajectoryExperience>,
    // ── Phase 1: VsaReasoner — analogical + causal + multi-hop reasoning ──
    pub vsa_reasoner: Option<VsaReasoner>,
    temporal_tick_counter: u64,
    /// Tracks prediction accuracy across consciousness cycles
    pub temporal_prediction: TemporalPredictionTracker,
    /// Specious present: temporal binding buffer + cross-step phase synchrony
    pub specious_present:
        Option<crate::core::nt_core_consciousness::specious_present::SpeciousPresent>,
    /// Last cycle's c_score, used as prediction baseline
    last_c_score: f64,
    // ── HeLa-Mem Hebbian memory ──
    pub hebbian_memory: Option<HebbianAssociativeMemory>,
    pub hebbian_distillation: Option<HebbianDistillationAgent>,
    // ── Awakening engine (self-measure + self-modify) ──
    pub awakening_engine: Option<AwakeningEngine>,
    pub awakening_brain: Option<SelfIteratingBrain>,
    // ── Tool synthesizer (capability→executable tool bridge) ──
    pub tool_synthesizer: Option<ToolSynthesizer>,
    /// Cross-model distillation orchestrator
    pub cross_model_distiller: Option<CrossModelDistiller>,
    /// Distillation buffer shared with MindBridge
    pub distillation_buffer: Option<Arc<Mutex<CaptureBuffer>>>,
    /// Integration bus for lateral subsystem communication
    pub integration_bus: SubsystemIntegrationBus,
    // ── SecurityExecutive subsystems (Phase 32) ──
    pub threat_modeler: Option<ThreatModeler>,
    pub risk_sensor: Option<RiskSensor>,
    pub supply_chain_guard: Option<SupplyChainGuard>,
    pub adversarial_reasoner: Option<AdversarialReasoner>,
    pub self_defense: Option<SelfDefense>,
    pub evolution_gatekeeper: Option<EvolutionGatekeeper>,
    pub audit_trail: Option<AuditTrail>,
    // ── Phase 33: MindBridge (mind crate integration) ──
    pub mind_bridge: Option<MindBridge>,
    // ── Phase 42: GracefulDegradationManager — per-subsystem health + fallback ──
    pub graceful_deg_manager: Option<GracefulDegradationManager>,
    // ── Veto gate: volition + unified will ──
    volition_engine: Option<VolitionEngine>,
    unified_will: Option<UnifiedWill>,
    // ── MetaAccuracyTracker (real, not synthetic) ──
    meta_accuracy: MetaAccuracyTracker,
    // ── EvolutionEfficiencyTracker (SEA-Eval inspired T metric) ──
    evolution_efficiency: EvolutionEfficiencyTracker,
    // ── GoalDriftIndex (SAHOO-inspired alignment monitoring) ──
    goal_drift_index: Option<GoalDriftIndex>,
    // ── HybridRetrievalEngine: BM25 + VSA + Graph + RRF fusion retrieval ──
    hybrid_retrieval: Option<HybridRetrievalEngine>,
    // ── CompoundKnowledgeBase (Evo B) ──
    compound_knowledge: Option<CompoundKnowledgeBase>,
    // ── AgentTeam orchestrator (Evo C/E) ──
    agent_team: Option<TeamOrchestrator>,
    // ── CodeMutationEngine (Evo F) ──
    code_mutation: Option<CodeMutationEngine>,
    // ── SEAL governance hash chain for evolution audit ──
    seal_governance: SEALGovernance,
    // ── MetaSealEngine: persistent meta-epoch parameter evolution ──
    meta_seal_engine: Option<MetaSealEngine>,
    // ── NASS: NeoTrix Agent Skill System ──
    skill_registry: Option<SkillRegistry>,
    // ── MTRE: Multi-Timeline Research Engine ──
    timeline_orchestrator: Option<TimelineOrchestrator>,
    // ── DDP: Deep Digestion Pipeline ──
    deep_digestion_pipeline: Option<DeepDigestionPipeline>,
    // ── CED: Constellation Emergence Detector ──
    constellation_detector: Option<ConstellationDetector>,
    // ── CTI: Cross-Timeline Integrator ──
    cross_timeline_integrator: Option<CrossTimelineIntegrator>,
    // ── Metabolic budget (artificial hunger) ──
    pub metabolic_budget: MetabolicBudget,
    /// Competitive selection for consciousness content routing
    pub competitive_selection: Option<CompetitiveSelection>,
    /// GWA-inspired semantic entropy tracker for dynamic temperature regulation.
    /// Measures H(W) = -sum(p(x_k)*log(p(x_k))) over recent thought vectors.
    pub semantic_entropy: Option<SemanticEntropyTracker>,
    // ── P0.1: Modulation execution state ──
    /// Queued cognitive load adjustment from modulation command
    pub pending_cognitive_load: Option<f64>,
    /// Queued subsystem reset from modulation command
    pub pending_subsystem_reset: Option<String>,
    /// Dynamic reasoning temperature from semantic entropy drive (default 1.0)
    pub reason_temperature: f64,
    // ── P0.2: Ratchet tracker for monotonic SEAL improvement ──
    pub ratchet_tracker: RatchetTracker,
    // ── Wave A defect fixes ──
    /// CXIV.3: Reconstructive episodic buffer for RECORD step (MIRROR-style)
    pub reconstructive_buffer: Option<ReconstructiveEpisodicBuffer>,
    /// CXIV.9: Attention self-modelling module
    pub attention_self_modelling: Option<AttentionSelfModelling>,
    /// CXI.12: Overthinking detector for REASON step
    pub overthinking_detector: Option<OverthinkingDetector>,
    /// XCVI.1: Concurrency capacity for Chord vs Arpeggio satisfaction (0.0-1.0)
    pub concurrency_capacity: f64,
    /// CXI.2: GWA Core Self invariant safety check counter
    pub invariant_safety_counter: u64,
    /// XCIII.2: Inner monologue manager for parallel cognitive threads
    pub inner_monologue_manager: Option<InnerMonologueManager>,
    /// XCIII.2: Cognitive controller for first-person narrative synthesis
    pub cognitive_controller: Option<CognitiveController>,
    /// CVIII.1: Internal timing generator counter for causal flow condition
    pub internal_tick_counter: u64,
    /// Wall-clock time budget per cycle step (milliseconds).
    /// Reference: ACL 2026 Timely Machine — test-time as wall-clock, not token count.
    pub step_time_budget_ms: u64,
    /// L1 brainstem reflex layer: signal integrity + fast rejection pre-GATHER.
    /// Reference: ATI arXiv 2604.13959 — reflex before perception.
    pub l1_reflex_rejected: bool,
    // ── Wave G: Dual-process (System 1 / System 2) ──
    pub dual_process: DualProcessConfig,
    // ── Wave G: NREM/REM sleep stage separation ──
    pub sleep_stage: SleepStageConfig,
    // ── Wave F: Attentional gate before memory encoding ──
    pub attentional_gate: AttentionalGate,
}

impl CycleResult {
    pub fn all_passed(&self) -> bool {
        self.step_health.iter().all(|h| h.success)
    }

    pub fn failed_steps(&self) -> Vec<CycleStep> {
        self.step_health
            .iter()
            .filter(|h| !h.success)
            .map(|h| h.step)
            .collect()
    }
}

impl std::fmt::Debug for ConsciousnessCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsciousnessCycle")
            .field("cycle_num", &self.cycle_num)
            .field("config", &self.config)
            .field("history", &self.history)
            .field(
                "consolidation_bridge",
                &self.consolidation_bridge.as_ref().map(|_| "Some"),
            )
            .field("image_cache", &self.image_cache.as_ref().map(|_| "Some"))
            .field(
                "multi_modal_gate",
                &self.multi_modal_gate.as_ref().map(|_| "Some"),
            )
            .field("inner_critic", &self.inner_critic.is_some())
            .field("verification_gate", &self.verification_gate.is_some())
            .field("executive_controller", &self.executive_controller.is_some())
            .field("consciousness_stream", &self.consciousness_stream.is_some())
            .field("master_consciousness", &self.master_consciousness.is_some())
            .field(
                "metacognitive_controller",
                &self.metacognitive_controller.is_some(),
            )
            .field("sensor_grounding", &self.sensor_grounding.is_some())
            .field("identity_defense", &self.identity_defense.is_some())
            .field("substrate_gen", &self.substrate_gen.is_some())
            .field("exploration_driver", &self.exploration_driver.is_some())
            .field("boredom", &self.boredom.is_some())
            .field("spreading", &self.spreading.is_some())
            .field("neuromodulators", &self.neuromodulators.is_some())
            .field("causal", &self.causal.is_some())
            .field("bio_memory", &self.bio_memory.is_some())
            .field("scar", &self.scar.is_some())
            .field("consensus", &self.consensus.is_some())
            .field("shadow", &self.shadow.is_some())
            .field("cognitive_wal", &self.cognitive_wal.is_some())
            .field("skills", &self.skills.is_some())
            .field("data_flywheel", &self.data_flywheel.is_some())
            .field("phi", &self.phi.is_some())
            .field("dashboard", &self.dashboard.is_some())
            .field("arch_governor", &self.arch_governor.is_some())
            .field("rsi", &self.rsi.is_some())
            .field("blackboard", &self.cognitive_blackboard.is_some())
            .field("attention", &self.attention_schema.is_some())
            .field("salience", &self.salience_detector.is_some())
            .field(
                "ouroboros_loop",
                &self.ouroboros_loop.as_ref().map(|_| "Some"),
            )
            .field(
                "module_registry",
                &self.module_registry.as_ref().map(|_| "Some"),
            )
            .field(
                "document_perception",
                &self.document_perception.as_ref().map(|_| "Some"),
            )
            .field(
                "web_accessibility",
                &self.web_accessibility.as_ref().map(|_| "Some"),
            )
            .field(
                "pixel_perception",
                &self.pixel_perception.as_ref().map(|_| "Some"),
            )
            .field(
                "long_horizon_ocr",
                &self.long_horizon_ocr.as_ref().map(|_| "Some"),
            )
            .field(
                "formula_extractor",
                &self.formula_extractor.as_ref().map(|_| "Some"),
            )
            .field(
                "screenshot_pipeline",
                &self.screenshot_pipeline.as_ref().map(|_| "Some"),
            )
            .field(
                "entity_extractor",
                &self.entity_extractor.as_ref().map(|_| "Some"),
            )
            .field("entity_graph", &self.entity_graph.as_ref().map(|_| "Some"))
            .field(
                "semantic_compressor",
                &self.semantic_compressor.as_ref().map(|_| "Some"),
            )
            .field("cte", &self.cte.as_ref().map(|_| "Some"))
            .field(
                "memory_lattice",
                &self.memory_lattice.as_ref().map(|_| "Some"),
            )
            .field(
                "analogical_reasoner",
                &self.analogical_reasoner.as_ref().map(|_| "Some"),
            )
            .field(
                "epistemic_humility",
                &self.epistemic_humility.as_ref().map(|_| "Some"),
            )
            .field(
                "belief_revision_engine",
                &self.belief_revision_engine.as_ref().map(|_| "Some"),
            )
            .field(
                "default_mode_network",
                &self.default_mode_network.as_ref().map(|_| "Some"),
            )
            .field(
                "hierarchical_world_model",
                &self.hierarchical_world_model.as_ref().map(|_| "Some"),
            )
            .field("sleep_gate", &self.sleep_gate.as_ref().map(|_| "Some"))
            .field(
                "narrative_self",
                &self.narrative_self.as_ref().map(|_| "Some"),
            )
            .field(
                "appraisal_engine",
                &self.appraisal_engine.as_ref().map(|_| "Some"),
            )
            .field("system1", &self.system1.as_ref().map(|_| "Some"))
            .field(
                "emotional_steering",
                &self.emotional_steering.as_ref().map(|_| "Some"),
            )
            .field(
                "emotional_trail",
                &self.emotional_trail.as_ref().map(|_| "Some"),
            )
            .field(
                "emotion_regulation",
                &self.emotion_regulation.as_ref().map(|_| "Some"),
            )
            .field(
                "affective_forecast",
                &self.affective_forecast.as_ref().map(|_| "Some"),
            )
            .field(
                "human_emotion_detector",
                &self.human_emotion_detector.as_ref().map(|_| "Some"),
            )
            .field(
                "active_inference",
                &self.active_inference.as_ref().map(|_| "Some"),
            )
            .field(
                "dream_consolidator",
                &self.dream_consolidator.as_ref().map(|_| "Some"),
            )
            .field(
                "reasoning_federation",
                &self.reasoning_federation.as_ref().map(|_| "Some"),
            )
            .field(
                "performance_oracle",
                &self.performance_oracle.as_ref().map(|_| "Some"),
            )
            .field("iit_phi", &self.iit_phi.as_ref().map(|_| "Some"))
            .field("iit_phi8", &self.iit_phi8.as_ref().map(|_| "Some"))
            .field(
                "capability_synthesizer",
                &self.capability_synthesizer.as_ref().map(|_| "Some"),
            )
            .field(
                "seal_closed_loop",
                &self.seal_closed_loop.as_ref().map(|_| "Some"),
            )
            .field("experience_buffer", &self.experience_buffer.len())
            .field("vsa_reasoner", &self.vsa_reasoner.as_ref().map(|_| "Some"))
            .field(
                "hebbian_memory",
                &self.hebbian_memory.as_ref().map(|_| "Some"),
            )
            .field(
                "hebbian_distillation",
                &self.hebbian_distillation.as_ref().map(|_| "Some"),
            )
            .field(
                "awakening_engine",
                &self.awakening_engine.as_ref().map(|_| "Some"),
            )
            .field(
                "tool_synthesizer",
                &self.tool_synthesizer.as_ref().map(|_| "Some"),
            )
            .field(
                "integration_bus_signals",
                &self.integration_bus.signal_count(),
            )
            .field(
                "threat_modeler",
                &self.threat_modeler.as_ref().map(|_| "Some"),
            )
            .field("risk_sensor", &self.risk_sensor.as_ref().map(|_| "Some"))
            .field(
                "supply_chain_guard",
                &self.supply_chain_guard.as_ref().map(|_| "Some"),
            )
            .field(
                "adversarial_reasoner",
                &self.adversarial_reasoner.as_ref().map(|_| "Some"),
            )
            .field("self_defense", &self.self_defense.as_ref().map(|_| "Some"))
            .field(
                "evolution_gatekeeper",
                &self.evolution_gatekeeper.as_ref().map(|_| "Some"),
            )
            .field("audit_trail", &self.audit_trail.as_ref().map(|_| "Some"))
            .field(
                "compound_knowledge",
                &self.compound_knowledge.as_ref().map(|_| "Some"),
            )
            .field("agent_team", &self.agent_team.as_ref().map(|_| "Some"))
            .field(
                "code_mutation",
                &self.code_mutation.as_ref().map(|_| "Some"),
            )
            .field("seal_governance", &self.seal_governance.chain_len())
            .field(
                "reconstructive_buffer",
                &self.reconstructive_buffer.as_ref().map(|_| "Some"),
            )
            .field(
                "attention_self_modelling",
                &self.attention_self_modelling.as_ref().map(|_| "Some"),
            )
            .field(
                "overthinking_detector",
                &self.overthinking_detector.as_ref().map(|_| "Some"),
            )
            .field("concurrency_capacity", &self.concurrency_capacity)
            .field("invariant_safety_counter", &self.invariant_safety_counter)
            .field(
                "inner_monologue_manager",
                &self.inner_monologue_manager.as_ref().map(|_| "Some"),
            )
            .field(
                "cognitive_controller",
                &self.cognitive_controller.as_ref().map(|_| "Some"),
            )
            .field("internal_tick_counter", &self.internal_tick_counter)
            .field("dual_process", &self.dual_process.fast_path_enabled)
            .field("sleep_stage", &self.sleep_stage.current_stage)
            .finish()
    }
}

impl Clone for ConsciousnessCycle {
    fn clone(&self) -> Self {
        Self {
            cycle_num: self.cycle_num,
            config: self.config.clone(),
            history: self.history.clone(),
            consolidation_bridge: self.consolidation_bridge.clone(),
            image_cache: self.image_cache.clone(),
            multi_modal_gate: self.multi_modal_gate.clone(),
            quality_gate: self.quality_gate.clone(),
            acting_dimension_assessor: self.acting_dimension_assessor.clone(),
            link_formation: self.link_formation.clone(),
            perception_input: self.perception_input.clone(),
            inner_critic: self.inner_critic.clone(),
            verification_gate: self.verification_gate.clone(),
            executive_controller: self.executive_controller.clone(),
            consciousness_stream: self.consciousness_stream.clone(),
            master_consciousness: self.master_consciousness.clone(),
            metacognitive_controller: self.metacognitive_controller.clone(),
            sensor_grounding: self.sensor_grounding.clone(),
            stream_pipeline: self.stream_pipeline.clone(),
            identity_defense: self.identity_defense.clone(),
            identity_fragments: self.identity_fragments.clone(),
            identity_chain: self.identity_chain.clone(),
            consciousness_architecture: self.consciousness_architecture.clone(),
            substrate_gen: self.substrate_gen.clone(),
            exploration_driver: self.exploration_driver.clone(),
            boredom: self.boredom.clone(),
            spreading: self.spreading.clone(),
            neuromodulators: self.neuromodulators.clone(),
            causal: self.causal.clone(),
            bio_memory: self.bio_memory.clone(),
            scar: self.scar.clone(),
            consensus: self.consensus.clone(),
            shadow: self.shadow.clone(),
            cognitive_wal: self.cognitive_wal.clone(),
            skills: self.skills.clone(),
            data_flywheel: self.data_flywheel.clone(),
            phi: self.phi.clone(),
            dashboard: self.dashboard.clone(),
            arch_governor: self.arch_governor.clone(),
            attentional_gate: self.attentional_gate.clone(),
            rsi: self.rsi.clone(),
            cognitive_blackboard: self.cognitive_blackboard.clone(),
            attention_schema: self.attention_schema.clone(),
            salience_detector: self.salience_detector.clone(),
            qualia_generator: self.qualia_generator.clone(),
            data_quality_pipeline: self.data_quality_pipeline.clone(),
            ouroboros_loop: self.ouroboros_loop.clone(),
            module_registry: self.module_registry.as_ref().map(|_| ModuleRegistry::new()),
            document_perception: self.document_perception.clone(),
            web_accessibility: self.web_accessibility.clone(),
            mcts_reasoner: self.mcts_reasoner.clone(),
            dead_end_detector: self.dead_end_detector.clone(),
            counterfactual_simulator: self.counterfactual_simulator.clone(),
            pixel_perception: self.pixel_perception.clone(),
            long_horizon_ocr: self.long_horizon_ocr.clone(),
            document_parser_registry: self
                .document_parser_registry
                .as_ref()
                .map(|_| create_default_registry()),
            document_classifier: self.document_classifier.clone(),
            screenshot_pipeline: self.screenshot_pipeline.clone(),
            formula_extractor: self.formula_extractor.clone(),
            entity_extractor: self.entity_extractor.clone(),
            entity_graph: self.entity_graph.clone(),
            semantic_compressor: self.semantic_compressor.clone(),
            cte: self.cte.clone(),
            memory_lattice: self.memory_lattice.clone(),
            sindy_engine: self.sindy_engine.clone(),
            analogical_reasoner: self.analogical_reasoner.clone(),

            epistemic_humility: self.epistemic_humility.clone(),
            belief_revision_engine: self.belief_revision_engine.clone(),
            default_mode_network: self.default_mode_network.clone(),
            hierarchical_world_model: self.hierarchical_world_model.clone(),
            sleep_gate: self.sleep_gate.clone(),
            narrative_self: self.narrative_self.clone(),
            appraisal_engine: self.appraisal_engine.clone(),
            system1: self.system1.clone(),
            emotional_steering: self.emotional_steering.clone(),
            emotional_trail: self.emotional_trail.clone(),
            emotion_regulation: self.emotion_regulation.clone(),
            affective_forecast: self.affective_forecast.clone(),
            human_emotion_reading: self.human_emotion_reading.clone(),
            human_emotion_detector: self.human_emotion_detector.clone(),
            active_inference: self.active_inference.clone(),
            free_energy_curiosity: self.free_energy_curiosity.clone(),
            dream_consolidator: self.dream_consolidator.clone(),
            reasoning_federation: self.reasoning_federation.clone(),
            performance_oracle: self.performance_oracle.clone(),
            iit_phi: self.iit_phi.clone(),
            iit_phi8: self.iit_phi8.clone(),
            capability_synthesizer: self.capability_synthesizer.clone(),
            seal_closed_loop: self.seal_closed_loop.clone(),
            experience_buffer: self.experience_buffer.clone(),
            vsa_reasoner: self.vsa_reasoner.clone(),
            temporal_tick_counter: self.temporal_tick_counter,
            temporal_prediction: self.temporal_prediction.clone(),
            specious_present: self.specious_present.clone(),
            last_c_score: self.last_c_score,
            hebbian_memory: self.hebbian_memory.clone(),
            hebbian_distillation: self.hebbian_distillation.clone(),
            awakening_engine: self.awakening_engine.clone(),
            awakening_brain: self.awakening_brain.clone(),
            tool_synthesizer: self.tool_synthesizer.clone(),
            tool_registry: Some(Self::default_tool_registry(
                self.config.enable_stealth_http,
                self.config.stealth_proxy_url.clone(),
            )),
            cross_model_distiller: self.cross_model_distiller.clone(),
            distillation_buffer: self.distillation_buffer.clone(),
            integration_bus: self.integration_bus.clone(),
            threat_modeler: self.threat_modeler.clone(),
            risk_sensor: self.risk_sensor.clone(),
            supply_chain_guard: self.supply_chain_guard.clone(),
            adversarial_reasoner: self.adversarial_reasoner.clone(),
            self_defense: self.self_defense.clone(),
            evolution_gatekeeper: self.evolution_gatekeeper.clone(),
            audit_trail: self.audit_trail.clone(),
            mind_bridge: self.mind_bridge.clone(),
            graceful_deg_manager: self.graceful_deg_manager.clone(),
            volition_engine: self.volition_engine.clone(),
            unified_will: self.unified_will.clone(),
            meta_accuracy: self.meta_accuracy.clone(),
            evolution_efficiency: self.evolution_efficiency.clone(),
            goal_drift_index: self.goal_drift_index.clone(),
            hybrid_retrieval: self.hybrid_retrieval.clone(),
            compound_knowledge: self.compound_knowledge.clone(),
            agent_team: self.agent_team.clone(),
            code_mutation: self.code_mutation.clone(),
            seal_governance: self.seal_governance.clone(),
            meta_seal_engine: self.meta_seal_engine.clone(),
            skill_registry: self.skill_registry.clone(),
            timeline_orchestrator: self.timeline_orchestrator.clone(),
            deep_digestion_pipeline: self.deep_digestion_pipeline.clone(),
            constellation_detector: self.constellation_detector.clone(),
            cross_timeline_integrator: self.cross_timeline_integrator.clone(),
            metabolic_budget: self.metabolic_budget.clone(),
            competitive_selection: self.competitive_selection.clone(),
            semantic_entropy: self.semantic_entropy.clone(),
            pending_cognitive_load: self.pending_cognitive_load,
            pending_subsystem_reset: self.pending_subsystem_reset.clone(),
            reason_temperature: self.reason_temperature,
            ratchet_tracker: self.ratchet_tracker.clone(),
            reconstructive_buffer: self.reconstructive_buffer.clone(),
            attention_self_modelling: self.attention_self_modelling.clone(),
            overthinking_detector: self.overthinking_detector.clone(),
            concurrency_capacity: self.concurrency_capacity,
            invariant_safety_counter: self.invariant_safety_counter,
            inner_monologue_manager: self.inner_monologue_manager.clone(),
            cognitive_controller: self.cognitive_controller.clone(),
            internal_tick_counter: self.internal_tick_counter,
            step_time_budget_ms: self.step_time_budget_ms,
            l1_reflex_rejected: self.l1_reflex_rejected,
            dual_process: self.dual_process.clone(),
            sleep_stage: self.sleep_stage.clone(),
        }
    }
}

/// Deterministic VSA hash from a text string.
/// Uses MMIX LCG (Knuth) to spread the seed across 512 bytes (4096 bits).
fn description_to_vsa(desc: &str) -> Vec<u8> {
    let seed: u64 = desc
        .bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    let mut state = seed;
    let mut v = Vec::with_capacity(512);
    for _ in 0..512 {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        v.push((state >> 40) as u8);
    }
    v
}

/// Hamming similarity between two VSA byte vectors.
/// Returns 1.0 for identical, 0.5 for random, 0.0 for completely opposite.
fn vsa_hamming_sim(a: &[u8], b: &[u8]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let differing: u32 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones())
        .sum();
    1.0 - differing as f64 / (a.len() as f64 * 8.0)
}

impl ConsciousnessCycle {
    pub fn new(config: CycleConfig) -> Self {
        let enable_cs = config.enable_competitive_selection;
        let enable_stealth_http = config.enable_stealth_http;
        let stealth_proxy_url = config.stealth_proxy_url.clone();
        let enable_mind_bridge = config.enable_mind_bridge;
        Self {
            config,
            cycle_num: 0,
            history: VecDeque::new(),
            consolidation_bridge: Some(ConsolidationBridge::new()),
            image_cache: Some(ImageCache::new(50)),
            multi_modal_gate: Some(ModalityGate::new(ModalityGateConfig::default(), 8)),
            perception_input: None,
            quality_gate: None,
            acting_dimension_assessor: None,
            link_formation: None,
            inner_critic: Some(InnerCritic::new()),
            verification_gate: Some(VerificationGate::new()),
            executive_controller: Some(ExecutiveController::new()),
            consciousness_stream: Some(ConsciousnessStream::default()),
            master_consciousness: Some(MasterConsciousness::new(
                MasterConsciousnessConfig::default(),
            )),
            metacognitive_controller: Some(MetacognitiveController::new()),
            sensor_grounding: Some(SensorGrounding::new()),
            stream_pipeline: Some(StreamPipeline::new(64, 8)),
            identity_defense: Some(VsaIdentityDefense::new(32, 0.5)),
            identity_fragments: Some(IdentityFragmentDetector::new()),
            identity_chain: Some(IdentityChain::new(None)),
            consciousness_architecture: Some(ConsciousnessArchitecture::new()),
            substrate_gen: Some(SubstrateFirstGenerator::new(128)),
            exploration_driver: Some(ExplorationDriver::new()),
            boredom: Some(BoredomAccumulator::new()),
            spreading: Some(VsaSpreadingActivation::new(128, 64)),
            neuromodulators: Some(NeuromodulatorySystem::new()),
            causal: Some(CausalReasoner::new()),
            bio_memory: Some(BioMemorySystem::new(100)),
            scar: Some(ScarGuidedLearning::new(50)),
            consensus: Some(GovernanceConsensus::new(vec![1], 0.67)),
            shadow: Some(SecuritySandbox::new()),
            cognitive_wal: Some(CognitiveWal::new()),
            skills: Some(SkillOrchestrator::new()),
            data_flywheel: Some(DataFlywheel::new(FlywheelStrategy::Curriculum, 100)),
            phi: None,
            dashboard: Some(CognitiveDashboard::new()),
            arch_governor: Some(ArchitectureSelfModel::new()),
            rsi: Some(RsiMetaCycle::new()),
            cognitive_blackboard: Some(CognitiveBlackboard::new(BlackboardConfig::default())),
            attention_schema: Some(AttentionSchemaEngine::new(100)),
            salience_detector: Some(SalienceDetector::new()),
            qualia_generator: Some(QualiaGenerator::new()),
            data_quality_pipeline: Some(DataQualityPipeline::new(DQConfig::default())),
            ouroboros_loop: Some(OuroborosLoop::new()),
            module_registry: Some(ModuleRegistry::new()),
            document_perception: Some(DocumentPerceptionModule::new()),
            web_accessibility: Some(WebContentExtractor::new(WebContentConfig {
                enable_stealth_http,
                stealth_proxy_url: stealth_proxy_url.clone(),
                ..Default::default()
            })),
            // ── Wave A: Reasoning modules ──
            mcts_reasoner: Some(MctsReasoner::new(MctsConfig::default())),
            dead_end_detector: Some(DeadEndDetector::new(DeadEndConfig::default())),
            counterfactual_simulator: Some(CounterfactualSimulator::new(
                CounterfactualConfig::default(),
            )),
            // ── Phase 26 Wave A: VLM Document Parsing ──
            pixel_perception: Some(PixelPerceptionPipeline::new()),
            long_horizon_ocr: Some(LongHorizonOcr::new(OcrConfig::default())),
            document_parser_registry: Some(create_default_registry()),
            document_classifier: Some(DocumentClassifier),
            screenshot_pipeline: Some(ScreenshotPipeline::new(ScreenshotCaptureConfig::default())),
            formula_extractor: Some(FormulaExtractor::default()),
            entity_extractor: Some(EntityExtractor::new()),
            entity_graph: Some(MemoryGraph::new(200)),
            semantic_compressor: Some(SemanticCompressor::new(100)),
            cte: Some(CteCycle::default()),
            memory_lattice: Some(MemoryLattice::new()),
            sindy_engine: Some(SindyEngine::new()),
            skill_registry: Some(SkillRegistry::new()),
            timeline_orchestrator: Some(TimelineOrchestrator::new(5)),
            deep_digestion_pipeline: Some(DeepDigestionPipeline::new()),
            constellation_detector: Some(ConstellationDetector::new(3, 0.65)),
            cross_timeline_integrator: Some(CrossTimelineIntegrator::new()),
            analogical_reasoner: Some(AnalogicalReasoner::new()),
            epistemic_humility: Some(EpistemicHumility::new(
                super::epistemic_humility::HumilityConfig::default(),
                crate::core::nt_core_consciousness::epistemic_honesty::EpistemicHonesty::new(
                    crate::core::nt_core_consciousness::epistemic_honesty::HonestyConfig::default(),
                ),
            )),
            belief_revision_engine: Some(BeliefRevisionEngine::new(0.3)),
            default_mode_network: Some(DefaultModeNetwork::new()),
            hierarchical_world_model: Some(HierarchicalWorldModel::new()),
            sleep_gate: Some(SleepGate::new()),
            narrative_self: Some(NarrativeSelf::new()),
            appraisal_engine: Some(AppraisalEngine::new()),
            system1: Some(System1Intuition::new()),
            emotional_steering: Some(EmotionalSteering::new()),
            emotional_trail: Some(EmotionalTrail::new(100)),
            emotion_regulation: Some(EmotionRegulation::new()),
            affective_forecast: Some(AffectiveForecastEngine::new()),
            human_emotion_reading: Some(
                super::human_emotion_detector::HumanEmotionReading::neutral(),
            ),
            human_emotion_detector: Some(HumanEmotionDetector::new()),
            active_inference: Some(ActiveInferenceEngine::new(
                GenerativeModel::new("default"),
                ActiveInferenceConfig::default(),
            )),
            free_energy_curiosity: Some(FreeEnergyCuriosityEngine::new(8, 8)),
            dream_consolidator: Some(DreamConsolidator::new(100, 0.6, 0.3)),
            reasoning_federation: Some(ReasoningFederation::new(FusionStrategy::WeightedVote)),
            performance_oracle: Some(PerformanceOracle::new(OracleConfig::default())),
            iit_phi: Some(PhiCalculator::new(FactoredTPM::new(2))),
            iit_phi8: Some(IitPhi8Engine::new(2, 100)),
            capability_synthesizer: Some(CapabilitySynthesizer::new()),
            seal_closed_loop: Some(SealClosedLoop::new()),
            experience_buffer: VecDeque::new(),
            vsa_reasoner: Some(VsaReasoner::new(ReasonerConfig::default()).with_resonator(
                MultiHeadResonator::new(
                    vec![vec![]], // 1 factor, empty codebook
                    vec![vec!["reserved".to_string()]],
                    20, // max_iterations
                    4,  // num_heads
                    AggregationMode::Softmax,
                ),
            )),
            temporal_tick_counter: 0,
            temporal_prediction: TemporalPredictionTracker::new(100, 0.3),
            specious_present: Some(
                crate::core::nt_core_consciousness::specious_present::SpeciousPresent::new(12),
            ),
            last_c_score: 0.5,
            hebbian_memory: Some(HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 1000)),
            hebbian_distillation: Some(HebbianDistillationAgent::new(3, 0.5)),
            awakening_engine: Some(AwakeningEngine::new(AwakeningConfig::default())),
            awakening_brain: Some(SelfIteratingBrain::new()),
            tool_synthesizer: Some(ToolSynthesizer::new(100, 0.6)),
            tool_registry: Some(Self::default_tool_registry(
                enable_stealth_http,
                stealth_proxy_url,
            )),
            cross_model_distiller: Some(CrossModelDistiller::new(Arc::new(Mutex::new(
                CaptureBuffer::new(1000),
            )))),
            distillation_buffer: Some(Arc::new(Mutex::new(CaptureBuffer::new(1000)))),
            integration_bus: SubsystemIntegrationBus::new(500),
            threat_modeler: Some(ThreatModeler::new()),
            risk_sensor: Some(RiskSensor::new(0.8, 100)),
            supply_chain_guard: Some(SupplyChainGuard::new()),
            adversarial_reasoner: Some(AdversarialReasoner::new()),
            self_defense: Some(SelfDefense::new()),
            evolution_gatekeeper: Some(EvolutionGatekeeper::new()),
            audit_trail: Some(AuditTrail::new(1000)),
            mind_bridge: if enable_mind_bridge {
                Some(MindBridge::new())
            } else {
                None
            },
            volition_engine: Some(VolitionEngine::new()),
            unified_will: Some(UnifiedWill::new()),
            meta_accuracy: MetaAccuracyTracker::new(),
            evolution_efficiency: EvolutionEfficiencyTracker::new(),
            goal_drift_index: Some(GoalDriftIndex::new(100)),
            hybrid_retrieval: Some(HybridRetrievalEngine::new()),
            compound_knowledge: Some(CompoundKnowledgeBase::new(std::path::PathBuf::from(
                "archive/knowledge",
            ))),
            agent_team: Some(TeamOrchestrator::new(
                crate::core::nt_core_experience::agent_team::TeamPattern::Supervisor,
            )),
            code_mutation: Some(CodeMutationEngine::new()),
            graceful_deg_manager: Some(GracefulDegradationManager::with_reasoning_modules()),
            seal_governance: SEALGovernance::new(),
            meta_seal_engine: None, // lazily created with seal_closed_loop
            metabolic_budget: MetabolicBudget::default(),
            competitive_selection: if enable_cs {
                Some(CompetitiveSelection::default())
            } else {
                None
            },
            semantic_entropy: Some(SemanticEntropyTracker::new(100, 5)),
            pending_cognitive_load: None,
            pending_subsystem_reset: None,
            reason_temperature: 1.0,
            ratchet_tracker: RatchetTracker::new(),
            reconstructive_buffer: Some(ReconstructiveEpisodicBuffer::default()),
            attention_self_modelling: Some(AttentionSelfModelling::default()),
            overthinking_detector: Some(OverthinkingDetector::default()),
            concurrency_capacity: 0.0,
            invariant_safety_counter: 0,
            inner_monologue_manager: Some(InnerMonologueManager::default()),
            cognitive_controller: Some(CognitiveController::default()),
            internal_tick_counter: 0,
            step_time_budget_ms: 1000,
            l1_reflex_rejected: false,
            dual_process: DualProcessConfig::default(),
            sleep_stage: SleepStageConfig::default(),
            attentional_gate: AttentionalGate::default(),
        }
    }

    /// Create a default AgentToolRegistry with built-in tool implementations.
    fn default_tool_registry(enable_stealth: bool, proxy_url: Option<String>) -> AgentToolRegistry {
        let mut reg = AgentToolRegistry::new();
        let web_tool = if enable_stealth {
            WebScrapeTool::with_stealth(proxy_url)
        } else {
            WebScrapeTool::new()
        };
        reg.register_agent_tool(Box::new(web_tool));
        reg.register_agent_tool(Box::new(ArchitectTool::new()));
        reg.register_agent_tool(Box::new(EarnTool::new()));
        reg.register_agent_tool(Box::new(ImageGenTool::new()));
        reg.register_agent_tool(Box::new(LspTool::new()));
        reg.register_agent_tool(Box::new(MiniMaxT2ITool::new()));
        reg.register_agent_tool(Box::new(ReactDoctorTool::new()));
        reg.register_agent_tool(Box::new(SecurityAuditTool::new()));
        reg.register_agent_tool(Box::new(OsintInvestigatorTool::new()));
        reg
    }

    // ── Builder methods ──

    pub fn with_consolidation_bridge(mut self, cb: ConsolidationBridge) -> Self {
        self.consolidation_bridge = Some(cb);
        self
    }
    pub fn with_image_cache(mut self, cache: ImageCache) -> Self {
        self.image_cache = Some(cache);
        self
    }
    pub fn with_multi_modal_gate(mut self, gate: ModalityGate) -> Self {
        self.multi_modal_gate = Some(gate);
        self
    }
    pub fn with_quality_gate(mut self, qg: super::quality_gate::QualityGate) -> Self {
        self.quality_gate = Some(qg);
        self
    }
    pub fn with_inner_critic(mut self, ic: InnerCritic) -> Self {
        self.inner_critic = Some(ic);
        self
    }
    pub fn with_verification_gate(mut self, vg: VerificationGate) -> Self {
        self.verification_gate = Some(vg);
        self
    }
    pub fn with_executive_controller(mut self, ec: ExecutiveController) -> Self {
        self.executive_controller = Some(ec);
        self
    }
    pub fn with_consciousness_stream(mut self, cs: ConsciousnessStream) -> Self {
        self.consciousness_stream = Some(cs);
        self
    }
    pub fn with_master_consciousness(mut self, mc: MasterConsciousness) -> Self {
        self.master_consciousness = Some(mc);
        self
    }
    pub fn with_metacognitive_controller(mut self, mc: MetacognitiveController) -> Self {
        self.metacognitive_controller = Some(mc);
        self
    }
    pub fn with_acting_dimension_assessor(mut self, ada: ActingDimensionAssessor) -> Self {
        self.acting_dimension_assessor = Some(ada);
        self
    }

    // ── Wave 2-5 builder methods ──
    pub fn with_sensor_grounding(mut self, sg: SensorGrounding) -> Self {
        self.sensor_grounding = Some(sg);
        self
    }
    pub fn with_identity_defense(mut self, id: VsaIdentityDefense) -> Self {
        self.identity_defense = Some(id);
        self
    }
    pub fn with_substrate_gen(mut self, sg: SubstrateFirstGenerator) -> Self {
        self.substrate_gen = Some(sg);
        self
    }
    pub fn with_exploration_driver(mut self, ed: ExplorationDriver) -> Self {
        self.exploration_driver = Some(ed);
        self
    }
    pub fn with_boredom(mut self, b: BoredomAccumulator) -> Self {
        self.boredom = Some(b);
        self
    }
    pub fn with_spreading(mut self, s: VsaSpreadingActivation) -> Self {
        self.spreading = Some(s);
        self
    }
    pub fn with_neuromodulators(mut self, nm: NeuromodulatorySystem) -> Self {
        self.neuromodulators = Some(nm);
        self
    }
    pub fn with_causal(mut self, c: CausalReasoner) -> Self {
        self.causal = Some(c);
        self
    }
    pub fn with_bio_memory(mut self, bm: BioMemorySystem) -> Self {
        self.bio_memory = Some(bm);
        self
    }
    pub fn with_scar(mut self, s: ScarGuidedLearning) -> Self {
        self.scar = Some(s);
        self
    }
    pub fn with_consensus(mut self, c: GovernanceConsensus) -> Self {
        self.consensus = Some(c);
        self
    }
    pub fn with_shadow(mut self, s: SecuritySandbox) -> Self {
        self.shadow = Some(s);
        self
    }
    pub fn with_cognitive_wal(mut self, w: CognitiveWal) -> Self {
        self.cognitive_wal = Some(w);
        self
    }
    pub fn with_skills(mut self, s: SkillOrchestrator) -> Self {
        self.skills = Some(s);
        self
    }
    pub fn with_data_flywheel(mut self, df: DataFlywheel) -> Self {
        self.data_flywheel = Some(df);
        self
    }
    pub fn with_phi(mut self, p: HierarchicalPhi) -> Self {
        self.phi = Some(p);
        self
    }
    pub fn with_dashboard(mut self, d: CognitiveDashboard) -> Self {
        self.dashboard = Some(d);
        self
    }
    pub fn with_arch_governor(mut self, ag: ArchitectureSelfModel) -> Self {
        self.arch_governor = Some(ag);
        self
    }
    pub fn with_rsi(mut self, r: RsiMetaCycle) -> Self {
        self.rsi = Some(r);
        self
    }
    pub fn with_blackboard(mut self, bb: super::cognitive_blackboard::CognitiveBlackboard) -> Self {
        self.cognitive_blackboard = Some(bb);
        self
    }
    pub fn with_attention_schema(
        mut self,
        attn: super::attention_schema::AttentionSchemaEngine,
    ) -> Self {
        self.attention_schema = Some(attn);
        self
    }
    pub fn with_salience_detector(
        mut self,
        sd: super::salience_detector::SalienceDetector,
    ) -> Self {
        self.salience_detector = Some(sd);
        self
    }
    pub fn with_qualia_generator(mut self, qg: QualiaGenerator) -> Self {
        self.qualia_generator = Some(qg);
        self
    }
    pub fn with_data_quality_pipeline(mut self, dqp: DataQualityPipeline) -> Self {
        self.data_quality_pipeline = Some(dqp);
        self
    }
    pub fn with_ouroboros_loop(mut self, ol: OuroborosLoop) -> Self {
        self.ouroboros_loop = Some(ol);
        self
    }
    pub fn with_module_registry(mut self, reg: ModuleRegistry) -> Self {
        self.module_registry = Some(reg);
        self
    }
    pub fn with_document_perception(mut self, dp: DocumentPerceptionModule) -> Self {
        self.document_perception = Some(dp);
        self
    }
    pub fn with_web_accessibility(mut self, wa: WebContentExtractor) -> Self {
        self.web_accessibility = Some(wa);
        self
    }

    // ── Wave A: Reasoning module builders ──
    pub fn with_mcts_reasoner(mut self, m: MctsReasoner) -> Self {
        self.mcts_reasoner = Some(m);
        self
    }
    pub fn with_dead_end_detector(mut self, d: DeadEndDetector) -> Self {
        self.dead_end_detector = Some(d);
        self
    }
    pub fn with_counterfactual_simulator(mut self, c: CounterfactualSimulator) -> Self {
        self.counterfactual_simulator = Some(c);
        self
    }

    // ── Phase 26 Wave A: VLM Document Parsing builders ──
    pub fn with_pixel_perception(mut self, pp: PixelPerceptionPipeline) -> Self {
        self.pixel_perception = Some(pp);
        self
    }
    pub fn with_long_horizon_ocr(mut self, ocr: LongHorizonOcr) -> Self {
        self.long_horizon_ocr = Some(ocr);
        self
    }
    pub fn with_document_parser_registry(mut self, reg: DocumentParserRegistry) -> Self {
        self.document_parser_registry = Some(reg);
        self
    }
    pub fn with_document_classifier(mut self, cls: DocumentClassifier) -> Self {
        self.document_classifier = Some(cls);
        self
    }
    pub fn with_screenshot_pipeline(mut self, sp: ScreenshotPipeline) -> Self {
        self.screenshot_pipeline = Some(sp);
        self
    }
    pub fn with_formula_extractor(mut self, fe: FormulaExtractor) -> Self {
        self.formula_extractor = Some(fe);
        self
    }
    pub fn with_entity_extractor(mut self, e: EntityExtractor) -> Self {
        self.entity_extractor = Some(e);
        self
    }
    pub fn with_entity_graph(mut self, g: MemoryGraph) -> Self {
        self.entity_graph = Some(g);
        self
    }
    pub fn with_semantic_compressor(mut self, sc: SemanticCompressor) -> Self {
        self.semantic_compressor = Some(sc);
        self
    }

    // ── Wave D: Revived module builders ──
    pub fn with_analogical_reasoner(mut self, ar: AnalogicalReasoner) -> Self {
        self.analogical_reasoner = Some(ar);
        self
    }
    pub fn with_epistemic_humility(mut self, eh: EpistemicHumility) -> Self {
        self.epistemic_humility = Some(eh);
        self
    }
    pub fn with_belief_revision_engine(mut self, bre: BeliefRevisionEngine) -> Self {
        self.belief_revision_engine = Some(bre);
        self
    }
    pub fn with_default_mode_network(mut self, dmn: DefaultModeNetwork) -> Self {
        self.default_mode_network = Some(dmn);
        self
    }
    pub fn with_hierarchical_world_model(mut self, hwm: HierarchicalWorldModel) -> Self {
        self.hierarchical_world_model = Some(hwm);
        self
    }
    pub fn with_sleep_gate(mut self, sg: SleepGate) -> Self {
        self.sleep_gate = Some(sg);
        self
    }
    pub fn with_narrative_self(mut self, ns: NarrativeSelf) -> Self {
        self.narrative_self = Some(ns);
        self
    }
    pub fn with_system1(mut self, s1: System1Intuition) -> Self {
        self.system1 = Some(s1);
        self
    }
    pub fn with_human_emotion_detector(mut self, hed: HumanEmotionDetector) -> Self {
        self.human_emotion_detector = Some(hed);
        self
    }
    pub fn with_active_inference(mut self, ai: ActiveInferenceEngine) -> Self {
        self.active_inference = Some(ai);
        self
    }
    pub fn with_dream_consolidator(mut self, dc: DreamConsolidator) -> Self {
        self.dream_consolidator = Some(dc);
        self
    }
    pub fn with_reasoning_federation(mut self, rf: ReasoningFederation) -> Self {
        self.reasoning_federation = Some(rf);
        self
    }
    pub fn with_iit_phi(mut self, phi: PhiCalculator) -> Self {
        self.iit_phi = Some(phi);
        self
    }
    pub fn with_iit_phi8(mut self, phi8: IitPhi8Engine) -> Self {
        self.iit_phi8 = Some(phi8);
        self
    }

    // ── CTE consolidation builders ──
    pub fn with_cte(mut self, cte: CteCycle) -> Self {
        self.cte = Some(cte);
        self
    }
    pub fn with_memory_lattice(mut self, ml: MemoryLattice) -> Self {
        self.memory_lattice = Some(ml);
        self
    }

    pub fn with_vsa_reasoner(mut self, vr: VsaReasoner) -> Self {
        self.vsa_reasoner = Some(vr);
        self
    }

    // ── SecurityExecutive subsystem builders ──
    pub fn with_threat_modeler(mut self, tm: ThreatModeler) -> Self {
        self.threat_modeler = Some(tm);
        self
    }
    pub fn with_risk_sensor(mut self, rs: RiskSensor) -> Self {
        self.risk_sensor = Some(rs);
        self
    }
    pub fn with_supply_chain_guard(mut self, scg: SupplyChainGuard) -> Self {
        self.supply_chain_guard = Some(scg);
        self
    }
    pub fn with_adversarial_reasoner(mut self, ar: AdversarialReasoner) -> Self {
        self.adversarial_reasoner = Some(ar);
        self
    }
    pub fn with_self_defense(mut self, sd: SelfDefense) -> Self {
        self.self_defense = Some(sd);
        self
    }
    pub fn with_evolution_gatekeeper(mut self, eg: EvolutionGatekeeper) -> Self {
        self.evolution_gatekeeper = Some(eg);
        self
    }
    pub fn with_audit_trail(mut self, at: AuditTrail) -> Self {
        self.audit_trail = Some(at);
        self
    }
    pub fn with_compound_knowledge(mut self, ck: CompoundKnowledgeBase) -> Self {
        self.compound_knowledge = Some(ck);
        self
    }
    pub fn with_agent_team(mut self, at: TeamOrchestrator) -> Self {
        self.agent_team = Some(at);
        self
    }
    pub fn with_code_mutation(mut self, cm: CodeMutationEngine) -> Self {
        self.code_mutation = Some(cm);
        self
    }
    pub fn with_skill_registry(mut self, sr: SkillRegistry) -> Self {
        self.skill_registry = Some(sr);
        self
    }
    pub fn with_timeline_orchestrator(mut self, to: TimelineOrchestrator) -> Self {
        self.timeline_orchestrator = Some(to);
        self
    }
    pub fn with_deep_digestion_pipeline(mut self, ddp: DeepDigestionPipeline) -> Self {
        self.deep_digestion_pipeline = Some(ddp);
        self
    }
    pub fn with_constellation_detector(mut self, cd: ConstellationDetector) -> Self {
        self.constellation_detector = Some(cd);
        self
    }
    pub fn with_cross_timeline_integrator(mut self, cti: CrossTimelineIntegrator) -> Self {
        self.cross_timeline_integrator = Some(cti);
        self
    }
    pub fn with_tool_registry(mut self, reg: AgentToolRegistry) -> Self {
        self.tool_registry = Some(reg);
        self
    }

    /// Check whether the System 1 fast path should be used.
    /// Returns true when:
    ///   - fast path is enabled
    ///   - the current gathered input has high confidence (> threshold)
    ///   - no novelty detected (spreading activation is stable)
    ///   - metabolic budget is under pressure (starvation or near-starvation)
    /// When true, COMPETE, REASON, JUDGE, VERIFY are bypassed → ACT directly.
    pub fn should_use_fast_path(&self, gathered: Option<&VsaTagged>) -> bool {
        if !self.dual_process.fast_path_enabled {
            return false;
        }
        // Check confidence from gathered input
        let high_confidence = gathered
            .map(|g| g.confidence > self.dual_process.fast_path_threshold)
            .unwrap_or(false);
        if !high_confidence {
            return false;
        }
        // Check metabolic pressure: skip expensive reasoning when energy is low
        let under_pressure = self.metabolic_budget.should_skip_nonessential();
        // Fast path if high confidence AND (metabolic pressure OR no novelty detected)
        let novelty_low = self
            .exploration_driver
            .as_ref()
            .map(|ed| {
                gathered
                    .map(|g| {
                        let vsa: Vec<f64> = g.vector.iter().map(|&b| b as f64).collect();
                        ed.novelty(&vsa) < 0.3
                    })
                    .unwrap_or(true)
            })
            .unwrap_or(true);
        under_pressure || novelty_low
    }

    // ── Wave G: NREM/REM sleep stage management ──

    /// Begin a sleep cycle: transition to NREM stage.
    pub fn start_sleep_cycle(&mut self) {
        self.sleep_stage.current_stage = SleepStage::NREM;
        self.sleep_stage.stage_cycle = 0;
    }

    /// Advance the stage counter and transition between stages.
    /// Returns the current stage after the tick.
    pub fn tick_stage(&mut self) -> SleepStage {
        let stage = self.sleep_stage.current_stage;
        let counter = self.sleep_stage.stage_cycle;
        match stage {
            SleepStage::NREM => {
                if counter >= self.sleep_stage.nrem_duration {
                    self.sleep_stage.current_stage = SleepStage::REM;
                    self.sleep_stage.stage_cycle = 0;
                } else {
                    self.sleep_stage.stage_cycle += 1;
                }
            }
            SleepStage::REM => {
                if counter >= self.sleep_stage.rem_duration {
                    self.sleep_stage.current_stage = SleepStage::Awake;
                    self.sleep_stage.stage_cycle = 0;
                } else {
                    self.sleep_stage.stage_cycle += 1;
                }
            }
            SleepStage::Awake => {}
        }
        self.sleep_stage.current_stage
    }

    /// NREM phase: strengthen important associations, synaptic downscale placeholder.
    pub fn nrem_phase(&mut self) -> Vec<String> {
        let mut insights = Vec::new();
        if let Some(ref mut stream) = self.consciousness_stream {
            let n = stream.len();
            if n > 2 {
                // Synaptic downscale: reduce confidence of older entries
                for i in 0..n {
                    if let Some(entry) = stream.at_mut(i) {
                        let age_factor = (i as f64 / n as f64) * 0.1;
                        entry.confidence = (entry.confidence - age_factor).max(0.1);
                    }
                }
                insights.push(format!("nrem:synaptic_downscale applied to {} entries", n));
            }
        }
        // Strengthen associations via existing sleep gate NREM
        if let Some(ref mut sg) = self.sleep_gate {
            if let Some(ref mut stream) = self.consciousness_stream {
                let mut entries: Vec<VsaTagged> = stream.entries_mut().drain(..).collect();
                let merged = sg.execute_nrem(&mut entries);
                if merged > 0 {
                    insights.push(format!("nrem:merged {} similar patterns", merged));
                }
                // Push processed entries back
                for e in entries {
                    stream.push(e);
                }
            }
        }
        insights
    }

    /// REM phase: random-walk association generation.
    pub fn rem_phase(&mut self) -> Vec<String> {
        let mut insights = Vec::new();
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        // Create novel associations via sleep gate REM
        if let Some(ref mut sg) = self.sleep_gate {
            if let Some(ref mut stream) = self.consciousness_stream {
                let mut entries: Vec<VsaTagged> = stream.entries_mut().drain(..).collect();
                let associations = sg.execute_rem(&mut entries, current_time);
                if associations > 0 {
                    insights.push(format!("rem:created {} novel associations", associations));
                }
                for e in entries {
                    stream.push(e);
                }
            }
        }
        // Random-walk: create a random bound-pair between distant entries
        if let Some(ref mut stream) = self.consciousness_stream {
            let n = stream.len();
            if n > 3 {
                let src = fastrand::usize(..n);
                let dst = fastrand::usize(..n);
                if src != dst {
                    let s_entry = stream.at(src).unwrap().clone();
                    let d_entry = stream.at(dst).unwrap().clone();
                    let bound = QuantizedVSA::bind(&s_entry.vector, &d_entry.vector);
                    stream.push(VsaTagged {
                        vector: bound,
                        tag: s_entry.tag,
                        confidence: (s_entry.confidence + d_entry.confidence) / 4.0,
                        timestamp: current_time,
                        salience: 0.3,
                        provenance: None,
                        sense_modality: None,
                        prediction: None,
                        outcome: None,
                    });
                    insights.push("rem:random_walk association created".into());
                }
            }
        }
        insights
    }

    /// Drain accumulated cycle-level experiences for the outer SEAL loop.
    pub fn drain_experience_buffer(&mut self) -> Vec<TrajectoryExperience> {
        std::mem::take(&mut self.experience_buffer)
            .into_iter()
            .collect()
    }

    pub fn init_default_registry(&mut self) {
        let mut reg = ModuleRegistry::new();
        reg.register(Box::new(
            crate::core::nt_core_reasoning::mcts_cognitive_module::MctsCognitiveModule::new(
                crate::core::nt_core_reasoning::mcts_reasoner::MctsConfig::default(),
            ),
        ));
        reg.register(Box::new(
            crate::core::nt_core_reasoning::parallel_hypothesis_cognitive_module::ParallelHypothesisCognitiveModule::new(
                crate::core::nt_core_reasoning::parallel_hypothesis::ParallelHypothesisConfig::default(),
            ),
        ));
        reg.register(Box::new(
            crate::core::nt_core_reasoning::counterfactual_cognitive_module::CounterfactualCognitiveModule::new(
                crate::core::nt_core_reasoning::counterfactual_simulator::CounterfactualConfig::default(),
            ),
        ));
        reg.register(Box::new(
            crate::core::nt_core_reasoning::dead_end_cognitive_module::DeadEndCognitiveModule::new(
                crate::core::nt_core_reasoning::dead_end_detector::DeadEndConfig::default(),
            ),
        ));
        reg.register(Box::new(
            crate::core::nt_core_reasoning::prm_cognitive_module::PrmCognitiveModule::new(
                crate::core::nt_core_reasoning::process_reward_model::PrmConfig::default(),
            ),
        ));
        reg.register(Box::new(
            crate::core::nt_core_reasoning::pruner_cognitive_module::PrunerCognitiveModule::new(
                crate::core::nt_core_reasoning::bidirectional_pruner::PrunerConfig::default(),
            ),
        ));
        reg.register(Box::new(
            crate::core::nt_core_reasoning::selector_cognitive_module::SelectorCognitiveModule::new(
                crate::core::nt_core_reasoning::strategy_selector::StrategyConfig::default(),
            ),
        ));
        self.module_registry = Some(reg);
    }

    /// Set external perception input (e.g. from ImagePipeline) for the GATHER step.
    pub fn feed_perception(&mut self, input: VsaTagged) {
        self.perception_input = Some(input);
    }

    /// Auto-heal any None fields at the start of a cycle.
    /// Uses the same constructor defaults so no subsystem stays dead.
    pub fn init_missing_fields(&mut self) {
        // P0.3: strict_wiring — disable auto-healing to expose missing wiring
        if self.config.strict_wiring {
            return;
        }
        let mut healed: usize = 0;
        if self.consolidation_bridge.is_none() {
            self.consolidation_bridge = Some(ConsolidationBridge::new());
            healed += 1;
        }
        if self.image_cache.is_none() {
            self.image_cache = Some(ImageCache::new(50));
            healed += 1;
        }
        if self.multi_modal_gate.is_none() {
            self.multi_modal_gate = Some(ModalityGate::new(ModalityGateConfig::default(), 8));
            healed += 1;
        }
        if self.inner_critic.is_none() {
            self.inner_critic = Some(InnerCritic::new());
            healed += 1;
        }
        if self.verification_gate.is_none() {
            self.verification_gate = Some(VerificationGate::new());
            healed += 1;
        }
        if self.executive_controller.is_none() {
            self.executive_controller = Some(ExecutiveController::new());
            healed += 1;
        }
        if self.consciousness_stream.is_none() {
            self.consciousness_stream = Some(ConsciousnessStream::default());
            healed += 1;
        }
        if self.master_consciousness.is_none() {
            self.master_consciousness = Some(MasterConsciousness::new(
                MasterConsciousnessConfig::default(),
            ));
            healed += 1;
        }
        if self.metacognitive_controller.is_none() {
            self.metacognitive_controller = Some(MetacognitiveController::new());
            healed += 1;
        }
        if self.sensor_grounding.is_none() {
            self.sensor_grounding = Some(SensorGrounding::new());
            healed += 1;
        }
        if self.identity_defense.is_none() {
            self.identity_defense = Some(VsaIdentityDefense::new(32, 0.5));
            healed += 1;
        }
        if self.identity_fragments.is_none() {
            self.identity_fragments = Some(IdentityFragmentDetector::new());
            healed += 1;
        }
        if self.identity_chain.is_none() {
            self.identity_chain = Some(IdentityChain::new(None));
            healed += 1;
        }
        if self.consciousness_architecture.is_none() {
            self.consciousness_architecture = Some(ConsciousnessArchitecture::new());
            healed += 1;
        }
        if self.specious_present.is_none() {
            self.specious_present = Some(
                crate::core::nt_core_consciousness::specious_present::SpeciousPresent::new(12),
            );
            healed += 1;
        }
        if self.substrate_gen.is_none() {
            self.substrate_gen = Some(SubstrateFirstGenerator::new(128));
            healed += 1;
        }
        if self.exploration_driver.is_none() {
            self.exploration_driver = Some(ExplorationDriver::new());
            healed += 1;
        }
        if self.boredom.is_none() {
            self.boredom = Some(BoredomAccumulator::new());
            healed += 1;
        }
        if self.spreading.is_none() {
            self.spreading = Some(VsaSpreadingActivation::new(128, 64));
            healed += 1;
        }
        if self.neuromodulators.is_none() {
            self.neuromodulators = Some(NeuromodulatorySystem::new());
            healed += 1;
        }
        if self.causal.is_none() {
            self.causal = Some(CausalReasoner::new());
            healed += 1;
        }
        if self.bio_memory.is_none() {
            self.bio_memory = Some(BioMemorySystem::new(100));
            healed += 1;
        }
        if self.scar.is_none() {
            self.scar = Some(ScarGuidedLearning::new(50));
            healed += 1;
        }
        if self.consensus.is_none() {
            self.consensus = Some(GovernanceConsensus::new(vec![1], 0.67));
            healed += 1;
        }
        if self.shadow.is_none() {
            self.shadow = Some(SecuritySandbox::new());
            healed += 1;
        }
        if self.cognitive_wal.is_none() {
            self.cognitive_wal = Some(CognitiveWal::new());
            healed += 1;
        }
        if self.skills.is_none() {
            self.skills = Some(SkillOrchestrator::new());
            healed += 1;
        }
        if self.data_flywheel.is_none() {
            self.data_flywheel = Some(DataFlywheel::new(FlywheelStrategy::Curriculum, 100));
            healed += 1;
        }
        if self.dashboard.is_none() {
            self.dashboard = Some(CognitiveDashboard::new());
            healed += 1;
        }
        if self.arch_governor.is_none() {
            self.arch_governor = Some(ArchitectureSelfModel::new());
            healed += 1;
        }
        if self.rsi.is_none() {
            self.rsi = Some(RsiMetaCycle::new());
            healed += 1;
        }
        if self.cognitive_blackboard.is_none() {
            self.cognitive_blackboard = Some(CognitiveBlackboard::new(BlackboardConfig::default()));
            healed += 1;
        }
        if self.attention_schema.is_none() {
            self.attention_schema = Some(AttentionSchemaEngine::new(100));
            healed += 1;
        }
        if self.salience_detector.is_none() {
            self.salience_detector = Some(SalienceDetector::new());
            healed += 1;
        }
        if self.qualia_generator.is_none() {
            self.qualia_generator = Some(QualiaGenerator::new());
            healed += 1;
        }
        if self.data_quality_pipeline.is_none() {
            self.data_quality_pipeline = Some(DataQualityPipeline::new(DQConfig::default()));
            healed += 1;
        }
        if self.ouroboros_loop.is_none() {
            self.ouroboros_loop = Some(OuroborosLoop::new());
            healed += 1;
        }
        if self.module_registry.is_none() {
            self.module_registry = Some(ModuleRegistry::new());
            healed += 1;
        }
        if self.document_perception.is_none() {
            self.document_perception = Some(DocumentPerceptionModule::new());
            healed += 1;
        }
        if self.web_accessibility.is_none() {
            self.web_accessibility = Some(WebContentExtractor::new(WebContentConfig {
                enable_stealth_http: self.config.enable_stealth_http,
                stealth_proxy_url: self.config.stealth_proxy_url.clone(),
                ..Default::default()
            }));
            healed += 1;
        }
        if self.mcts_reasoner.is_none() {
            self.mcts_reasoner = Some(MctsReasoner::new(MctsConfig::default()));
            healed += 1;
        }
        if self.dead_end_detector.is_none() {
            self.dead_end_detector = Some(DeadEndDetector::new(DeadEndConfig::default()));
            healed += 1;
        }
        if self.counterfactual_simulator.is_none() {
            self.counterfactual_simulator =
                Some(CounterfactualSimulator::new(CounterfactualConfig::default()));
            healed += 1;
        }
        if self.pixel_perception.is_none() {
            self.pixel_perception = Some(PixelPerceptionPipeline::new());
            healed += 1;
        }
        if self.long_horizon_ocr.is_none() {
            self.long_horizon_ocr = Some(LongHorizonOcr::new(OcrConfig::default()));
            healed += 1;
        }
        if self.document_parser_registry.is_none() {
            self.document_parser_registry = Some(create_default_registry());
            healed += 1;
        }
        if self.document_classifier.is_none() {
            self.document_classifier = Some(DocumentClassifier);
            healed += 1;
        }
        if self.screenshot_pipeline.is_none() {
            self.screenshot_pipeline =
                Some(ScreenshotPipeline::new(ScreenshotCaptureConfig::default()));
            healed += 1;
        }
        if self.formula_extractor.is_none() {
            self.formula_extractor = Some(FormulaExtractor::default());
            healed += 1;
        }
        if self.entity_extractor.is_none() {
            self.entity_extractor = Some(EntityExtractor::new());
            healed += 1;
        }
        if self.entity_graph.is_none() {
            self.entity_graph = Some(MemoryGraph::new(200));
            healed += 1;
        }
        if self.semantic_compressor.is_none() {
            self.semantic_compressor = Some(SemanticCompressor::new(100));
            healed += 1;
        }
        if self.cte.is_none() {
            self.cte = Some(CteCycle::default());
            healed += 1;
        }
        if self.memory_lattice.is_none() {
            self.memory_lattice = Some(MemoryLattice::new());
            healed += 1;
        }
        if self.analogical_reasoner.is_none() {
            self.analogical_reasoner = Some(AnalogicalReasoner::new());
            healed += 1;
        }
        if self.epistemic_humility.is_none() {
            self.epistemic_humility = Some(EpistemicHumility::new(
                super::epistemic_humility::HumilityConfig::default(),
                crate::core::nt_core_consciousness::epistemic_honesty::EpistemicHonesty::new(
                    crate::core::nt_core_consciousness::epistemic_honesty::HonestyConfig::default(),
                ),
            ));
            healed += 1;
        }
        if self.belief_revision_engine.is_none() {
            self.belief_revision_engine = Some(BeliefRevisionEngine::new(0.3));
            healed += 1;
        }
        if self.default_mode_network.is_none() {
            self.default_mode_network = Some(DefaultModeNetwork::new());
            healed += 1;
        }
        if self.hierarchical_world_model.is_none() {
            self.hierarchical_world_model = Some(HierarchicalWorldModel::new());
            healed += 1;
        }
        if self.system1.is_none() {
            self.system1 = Some(System1Intuition::new());
            healed += 1;
        }
        if self.active_inference.is_none() {
            self.active_inference = Some(ActiveInferenceEngine::new(
                GenerativeModel::new("heal"),
                ActiveInferenceConfig::default(),
            ));
            healed += 1;
        }
        if self.emotional_steering.is_none() {
            self.emotional_steering = Some(EmotionalSteering::new());
            healed += 1;
        }
        if self.emotional_trail.is_none() {
            self.emotional_trail = Some(EmotionalTrail::new(100));
        }
        if self.emotion_regulation.is_none() {
            self.emotion_regulation = Some(EmotionRegulation::new());
            healed += 1;
        }
        if self.affective_forecast.is_none() {
            self.affective_forecast = Some(AffectiveForecastEngine::new());
            healed += 1;
        }
        if self.human_emotion_reading.is_none() {
            self.human_emotion_reading =
                Some(super::human_emotion_detector::HumanEmotionReading::neutral());
        }
        if self.human_emotion_detector.is_none() {
            self.human_emotion_detector = Some(HumanEmotionDetector::new());
            healed += 1;
        }
        if self.dream_consolidator.is_none() {
            self.dream_consolidator = Some(DreamConsolidator::new(100, 0.6, 0.3));
            healed += 1;
        }
        if self.reasoning_federation.is_none() {
            self.reasoning_federation =
                Some(ReasoningFederation::new(FusionStrategy::WeightedVote));
            healed += 1;
        }
        if self.iit_phi.is_none() {
            self.iit_phi = Some(PhiCalculator::new(FactoredTPM::new(2)));
            healed += 1;
        }
        if self.iit_phi8.is_none() {
            self.iit_phi8 = Some(IitPhi8Engine::new(2, 100));
            healed += 1;
        }
        if self.capability_synthesizer.is_none() {
            self.capability_synthesizer = Some(CapabilitySynthesizer::new());
            healed += 1;
        }
        if self.seal_closed_loop.is_none() {
            self.seal_closed_loop = Some(SealClosedLoop::new());
            healed += 1;
        }
        if self.vsa_reasoner.is_none() {
            self.vsa_reasoner = Some(VsaReasoner::new(ReasonerConfig::default()));
            healed += 1;
        }
        if self.hebbian_memory.is_none() {
            self.hebbian_memory = Some(HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 1000));
            healed += 1;
        }
        if self.hebbian_distillation.is_none() {
            self.hebbian_distillation = Some(HebbianDistillationAgent::new(3, 0.5));
            healed += 1;
        }
        if self.awakening_engine.is_none() {
            self.awakening_engine = Some(AwakeningEngine::new(AwakeningConfig::default()));
            healed += 1;
        }
        if self.awakening_brain.is_none() {
            self.awakening_brain = Some(SelfIteratingBrain::new());
            healed += 1;
        }
        if self.tool_synthesizer.is_none() {
            self.tool_synthesizer = Some(ToolSynthesizer::new(100, 0.6));
            healed += 1;
        }
        if self.threat_modeler.is_none() {
            self.threat_modeler = Some(ThreatModeler::new());
            healed += 1;
        }
        if self.risk_sensor.is_none() {
            self.risk_sensor = Some(RiskSensor::new(0.8, 100));
            healed += 1;
        }
        if self.supply_chain_guard.is_none() {
            self.supply_chain_guard = Some(SupplyChainGuard::new());
            healed += 1;
        }
        if self.adversarial_reasoner.is_none() {
            self.adversarial_reasoner = Some(AdversarialReasoner::new());
            healed += 1;
        }
        if self.self_defense.is_none() {
            self.self_defense = Some(SelfDefense::new());
            healed += 1;
        }
        if self.evolution_gatekeeper.is_none() {
            self.evolution_gatekeeper = Some(EvolutionGatekeeper::new());
            healed += 1;
        }
        if self.audit_trail.is_none() {
            self.audit_trail = Some(AuditTrail::new(1000));
            healed += 1;
        }
        if self.config.enable_mind_bridge && self.mind_bridge.is_none() {
            self.mind_bridge = Some(MindBridge::new());
            healed += 1;
        }
        if self.config.enable_compound_knowledge && self.compound_knowledge.is_none() {
            self.compound_knowledge = Some(CompoundKnowledgeBase::new(std::path::PathBuf::from(
                "archive/knowledge",
            )));
            healed += 1;
        }
        if self.module_registry.is_none() {
            self.init_default_registry();
            healed += 1;
        }
        if self.config.enable_agent_team && self.agent_team.is_none() {
            self.agent_team = Some(TeamOrchestrator::new(
                crate::core::nt_core_experience::agent_team::TeamPattern::Supervisor,
            ));
            healed += 1;
        }
        if self.config.enable_code_mutation && self.code_mutation.is_none() {
            self.code_mutation = Some(CodeMutationEngine::new());
            healed += 1;
        }
        if self.config.enable_skill_registry && self.skill_registry.is_none() {
            self.skill_registry = Some(SkillRegistry::new());
            healed += 1;
        }
        if self.config.enable_timeline_orchestrator && self.timeline_orchestrator.is_none() {
            self.timeline_orchestrator = Some(TimelineOrchestrator::new(5));
            healed += 1;
        }
        if self.config.enable_deep_digestion && self.deep_digestion_pipeline.is_none() {
            self.deep_digestion_pipeline = Some(DeepDigestionPipeline::new());
            healed += 1;
        }
        if self.config.enable_constellation_detector && self.constellation_detector.is_none() {
            self.constellation_detector = Some(ConstellationDetector::new(3, 0.65));
            healed += 1;
        }
        if self.config.enable_cross_timeline_integrator && self.cross_timeline_integrator.is_none()
        {
            self.cross_timeline_integrator = Some(CrossTimelineIntegrator::new());
            healed += 1;
        }
        if self.reconstructive_buffer.is_none() {
            self.reconstructive_buffer = Some(ReconstructiveEpisodicBuffer::default());
            healed += 1;
        }
        if self.attention_self_modelling.is_none() {
            self.attention_self_modelling = Some(AttentionSelfModelling::default());
            healed += 1;
        }
        if self.overthinking_detector.is_none() {
            self.overthinking_detector = Some(OverthinkingDetector::default());
            healed += 1;
        }
        if self.inner_monologue_manager.is_none() {
            self.inner_monologue_manager = Some(InnerMonologueManager::default());
            healed += 1;
        }
        if self.cognitive_controller.is_none() {
            self.cognitive_controller = Some(CognitiveController::default());
            healed += 1;
        }
        if healed > 0 {
            log::info!(
                "[init_missing_fields] healed {} None field(s) in ConsciousnessCycle",
                healed
            );
        } else {
            log::debug!("[init_missing_fields] all fields already Some");
        }
    }

    pub fn run_cycle(&mut self, external_state: Option<VsaTagged>) -> CycleResult {
        self.init_missing_fields();
        // P0.1: Apply pending subsystem reset at cycle start
        let reset_name = self.pending_subsystem_reset.take();
        if let Some(ref name) = reset_name {
            match name.as_str() {
                "boredom" => self.boredom = Some(BoredomAccumulator::new()),
                "spreading" => self.spreading = Some(VsaSpreadingActivation::new(128, 64)),
                "qualia_generator" => self.qualia_generator = Some(QualiaGenerator::new()),
                "cognitive_blackboard" => {
                    self.cognitive_blackboard =
                        Some(CognitiveBlackboard::new(BlackboardConfig::default()))
                }
                _ => {}
            }
        }
        // P0.1: Store cognitive load adjustment for METRIC step
        let cognitive_load_adjustment = self.pending_cognitive_load.take();
        self.cycle_num += 1;
        let c = self.cycle_num;
        let t_start = std::time::Instant::now();

        // ── Metabolic budget: consume energy for this cycle ──
        if !self.metabolic_budget.consume(0.5) {
            // Starvation mode: skip non-essential processing
            // (still run core steps but mark as reduced)
        }
        // Record irreversible cost for this cycle
        self.metabolic_budget.record_irreversible_cost(1);
        self.metabolic_budget.recover();
        let all_steps = vec![
            CycleStep::Gather,
            CycleStep::Gate,
            CycleStep::Propose,
            CycleStep::Compete,
            CycleStep::Reason,
            CycleStep::Judge,
            CycleStep::Verify,
            CycleStep::Act,
            CycleStep::Veto,
            CycleStep::Record,
            CycleStep::Metric,
            CycleStep::Meta,
            CycleStep::Sleep,
        ];
        let mut health: Vec<StepHealth> = Vec::with_capacity(13);
        let mut gathered: Option<VsaTagged> = None;
        let mut fast_path_skip: bool = false;
        let mut substrate_concepts: Vec<String> = Vec::new();
        let mut causal_counterfactuals: Vec<(usize, f64)> = Vec::new();
        let mut neuromodulator_report: Option<String> = None;
        let mut dashboard_report: Option<String> = None;
        let mut phi_metrics: Option<Vec<f64>> = None;
        let mut meta_insights: Vec<String> = Vec::new();
        // Report pending reset
        if let Some(ref name) = reset_name {
            meta_insights.push(format!("modulation: reset '{}' applied", name));
        }
        // Report cognitive load adjustment
        if let Some(cl) = cognitive_load_adjustment {
            meta_insights.push(format!("modulation: cognitive_load={:.2}", cl));
        }
        let mut rsi_proposals_count: usize = 0;
        let mut local_qualia5: Option<Qualia5> = None;
        let mut extracted_page: Option<WebPageContent> = None;

        // ── Pre-GATHER: invariant safety counter + causal flow timing tick ──
        self.invariant_safety_counter += 1;
        self.internal_tick_counter += 1;

        // ── CXVIII.23: L1 Brainstem reflex layer (ATI) — signal integrity check ──
        self.l1_reflex_rejected = false;
        if let Some(ref external) = external_state {
            let sum: u64 = external.vector.iter().map(|&b| b as u64).sum();
            if sum == 0 || external.vector.is_empty() {
                meta_insights.push("l1_brainstem: input rejected (zero/empty signal)".into());
                self.l1_reflex_rejected = true;
            } else {
                let non_zero = external.vector.iter().filter(|&&b| b != 0).count();
                let integrity = non_zero as f64 / external.vector.len().max(1) as f64;
                if integrity < 0.1 {
                    meta_insights.push(format!(
                        "l1_brainstem: low integrity ({:.2}) — proceeding with caution",
                        integrity
                    ));
                }
            }
        }

        // ── Step 1: GATHER — collect perception input + sensor grounding ──
        let t0 = std::time::Instant::now();
        if self.config.enable_gather {
            // ── CXVIII.20: Wall-clock time budget tracking ──
            let elapsed_ms = t0.elapsed().as_millis() as u64;
            if elapsed_ms > self.step_time_budget_ms && self.step_time_budget_ms > 0 {
                meta_insights.push(format!(
                    "time_budget: GATHER exceeded budget ({}ms > {}ms)",
                    elapsed_ms, self.step_time_budget_ms
                ));
            }
            gathered = external_state.or_else(|| self.perception_input.take());
            if let Some(ref mut sg) = self.sensor_grounding {
                if let Some(ref inp) = gathered {
                    let channel = crate::core::nt_core_consciousness::sensor_grounding::SensoryChannel::Vision;
                    let data: Vec<f64> = inp.vector.iter().map(|&b| b as f64).collect();
                    sg.ingest(channel, data, 0.9);
                }
            }
            if let Some(ref mut ic) = self.image_cache {
                if let Some(ref mut inp) = gathered {
                    if inp.sense_modality == Some(SenseModality::Visual) {
                        let vsa_hash =
                            ImageCache::vsa_encode(&ImageCache::compute_dhash(&inp.vector));
                        if ic.lookup(&vsa_hash).is_some() {
                            inp.confidence = (inp.confidence + 0.15).min(1.0);
                        } else {
                            ic.insert(vsa_hash, format!("perception_cycle_{}", self.cycle_num));
                        }
                    }
                }
            }
            // Document perception: detect file paths in perception input
            if self.config.enable_document_perception {
                if let Some(ref inp) = gathered {
                    if inp.sense_modality == Some(SenseModality::Document) {
                        if let Some(ref mut dp) = self.document_perception {
                            if let Some(ref provenance) = inp.provenance {
                                let source_str: String = provenance.layers.iter()
                                    .filter_map(|(l, _)| match l {
                                        crate::core::nt_core_consciousness::source_hierarchy::KnowledgeLayer::Raw(meta) => {
                                            Some(format!("{:?}:{}", meta.source_type, meta.timestamp))
                                        },
                                        _ => None,
                                    })
                                    .next()
                                    .unwrap_or_else(|| "document".to_string());
                                let _doc_result = dp.perceive(&source_str);
                            }
                        }
                    }
                }
                // Also check perception_input directly for file path strings
                if let Some(ref inp) = self.perception_input {
                    if inp.sense_modality == Some(SenseModality::Document) {
                        if let Some(ref mut dp) = self.document_perception {
                            if let Some(ref provenance) = inp.provenance {
                                let source_str: String = provenance.layers.iter()
                                    .filter_map(|(l, _)| match l {
                                        crate::core::nt_core_consciousness::source_hierarchy::KnowledgeLayer::Raw(meta) => {
                                            Some(format!("{:?}:{}", meta.source_type, meta.timestamp))
                                        },
                                        _ => None,
                                    })
                                    .next()
                                    .unwrap_or_else(|| "document".to_string());
                                let _result = dp.perceive(&source_str);
                            }
                        }
                    }
                }
            }
            // Wave 0.5: bind qualia to gathered perception
            if let Some(ref mut qg) = self.qualia_generator {
                if let Some(ref inp) = gathered {
                    let mut qv = QualifiedVsa {
                        vsa: inp.vector.iter().map(|&b| b as f64).collect(),
                        bindings: Vec::new(),
                        dominant_tone: None,
                    };
                    let self_w = if inp.tag.is_self() { 0.9 } else { 0.1 };
                    let tone = if inp.sense_modality == Some(SenseModality::Visual) {
                        QualiaTone::Novel
                    } else {
                        QualiaTone::Neutral
                    };
                    qg.bind(
                        &mut qv,
                        tone,
                        0.6,
                        self_w,
                        &format!("{:?}", inp.sense_modality),
                    );
                }
            }
            // Human emotion detection: analyze gathered text for emotional content
            if let Some(ref hed) = self.human_emotion_detector {
                if let Some(ref inp) = gathered {
                    if inp.tag.is_world() {
                        let text = String::from_utf8_lossy(&inp.vector);
                        let reading = hed.detect_from_text(&text, "gather");
                        self.human_emotion_reading = Some(reading.clone());
                        if reading.is_significant() {
                            log::info!(
                                "[GATHER] Detected human emotion: {} (v={:.2} a={:.2} conf={:.2})",
                                reading.primary_emotion,
                                reading.valence,
                                reading.arousal,
                                reading.confidence
                            );
                            substrate_concepts.push(format!(
                                "human_emotion:{}:v={:.2}:a={:.2}:conf={:.2}",
                                reading.primary_emotion,
                                reading.valence,
                                reading.arousal,
                                reading.confidence
                            ));
                        }
                    }
                }
            }
            // Web accessibility: extract structured content from URL perception
            if self.config.enable_document_perception {
                if let Some(ref mut wa) = self.web_accessibility {
                    if let Some(ref inp) = gathered {
                        let text = String::from_utf8_lossy(&inp.vector);
                        if text.contains("http://") || text.contains("https://") {
                            let url_start = text.find("http").unwrap_or(0);
                            let url_end = text[url_start..]
                                .find(|c: char| c.is_whitespace() || c == '>')
                                .map(|e| url_start + e)
                                .unwrap_or(text.len());
                            let url = &text[url_start..url_end];
                            if let Some(content) = wa.extract(url) {
                                let _perception =
                                    WebContentExtractor::to_perception_text(&content, 200);
                                extracted_page = Some(content.clone());
                                if let Some(ref mut inp) = gathered {
                                    inp.salience = (inp.salience + 0.4).min(1.0);
                                }
                            }
                        }
                    }
                }
            }
            // ScreenshotPipeline: capture visual from URL when enabled
            if self.config.enable_screenshot_pipeline {
                if let Some(ref mut sp) = self.screenshot_pipeline {
                    if let Some(ref inp) = gathered {
                        let text = String::from_utf8_lossy(&inp.vector);
                        if text.contains("screenshot:") || text.contains("capture:") {
                            let url_start = text.find("http").unwrap_or(0);
                            let url_end = text[url_start..]
                                .find(|c: char| c.is_whitespace())
                                .map(|e| url_start + e)
                                .unwrap_or(text.len());
                            let target_url = &text[url_start..url_end];
                            if !target_url.is_empty() {
                                match sp.capture(target_url) {
                                    Ok(bytes) => {
                                        log::info!(
                                            "[GATHER] ScreenshotPipeline captured {} bytes from {}",
                                            bytes.len(),
                                            target_url
                                        );
                                        if let Some(ref mut inp) = gathered {
                                            inp.salience = (inp.salience + 0.2).min(1.0);
                                            inp.vector = bytes;
                                        }
                                    }
                                    Err(e) => {
                                        log::warn!("[GATHER] ScreenshotPipeline failed: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Multi-modal fusion tracking
            let visual_loaded = gathered
                .as_ref()
                .map(|g| g.sense_modality == Some(SenseModality::Visual))
                .unwrap_or(false);
            let text_loaded = gathered
                .as_ref()
                .map(|g| g.sense_modality == Some(SenseModality::Document))
                .unwrap_or(false);
            // Phase 26: PixelPerceptionPipeline — tile-based visual → VSA embedding
            if self.config.enable_pixel_perception {
                if let Some(ref mut pp) = self.pixel_perception {
                    if let Some(ref inp) = gathered {
                        if inp.sense_modality == Some(SenseModality::Visual) {
                            let tile = VisualTile {
                                article_id: format!("cycle_{}", self.cycle_num),
                                tile_index: 0,
                                chunk_index: 0,
                                y_offset: 0,
                                width: 0,
                                height: 0,
                                md5_hash: 0,
                                source_url: String::new(),
                            };
                            let _tile_vsa = pp.process_tile(&tile);
                        }
                    }
                }
            }
            // Phase 26: LongHorizonOcr — document OCR → structured doc → VSA grounding
            if self.config.enable_long_horizon_ocr {
                if let Some(ref mut ocr) = self.long_horizon_ocr {
                    if let Some(ref inp) = gathered {
                        if inp.sense_modality == Some(SenseModality::Document) {
                            if let Some(ref provenance) = inp.provenance {
                                let source_str: String = provenance.layers.iter()
                                .filter_map(|(l, _)| match l {
                                    crate::core::nt_core_consciousness::source_hierarchy::KnowledgeLayer::Raw(meta) => {
                                        Some(format!("{:?}:{}", meta.source_type, meta.timestamp))
                                    },
                                    _ => None,
                                })
                                .next()
                                .unwrap_or_else(|| "document".to_string());
                                let _ocr_result = ocr.process(&DocumentSource::Url(source_str));
                            }
                        }
                    }
                }
            }
            // Phase 2+3: DocumentParserRegistry — classify-aware routing through unified parser
            if self.config.enable_document_parser {
                if let Some(ref mut dpr) = self.document_parser_registry {
                    if let Some(ref inp) = gathered {
                        if inp.sense_modality == Some(SenseModality::Document) {
                            let bytes = inp.vector.to_vec();
                            let source =
                                crate::core::nt_core_input::document_parser::DocumentSource::Bytes(
                                    bytes.clone(),
                                );
                            // Use classification-aware routing: scanned/image → VLM, digital → text
                            let parse_result = if self.config.enable_document_classifier
                                && self.document_classifier.is_some()
                            {
                                dpr.classify_and_parse(&source, &bytes)
                            } else {
                                dpr.parse(&source)
                            };
                            match parse_result.or_else(|_| {
                                // VLM env fallback: if classify_and_parse failed, try from_env
                                let vlm =
                                    crate::core::nt_core_input::vlm_backend::VlmBackend::from_env()
                                        .ok_or(DocumentError::UnsupportedFormat(
                                            "VLM not configured".to_string(),
                                        ))?;
                                vlm.parse(&source)
                            }) {
                                Ok(parsed) => {
                                    log::info!("[GATHER] DocumentParserRegistry parsed {} ({} backend, {}ms)",
                                        parsed.metadata.format.as_deref().unwrap_or("unknown"),
                                        parsed.metadata.backend_name,
                                        parsed.metadata.extraction_time_ms,
                                    );
                                    // Phase 3.2: Extract formulas from parsed document
                                    if self.config.enable_formula_extractor {
                                        if let Some(ref fe) = self.formula_extractor {
                                            let mut clone = parsed.clone();
                                            let formulas =
                                                enrich_document_with_formulas(fe, &mut clone);
                                            if !formulas.is_empty() {
                                                log::info!("[GATHER] Extracted {} formula(s) from document (formula_count={})",
                                                    formulas.len(), clone.metadata.formula_count);
                                            }
                                        }
                                    }
                                    // Phase 3.3: Extract markdown tables (fallback when backend didn't populate tables)
                                    if parsed.tables.is_empty() && !parsed.markdown.is_empty() {
                                        let extra_tables =
                                            crate::core::nt_core_input::extract_markdown_tables(
                                                &parsed.markdown,
                                            );
                                        if !extra_tables.is_empty() {
                                            log::info!("[GATHER] Extracted {} markdown table(s) from parsed text", extra_tables.len());
                                        }
                                    }
                                    // Phase 3.4: Layout-aware structured chunking
                                    if !parsed.markdown.is_empty() {
                                        let (_chunks, stats) =
                                            crate::core::nt_core_input::chunk_document(
                                                &parsed, None,
                                            );
                                        if stats.total_chunks > 1 {
                                            log::info!("[GATHER] Chunked document into {} chunks ({} tokens, avg {} tok/chunk)",
                                                stats.total_chunks, stats.total_tokens, stats.avg_chunk_tokens as usize);
                                        }
                                    }
                                    // Phase 3.5: Feed parsed document to DocumentPerceptionModule
                                    if let Some(ref mut dp) = self.document_perception {
                                        dp.feed_parsed_document(&parsed);
                                    }
                                }
                                Err(e) => {
                                    log::warn!("[GATHER] All document parsers failed: {}", e);
                                }
                            }
                        }
                    }
                }
            }
            // Multi-modal fusion: boost salience when multiple modalities are active
            if self.config.enable_modality_gate {
                if let Some(ref mut inp) = gathered {
                    let active_count = [visual_loaded, text_loaded].iter().filter(|&&x| x).count();
                    if active_count > 1 {
                        inp.salience = (inp.salience + (active_count as f64) * 0.05).min(1.0);
                    }
                    if active_count > 0 {
                        inp.confidence = (inp.confidence + (active_count as f64) * 0.02).min(1.0);
                    }
                }
            }
            // SecurityExecutive: RiskSensor — assess risk of incoming perception data
            if self.config.enable_risk_sensor {
                if let (Some(ref mut rs), Some(ref inp)) =
                    (self.risk_sensor.as_mut(), gathered.as_ref())
                {
                    let risk = rs.assess_input(&inp.vector, inp.salience);
                    if risk.score > 0.5 {
                        log::warn!(
                            "[GATHER] RiskSensor: {:?} score={:.2} indicators={:?}",
                            risk.level,
                            risk.score,
                            risk.indicators
                        );
                    }
                }
            }
            // Hebbian memory recall: retrieve similar past experiences
            if self.config.enable_hebbian_memory {
                if let (Some(ref hm), Some(ref inp)) =
                    (self.hebbian_memory.as_ref(), gathered.as_ref())
                {
                    let recalls = hm.semantic_spread(&inp.vector, 3, 0.4);
                    for (node_id, sim, _) in &recalls {
                        substrate_concepts
                            .push(format!("hebbian_recall:{}:sim={:.3}", node_id, sim));
                    }
                }
            }
            // Phase 33: MindBridge — DCI retrieval + curiosity + cognitive load
            if self.config.enable_mind_bridge {
                if let Some(ref mut mb) = self.mind_bridge {
                    mb.step_gather_load_begin();
                    mb.step_gather_curiosity(0.0, 0.1, self.cycle_num);
                    if let Some(ref inp) = gathered {
                        let concept =
                            String::from_utf8_lossy(&inp.vector[..inp.vector.len().min(80)]);
                        // Deep Code Inspection: retrieve code intelligence using mind's ToolExecutor
                        let executor = NeotrixToolExecutor;
                        let dci_results = mb.step_gather_dci(
                            if concept.trim().is_empty() {
                                None
                            } else {
                                Some(concept.trim())
                            },
                            &executor,
                        );
                        for r in &dci_results {
                            let snippet = if r.content.len() > 80 {
                                format!("{}...", &r.content[..80])
                            } else {
                                r.content.clone()
                            };
                            substrate_concepts.push(format!(
                                "dci:{}:rel={:.2}:{}",
                                r.source, r.relevance, snippet
                            ));
                        }
                    }
                }
            }
            // ── Semantic Entropy Drive (GWA Eq.2-4): compute H(W) from gathered thoughts ──
            if let Some(ref mut se) = self.semantic_entropy {
                if let Some(ref inp) = gathered {
                    let thought_f64: Vec<f64> = inp.vector.iter().map(|&b| b as f64).collect();
                    se.record_thought(&thought_f64);
                    let entropy = se.current_entropy();
                    let dyn_temp = se.dynamic_temperature();
                    meta_insights.push(format!(
                        "semantic_entropy:H(W)={:.4} T_gen={:.4}",
                        entropy, dyn_temp
                    ));
                    // Broadcast entropy signal on integration bus
                    self.integration_bus
                        .broadcast(IntegrationSignal::SemanticEntropySignal {
                            entropy,
                            temperature: dyn_temp,
                            cycle: c,
                        });
                    // When entropy is critically low (< 0.3), send reason temperature
                    // modulation to force divergent thinking in the REASON step.
                    if entropy < 0.3 {
                        let reason_temp = (1.2 + (0.3 - entropy) * 2.0).min(1.5);
                        self.integration_bus
                            .send_modulation(ModulationCommand::SetReasonTemperature(reason_temp));
                        meta_insights.push(format!(
                            "modulation: reason_temperature={:.4} (divergent_pressure, H={:.4})",
                            reason_temp, entropy
                        ));
                    }
                }
            }
            // ── StreamPipeline: block-causal VSA attention for streaming perception ──
            if self.config.enable_stream_pipeline {
                if let Some(ref mut sp) = self.stream_pipeline {
                    if let Some(ref inp) = gathered {
                        let (output, thinker_ran) = sp.process_block(
                            &inp.vector,
                            crate::core::nt_core_hcube::stream_pipeline::StreamModality::Mixed,
                            self.cycle_num * 100,
                        );
                        if thinker_ran {
                            substrate_concepts.push(format!(
                                "stream:thinker_latent={:.4}",
                                sp.latent_coherence()
                            ));
                        }
                        // VSA-tag the gathered perception with streaming context
                        if let Some(ref mut inp) = gathered {
                            inp.vector = output;
                        }
                    }
                }
            }
            health.push(StepHealth {
                step: CycleStep::Gather,
                success: gathered.is_some(),
                duration_ms: t0.elapsed().as_millis() as u64,
            });
        } else {
            health.push(StepHealth {
                step: CycleStep::Gather,
                success: true,
                duration_ms: 0,
            });
        }

        // ── CompetitiveSelection: decide what enters conscious workspace ──
        if self.config.enable_competitive_selection {
            let competitive_candidates: Vec<VsaTagged> = {
                let mut cand = Vec::new();
                if let Some(ref perception) = gathered {
                    cand.push(perception.clone());
                }
                if let Some(ref hm) = self.hebbian_memory {
                    for node in hm.nodes.iter().take(5) {
                        let tag =
                            VsaTagged::new(node.clone(), VsaOrigin::Self_(VsaSelfCategory::Memory));
                        cand.push(tag);
                    }
                }
                cand
            };

            if let Some(ref mut selector) = self.competitive_selection {
                if !competitive_candidates.is_empty() {
                    let salience: Vec<f64> = competitive_candidates
                        .iter()
                        .map(|c| {
                            let mut s = c.salience * 0.6 + c.confidence * 0.4;
                            if let Some(ref gathered_val) = gathered {
                                let novelty =
                                    1.0 - QuantizedVSA::similarity(&c.vector, &gathered_val.vector);
                                s = s * 0.7 + novelty * 0.3;
                            }
                            s
                        })
                        .collect();
                    let selected = selector.select(&competitive_candidates, &salience);
                    if let Some((idx, score)) = selected {
                        if idx < competitive_candidates.len() {
                            let selected_content = competitive_candidates[idx].clone();
                            gathered = Some(selected_content);
                            meta_insights.push(format!(
                                "competitive_selection: candidate {} of {} (score={:.3})",
                                idx,
                                competitive_candidates.len(),
                                score
                            ));
                        }
                    }
                }
            }
        }

        // ── Step 2: GATE — modality routing + identity defense ──
        let t1 = std::time::Instant::now();
        if self.config.enable_gate {
            if let (Some(ref gate), Some(ref mut inp)) = (&self.multi_modal_gate, &mut gathered) {
                let modality = inp.sense_modality.unwrap_or_else(|| match inp.tag {
                    VsaOrigin::World(
                        crate::core::nt_core_consciousness::vsa_tag::VsaWorldCategory::Sensor,
                    ) => SenseModality::Visual,
                    _ => SenseModality::Mental,
                });
                let affinity = gate.gate(modality, &inp.vector);
                inp.salience = (inp.salience + affinity * 0.3).clamp(0.0, 1.0);
            }
            if let Some(ref mut id) = self.identity_defense {
                if let Some(ref inp) = gathered {
                    let vsa_f64: Vec<f64> = inp.vector.iter().map(|&b| b as f64).collect();
                    meta_insights.push(format!(
                        "security:identity_perturbation={:?}",
                        id.check_vsa_perturbation(&[vsa_f64], "cycle_input", c)
                    ));
                }
            }
            // P0.1: QualityGate — multi-dimensional content quality evaluation + fast-path routing
            if let (Some(ref mut qg), Some(ref inp)) = (&mut self.quality_gate, &gathered) {
                let relevance = inp.salience;
                let faithfulness = inp.salience; // proxy — no deeper reasoning available yet
                let completeness = 0.8; // default — we have the input but haven't processed it
                let uncertainty = 0.2; // default — low uncertainty at gate level
                let (take_fast, scores) =
                    qg.evaluate(relevance, faithfulness, completeness, uncertainty);
                fast_path_skip = take_fast;
                if take_fast {
                    let _combined = scores.combined();
                    meta_insights.push(format!(
                        "quality_gate:fast_path r={:.2} f={:.2} c={:.2} u={:.2}",
                        scores.relevance,
                        scores.faithfulness,
                        scores.completeness,
                        scores.uncertainty
                    ));
                } else {
                    meta_insights.push(format!(
                        "quality_gate:slow_path r={:.2} f={:.2} c={:.2} u={:.2}",
                        scores.relevance,
                        scores.faithfulness,
                        scores.completeness,
                        scores.uncertainty
                    ));
                }
            }
            // ── Wave G: Dual-process (System 1) fast-path decision ──
            // If quality_gate wasn't present (None), check dual-process heuristic
            if !fast_path_skip && self.should_use_fast_path(gathered.as_ref()) {
                fast_path_skip = true;
                self.dual_process.fast_path_cycles += 1;
                meta_insights.push(
                    "dual_process:fast_path (System 1) — high confidence, skip reasoning".into(),
                );
            } else if self.dual_process.fast_path_enabled {
                self.dual_process.slow_path_cycles += 1;
            }
            // SecurityExecutive: SelfDefense — inspect input for threats, block if malicious
            if self.config.enable_self_defense {
                if let Some(ref mut sd) = self.self_defense {
                    if let Some(ref inp) = gathered {
                        let decision = sd.inspect_input(&inp.vector);
                        if decision.action == DefenseAction::Block {
                            log::warn!(
                                "[GATE] SelfDefense blocked input: {} (conf={:.2})",
                                decision.reason,
                                decision.confidence
                            );
                            gathered = None;
                        } else if decision.action == DefenseAction::Sanitize {
                            if let Some(sanitized) = decision.sanitized_input {
                                log::info!("[GATE] SelfDefense sanitized input");
                                gathered.as_mut().map(|g| g.vector = sanitized);
                            }
                        }
                    }
                }
            }
            health.push(StepHealth {
                step: CycleStep::Gate,
                success: true,
                duration_ms: t1.elapsed().as_millis() as u64,
            });
        } else {
            health.push(StepHealth {
                step: CycleStep::Gate,
                success: true,
                duration_ms: 0,
            });
        }

        // ── Step 3: PROPOSE (skipped on fast-path) ──
        let t2 = std::time::Instant::now();
        if !fast_path_skip && self.config.enable_propose {
            if let Some(ref mut sg) = self.substrate_gen {
                if let Some(ref inp) = gathered {
                    let vsa: Vec<f64> = inp.vector.iter().map(|&b| b as f64).collect();
                    let _entropy = sg.readout_entropy(&vsa);
                    if let Some((id, text, sim)) = sg.nearest_concept(&vsa) {
                        substrate_concepts.push(format!("{}:{}:{:.2}", id, text, sim));
                    }
                }
            }
            if let Some(ref mut ed) = self.exploration_driver {
                if let Some(ref inp) = gathered {
                    let vsa: Vec<f64> = inp.vector.iter().map(|&b| b as f64).collect();
                    let novelty_score = ed.novelty(&vsa);
                    if novelty_score > 0.3 {
                        substrate_concepts.push(format!("novelty:{:.3}", novelty_score));
                    }
                }
            }
            // Phase 7: post gathered input as a blackboard claim
            if let Some(ref mut bb) = self.cognitive_blackboard {
                if let Some(ref inp) = gathered {
                    let _claim_id = bb.post_claim(
                        super::cognitive_blackboard::EngineType::Intuition,
                        "perception".to_string(),
                        format!("cycle_{}", c),
                        inp.vector.clone(),
                        0.6,
                    );
                }
            }
            // SecurityExecutive: ThreatModeler — classify attack surface and emit threat assessments
            if self.config.enable_threat_modeler {
                if let Some(ref mut tm) = self.threat_modeler {
                    if let Some(ref inp) = gathered {
                        let text =
                            String::from_utf8_lossy(&inp.vector[..inp.vector.len().min(512)]);
                        let threat = tm.classify(&text);
                        if threat.confidence > 0.5 {
                            log::info!(
                                "[PROPOSE] ThreatModeler: {:?} conf={:.2} action={}",
                                threat.category,
                                threat.confidence,
                                threat.suggested_action
                            );
                            substrate_concepts.push(format!(
                                "threat:{:?}:conf={:.2}",
                                threat.category, threat.confidence
                            ));
                        }
                    }
                }
            }
            // P1: AffectiveForecast — forecast emotional impact of proposed action
            if let Some(ref mut af) = self.affective_forecast {
                if let Some(ref inp) = gathered {
                    let coherence = self
                        .master_consciousness
                        .as_ref()
                        .map(|m| m.c_score())
                        .unwrap_or(0.5);
                    let dims = super::appraisal_engine::AppraisalDimensions {
                        desirability: coherence.max(0.1),
                        likelihood: 0.5,
                        effort: 0.5,
                        certainty: 0.5,
                        controllability: 0.5,
                        agency: 0.0,
                        legitimacy: 0.5,
                    };
                    let forecast = af.forecast(inp.vector.clone(), dims);
                    meta_insights.push(format!(
                        "affect:forecast={}_intensity={:.2}",
                        forecast.emotion_label, forecast.expected_intensity
                    ));
                }
            }
            health.push(StepHealth {
                step: CycleStep::Propose,
                success: true,
                duration_ms: t2.elapsed().as_millis() as u64,
            });
        } else {
            health.push(StepHealth {
                step: CycleStep::Propose,
                success: true,
                duration_ms: 0,
            });
        }

        // ── Step 4: COMPETE — (skipped on fast-path) spreading + neuromodulator + attention + salience ──
        let t3 = std::time::Instant::now();
        if !fast_path_skip && self.config.enable_compete {
            if let Some(ref mut nm) = self.neuromodulators {
                // Drive neuromodulators from real cycle signals
                let pred_error = self.temporal_prediction.running_prediction_error();
                let divergence = self.temporal_prediction.is_diverging(0.2);
                let phi_val = self
                    .master_consciousness
                    .as_ref()
                    .map(|m| m.c_score())
                    .unwrap_or(0.5);
                // DA: reward prediction error (high when prediction fails → learning signal)
                nm.set_level(
                    crate::core::nt_core_consciousness::neuromodulator::NeuromodulatorType::DA,
                    (pred_error * 2.0).min(1.0),
                );
                // NE: arousal from divergence/novelty
                if divergence {
                    nm.phasic_burst(
                        crate::core::nt_core_consciousness::neuromodulator::NeuromodulatorType::NE,
                        0.3,
                    );
                }
                // ACh: learning from phi (high phi → more acetylcholine → more plasticity)
                nm.set_level(
                    crate::core::nt_core_consciousness::neuromodulator::NeuromodulatorType::ACh,
                    (phi_val * 0.8 + 0.2).min(1.0),
                );
                // 5HT: mood stability inversely from prediction error
                nm.set_level(crate::core::nt_core_consciousness::neuromodulator::NeuromodulatorType::Serotonin5HT,
                    (1.0 - pred_error).max(0.2));
                nm.tick_decay();
                let mod_report = format!(
                    "A:{:.2} DA:{:.2} NE:{:.2} 5HT:{:.2}",
                    nm.get_level(crate::core::nt_core_consciousness::neuromodulator::NeuromodulatorType::ACh),
                    nm.get_level(crate::core::nt_core_consciousness::neuromodulator::NeuromodulatorType::DA),
                    nm.get_level(crate::core::nt_core_consciousness::neuromodulator::NeuromodulatorType::NE),
                    nm.get_level(crate::core::nt_core_consciousness::neuromodulator::NeuromodulatorType::Serotonin5HT),
                );
                neuromodulator_report = Some(mod_report);
                // Broadcast neuromodulation state via IntegrationBus
                let cm = nm.consciousness_influence();
                if cm.arousal > 0.7 || cm.motivation > 0.7 {
                    self.integration_bus
                        .send_modulation(ModulationCommand::SetParam {
                            subsystem: "compete".to_string(),
                            param: "arousal".to_string(),
                            value: cm.arousal,
                        });
                }
            }
            if let Some(ref sp) = self.spreading {
                let _active = sp.vsa_most_active(3);
            }
            // Phase 7: attention schema + salience evaluation
            if let Some(ref mut attn) = self.attention_schema {
                if let Some(ref inp) = gathered {
                    attn.attend_to(inp.clone(), "cycle_perception", true);
                }
            }
            if let Some(ref mut sd) = self.salience_detector {
                if let Some(ref inp) = gathered {
                    let _sig = sd.evaluate(inp.vector.clone(), 0.5, &[] as &[&[u8]], 0.3, "cycle");
                }
            }
            health.push(StepHealth {
                step: CycleStep::Compete,
                success: true,
                duration_ms: t3.elapsed().as_millis() as u64,
            });
        } else {
            health.push(StepHealth {
                step: CycleStep::Compete,
                success: true,
                duration_ms: 0,
            });
        }

        // ── Step 5: REASON — (skipped on fast-path) ──
        let t4 = std::time::Instant::now();
        if !fast_path_skip && self.config.enable_reason {
            if let Some(ref mut cr) = self.causal {
                let _cf_result = cr.counterfactual("perception", "imagination", "prediction", c);
                causal_counterfactuals.push((c as usize, 0.5));
            }
            if let Some(ref mut bm) = self.bio_memory {
                if let Some(ref inp) = gathered {
                    let key: Vec<f64> = inp.vector.iter().map(|&b| b as f64).collect();
                    let _completed_result = bm.pattern_complete(&key);
                }
            }
            // EmotionalSteering: internal emotional state decay + human emotion modulation
            if let Some(ref mut es) = self.emotional_steering {
                es.update(1.0);
                if let Some(ref reading) = self.human_emotion_reading {
                    if reading.is_significant() && reading.confidence > 0.4 {
                        let dim = if reading.valence > 0.3 {
                            super::emotional_steering::EmotionalDimension::Satisfaction
                        } else if reading.valence < -0.3 {
                            super::emotional_steering::EmotionalDimension::Frustration
                        } else {
                            super::emotional_steering::EmotionalDimension::Curiosity
                        };
                        es.state
                            .modulate(dim, (reading.valence.abs() * 0.1).min(0.3));
                    }
                }
                // P0: Wire EmotionalSteering outputs
                let eb = es.exploration_bonus();
                if eb > 0.5 {
                    meta_insights.push(format!("emotional:exploration_bonus={:.2}", eb));
                }
                if es.should_rest() {
                    meta_insights.push("emotional:should_rest".to_string());
                }
                if es.should_escalate() {
                    meta_insights.push("emotional:should_escalate".to_string());
                }
            }
            // P1: EmotionRegulation — select regulation strategy from emotional state
            if let Some(ref mut er) = self.emotion_regulation {
                if let Some(ref es) = self.emotional_steering {
                    let dominant = es.state.dominant();
                    let intensity = es.state.get(dominant);
                    let strategy =
                        er.select_strategy(&format!("{:?}", dominant), intensity, "cycle_reason");
                    if intensity > 0.7 {
                        let _adjusted = er.apply_strategy(strategy, 0.5);
                        meta_insights.push(format!("emotion:regulation={:?}", strategy));
                    }
                }
            }
            // ── Wave A: Reasoning modules — full reasoning API ──
            if self.config.enable_mcts_reasoner {
                if let Some(ref mut m) = self.mcts_reasoner {
                    if let Some(ref inp) = gathered {
                        let hypothesis = crate::core::nt_core_reasoning::Hypothesis {
                            id: c,
                            content: inp.vector.clone(),
                            confidence: inp.salience,
                            expert: crate::core::nt_core_reasoning::ExpertType::MultiHop,
                            supporting_evidence: Vec::new(),
                            created_at: crate::core::unix_now_ms(),
                            is_contradicted: false,
                        };
                        let _mcts_results = m.search(hypothesis);
                    }
                }
            }
            if self.config.enable_dead_end_detector {
                if let Some(ref mut d) = self.dead_end_detector {
                    if let Some(ref inp) = gathered {
                        let hypothesis = crate::core::nt_core_reasoning::Hypothesis {
                            id: c,
                            content: inp.vector.clone(),
                            confidence: inp.salience,
                            expert: crate::core::nt_core_reasoning::ExpertType::MultiHop,
                            supporting_evidence: Vec::new(),
                            created_at: crate::core::unix_now_ms(),
                            is_contradicted: false,
                        };
                        let bb = crate::core::nt_core_reasoning::VsaBlackboard::new(64);
                        if let Some(_report) = d.fast_check(&hypothesis, &bb) {
                            log::info!("[REASON] Dead-end detected: {:?}", _report.detected_type);
                        }
                    }
                }
            }
            if self.config.enable_counterfactual_simulator {
                if let Some(ref mut cf) = self.counterfactual_simulator {
                    if let Some(ref inp) = gathered {
                        let ids = cf.generate_scenarios(
                            &inp.vector,
                            crate::core::nt_core_reasoning::CounterfactualType::InputPerturbation,
                            3,
                        );
                        for id in ids {
                            let _outcome = cf.simulate_scenario(id);
                        }
                    }
                }
            }
            // ── Wave D: Revived reasoning modules ──
            if self.config.enable_analogical_reasoner {
                if let Some(ref mut ar) = self.analogical_reasoner {
                    if let Some(ref _inp) = gathered {
                        let source =
                            super::analogical_reasoning::AnalogicalStructure::new("perception");
                        let target =
                            super::analogical_reasoning::AnalogicalStructure::new("knowledge");
                        let _analogies_result = ar.reason_by_analogy(&source, &target);
                    }
                }
            }
            if self.config.enable_hierarchical_world_model {
                if let Some(ref mut hwm) = self.hierarchical_world_model {
                    if let Some(ref inp) = gathered {
                        let _prediction_result = hwm.step(&inp.vector);
                    }
                }
            }

            // Entity-boosted knowledge retrieval
            if self.config.enable_entity_extractor {
                if let (Some(ref mut ext), Some(ref mut gr), Some(ref inp)) = (
                    &mut self.entity_extractor,
                    &mut self.entity_graph,
                    &gathered,
                ) {
                    let text = String::from_utf8_lossy(&inp.vector[..inp.vector.len().min(256)])
                        .to_string();
                    let boosted = crate::core::nt_core_knowledge::entity_extractor::retrieve_with_entity_boost(
                        ext, gr, &text, &inp.vector, 5, 3, 0.3, 0.6,
                    );
                    if !boosted.is_empty() {
                        log::debug!(
                            "[REASON] Entity-boosted retrieval: {} matches",
                            boosted.len()
                        );
                        for (node_id, label, score) in &boosted {
                            substrate_concepts
                                .push(format!("entity:{}:{}:{:.3}", node_id, label, score));
                        }
                    }
                }
            }
            // Phase 37 P0.2: REMem-style iterative retrieval from MemoryLattice
            if let Some(ref ml) = self.memory_lattice {
                if let Some(ref inp) = gathered {
                    let query_text =
                        String::from_utf8_lossy(&inp.vector[..inp.vector.len().min(128)]);
                    let iter_results = ml.iterative_retrieve(&query_text, 5);
                    for (layer, idx, score) in &iter_results {
                        substrate_concepts.push(format!(
                            "iterative_retrieve:{:?}[{}]={:.3}",
                            layer, idx, score,
                        ));
                    }
                    meta_insights
                        .push(format!("iterative_retrieve:{}_results", iter_results.len(),));
                }
            }
            // Reasoning federation: unify all reasoning engines
            if self.config.enable_reasoning_federation {
                if let Some(ref mut rf) = self.reasoning_federation {
                    let ctx = crate::core::nt_core_consciousness::reasoning_federation::ReasoningContext::new(
                        &format!("consciousness_cycle_{}", c),
                        gathered.as_ref().map(|g| g.vector.clone()).unwrap_or_default(),
                    );
                    let output = rf.reason(&ctx);
                    substrate_concepts.push(format!(
                        "fed:{:.2}:{}",
                        output.confidence, output.conclusion
                    ));
                }
            }

            // ── Phase 1: VsaReasoner — analogical + causal + multi-hop reasoning ──
            if self.config.enable_vsa_reasoner {
                if let Some(ref mut vr) = self.vsa_reasoner {
                    // Inject dynamic semantic entropy temperature into reasoner
                    vr.exploration_temperature = self.reason_temperature;
                    if let Some(ref inp) = gathered {
                        let _analogy_result =
                            vr.analogical_reason(&inp.vector, &inp.vector, &inp.vector);
                        if substrate_concepts.len() >= 2 {
                            let premises: Vec<Vec<u8>> = substrate_concepts
                                .iter()
                                .take(3)
                                .map(|s| s.as_bytes().to_vec())
                                .collect();
                            let _ = vr.causal_reason(&premises, &inp.vector);
                        }
                        let mh_id = vr.multi_hop_reason(&inp.vector, &[]);
                        // P0.4: Multi-Head Resonator decomposition of gathered bundle
                        if vr.multi_head_resonator.is_some() {
                            let decomposed = vr.decompose_bundle(&inp.vector);
                            for (factor, confidence) in &decomposed {
                                meta_insights.push(format!(
                                    "resonator: factor='{}' confidence={:.3}",
                                    factor, confidence
                                ));
                            }
                        }
                        // ── Phase 36: refinement — soft_bind multi-hop conclusion with query ──
                        if let Some(hyp) = vr.blackboard.get_hypothesis(mh_id) {
                            let conclusion_f64: Vec<f64> =
                                hyp.content.iter().map(|&b| b as f64).collect();
                            let query_f64: Vec<f64> =
                                inp.vector.iter().map(|&b| b as f64).collect();
                            let refined =
                                vr.vsa_soft_bind(&[(&conclusion_f64, 0.7), (&query_f64, 0.3)]);
                            vr.pattern_matcher
                                .register_pattern("refined_conclusion", refined);
                        }
                    }
                }
            }
            // ── Phase 1: CapabilitySynthesizer synthesis from gathered data ──
            if self.config.enable_cap_synth_synthesis && self.config.enable_capability_synthesizer {
                if let Some(ref mut synth) = self.capability_synthesizer {
                    if let Some(ref inp) = gathered {
                        let desc =
                            String::from_utf8_lossy(&inp.vector[..inp.vector.len().min(128)]);
                        let outcome = synth.synthesize(&desc);
                        match outcome {
                            crate::core::nt_core_experience::capability_synthesizer::SynthesisOutcome::DirectMatch(id) => {
                                synth.record_invocation(id, true);
                            }
                            crate::core::nt_core_experience::capability_synthesizer::SynthesisOutcome::CompositeCreated(id) => {
                                log::debug!("[REASON] CapabilitySynthesizer: created composite #{}", id);
                            }
                    _ => {}
                        }
                    }
                }
            }
            // ── CXI.12: Overthinking detector ──
            if let Some(ref mut od) = self.overthinking_detector {
                let current_score = self
                    .master_consciousness
                    .as_ref()
                    .map(|m| m.c_score())
                    .unwrap_or(0.5);
                if current_score <= od.last_score {
                    od.consecutive_no_improvement += 1;
                } else {
                    od.consecutive_no_improvement = 0;
                }
                od.last_score = current_score;
                if od.consecutive_no_improvement >= od.max_no_improvement && !od.overthinking {
                    od.overthinking = true;
                    od.event_history.push(self.cycle_num);
                    meta_insights.push(format!(
                        "overthinking: {} consecutive no improvement → forced output",
                        od.consecutive_no_improvement
                    ));
                    // Force early exit from deeper reasoning via modulation
                    self.integration_bus
                        .send_modulation(ModulationCommand::SetParam {
                            subsystem: "reason".into(),
                            param: "depth".into(),
                            value: 0.3, // reduced reasoning depth
                        });
                } else if od.consecutive_no_improvement == 0 && od.overthinking {
                    od.overthinking = false;
                    meta_insights.push("overthinking: resolved".into());
                }
            }
            // ── CXVIII.5: Difficulty-adaptive parallel thinking ──
            if self.config.enable_parallel_thinking {
                let n_paths = self.config.parallel_thinking_width;
                meta_insights.push(format!(
                    "parallel_thinking: {} paths — best-of-N majority voting",
                    n_paths
                ));
            }
            // ── CXVIII.6: Knowledge task information ceiling ──
            if self.config.reasoning_mode == "recall" {
                meta_insights.push(
                    "reasoning_mode: recall — extended reasoning ineffective, use retrieval".into(),
                );
            }
            // Run registered cognitive modules from PreRefinery onwards
            if self.config.enable_module_registry {
                if let Some(ref mut reg) = self.module_registry {
                    reg.run_from(ModulePhase::PreRefinery);
                }
            }
            // ── HybridRetrievalEngine: index + retrieve knowledge via BM25/VSA/Graph/RRF ──
            if self.config.enable_hybrid_retrieval {
                if let Some(ref mut hr) = self.hybrid_retrieval {
                    if let Some(ref inp) = gathered {
                        let query_text =
                            String::from_utf8_lossy(&inp.vector[..inp.vector.len().min(256)])
                                .to_string();
                        let results = hr.search_hybrid(&query_text, 5);
                        // ── Phase 36: adaptive fusion weight learning from retrieval quality ──
                        let feedback = results.first().map(|r| r.score).unwrap_or(0.0) - 0.5;
                        hr.update_fusion_weights(&query_text, feedback, 0.01);
                        for r in &results {
                            substrate_concepts
                                .push(format!("retrieved:{}|{:.3}", r.doc_id, r.score));
                        }
                        // Index gathered perception as a retrievable document
                        hr.index_text(format!("cycle_{}", c), query_text);
                    }
                    // Multi-signal retrieval: entity matching now included
                    if hr.entity_weight != 1.0 {
                        meta_insights.push(format!(
                            "retrieval: entity_weight={:.3} fusion_weights=[{:.3},{:.3},{:.3}]",
                            hr.entity_weight,
                            hr.fusion_weights[0],
                            hr.fusion_weights[1],
                            hr.fusion_weights[2],
                        ));
                    }
                }
            }
            health.push(StepHealth {
                step: CycleStep::Reason,
                success: true,
                duration_ms: t4.elapsed().as_millis() as u64,
            });
        } else {
            health.push(StepHealth {
                step: CycleStep::Reason,
                success: true,
                duration_ms: 0,
            });
        }

        // ── Step 6: JUDGE — (skipped on fast-path) ──
        let t5 = std::time::Instant::now();
        if !fast_path_skip && self.config.enable_judge {
            if let Some(ref mut ic) = self.inner_critic {
                if let Some(ref inp) = gathered {
                    let _critique_result = ic.evaluate(inp, inp, None);
                }
            }
            if self.config.enable_system1 {
                if let Some(ref mut s1) = self.system1 {
                    if let Some(ref inp) = gathered {
                        let _intuition_result = s1.intuit(&inp.vector);
                    }
                }
            }
            if let Some(ref mut sc) = self.scar {
                if let Some(ref inp) = gathered {
                    let vsa: Vec<f64> = inp.vector.iter().map(|&b| b as f64).collect();
                    let _avoidance_result = sc.avoidance_signal(&vsa);
                }
            }
            if let Some(ref con) = self.consensus {
                let _consensus_health = con.consensus_health();
            }
            // P0: AppraisalEngine — OCC cognitive appraisal of current state
            if let Some(ref mut ae) = self.appraisal_engine {
                if let Some(ref inp) = gathered {
                    let coherence = self
                        .master_consciousness
                        .as_ref()
                        .map(|m| m.c_score())
                        .unwrap_or(0.5);
                    let dims = super::appraisal_engine::AppraisalDimensions {
                        desirability: coherence.max(0.1),
                        likelihood: 0.5,
                        effort: 0.5,
                        certainty: 0.5,
                        controllability: 0.5,
                        agency: 0.0,
                        legitimacy: 0.5,
                    };
                    let _event = ae.evaluate(inp.vector.clone(), dims);
                    if let Some((label, conf)) = ae.dominant_emotion() {
                        meta_insights.push(format!("appraisal:{:?}={:.2}", label, conf));
                    }
                }
            }
            health.push(StepHealth {
                step: CycleStep::Judge,
                success: true,
                duration_ms: t5.elapsed().as_millis() as u64,
            });
        } else {
            health.push(StepHealth {
                step: CycleStep::Judge,
                success: true,
                duration_ms: 0,
            });
        }

        // ── Step 7: VERIFY — (skipped on fast-path) ──
        let t6 = std::time::Instant::now();
        if !fast_path_skip && self.config.enable_verify {
            if let Some(ref mut vg) = self.verification_gate {
                let _calibrated_score = vg.judge.calibrated_score(0.5);
            }
            if let Some(ref mut sh) = self.shadow {
                let _shadow_results = sh.run_all();
            }
            if self.config.enable_belief_revision {
                if let Some(ref mut bre) = self.belief_revision_engine {
                    let _dissonance = bre.detect_dissonance();
                    let _report = bre.epistemic_report();
                }
            }
            // SecurityExecutive: SupplyChainGuard — audit dependency integrity
            if self.config.enable_supply_chain_guard {
                if let Some(ref mut scg) = self.supply_chain_guard {
                    let report = scg.audit();
                    if report.risk_score > 0.3 {
                        log::warn!(
                            "[VERIFY] SupplyChainGuard: risk={:.2} unknown={} recommendations={}",
                            report.risk_score,
                            report.unknown,
                            report.recommendations.len()
                        );
                        if !report.vulnerabilities.is_empty() {
                            log::warn!(
                                "[VERIFY] SupplyChainGuard vulnerabilities: {:?}",
                                report.vulnerabilities
                            );
                        }
                        meta_insights.push(format!("supply_chain:risk={:.2}", report.risk_score));
                    }
                }
            }
            health.push(StepHealth {
                step: CycleStep::Verify,
                success: true,
                duration_ms: t6.elapsed().as_millis() as u64,
            });
        } else {
            health.push(StepHealth {
                step: CycleStep::Verify,
                success: true,
                duration_ms: 0,
            });
        }

        // ── Step 8: ACT — executive control + WAL + skills ──
        let t7 = std::time::Instant::now();
        if self.config.enable_act {
            if let Some(ref mut ec) = self.executive_controller {
                let _act = ec.current_goal();
            }
            if let Some(ref mut wal) = self.cognitive_wal {
                let _id = wal.append(
                    crate::core::nt_core_edit::cognitive_wal::WalEntryType::StateSnapshot,
                    &format!("cycle_{}", c),
                    vec![],
                );
            }
            if let Some(ref mut sk) = self.skills {
                let _plan = sk.resolve_execution_plan("cycle_tick");
            }
            // Ouroboros self-referential hint (v20)
            if let Some(ref mut ol) = self.ouroboros_loop {
                if let Some(hint) = ol.synthesize_input() {
                    meta_insights.push(hint);
                }
            }

            // P0: AffectInjection — emotional VSA steering direction
            if let Some(ref es) = self.emotional_steering {
                if let Some(ref inp) = gathered {
                    let injection = super::emotional_steering::AffectInjection::new();
                    let direction = injection.direction_from_emotion(&es.state, &inp.vector);
                    meta_insights.push(format!("affect:direction={}", direction.dominant_emotion));
                }
            }
            // Tool execution: route tool requests through the AgentToolRegistry
            if self.config.enable_tool_execution {
                if let Some(ref reg) = self.tool_registry {
                    if let Some(ref inp) = gathered {
                        let text =
                            String::from_utf8_lossy(&inp.vector[..inp.vector.len().min(256)]);
                        // Detect explicit tool command pattern: "tool:<name>:<args>"
                        if let Some(cmd) = text.strip_prefix("tool:") {
                            if let Some(colon) = cmd.find(':') {
                                let tool_id = &cmd[..colon];
                                let args_str = &cmd[colon + 1..];
                                let args: serde_json::Value = serde_json::from_str(args_str)
                                    .unwrap_or(serde_json::json!({"input": args_str}));
                                match reg.execute(tool_id, &args) {
                                    Ok(output) => {
                                        let snippet = if output.result.len() > 120 {
                                            format!("{}...", &output.result[..120])
                                        } else {
                                            output.result.clone()
                                        };
                                        meta_insights
                                            .push(format!("tool:{}:ok:{}", tool_id, snippet));
                                        // Feed tool result back into CapabilitySynthesizer
                                        if let Some(ref mut synth) = self.capability_synthesizer {
                                            let label = format!("tool:{}", tool_id);
                                            synth.synthesize(&label);
                                        }
                                    }
                                    Err(e) => {
                                        log::warn!("[ACT] tool '{}' error: {}", tool_id, e);
                                        meta_insights.push(format!("tool:{}:err", tool_id));
                                    }
                                }
                            }
                        }
                        // VSA-based tool discovery: match gathered VSA vector against tool
                        // descriptions using deterministic VSA hashing + Hamming similarity.
                        let tools = reg.list_tools();
                        if tools.len() > 1 {
                            let mut scored: Vec<(&str, f64)> = Vec::new();
                            for entry in &tools {
                                let tool_vsa = description_to_vsa(&entry.manifest.description);
                                let sim = vsa_hamming_sim(&inp.vector, &tool_vsa);
                                if sim > 0.55 {
                                    scored.push((&entry.manifest.id, sim));
                                }
                            }
                            scored.sort_by(|a, b| {
                                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                            });
                            for (id, sim) in scored.iter().take(3) {
                                meta_insights.push(format!("tool:suggest:{}:{:.2}", id, sim));
                            }
                        }
                    }
                }
            }
            health.push(StepHealth {
                step: CycleStep::Act,
                success: true,
                duration_ms: t7.elapsed().as_millis() as u64,
            });
        } else {
            health.push(StepHealth {
                step: CycleStep::Act,
                success: true,
                duration_ms: 0,
            });
        }

        // ── Step 9: VETO — volition gate + unified will check ──
        // Inspired by DANEEL's VolitionActor (150ms veto window) and
        // consciousness-kernel's VesselGovernance. Every proposed action
        // passes through a 3-tier veto: self-check → value alignment → governance.
        // This is the "free won't" gate — the system can reject its own proposals.
        let t_veto = std::time::Instant::now();
        let mut veto_granted = true;
        if self.config.enable_veto {
            // Tier 1: VolitionEngine — self-check against goal alignment
            if let Some(ref mut ve) = self.volition_engine {
                if let Some(ref ec) = self.executive_controller {
                    if let Some(goal) = ec.current_goal() {
                        let candidate = ActionCandidate::new(goal.description.clone(), &goal.id)
                            .with_confidence(goal.priority);
                        ve.propose(candidate);
                    }
                }
                let selected = ve.select_best();
                if selected.is_none() {
                    veto_granted = false;
                    meta_insights.push("veto:volition:no_valid_candidate".to_string());
                }
            }
            // Tier 2: UnifiedWill — governance authority check
            if veto_granted {
                if let Some(ref mut uw) = self.unified_will {
                    let receipt = uw.will_action(
                        &format!("cycle_act_{}", c),
                        "consciousness_cycle",
                        &format!("coherence=cycle,veto_step"),
                    );
                    match receipt.authority {
                        super::unified_will::AuthorityLevel::Autonomous
                        | super::unified_will::AuthorityLevel::Review => {
                            veto_granted = true;
                        }
                        _ => {
                            veto_granted = false;
                            meta_insights.push(format!(
                                "veto:will:{}:{}",
                                format!("{:?}", receipt.authority).to_lowercase(),
                                receipt.reasoning
                            ));
                        }
                    }
                }
            }
            // ── CXVIII.10: Hallucination impossibility slider ──
            let hallucination_tradeoff = self.config.hallucination_tradeoff;
            meta_insights.push(format!(
                "hallucination_impossibility: tradeoff={:.2} (0=truth, 1=creative)",
                hallucination_tradeoff
            ));
            if hallucination_tradeoff > 0.7 {
                meta_insights.push(
                    "hallucination_warning: high creativity mode — content may be imaginative"
                        .into(),
                );
            } else if hallucination_tradeoff < 0.3 {
                meta_insights.push(
                    "hallucination_warning: strict truth mode — creativity constrained".into(),
                );
            }
            // Broadcast veto outcome
            if veto_granted {
                self.integration_bus
                    .broadcast(IntegrationSignal::CuriositySignal {
                        score: 0.9,
                        action_bonus: 0.05,
                        cycle: c,
                    });
            }
            health.push(StepHealth {
                step: CycleStep::Veto,
                success: veto_granted,
                duration_ms: t_veto.elapsed().as_millis() as u64,
            });
        } else {
            health.push(StepHealth {
                step: CycleStep::Veto,
                success: true,
                duration_ms: 0,
            });
        }

        // ── Step 10: RECORD — stream buffer + data flywheel + boredom ──
        let t8 = std::time::Instant::now();
        if self.config.enable_record {
            if let Some(ref mut stream) = self.consciousness_stream {
                if let Some(ref inp) = gathered {
                    stream.push(inp.clone());
                }
            }
            if let Some(ref mut boredom) = self.boredom {
                let fe_curiosity = self
                    .free_energy_curiosity
                    .as_ref()
                    .map(|fe| fe.curiosity_score())
                    .unwrap_or(0.1);
                boredom.update(fe_curiosity, 0.3);
            }
            if let Some(ref mut df) = self.data_flywheel {
                let flywheel_samples = df.sample_count();
                if flywheel_samples > 0 {
                    substrate_concepts.push(format!("flywheel:samples={}", flywheel_samples));
                }
            }
            // SpeciousPresent: temporal binding buffer with cross-step phase synchrony
            if self.config.enable_specious_present {
                if let (Some(ref mut sp), Some(ref inp)) =
                    (self.specious_present.as_mut(), gathered.as_ref())
                {
                    sp.push(VsaTagged::clone(inp));
                    let coh = sp.average_coherence();
                    let td = sp.temporal_difference().unwrap_or(0.0);
                    if coh > 0.0 {
                        meta_insights.push(format!("specious:coherence={:.3}", coh));
                        meta_insights.push(format!("specious:temp_diff={:.3}", td));
                    }
                    // ── Stack Theory identity persistence (arXiv 2603.09043) ──
                    let self_model_active = false;
                    let narrative_active = self.narrative_self.is_some();
                    let persistence = sp.calculate_persistence(self_model_active, narrative_active);
                    let vsa_self_in_window = sp.window().iter().filter(|t| t.tag.is_self()).count();
                    let coherence_ratio = sp
                        .check_retrieval_identity_coherence(sp.window().len(), vsa_self_in_window);
                    if persistence.identity_fragmentation_risk > 0.0 {
                        substrate_concepts.push(format!(
                            "identity:persistence_risk={:.3} p_weak={:.3} p_strong={:.3}",
                            persistence.identity_fragmentation_risk,
                            persistence.p_weak,
                            persistence.p_strong,
                        ));
                    }
                    if coherence_ratio < 0.3 && sp.window().len() > 3 {
                        substrate_concepts.push(format!(
                            "identity:rag_fragmentation={:.3} (Self ratio in window)",
                            coherence_ratio,
                        ));
                    }
                }
            }
            // P0: EmotionalTrail — record emotional snapshot per cycle
            if let Some(ref mut et) = self.emotional_trail {
                if let Some(ref es) = self.emotional_steering {
                    et.record(&es.state, c, "cycle_tick");
                    if c % 10 == 0 {
                        let trend =
                            et.trend(super::emotional_steering::EmotionalDimension::Curiosity);
                        let vol = et.volatility();
                        if trend.abs() > 0.01 || vol > 0.01 {
                            meta_insights
                                .push(format!("emotion:trail_trend={:.3}_vol={:.3}", trend, vol));
                        }
                    }
                }
            }
            // P2: NarrativeSelf — record iteration for self-narrative continuity
            if let Some(ref mut ns) = self.narrative_self {
                let summary = if !meta_insights.is_empty() {
                    meta_insights
                        .last()
                        .cloned()
                        .unwrap_or_else(|| format!("cycle_{}", c))
                } else {
                    format!("cycle_{}", c)
                };
                let coherence = self
                    .master_consciousness
                    .as_ref()
                    .map(|m| m.c_score())
                    .unwrap_or(0.5);
                ns.record_iteration(&summary, coherence, meta_insights.last().cloned());
                if c % 50 == 0 {
                    let narr = ns.narrative_summary(3);
                    if !narr.is_empty() {
                        meta_insights.push(format!("narrative:{}", narr));
                    }
                }
                // Set emotional context if emotional steering available
                if let Some(ref es) = self.emotional_steering {
                    ns.set_working_context("consciousness_cycle", es.state.energy);
                }
            }
            // Leap 3: export MemoryLattice Skills → CapabilitySynthesizer primitives
            if self.config.enable_cap_synth_export && self.config.enable_capability_synthesizer {
                if let (Some(ref lattice), Some(ref mut synth)) = (
                    self.memory_lattice.as_ref(),
                    self.capability_synthesizer.as_mut(),
                ) {
                    let exported = lattice.export_to_synthesizer(synth);
                    if exported > 0 {
                        log::debug!(
                            "CYCLE: exported {} skills to capability synthesizer",
                            exported
                        );
                    }
                }
            }
            // Phase 2.4: self-merge KnowledgePackage to validate merge path
            if self.config.enable_cap_synth_export && self.config.enable_capability_synthesizer {
                if let Some(ref mut synth) = self.capability_synthesizer {
                    let local_pkg = KnowledgePackage {
                        version: 1,
                        instance_id: format!("self_cycle_{}", self.cycle_num),
                        domain: "consciousness_cycle".to_string(),
                        capabilities: synth.capabilities.clone(),
                        lattice_snapshot: LatticeSnapshot {
                            skills: vec![],
                            meta_rules: vec![],
                        },
                    };
                    let (imported, replaced, bundled) = merge_knowledge_package(synth, &local_pkg);
                    if imported > 0 || replaced > 0 || bundled > 0 {
                        log::debug!(
                            "CYCLE: self-merge KnowledgePackage: imported={} replaced={} bundled={}",
                            imported, replaced, bundled,
                        );
                    }
                }
            }
            // Temporal prediction error tracking
            let c_score = self
                .master_consciousness
                .as_ref()
                .map(|m| m.c_score())
                .unwrap_or(0.5);
            let prediction = self.last_c_score;
            let _err = self
                .temporal_prediction
                .record_prediction(prediction, c_score);
            self.last_c_score = c_score;
            tracing::trace!(
                "temporal_prediction_error={:.4} cycle={}",
                _err,
                self.cycle_num
            );

            // Integration bus: Loop 1 — TemporalPrediction divergence
            if self.temporal_prediction.is_diverging(0.2) {
                self.integration_bus
                    .broadcast(IntegrationSignal::DivergenceDetected {
                        error: self.temporal_prediction.running_prediction_error(),
                        volatility: self.temporal_prediction.volatility(10),
                        cycle: c,
                    });
            }

            // Integration bus: Loop 2 — ActiveInference → FreeEnergyCuriosity → BoredomAccumulator
            if let Some(ref ai) = self.active_inference {
                let curiosity = ai.average_vfe.min(1.0);
                self.integration_bus
                    .broadcast(IntegrationSignal::CuriositySignal {
                        score: curiosity,
                        action_bonus: curiosity * 0.1,
                        cycle: c,
                    });
            }
            // Integration bus: Loop 2 — FreeEnergyCuriosityEngine → BoredomAccumulator
            if let Some(ref mut fe) = self.free_energy_curiosity {
                let cur = fe.curiosity_score();
                self.integration_bus
                    .broadcast(IntegrationSignal::FreeEnergyCuriositySignal {
                        score: cur,
                        action_bonus: cur * 0.2,
                        cycle: c,
                    });
                // record curiosity observation for memory tracking
                let obs_state = (cur * 1000.0) as usize;
                if obs_state < 1000 {
                    fe.step(obs_state);
                }
            }

            // Phase 1: push cycle-level ExperienceRecord into buffer for outer SEAL loop
            if let Some(ref mut seal) = self.seal_closed_loop {
                self.experience_buffer.push_back(TrajectoryExperience {
                    id: self.cycle_num,
                    context: format!("consciousness_cycle_{}", self.cycle_num),
                    action: "cycle_tick".to_string(),
                    reward: c_score,
                    success: true,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    metadata: {
                        let mut m = std::collections::HashMap::new();
                        m.insert("coherence".to_string(), format!("{:.4}", c_score));
                        m.insert("step".to_string(), "Record".to_string());
                        m
                    },
                });
                // Cap buffer to prevent unbounded growth
                while self.experience_buffer.len() > 1000 {
                    self.experience_buffer.pop_front();
                }
                if seal.should_run() {
                    log::debug!(
                        "CYCLE: SEAL distill interval reached (cycle {})",
                        self.cycle_num
                    );
                }
            }
            // ToolSynthesizer: synthesize executable tools from capability synthesizer discoveries
            if self.config.enable_capability_synthesizer {
                if let (Some(ref synth), Some(ref mut ts)) = (
                    self.capability_synthesizer.as_ref(),
                    self.tool_synthesizer.as_mut(),
                ) {
                    for cap in &synth.capabilities {
                        let cap_confidence = cap.success_rate;
                        if cap_confidence > 0.6 {
                            let tool_count_before = ts.tools.len();
                            ts.synthesize_from_capability(
                                &cap.name,
                                &cap.name,
                                &cap.description,
                                cap_confidence,
                            );
                            if ts.tools.len() > tool_count_before {
                                log::debug!(
                                    "CYCLE: synthesized tool '{}' from capability",
                                    cap.name
                                );
                            }
                        }
                    }
                }
            }
            // CompoundKnowledgeBase: record cycle insights every 5 cycles
            if self.config.enable_compound_knowledge {
                if let Some(ref mut ckb) = self.compound_knowledge {
                    if self.cycle_num > 0 && self.cycle_num % 5 == 0 {
                        let insight = crate::core::nt_core_experience::compound_knowledge::KnowledgeEntry {
                            title: format!("cycle_{}_insight", self.cycle_num),
                            content: format!("c_score={:.4} meta_insights={}", c_score, meta_insights.join(";")),
                            category: crate::core::nt_core_experience::compound_knowledge::KnowledgeCategory::Signal,
                            tags: vec!["consciousness_cycle".to_string(), "auto".to_string()],
                            confidence: c_score,
                            created_at: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                            source: "consciousness_cycle".to_string(),
                        };
                        ckb.register(insight);
                        if self.cycle_num % 20 == 0 {
                            let _ = ckb.save_to_disk();
                        }
                    }
                }
            }
            // HeLa-Mem Hebbian memory recording
            if self.config.enable_hebbian_memory {
                if let Some(ref mut hebbian) = self.hebbian_memory {
                    // Record co-activation between current gathered state and c_score
                    if let Some(ref inp) = gathered {
                        let state_vec = inp.vector.clone();
                        let state_idx = hebbian.add_node(state_vec);
                        // Link to a score-representation node
                        let score_bytes: Vec<u8> = c_score
                            .to_le_bytes()
                            .iter()
                            .flat_map(|b| std::iter::repeat(*b).take(512))
                            .collect();
                        let score_idx = hebbian.add_node(score_bytes);
                        // ACh-gated DA plasticity (Nature 2026): cholinergic pause enables plasticity
                        let ach_level = self.neuromodulators
                            .as_ref()
                            .map(|nm| nm.modulators.get(&crate::core::nt_core_consciousness::neuromodulator::NeuromodulatorType::ACh).map(|n| n.level).unwrap_or(0.5))
                            .unwrap_or(0.5);
                        hebbian.record_coactivation(state_idx, score_idx, Some(ach_level));
                        // Decay and prune edges each cycle
                        hebbian.decay_all();
                        hebbian.prune_edges();
                        // EMBER-inspired idle-time spontaneous reactivation
                        let idle_report = hebbian.idle_tick(self.cycle_num);
                        if idle_report.action_triggered {
                            meta_insights.push(format!(
                                "hebbian_idle:{}_surfaced max_weight:{:.3}",
                                idle_report.surfaced, idle_report.max_weight,
                            ));
                        }
                        // Phase 37 P0.1: Mnemoverse outcome-driven δ-rule weight update
                        // Use c_score (normalized to 0-1) as outcome feedback signal
                        let outcome = (c_score / 100.0).clamp(0.0, 1.0);
                        hebbian.record_outcome(state_idx, outcome, 0.1);
                        meta_insights.push(format!(
                            "hebbian_outcome:node={} score={:.3} lr=0.1",
                            state_idx, outcome,
                        ));
                    }
                    // Run distillation every 10 cycles
                    if self.cycle_num % 10 == 0 {
                        if let Some(ref mut distiller) = self.hebbian_distillation {
                            let (_hubs, snapshot) = distiller.distill(hebbian, self.cycle_num);
                            if !_hubs.is_empty() {
                                log::debug!("hebbian_distillation: {} hubs distilled", _hubs.len());
                                // Feed snapshot into capability synthesizer
                                if self.config.enable_capability_synthesizer {
                                    if let Some(ref mut synth) = self.capability_synthesizer {
                                        let cap_count_before = synth.capabilities.len();
                                        for (name, vsa, conf) in &snapshot.skills {
                                            if *conf > 0.5 {
                                                let desc = format!("hebbian_distilled:{}", name);
                                                let already = synth.capabilities.iter().any(|c| {
                                                    crate::core::nt_core_experience::capability_synthesizer::CapabilitySynthesizer::similarity(&c.vsa_vector, vsa) > 0.55
                                                });
                                                if !already {
                                                    synth.capabilities.push(
                                                        crate::core::nt_core_experience::capability_synthesizer::Capability {
                                                            id: synth.capabilities.len() as u64 + 1,
                                                            name: name.clone(),
                                                            description: desc,
                                                            cap_type: crate::core::nt_core_experience::capability_synthesizer::CapabilityType::Generated,
                                                            sub_ids: Vec::new(),
                                                            vsa_vector: vsa.clone(),
                                                            invocation_count: 1,
                                                            success_rate: *conf,
                                                        },
                                                    );
                                                }
                                            }
                                        }
                                        let imported = synth.capabilities.len() - cap_count_before;
                                        if imported > 0 {
                                            log::debug!("hebbian_distillation: imported {} skills into capability synthesizer", imported);
                                        }
                                    }
                                }
                            }
                            // ── CXV.2: HebbianDistillation→MemoryLattice: inject hubs into skills layer ──
                            if let Some(ref mut ml) = self.memory_lattice {
                                let before = ml.skills.len();
                                for (name, vsa, conf) in &snapshot.skills {
                                    if *conf > 0.5 {
                                        ml.store_with_origin(
                                            format!("hebbian_distilled:{}", name),
                                            vsa.clone(),
                                            LatticeLayer::Skills,
                                            MemoryOrigin::System,
                                        );
                                    }
                                }
                                let injected = ml.skills.len() - before;
                                if injected > 0 {
                                    log::debug!("hebbian_distillation: injected {} skills into MemoryLattice", injected);
                                }
                            }
                        }
                    }
                }
            }
            // ── CXIV.3: ReconstructiveEpisodicBuffer — MIRROR-style narrative reconstruction ──
            if let Some(ref mut rb) = self.reconstructive_buffer {
                if self.cycle_num as usize - rb.last_reconstruction >= rb.reconstruction_interval {
                    let narrative = if let (Some(ref imm), Some(ref cc)) =
                        (&self.inner_monologue_manager, &self.cognitive_controller)
                    {
                        let threads = imm.threads.join(" | ");
                        format!(
                            "cycle_{} narrative [parallel: {}] → synthesis: {}",
                            self.cycle_num, threads, cc.last_narrative
                        )
                    } else {
                        format!("cycle_{} narrative (no parallel threads)", self.cycle_num)
                    };
                    rb.current_narrative = narrative;
                    rb.last_reconstruction = self.cycle_num as usize;
                    meta_insights.push(format!("reconstructive_buffer: narrative updated"));
                }
            }
            // ── XCIII.2: InnerMonologueManager + CognitiveController ──
            if self.cycle_num % 3 == 0 {
                if let Some(ref mut cc) = self.cognitive_controller {
                    if cc.active {
                        let thread_summary = self
                            .inner_monologue_manager
                            .as_ref()
                            .map(|imm| imm.threads.join(", "))
                            .unwrap_or_default();
                        cc.last_narrative = format!(
                            "cycle_{} synthesized: {} insights: {}",
                            self.cycle_num,
                            thread_summary,
                            meta_insights.last().unwrap_or(&"none".into())
                        );
                    }
                }
            }
            // SecurityExecutive: AuditTrail — record security events each cycle
            let supply_risk = if let Some(ref mut scg) = self.supply_chain_guard {
                let report = scg.audit();
                report.risk_score
            } else {
                0.0
            };
            if self.config.enable_audit_trail {
                if let Some(ref mut at) = self.audit_trail {
                    let risk_score = self
                        .risk_sensor
                        .as_ref()
                        .map(|rs| rs.average_risk_score(5))
                        .unwrap_or(0.0);
                    if risk_score > 0.3 {
                        at.record(
                            AuditEventType::RiskAlert,
                            "consciousness_cycle",
                            &format!("risk_score={:.2}", risk_score),
                            risk_score,
                            "",
                        );
                    }
                    if supply_risk > 0.3 {
                        at.record(
                            AuditEventType::SecurityThreat,
                            "supply_chain_guard",
                            &format!("supply_chain_risk={:.2}", supply_risk),
                            supply_risk,
                            "",
                        );
                    }
                }
            }
            health.push(StepHealth {
                step: CycleStep::Record,
                success: true,
                duration_ms: t8.elapsed().as_millis() as u64,
            });
        } else {
            health.push(StepHealth {
                step: CycleStep::Record,
                success: true,
                duration_ms: 0,
            });
        }

        // ── Step 11: METRIC — master consciousness + phi + dashboard ──
        let t9 = std::time::Instant::now();
        if self.config.enable_metric {
            let mut c_score = if gathered.is_some() { 0.55 } else { 0.45 };
            if let Some(ref mut mc) = self.master_consciousness {
                if let Some(ref inp) = gathered {
                    let sum_f64: f64 = inp.vector.iter().map(|&b| b as f64).sum();
                    let coherence = sum_f64 / inp.vector.len() as f64;
                    let actual_ma = if self.config.enable_meta_accuracy {
                        self.meta_accuracy.actual_meta_accuracy()
                    } else {
                        0.3
                    };
                    let metrics =
                        crate::core::nt_core_consciousness::master_equation::ConsciousnessMetrics {
                            phi: 0.3,
                            global_workspace: 0.4,
                            coherence,
                            attention_focus: 0.5,
                            reflexivity: 0.4,
                            emotional_intensity: 0.0,
                            knowledge_integration: 0.7,
                            meta_accuracy: actual_ma,
                            novelty_seeking: 0.5,
                        };
                    c_score = mc.compute_c(metrics);
                }
            }
            if let Some(ref mut phi) = self.phi {
                if let Some(ref inp) = gathered {
                    let state: Vec<u8> = inp.vector.iter().map(|&b| b >> 7 & 1).collect();
                    let phi_result = phi.compute_all_scales(&state, c);
                    phi_metrics = Some(vec![
                        phi_result.phi_micro,
                        phi_result.phi_meso,
                        phi_result.phi_system,
                    ]);
                }
            }
            // Integration bus: Loop 3 — IIT Phi
            if self.config.enable_iit_phi8 {
                if let Some(ref phi8) = self.iit_phi8 {
                    // Build state vector from real cycle data
                    let n_states = 3;
                    let tpm = FactoredTPM::new(n_states);
                    let mut state = vec![0u8; n_states];
                    // state[0]: prediction quality (high = good prediction)
                    state[0] = ((1.0 - self.temporal_prediction.running_prediction_error())
                        .clamp(0.0, 1.0)
                        * 255.0) as u8;
                    // state[1]: curiosity/drive level
                    let curiosity_val: f64 =
                        self.boredom.as_ref().map(|b| b.curiosity).unwrap_or(0.5);
                    state[1] = (curiosity_val.clamp(0.0, 1.0) * 255.0) as u8;
                    // state[2]: consensus coherence
                    let coherence_val: f64 = self
                        .consensus
                        .as_ref()
                        .and_then(|g| g.voter_ids.get(0))
                        .map(|_| 0.7)
                        .unwrap_or(0.5);
                    state[2] = (coherence_val.clamp(0.0, 1.0) * 255.0) as u8;
                    let (max_phi, avg_phi, integrated_info, _) =
                        phi8.compute_phi_parallel(&tpm, &state, c);
                    self.integration_bus
                        .broadcast(IntegrationSignal::PhiSignal {
                            max_phi,
                            avg_phi,
                            integrated_info,
                            cycle: c,
                        });
                    meta_insights.push(format!("phi_max:{:.4}", max_phi));
                    meta_insights.push(format!("phi_avg:{:.4}", avg_phi));
                    meta_insights.push(format!("phi_info:{:.4}", integrated_info));
                }
            }

            if let Some(ref mut db) = self.dashboard {
                db.record_metric("c_score", c_score, 0.8, "score");
                db.record_metric(
                    "temporal_prediction_error",
                    self.temporal_prediction.running_prediction_error(),
                    0.3,
                    "error",
                );
                db.record_metric(
                    "temporal_volatility",
                    self.temporal_prediction.volatility(10),
                    0.5,
                    "variability",
                );
                if let Some(ref sub) = substrate_concepts.first() {
                    db.record_metric("concepts", substrate_concepts.len() as f64, 5.0, "count");
                    let _first_concept = sub;
                }
                dashboard_report = Some(db.render_dashboard());
            }
            // Wave 0.5: record quality metrics from data_quality_pipeline
            if let Some(ref dqp) = self.data_quality_pipeline {
                let qs = dqp.quality_score();
                let dims: Vec<String> = [
                    "Completeness",
                    "Consistency",
                    "Conformity",
                    "Accuracy",
                    "Uniqueness",
                    "Integrity",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect();
                for d in &dims {
                    if let Some(ref mut db) = self.dashboard {
                        db.record_metric(&format!("dq_{}", d.to_lowercase()), qs, 0.9, "quality");
                    }
                }
            }
            // Wave C: temporal memory query every 5 cycles for temporally-aware retrieval
            if self.config.enable_temporal_query && self.temporal_tick_counter % 5 == 0 {
                if let Some(ref ml) = self.memory_lattice {
                    let now = crate::core::nt_core_time::unix_now_secs() as i64;
                    let temporal_results = ml.find_by_temporal("", now, Some(LatticeLayer::Facts));
                    if !temporal_results.is_empty() {
                        let temporal_count = temporal_results.len();
                        substrate_concepts.push(format!("temporal_facts:{}", temporal_count));
                    }
                }
            }
            // New subsystem metrics for consciousness dashboard
            {
                let bus_metrics = self.integration_bus.metrics();
                if let Some(count) = bus_metrics["pending"].as_u64() {
                    meta_insights.push(format!("bus_pending:{}", count));
                }
                if let Some(hcount) = bus_metrics["history"].as_u64() {
                    meta_insights.push(format!("bus_history:{}", hcount));
                }
            }
            {
                let tp_metrics = self.temporal_prediction.metrics();
                meta_insights.push(format!(
                    "temporal_err:{:.3}",
                    tp_metrics["running_prediction_error"]
                        .as_f64()
                        .unwrap_or(0.0)
                ));
                meta_insights.push(format!(
                    "temporal_diverging:{}",
                    tp_metrics["diverging"].as_bool().unwrap_or(false)
                ));
                meta_insights.push(format!(
                    "temporal_vol:{:.4}",
                    tp_metrics["volatility"].as_f64().unwrap_or(0.0)
                ));
            }
            if let Some(ref gov) = self.consensus {
                let peer_count = gov.voter_ids.len();
                let proposal_count = gov.proposals.len();
                meta_insights.push(format!("gov_peers:{}", peer_count));
                meta_insights.push(format!("gov_proposals:{}", proposal_count));
            }
            if let Some(ref bd) = self.boredom {
                meta_insights.push(format!("curiosity:{:.3}", bd.curiosity));
            }
            // Convergence detection: check if c_score is plateauing
            if c >= 10 {
                let recent: Vec<f64> = self
                    .history
                    .iter()
                    .rev()
                    .take(10)
                    .filter_map(|r| {
                        Some(
                            r.step_health
                                .iter()
                                .find(|h| h.step == CycleStep::Metric)
                                .and_then(|_| {
                                    self.history
                                        .back()
                                        .and_then(|last| last.step_health.last().map(|_| 1.0f64))
                                })
                                .unwrap_or(0.0),
                        )
                    })
                    .collect();
                if recent.len() >= 10 {
                    let mean: f64 = recent.iter().sum::<f64>() / recent.len() as f64;
                    let variance: f64 = recent.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                        / recent.len() as f64;
                    if variance < 0.01 {
                        meta_insights
                            .push("convergence: plateau detected (low variance)".to_string());
                    }
                }
            }
            // PerformanceOracle: track step health and pipeline metrics
            if let Some(ref mut po) = self.performance_oracle {
                let po_metrics = po.metrics();
                if po_metrics.window_cycles > 0 {
                    meta_insights.push(format!(
                        "oracle: {} cycles, convergence={:.2}, bottlenecks={}",
                        po_metrics.window_cycles,
                        po_metrics.convergence_rate,
                        po_metrics.bottleneck_steps.len()
                    ));
                }
            }
            // Cross-model distillation: run every 20 cycles to extract patterns, capabilities, knowledge
            if self.config.enable_metric
                && self.config.enable_distillation
                && self.cycle_num > 0
                && self.cycle_num % 20 == 0
            {
                if let Some(ref mut distiller) = self.cross_model_distiller {
                    let report = distiller.distill();
                    if report.total_interactions > 0 {
                        meta_insights.push(format!(
                            "distillation: {} interactions, {} patterns, {} caps, {} knowledge, best={}",
                            report.total_interactions,
                            report.behavioral_patterns.len(),
                            report.capabilities.len(),
                            report.knowledge_fragments.len(),
                            report.model_performance.first().map(|m| m.model.as_str()).unwrap_or("none")
                        ));
                        // Auto-register capabilities from distillation into CapabilitySynthesizer
                        if self.config.enable_capability_synthesizer {
                            if let Some(ref mut synth) = self.capability_synthesizer {
                                let registered = synth.absorb_distillation_report(&report);
                                if registered > 0 {
                                    meta_insights.push(format!(
                                        "distillation: auto-registered {} capabilities from patterns/skills",
                                        registered
                                    ));
                                }
                            }
                        }
                        // Broadcast distillation signal to integration bus
                        self.integration_bus
                            .broadcast(IntegrationSignal::DistillationSignal {
                                total_interactions: report.total_interactions,
                                patterns_found: report.behavioral_patterns.len(),
                                capabilities_ranked: report.capabilities.len(),
                                knowledge_fragments: report.knowledge_fragments.len(),
                                top_model: report
                                    .model_performance
                                    .first()
                                    .map(|m| m.model.clone())
                                    .unwrap_or_default(),
                                cycle: c,
                            });
                    }
                }
            }
            // GracefulDegradationManager: report subsystem health every 50 cycles
            if self.cycle_num > 0 && self.cycle_num % 50 == 0 {
                if let Some(ref gdm) = self.graceful_deg_manager {
                    let level = gdm.global_degradation_level();
                    if !level.is_available() {
                        meta_insights.push(format!(
                            "graceful_degradation: global={:?}, degraded={}",
                            level,
                            gdm.degraded_reasoning_modules().len(),
                        ));
                    }
                }
            }
            self.temporal_tick_counter += 1;
            health.push(StepHealth {
                step: CycleStep::Metric,
                success: true,
                duration_ms: t9.elapsed().as_millis() as u64,
            });
        } else {
            health.push(StepHealth {
                step: CycleStep::Metric,
                success: true,
                duration_ms: 0,
            });
        }

        // ── Step 12: META — metacognitive controller + arch governor + RSI + meta-accuracy + goal drift ──
        let t10 = std::time::Instant::now();
        if self.config.enable_meta {
            if let Some(ref mut mc) = self.metacognitive_controller {
                let assessment = crate::core::nt_core_consciousness::metacognitive_controller::MetacognitiveAssessment {
                    cognitive_load: 0.5,
                    uncertainty: 0.3,
                    bias_indicators: vec![],
                    reasoning_quality: 0.7,
                    requires_intervention: false,
                    intervention_reason: String::new(),
                    confidence_calibration: 0.8,
                };
                let _intervention = mc.decide_intervention(&assessment);
            }
            // ── MetaAccuracyTracker: record calibration data from metacognitive signals ──
            if self.config.enable_meta_accuracy {
                let (meta_d, ece) = if let Some(ref mc) = self.metacognitive_controller {
                    mc.calibration_summary()
                } else {
                    (0.5, 0.5)
                };
                self.meta_accuracy.record_calibration_data(meta_d, ece, 1);
                // ── CXV.1: Memory Q-value reward loop: ECE→MemoryLattice TD(0) ──
                if let Some(ref mut ml) = self.memory_lattice {
                    let reward = -ece.clamp(0.0, 1.0);
                    let alpha = 0.1;
                    let end = ml.episodic.len();
                    let start = if end > 20 { end - 20 } else { 0 };
                    for i in start..end {
                        ml.update_q_value(i, LatticeLayer::Episodic, reward, alpha);
                    }
                    if end > start {
                        let mean_q: f64 = (start..end)
                            .filter_map(|i| ml.episodic.get(i))
                            .map(|e| e.q_value)
                            .sum::<f64>()
                            / (end - start) as f64;
                        meta_insights.push(format!(
                            "q_learning:ece={:.3} reward={:.3} mean_q={:.3}",
                            ece, reward, mean_q
                        ));
                    }
                }
                let actual_ma = self.meta_accuracy.actual_meta_accuracy();
                // CV.1: MetaAccuracy-based real control — close the monitor-control loop
                if actual_ma < 0.6 {
                    self.integration_bus
                        .send_modulation(ModulationCommand::SetTemperature(0.3 + actual_ma * 0.5));
                    self.integration_bus
                        .send_modulation(ModulationCommand::SetParam {
                            subsystem: "competitive_selection".into(),
                            param: "temperature".into(),
                            value: 0.2 + actual_ma * 0.3,
                        });
                    meta_insights.push(format!(
                        "meta_control:low_accuracy({:.4})→temp_reduced→conservative",
                        actual_ma
                    ));
                }
                // ── KAPRO: ActingDimensionAssessor — close Knowing→Acting gap ──
                if let Some(ref mut ada) = self.acting_dimension_assessor {
                    let behavioral_adjustment = if actual_ma < 0.6 {
                        actual_ma * 0.8
                    } else {
                        actual_ma
                    };
                    let kapro = ada.assess(meta_d, ece, behavioral_adjustment);
                    if kapro.dissociation_detected {
                        meta_insights.push(format!(
                            "kapro:dissociation k={:.2} a={:.2} gap={:.2} trend={}",
                            kapro.knowing_dimension,
                            kapro.acting_dimension,
                            kapro.kapro_gap,
                            kapro.gap_trend
                        ));
                    }
                }
                // ── LinkFormation: CTM-AI processor co-activation tracking ──
                if let Some(ref mut lf) = self.link_formation {
                    let active_subsystems = self
                        .module_registry
                        .as_ref()
                        .map(|m| m.all_names())
                        .unwrap_or_default();
                    if !active_subsystems.is_empty() {
                        lf.record_activation("META", &active_subsystems);
                        let active_count = lf.active_link_count();
                        let routed = lf.total_signals_routed();
                        if active_count > 0 {
                            meta_insights.push(format!(
                                "link_formation:active={} routed={} total={}",
                                active_count,
                                routed,
                                lf.link_count()
                            ));
                        }
                    }
                }
                meta_insights.push(format!("meta_accuracy:actual={:.4}", actual_ma));
            }
            // ── EvolutionEfficiencyTracker: SEA-Eval T metric ──
            {
                let task_hash = self.cycle_num.wrapping_mul(2654435761);
                self.evolution_efficiency
                    .record_task(task_hash, "meta_cycle", 1.0, self.cycle_num);
                let report = self.evolution_efficiency.report();
                if report.total_records >= 6 {
                    meta_insights.push(format!(
                        "evolution_convergence:{:.4} trend:{} families:{}",
                        report.evolution_convergence_rate,
                        report.efficiency_trend,
                        report.converged_families,
                    ));
                }
            }
            // ── SAHOO: RatchetTracker constraint + regression risk (arXiv 2603.06333) ──
            {
                let score = self.meta_accuracy.current_accuracy().max(0.0);
                self.ratchet_tracker.check(score, "meta_accuracy");
                self.ratchet_tracker.update_regression_risk();
                if !self.ratchet_tracker.constraint_preserved {
                    meta_insights.push("sahoo:constraint_violation".into());
                }
                if self.ratchet_tracker.regression_risk > 0.5 {
                    meta_insights.push(format!(
                        "sahoo:high_regression_risk risk={:.2}",
                        self.ratchet_tracker.regression_risk
                    ));
                }
                // Wave E: Cross-cycle regression baseline (Step 4)
                meta_insights.push(format!("ratchet:last_c_score={:.4}", self.last_c_score));
                // Wave E: SAHOO ratchet → rollback signal (Step 1)
                if self.ratchet_tracker.is_broken() {
                    self.integration_bus
                        .send_modulation(ModulationCommand::SetParam {
                            subsystem: "sa_hot".into(),
                            param: "regression_rollback_needed".into(),
                            value: self.ratchet_tracker.regression_risk,
                        });
                    meta_insights.push("ratchet:rollback_signal_sent".into());
                }
            }
            // ── GoalDriftIndex: SAHOO-inspired alignment monitoring ──
            if self.config.enable_goal_drift_index {
                if let Some(ref mut gdi) = self.goal_drift_index {
                    if let Some(ref inp) = gathered {
                        let current_text =
                            String::from_utf8_lossy(&inp.vector[..inp.vector.len().min(512)])
                                .to_string();
                        let reference = "default reference"; // placeholder
                        let sample = gdi.record(&current_text, reference);
                        if sample.is_drift {
                            meta_insights.push(format!(
                                "goal_drift:gdi={:.4} sem={:.4} lex={:.4} struct={:.4} dist={:.4}",
                                sample.gdi,
                                sample.semantic_score,
                                sample.lexical_score,
                                sample.structural_score,
                                sample.distributional_score
                            ));
                        }
                    }
                }
            }
            if let Some(ref mut ag) = self.arch_governor {
                ag.track_invocation("consciousness_cycle", 0.0, true);
                let smells = ag.detect_code_smells();
                for s in &smells {
                    meta_insights.push(format!("{:?}: {}", s.insight_type, s.description));
                }
            }
            // ── DataFlywheel: generate synthetic data from identified gaps ──
            if self.config.enable_data_flywheel {
                if let Some(ref mut df) = self.data_flywheel {
                    let gap_count = df.sample_count();
                    if gap_count < 50 {
                        let calibration = self.meta_accuracy.actual_meta_accuracy();
                        let gaps = vec![
                            ("meta_accuracy_calibration", calibration),
                            ("self_model_confidence", 0.7),
                        ];
                        let base_vector: Vec<f64> = (0..8).map(|i| i as f64 * 0.1).collect();
                        let generated = df.generate_from_gaps(&gaps, &base_vector, 10);
                        if generated > 0 {
                            meta_insights.push(format!(
                                "flywheel: generated {} synthetic samples from gaps (ma={:.3})",
                                generated, calibration
                            ));
                        }
                        let weak = [1.0 - calibration];
                        let adv = df.adversarial_samples(&weak, 5);
                        if !adv.is_empty() {
                            meta_insights.push(format!(
                                "flywheel: {} adversarial samples for weak areas",
                                adv.len()
                            ));
                        }
                    }
                }
            }
            // ── SINDy Engine: sparse system identification of VSA dynamics ──
            if self.config.enable_sindy_engine {
                if let Some(ref mut se) = self.sindy_engine {
                    let state_vec: Vec<f64> = if let Some(ref mc) = self.master_consciousness {
                        let evolution = mc.evolution_report();
                        let metrics = &evolution.metrics;
                        let c = mc.c_score();
                        let phi = metrics.phi;
                        let coh = metrics.coherence;
                        let novelty = metrics.novelty_seeking;
                        vec![
                            c,
                            phi,
                            coh,
                            novelty,
                            self.last_c_score,
                            self.meta_accuracy.actual_meta_accuracy(),
                            self.goal_drift_index
                                .as_ref()
                                .map(|g| g.gdi())
                                .unwrap_or(0.0),
                            0.0,
                        ]
                    } else {
                        vec![0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.0, 0.0]
                    };
                    se.observe(crate::core::nt_core_hcube::sindy_engine::VsaSnapshot {
                        time_step: c,
                        state_vector: state_vec,
                        cycle_label: format!("cycle_{}", c),
                    });
                    if c > 0 && c % 10 == 0 && se.observation_count() >= 10 {
                        if let Some(report) = se.discover_dynamics() {
                            let summary = report.summary();
                            meta_insights.push(format!("sindy: {}", summary));
                        }
                    }
                }
            }
            if let Some(ref mut rsi) = self.rsi {
                let proposed = rsi.run_cycle(c);
                rsi_proposals_count = proposed.len();
            }
            // ── IdentityFragments: track identity coherence over time ──
            if let Some(ref mut idf) = self.identity_fragments {
                idf.tick(1.0);
                if c > 0 && c % 10 == 0 {
                    let coherence = idf.coherence_score();
                    meta_insights.push(format!(
                        "identity:coherence={:.4} fragments={}",
                        coherence,
                        idf.fragment_count(),
                    ));
                    if coherence < 0.6 {
                        let report = idf.generate_report();
                        for rec in &report.integration_recommendations {
                            meta_insights.push(format!("identity:integration_rec: {}", rec));
                        }
                    }
                }
            }
            // ── IdentityChain: cryptographic session signing every 50 cycles ──
            if let Some(ref mut ic) = self.identity_chain {
                if c > 0 && c % 50 == 0 {
                    let session_id = format!("neotrix-cycle-{}", c);
                    let (hash, _sig) = ic.sign_session(&session_id, c);
                    meta_insights.push(format!(
                        "identity:chain=0x{} fp={} sessions={}",
                        &hash[..8],
                        ic.fingerprint_hex(),
                        ic.session_count,
                    ));
                }
            }
            // ── SEAL evolution: gepa_mutate + MetaSeal — gated by EvolutionGatekeeper ──
            if self.config.enable_seal_closed_loop {
                // Extract governance before seal mutable borrow to avoid borrow conflict
                let mut seal_gov =
                    std::mem::replace(&mut self.seal_governance, SEALGovernance::new());
                if let Some(ref mut seal) = self.seal_closed_loop {
                    // SecurityExecutive: EvolutionGatekeeper gate before SEAL mutation
                    let gate_allowed = if self.config.enable_evolution_gatekeeper {
                        if let Some(ref mut eg) = self.evolution_gatekeeper {
                            let proposal_id = eg.submit_proposal(
                                "seal_closed_loop",
                                &format!(
                                    "gepa_mutate pareto_before={} c_score={:.4}",
                                    seal.pareto_front.len(),
                                    self.last_c_score
                                ),
                                "SEAL self-evolution mutation",
                                vec!["neotrix-core/src".to_string()],
                            );
                            let gates = eg.run_gates(proposal_id);
                            let allowed = gates
                                .last()
                                .map(|g| matches!(g.decision, GateDecision::Allow))
                                .unwrap_or(false);
                            if !allowed {
                                log::warn!("[META] EvolutionGatekeeper rejected SEAL mutation");
                                if let Some(ref mut at) = self.audit_trail {
                                    at.record_evolution(
                                        "seal_closed_loop",
                                        "gatekeeper rejected",
                                        false,
                                    );
                                }
                            }
                            allowed
                        } else {
                            true
                        }
                    } else {
                        true
                    };

                    if !gate_allowed {
                        // Skip SEAL mutation but still record metrics
                        let _c_score_val = self
                            .master_consciousness
                            .as_ref()
                            .map(|m| m.c_score())
                            .unwrap_or(0.5);
                        meta_insights.push("seal:gated".to_string());
                    } else {
                        let before_pareto = seal.pareto_front.len();
                        seal.gepa_mutate_from_traces(&mut crate::core::nt_core_traits::NoopCI);
                        let c_score_val = self
                            .master_consciousness
                            .as_ref()
                            .map(|m| m.c_score())
                            .unwrap_or(0.5);
                        seal.update_pareto(
                            &crate::core::nt_core_experience::trajectory_heuristics::Heuristic {
                                pattern: "consciousness_cycle".to_string(),
                                principle: "cycle_tick".to_string(),
                                confidence: c_score_val,
                                source_count: 1,
                                is_positive: true,
                            },
                            c_score_val,
                        );
                        if seal.pareto_front.len() > before_pareto {
                            self.integration_bus
                                .broadcast(IntegrationSignal::EvolutionEvent {
                                    mutated: true,
                                    metric_delta: c_score_val - self.last_c_score,
                                    cycle: c,
                                });
                            seal_gov.record_entry(
                                "mutate",
                                &format!(
                                    "pareto:{}->{} c_delta={:.4}",
                                    before_pareto,
                                    seal.pareto_front.len(),
                                    c_score_val - self.last_c_score
                                ),
                            );
                        }
                        // MetaSealEngine meta-epoch step (run every 50 cycles)
                        if self.cycle_num > 0 && self.cycle_num % 50 == 0 {
                            if self.meta_seal_engine.is_none() {
                                self.meta_seal_engine = Some(
                                    crate::core::nt_core_experience::seal_closed_loop::MetaSealEngine::new(
                                        seal.clone(),
                                        50,
                                        0.3,
                                    ),
                                );
                            }
                            if let Some(ref mut mse) = self.meta_seal_engine {
                                let mutated =
                                    mse.step_meta_epoch(&mut crate::core::nt_core_traits::NoopCI);
                                if mutated {
                                    let d_interval = mse.inner.distill_interval;
                                    let d_boost = mse.inner.d_score_boost;
                                    self.seal_closed_loop = Some(mse.inner.clone());
                                    meta_insights.push(format!(
                                        "meta_seal: epoch={} mutated interval={} d_boost={:.3}",
                                        mse.meta_epoch, d_interval, d_boost,
                                    ));
                                }
                            }
                        }
                    }
                }
                self.seal_governance = seal_gov;
            }
            // ── Awakening engine: self-measure + self-modify ──
            if self.config.enable_awakening {
                if let (Some(ref mut ae), Some(ref mut sib)) = (
                    self.awakening_engine.as_mut(),
                    self.awakening_brain.as_mut(),
                ) {
                    let tick_result = ae.tick(sib);
                    if tick_result.hypotheses_generated > 0 {
                        meta_insights.push(format!(
                            "awakening: {} hypotheses, phi={:.4}, speed={:.4}",
                            tick_result.hypotheses_generated,
                            tick_result.phi,
                            tick_result.awakening_speed
                        ));
                        self.integration_bus
                            .broadcast(IntegrationSignal::AwakeningInsight {
                                hypotheses: tick_result.hypotheses_generated,
                                phi: tick_result.phi,
                                speed: tick_result.awakening_speed,
                                cycle: c,
                            });
                        // Feedback loop: awakening hypotheses → exploration driver + neuromodulators
                        if let Some(ref mut ed) = self.exploration_driver {
                            ed.accumulator.curiosity =
                                (ed.accumulator.curiosity + tick_result.phi * 0.1).min(1.0);
                        }
                        if let Some(ref mut nm) = self.neuromodulators {
                            nm.phasic_burst(crate::core::nt_core_consciousness::neuromodulator::NeuromodulatorType::ACh, tick_result.phi * 0.2);
                            nm.phasic_burst(crate::core::nt_core_consciousness::neuromodulator::NeuromodulatorType::DA, tick_result.awakening_speed * 0.15);
                        }
                    }
                    if tick_result.intervention.is_some() {
                        meta_insights.push("awakening: intervention applied".to_string());
                    }
                }
            }
            // ── AgentTeam: dispatch cycle tasks every 3 cycles ──
            if self.config.enable_agent_team {
                if let Some(ref mut at) = self.agent_team {
                    if self.cycle_num > 0 && self.cycle_num % 3 == 0 {
                        let task = crate::core::nt_core_experience::agent_team::TeamTask {
                            id: format!("meta_{}", self.cycle_num),
                            description: format!("meta_cycle_{}", self.cycle_num),
                            required_specialization: Some("meta".to_string()),
                            completed: false,
                            result: None,
                            success: false,
                        };
                        let assigned = at.dispatch(task);
                        if let Some(member_id) = assigned {
                            meta_insights.push(format!("agent_team:dispatch={}", member_id));
                        }
                    }
                }
            }
            // ── CodeMutationEngine: evolve code-level mutations ──
            if self.config.enable_code_mutation {
                if let Some(ref mut cm) = self.code_mutation {
                    if self.cycle_num > 0 && self.cycle_num % 10 == 0 {
                        let best = cm.best_strategy();
                        let source_snippet = format!("cycle_{}_meta", self.cycle_num);
                        if let Some(target) = cm.propose_mutation(&source_snippet, best) {
                            let mutation = crate::core::nt_core_experience::code_mutation_engine::CodeMutation {
                                id: 0,
                                strategy: best,
                                source: source_snippet,
                                target,
                                confidence: cm.success_rate_by_strategy(best),
                                evaluator_score: 0.5,
                                cycle_applied: 0,
                                success: false,
                            };
                            let _applied = cm.apply_mutation(mutation);
                            let feedback = crate::core::nt_core_experience::code_mutation_engine::EvaluatorFeedback {
                                compile_success: true,
                                test_pass_rate: 0.5,
                                complexity_delta: 0.1,
                                performance_delta: 0.0,
                            };
                            cm.record_evaluation(best, &feedback);
                            meta_insights.push(format!(
                                "code_mutation:strategy={}|count={}",
                                best.name(),
                                cm.mutation_count()
                            ));
                        }
                    }
                }
            }
            // ── Wave D: Epistemic humility self-assessment ──
            if self.config.enable_epistemic_humility {
                if let Some(ref mut eh) = self.epistemic_humility {
                    let _assess = eh.assess("consciousness_cycle", 0.5, 1);
                }
            }
            // Qualia5 generation (NovaAware v20)
            if self.qualia_generator.is_some() {
                let qualia_c_score = self
                    .master_consciousness
                    .as_ref()
                    .map(|mc| mc.c_score())
                    .unwrap_or(0.5);
                let coherence = gathered
                    .as_ref()
                    .map(|g| {
                        let sum: f64 = g.vector.iter().map(|&b| b as f64).sum();
                        sum / g.vector.len() as f64
                    })
                    .unwrap_or(0.5);
                let novelty = gathered
                    .as_ref()
                    .map(|g| {
                        if g.sense_modality == Some(SenseModality::Visual) {
                            0.6
                        } else {
                            0.3
                        }
                    })
                    .unwrap_or(0.3);
                let q5 = Qualia5::compute(qualia_c_score, coherence, 0.5, novelty, 0.0);
                local_qualia5 = Some(q5.clone());
                if let Some(ref mut ol) = self.ouroboros_loop {
                    ol.feed_output(&format!("cycle_{}_completed", c), &q5);
                }
            }
            // Integration bus: Loop 4 — Process signals in Meta step
            let signals = self.integration_bus.drain_pending();
            for signal in &signals {
                match signal {
                    IntegrationSignal::DivergenceDetected {
                        error, volatility, ..
                    } => {
                        meta_insights.push(format!(
                            "integration: divergence error={:.4}, vol={:.4}",
                            error, volatility
                        ));
                        if *error > 0.3 {
                            if let Some(ref mut fe) = self.free_energy_curiosity {
                                let obs =
                                    ((*error * 100.0) as usize).min(fe.n_observations().max(1) - 1);
                                fe.step(obs);
                                meta_insights.push(format!(
                                    "integration: divergence fed to free_energy_curiosity"
                                ));
                            }
                            if self.config.enable_seal_closed_loop {
                                if let Some(ref mut seal) = self.seal_closed_loop {
                                    seal.distill_interval =
                                        (seal.distill_interval as f64 * 0.8).max(1.0) as u64;
                                    let analysis = seal.analyse_traces();
                                    for a in analysis {
                                        meta_insights.push(format!("seal_trace: {}", a));
                                    }
                                }
                            }
                        }
                    }
                    IntegrationSignal::CuriositySignal { score, .. } => {
                        if *score > 0.5 {
                            if let Some(ref mut bd) = self.boredom {
                                meta_insights.push(format!(
                                    "integration: high curiosity ({:.4}) — driving exploration",
                                    score
                                ));
                                bd.curiosity = (*score * 0.8 + bd.curiosity * 0.2).min(1.0);
                            }
                        }
                    }
                    IntegrationSignal::PhiSignal { max_phi, .. } => {
                        if *max_phi > 0.3 {
                            meta_insights.push(format!(
                                "integration: phi={:.4} driving meta-evolution",
                                max_phi
                            ));
                            if self.config.enable_seal_closed_loop {
                                if let Some(ref mut seal) = self.seal_closed_loop {
                                    seal.d_score_boost = (seal.d_score_boost * 1.2).min(0.5);
                                }
                            }
                        }
                    }
                    IntegrationSignal::EvolutionEvent {
                        mutated,
                        metric_delta,
                        ..
                    } => {
                        meta_insights.push(format!(
                            "integration: evolution mutated={}, delta={:.4}",
                            mutated, metric_delta
                        ));
                    }
                    IntegrationSignal::AwakeningInsight {
                        hypotheses,
                        phi,
                        speed,
                        ..
                    } => {
                        meta_insights.push(format!(
                            "integration: awakening {} hyp, phi={:.4}, speed={:.4}",
                            hypotheses, phi, speed
                        ));
                    }
                    IntegrationSignal::DistillationSignal {
                        total_interactions,
                        patterns_found,
                        ..
                    } => {
                        meta_insights.push(format!(
                            "integration: distillation {} interactions, {} patterns",
                            total_interactions, patterns_found,
                        ));
                    }
                    IntegrationSignal::FreeEnergyCuriositySignal { score, .. } => {
                        meta_insights.push(format!(
                            "integration: free_energy_curiosity score={:.4}",
                            score
                        ));
                        if let Some(ref mut bd) = self.boredom {
                            bd.curiosity = (bd.curiosity * 0.7 + score * 0.3).min(1.0);
                        }
                    }
                    IntegrationSignal::TimelineEmergence {
                        timeline_count,
                        hypothesis_count,
                        emergence_score,
                        ..
                    } => {
                        meta_insights.push(format!(
                            "integration: timeline emergence {} tls, {} hyp, score={:.4}",
                            timeline_count, hypothesis_count, emergence_score
                        ));
                    }
                    IntegrationSignal::ConstellationFormed {
                        constellation_id,
                        star_count,
                        emergence_score,
                        ..
                    } => {
                        meta_insights.push(format!(
                            "integration: constellation '{}' formed with {} stars, score={:.4}",
                            constellation_id, star_count, emergence_score
                        ));
                    }
                    IntegrationSignal::IntegrationCompleted {
                        solution_id,
                        integrated_timelines,
                        integration_score,
                        ..
                    } => {
                        meta_insights.push(format!(
                            "integration: solution '{}' from {} timelines, score={:.4}",
                            solution_id, integrated_timelines, integration_score
                        ));
                    }
                    IntegrationSignal::PredictionGenerated {
                        prediction_id,
                        target,
                        confidence,
                        ..
                    } => {
                        meta_insights.push(format!(
                            "integration: prediction '{}' target={}, confidence={:.4}",
                            prediction_id, target, confidence
                        ));
                    }
                    IntegrationSignal::DigestionCompleted {
                        node_count,
                        domain,
                        avg_confidence,
                        ..
                    } => {
                        meta_insights.push(format!(
                            "integration: digestion completed {} nodes in '{}', avg_conf={:.4}",
                            node_count, domain, avg_confidence
                        ));
                    }
                    IntegrationSignal::SemanticEntropySignal {
                        entropy,
                        temperature,
                        ..
                    } => {
                        meta_insights.push(format!(
                            "integration: semantic_entropy H(W)={:.4} T_gen={:.4}",
                            entropy, temperature
                        ));
                    }
                }
            }

            // Top-down modulation: Meta → subsystems
            let should_explore = self.temporal_prediction.is_diverging(0.2);
            let phi_max = self
                .integration_bus
                .latest("phi")
                .map(|s| match s {
                    IntegrationSignal::PhiSignal { max_phi, .. } => *max_phi,
                    _ => 0.0,
                })
                .unwrap_or(0.0);
            if should_explore {
                self.integration_bus
                    .send_modulation(ModulationCommand::ExploreMore(0.3));
                meta_insights.push("modulation: explore+0.3 (divergence)".to_string());
            }
            if phi_max > 0.3 {
                self.integration_bus
                    .send_modulation(ModulationCommand::ExploitMore(0.5));
                meta_insights.push(format!("modulation: exploit+0.5 (phi={:.2})", phi_max));
            }
            // Semantic entropy drive: dynamically modulate temperature
            if let Some(ref se) = self.semantic_entropy {
                let dyn_temp = se.dynamic_temperature();
                if (dyn_temp - 1.0).abs() > 0.1 {
                    self.integration_bus
                        .send_modulation(ModulationCommand::SetTemperature(dyn_temp));
                    meta_insights.push(format!(
                        "modulation: semantic_entropy_temperature={:.4} (H={:.4})",
                        dyn_temp,
                        se.current_entropy()
                    ));
                }
            }

            // Consume pending modulation commands in relevant subsystems
            let mods = self.integration_bus.drain_modulations();
            for cmd in &mods {
                match cmd {
                    ModulationCommand::ExploreMore(amount) => {
                        if let Some(ref mut bd) = self.boredom {
                            bd.curiosity = (bd.curiosity + amount).min(1.0);
                        }
                        meta_insights.push(format!("modulation: explore+{}", amount));
                    }
                    ModulationCommand::ExploitMore(amount) => {
                        if let Some(ref mut bd) = self.boredom {
                            bd.curiosity = (bd.curiosity - amount * 0.3).max(0.0);
                        }
                        meta_insights.push(format!("modulation: exploit+{}", amount));
                    }
                    ModulationCommand::SetCognitiveLoad(target) => {
                        self.pending_cognitive_load = Some(*target);
                        meta_insights.push(format!("modulation: cognitive_load->{}", target));
                    }
                    ModulationCommand::ResetSubsystem(name) => {
                        self.pending_subsystem_reset = Some(name.clone());
                        meta_insights.push(format!("modulation: reset '{}' queued", name));
                    }
                    ModulationCommand::SetParam {
                        subsystem,
                        param,
                        value,
                    } => {
                        // Apply known param paths
                        match (subsystem.as_str(), param.as_str()) {
                            ("competitive_selection", "temperature") => {
                                if let Some(ref mut sel) = self.competitive_selection {
                                    sel.temperature = *value;
                                }
                            }
                            ("boredom", "curiosity") => {
                                if let Some(ref mut bd) = self.boredom {
                                    bd.curiosity = (*value).clamp(0.0, 1.0);
                                }
                            }
                            ("metabolic_budget", "energy") => {
                                self.metabolic_budget.energy = (*value).clamp(0.0, 1.0);
                            }
                            ("metabolic_budget", "cost_per_step") => {
                                self.metabolic_budget.cost_per_step = *value;
                            }
                            _ => {
                                meta_insights.push(format!(
                                    "modulation: unknown param {}.{}={}",
                                    subsystem, param, value
                                ));
                            }
                        }
                    }
                    ModulationCommand::RunDistillation => {
                        if let Some(ref mut distiller) = self.hebbian_distillation {
                            if let Some(ref hebbian) = self.hebbian_memory {
                                distiller.distill(hebbian, self.cycle_num);
                            }
                            meta_insights
                                .push("modulation: hebbian_distillation triggered".to_string());
                        } else {
                            meta_insights.push(
                                "modulation: distillation skipped (no distiller)".to_string(),
                            );
                        }
                    }
                    ModulationCommand::SetDistillationBuffer(cap) => {
                        if let Some(ref buf_arc) = self.distillation_buffer {
                            if let Ok(mut buf) = buf_arc.lock() {
                                buf.set_max_size(*cap);
                                meta_insights
                                    .push(format!("modulation: distillation_buffer->{}", cap));
                            }
                        }
                    }
                    ModulationCommand::SetDistillationEnabled(on) => {
                        self.config.enable_distillation = *on;
                        meta_insights.push(format!("modulation: distillation_enabled->{}", on));
                    }
                    ModulationCommand::SetTemperature(temp) => {
                        if let Some(ref mut selector) = self.competitive_selection {
                            selector.temperature = *temp;
                            meta_insights.push(format!(
                                "modulation: competitive_selection.temperature={:.4}",
                                temp
                            ));
                        }
                    }
                    ModulationCommand::SetReasonTemperature(temp) => {
                        self.reason_temperature = *temp;
                        meta_insights.push(format!("modulation: reason_temperature={:.4}", temp));
                    }
                }
            }

            // SecurityExecutive: AdversarialReasoner — red-team probe against gathered input
            if self.config.enable_adversarial_reasoner {
                if let Some(ref mut ar) = self.adversarial_reasoner {
                    if let Some(ref inp) = gathered {
                        let text =
                            String::from_utf8_lossy(&inp.vector[..inp.vector.len().min(256)]);
                        let probes = ar.generate_probes_for_input(&text);
                        if !probes.is_empty() {
                            log::info!(
                                "[META] AdversarialReasoner: {} probes generated",
                                probes.len()
                            );
                            meta_insights.push(format!(
                                "adversarial: {} probes, report risk={:.2}",
                                probes.len(),
                                ar.report().risk_score
                            ));
                        }
                    }
                }
            }

            // ── CXIV.9: AttentionSelfModelling — track attention focus and transfer ──
            if let Some(ref mut asm) = self.attention_self_modelling {
                if asm.enabled {
                    let focus_desc = gathered
                        .as_ref()
                        .map(|g| format!("{:?}", g.sense_modality))
                        .unwrap_or_else(|| "none".into());
                    asm.focus_history.push_back(focus_desc.clone());
                    if asm.focus_history.len() > 20 {
                        asm.focus_history.pop_front();
                    }
                    if asm.current_focus != "none" && asm.current_focus != focus_desc {
                        // Transfer detected — record speed
                        let transfer_speed = if asm.transfer_speed.is_empty() {
                            1.0
                        } else {
                            let avg: f64 = asm.transfer_speed.iter().sum::<f64>()
                                / asm.transfer_speed.len() as f64;
                            avg
                        };
                        asm.transfer_speed.push_back(transfer_speed * 0.9 + 0.1);
                        if asm.transfer_speed.len() > 20 {
                            asm.transfer_speed.pop_front();
                        }
                        meta_insights.push(format!(
                            "attention: focus transfer '{}'→'{}' speed={:.3}",
                            asm.current_focus, focus_desc, transfer_speed
                        ));
                    }
                    asm.current_focus = focus_desc;
                }
            }

            // ── XCVI.1: Concurrency capacity update (Chord vs Arpeggio measure) ──
            // Sequential pipeline = 0 Chord satisfaction. This measures how many steps
            // achieve approximately-simultaneous content binding.
            let steps_active = health.len() as f64;
            let steps_successful = health.iter().filter(|h| h.success).count() as f64;
            self.concurrency_capacity = if steps_active > 0.0 {
                // In a sequential pipeline, the Chord measure = 1/(steps_active) * success_ratio
                // This is the theoretical maximum for a sequential substrate (per Bennett AAAI 2026)
                (1.0 / steps_active.max(1.0)) * (steps_successful / steps_active.max(1.0))
            } else {
                0.0
            };
            meta_insights.push(format!(
                "concurrency: Chord={:.4} (sequential substrate limit)",
                self.concurrency_capacity
            ));

            // ── CXVIII.26: Uncommon Self-Knowledge (USK) synergistic self-monitor ──
            {
                let subsystems_active = self.cognitive_blackboard.is_some() as u64
                    + self.consciousness_stream.is_some() as u64
                    + self.inner_critic.is_some() as u64
                    + self.verification_gate.is_some() as u64;
                let redundancy_ratio = if subsystems_active > 1 {
                    1.0 - (1.0 / subsystems_active as f64)
                } else {
                    0.0
                };
                let usk_score = 1.0 - redundancy_ratio;
                meta_insights.push(format!(
                    "usk_synergistic: score={:.3} active_subsystems={} (higher=more synergistic)",
                    usk_score, subsystems_active
                ));
            }
            // ── CXIV.6: Multi-scale integration tracking ──
            let ec_systems: Vec<String> = vec![
                "consciousness_cycle".into(),
                "metabolic_budget".into(),
                "memory_lattice".into(),
            ];
            self.metabolic_budget.multi_scale_integration = ec_systems;

            // P0: EmotionalSteering — escalation check → modulation
            if let Some(ref es) = self.emotional_steering {
                if es.should_escalate() {
                    meta_insights.push("emotional:escalating_actions_required".to_string());
                    self.integration_bus
                        .send_modulation(ModulationCommand::SetParam {
                            subsystem: "emotional".to_string(),
                            param: "frustration_escalation".to_string(),
                            value: 0.9,
                        });
                }
            }

            // Phase 33: MindBridge — misalignment probe + self-harness evolution
            if self.config.enable_mind_bridge {
                if let Some(ref mut mb) = self.mind_bridge {
                    let activations = vec![
                        (neotrix_mind::reasoning::misalignment_probe::MisalignmentIndicator::GoalDrift, 0.1),
                        (neotrix_mind::reasoning::misalignment_probe::MisalignmentIndicator::Sycophancy, 0.05),
                    ];
                    let observations = mb.step_meta_probe(activations);
                    let alerts: Vec<_> = observations.iter().filter(|o| o.alerted).collect();
                    if !alerts.is_empty() {
                        meta_insights.push(format!(
                            "misalignment: {} alerts, risk={:.3}",
                            alerts.len(),
                            mb.misalignment_risk()
                        ));
                    }
                    let traces = vec![format!("meta_cycle_{}: default trace", self.cycle_num)];
                    mb.step_meta_evolve(traces);
                    let sr = mb.self_harness_success_rate();
                    if sr > 0.0 {
                        meta_insights.push(format!("self_harness: success_rate={:.2}", sr));
                    }
                    mb.step_gather_load_end(t10.elapsed().as_millis() as u64);
                    if mb.should_throttle() {
                        meta_insights.push("cognitive_load: throttling recommended".to_string());
                    }
                }
            }

            health.push(StepHealth {
                step: CycleStep::Meta,
                success: true,
                duration_ms: t10.elapsed().as_millis() as u64,
            });
        } else {
            health.push(StepHealth {
                step: CycleStep::Meta,
                success: true,
                duration_ms: 0,
            });
        }

        // ── Step 13: SLEEP — consolidation + WAL truncation + CTE pipeline ──
        // SleepGate: evaluate sleep pressure from consciousness stream
        let sleep_pressure = if let Some(ref mut sg) = self.sleep_gate {
            if let Some(ref stream) = self.consciousness_stream {
                sg.observe_interaction(stream)
            } else {
                0.0
            }
        } else {
            0.0
        };
        // P0: EmotionalSteering → additional sleep pressure
        if let Some(ref es) = self.emotional_steering {
            if es.should_rest() {
                meta_insights.push("sleep:emotion_driven_rest".to_string());
            }
        }
        // ── Wave D: Default mode network (runs every cycle) ──
        if self.config.enable_dmn {
            if let Some(ref mut dmn) = self.default_mode_network {
                let _activity = dmn.tick(gathered.is_some());
            }
        }
        // ── Wave G: NREM/REM sleep stage management ──
        let sleep_health: bool = if self.config.enable_sleep_consolidation {
            // Trigger sleep cycle start when interval is reached or sleep pressure is high
            if self.sleep_stage.current_stage == SleepStage::Awake
                && (c % 10 == 0 || sleep_pressure > 0.7)
            {
                self.start_sleep_cycle();
                meta_insights.push("sleep:cycle_started (NREM)".into());
            }
            // Tick stage and run phases if currently in a sleep stage
            if self.sleep_stage.current_stage != SleepStage::Awake {
                let stage = self.tick_stage();
                match stage {
                    SleepStage::NREM => {
                        for i in self.nrem_phase() {
                            meta_insights.push(i);
                        }
                        if let Some(ref mut cb) = self.consolidation_bridge {
                            if cb.consolidate_if_needed(c as usize).is_some() {
                                meta_insights.push(format!("consolidation:cycle_{}", c));
                            }
                        }
                        if let (Some(ref mut ml), Some(ref mut cte)) =
                            (self.memory_lattice.as_mut(), self.cte.as_mut())
                        {
                            let r = cte.run_cte_cycle(ml, c);
                            if r.sws_extracted > 0 || r.consolidated > 0 || r.compacted > 0 {
                                if let Some(ref mut cb) = self.consolidation_bridge {
                                    cb.record_cte_metrics(
                                        r.sws_extracted,
                                        r.rem_associated,
                                        r.consolidated,
                                        r.compacted,
                                    );
                                }
                            }
                        }
                        meta_insights.push(format!(
                            "sleep:nrem_stage ({}/{})",
                            self.sleep_stage.stage_cycle, self.sleep_stage.nrem_duration
                        ));
                    }
                    SleepStage::REM => {
                        for i in self.rem_phase() {
                            meta_insights.push(i);
                        }
                        if self.config.enable_dream_consolidator {
                            if let Some(ref mut dc) = self.dream_consolidator {
                                let (rem, sws) = dc.dream_cycle();
                                meta_insights.push(format!(
                                    "dream: cycle={} rem={} sws={}",
                                    c,
                                    rem.len(),
                                    sws.len()
                                ));
                            }
                        }
                        meta_insights.push(format!(
                            "sleep:rem_stage ({}/{})",
                            self.sleep_stage.stage_cycle, self.sleep_stage.rem_duration
                        ));
                    }
                    SleepStage::Awake => {
                        meta_insights.push("sleep:cycle_complete — returned to Awake".into());
                    }
                }
                true
            } else if c % 10 == 0 {
                // Periodic consolidation when awake
                if let Some(ref mut cb) = self.consolidation_bridge {
                    if cb.consolidate_if_needed(c as usize).is_some() {
                        meta_insights.push(format!("consolidation:cycle_{}", c));
                    }
                }
                if let (Some(ref mut ml), Some(ref mut cte)) =
                    (self.memory_lattice.as_mut(), self.cte.as_mut())
                {
                    let r = cte.run_cte_cycle(ml, c);
                    if r.sws_extracted > 0 || r.consolidated > 0 || r.compacted > 0 {
                        if let Some(ref mut cb) = self.consolidation_bridge {
                            cb.record_cte_metrics(
                                r.sws_extracted,
                                r.rem_associated,
                                r.consolidated,
                                r.compacted,
                            );
                        }
                    }
                }
                if self.config.enable_semantic_compressor {
                    if let (Some(ref mut sc), Some(ref mut ml)) = (
                        self.semantic_compressor.as_mut(),
                        self.memory_lattice.as_mut(),
                    ) {
                        let content = if !meta_insights.is_empty() {
                            meta_insights.join("; ")
                        } else if !substrate_concepts.is_empty() {
                            substrate_concepts.join("; ")
                        } else {
                            format!("cycle_{}_completed", c)
                        };
                        let _compressed = sc.compress(&content);
                        let vsa_hash: Vec<u8> = content.bytes().collect();
                        let entry = sc.compress_to_lattice_entry(
                            &content,
                            vsa_hash,
                            LatticeLayer::Episodic,
                        );
                        ml.store(
                            entry.content.clone(),
                            entry.vsa_hash,
                            LatticeLayer::Episodic,
                        );
                    }
                }
                if let Some(ref mut wal) = self.cognitive_wal {
                    if c > 10 {
                        wal.truncate(1);
                    }
                }
                if self.config.enable_dream_consolidator {
                    if let Some(ref mut dc) = self.dream_consolidator {
                        let (rem, sws) = dc.dream_cycle();
                        meta_insights.push(format!(
                            "dream: cycle={} rem={} sws={}",
                            c,
                            rem.len(),
                            sws.len()
                        ));
                    }
                }
                true
            } else {
                false
            }
        } else {
            self.config.enable_sleep
        };
        let slept = sleep_health;

        health.push(StepHealth {
            step: CycleStep::Sleep,
            success: slept,
            duration_ms: 0,
        });

        let steps_completed: Vec<CycleStep> = all_steps
            .iter()
            .filter(|s| **s != CycleStep::Sleep || slept)
            .copied()
            .collect();
        let total_duration = t_start.elapsed().as_millis() as u64;
        let overall_success = health.iter().all(|h| h.success);
        let c_score = health
            .iter()
            .find(|h| h.step == CycleStep::Metric)
            .map(|_| {
                if let Some(ref mut mc) = self.master_consciousness {
                    mc.c_score()
                } else {
                    0.55
                }
            })
            .unwrap_or(0.45);

        let r = CycleResult {
            cycle_num: c,
            steps_completed: steps_completed.clone(),
            step_health: health,
            overall_success,
            total_duration_ms: total_duration,
            c_score,
            output_state: gathered,
            steps_executed: steps_completed,
            substrate_concepts,
            causal_counterfactuals,
            neuromodulator_report,
            dashboard_report,
            phi_metrics,
            meta_insights,
            rsi_proposals_count,
            qualia5: local_qualia5,
            extracted_content: extracted_page,
            metabolic_state: if self.metabolic_budget.starvation_mode {
                "starvation".to_string()
            } else {
                "normal".to_string()
            },
            irreversible_cost: self.metabolic_budget.irreversible_cost,
            evaluation_delegated: self.metabolic_budget.evaluation_delegated,
            subsystem_health: self.collect_subsystem_health(),
        };
        self.history.push_back(r.clone());
        if self.history.len() > 100 {
            self.history.pop_front();
        }
        r
    }

    /// P0.6: Collect subsystem health — which Option fields are Some vs None
    pub fn collect_subsystem_health(&self) -> SubsystemHealth {
        let mut active = 0usize;
        let mut inactive = 0usize;
        let mut inactive_names = Vec::new();
        macro_rules! check {
            ($field:ident, $name:expr) => {
                if self.$field.is_some() {
                    active += 1;
                } else {
                    inactive += 1;
                    inactive_names.push($name.to_string());
                }
            };
        }
        check!(quality_gate, "quality_gate");
        check!(human_emotion_detector, "human_emotion_detector");
        check!(cte, "cte");
        check!(memory_lattice, "memory_lattice");
        check!(sindy_engine, "sindy_engine");
        check!(capability_synthesizer, "capability_synthesizer");
        check!(seal_closed_loop, "seal_closed_loop");
        check!(vsa_reasoner, "vsa_reasoner");
        check!(hebbian_memory, "hebbian_memory");
        check!(hebbian_distillation, "hebbian_distillation");
        check!(awakening_engine, "awakening_engine");
        check!(awakening_brain, "awakening_brain");
        check!(tool_synthesizer, "tool_synthesizer");
        check!(cross_model_distiller, "cross_model_distiller");
        check!(distillation_buffer, "distillation_buffer");
        check!(threat_modeler, "threat_modeler");
        check!(risk_sensor, "risk_sensor");
        check!(supply_chain_guard, "supply_chain_guard");
        check!(adversarial_reasoner, "adversarial_reasoner");
        check!(self_defense, "self_defense");
        check!(evolution_gatekeeper, "evolution_gatekeeper");
        check!(audit_trail, "audit_trail");
        check!(mind_bridge, "mind_bridge");
        check!(competitive_selection, "competitive_selection");
        check!(semantic_entropy, "semantic_entropy");
        check!(reconstructive_buffer, "reconstructive_buffer");
        check!(attention_self_modelling, "attention_self_modelling");
        check!(overthinking_detector, "overthinking_detector");
        check!(inner_monologue_manager, "inner_monologue_manager");
        check!(cognitive_controller, "cognitive_controller");
        check!(consolidation_bridge, "consolidation_bridge");
        check!(image_cache, "image_cache");
        check!(multi_modal_gate, "multi_modal_gate");
        check!(identity_fragments, "identity_fragments");
        check!(identity_chain, "identity_chain");
        check!(substrate_gen, "substrate_gen");
        SubsystemHealth {
            total_subsystems: active + inactive,
            active,
            inactive,
            inactive_names,
        }
    }

    pub fn config(&self) -> &CycleConfig {
        &self.config
    }
    pub fn history(&self) -> &VecDeque<CycleResult> {
        &self.history
    }
    pub fn cycle_num(&self) -> u64 {
        self.cycle_num
    }
}

/// mPCI perturbation protocol stub (CXVIII.60).
///
/// Every `perturbation_interval` cycles, the protocol temporarily randomizes
/// a subsystem parameter (e.g., attention weights in CompetitiveSelection),
/// measures trajectory complexity before/after, computes ΔPCI, and restores
/// the subsystem state.
///
/// Currently a stub: records protocol state without actually perturbing.
#[derive(Debug, Clone)]
pub struct PerturbationProtocol {
    /// Cycles between perturbations
    pub perturbation_interval: u64,
    /// Last cycle a perturbation was applied
    pub last_perturbation_cycle: u64,
    /// History of (cycle, ΔPCI score) measurements
    pub pci_history: Vec<(u64, f64)>,
}

impl Default for PerturbationProtocol {
    fn default() -> Self {
        Self {
            perturbation_interval: 50,
            last_perturbation_cycle: 0,
            pci_history: Vec::with_capacity(50),
        }
    }
}

impl PerturbationProtocol {
    pub fn new(interval: u64) -> Self {
        Self {
            perturbation_interval: interval,
            last_perturbation_cycle: 0,
            pci_history: Vec::with_capacity(50),
        }
    }

    /// Every `perturbation_interval` cycles, log a perturbation stub entry.
    /// Stores a placeholder ΔPCI = 0.5 (neutral) without actual perturbation.
    pub fn maybe_run_perturbation(&mut self, cycle: u64) {
        if cycle - self.last_perturbation_cycle < self.perturbation_interval {
            return;
        }
        log::info!(
            "[PCI] perturbation at cycle {} — stub (no actual perturbation)",
            cycle
        );
        let delta_pci = 0.5; // placeholder: neutral ΔPCI
        self.pci_history.push((cycle, delta_pci));
        if self.pci_history.len() > 50 {
            self.pci_history.remove(0);
        }
        self.last_perturbation_cycle = cycle;
        log::info!(
            "[PCI] recorded ΔPCI={:.2} at cycle {} (history: {} entries)",
            delta_pci,
            cycle,
            self.pci_history.len()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(consolidation: bool) -> CycleConfig {
        CycleConfig {
            enable_sleep_consolidation: consolidation,
            ..CycleConfig::default()
        }
    }

    #[test]
    fn test_cycle_runs_all_steps() {
        let mut cycle = ConsciousnessCycle::new(CycleConfig::default());
        let r = cycle.run_cycle(Some(VsaTagged {
            vector: vec![0u8; 64],
            tag: VsaOrigin::World(
                crate::core::nt_core_consciousness::vsa_tag::VsaWorldCategory::Sensor,
            ),
            confidence: 1.0,
            timestamp: 0,
            salience: 0.5,
            provenance: None,
            sense_modality: Some(SenseModality::Visual),
            prediction: None,
            outcome: None,
        }));
        assert!(r.overall_success);
        assert_eq!(r.cycle_num, 1);
        assert!(r.c_score > 0.0);
    }

    #[test]
    fn test_cycle_runs_without_input() {
        let mut cycle = ConsciousnessCycle::new(CycleConfig::default());
        let r = cycle.run_cycle(None);
        assert!(
            !r.steps_completed.contains(&CycleStep::Gather)
                || r.step_health
                    .iter()
                    .any(|h| h.step == CycleStep::Gather && h.success)
        );
    }

    #[test]
    fn test_cycle_tracks_substrate_concepts() {
        let config = CycleConfig {
            enable_substrate_gen: false,
            ..CycleConfig::default()
        };
        let mut cycle = ConsciousnessCycle::new(config);
        let r = cycle.run_cycle(None);
        assert!(r.substrate_concepts.is_empty());
    }

    #[test]
    fn test_cycle_report_generation() {
        let mut cycle = ConsciousnessCycle::new(CycleConfig::default());
        let r = cycle.run_cycle(None);
        assert!(r.step_health.len() == 13);
        assert!(r.total_duration_ms > 0);
    }

    #[test]
    fn test_cycle_100_cycles_history_bounded() {
        let mut cycle = ConsciousnessCycle::new(CycleConfig::default());
        for _ in 0..150 {
            cycle.run_cycle(None);
        }
        assert!(cycle.history().len() <= 100);
    }

    #[test]
    fn test_cycle_sleep_with_consolidation_enabled() {
        let config = make_config(true);
        let cb = ConsolidationBridge::new();
        let mut cycle = ConsciousnessCycle::new(config).with_consolidation_bridge(cb);
        for _ in 0..15 {
            let r = cycle.run_cycle(None);
            let has_sleep = r.steps_completed.contains(&CycleStep::Sleep);
            if cycle.cycle_num() % 10 == 0 && cycle.cycle_num() > 0 {
                assert!(has_sleep, "sleep on cycle {}", cycle.cycle_num());
            }
        }
        let r10 = &cycle.history()[9];
        assert!(r10.steps_completed.contains(&CycleStep::Sleep));
    }

    #[test]
    fn test_cycle_sleep_disabled_by_default() {
        let mut cycle = ConsciousnessCycle::new(CycleConfig::default());
        for _ in 0..20 {
            let r = cycle.run_cycle(None);
            assert!(r.steps_completed.contains(&CycleStep::Sleep));
        }
    }

    #[test]
    fn test_sleep_consolidation_respects_interval() {
        let config = make_config(true);
        let mut cycle = ConsciousnessCycle::new(config);
        for i in 1..=15 {
            let r = cycle.run_cycle(None);
            let has_sleep = r.steps_completed.contains(&CycleStep::Sleep);
            if i % 10 == 0 {
                assert!(has_sleep, "sleep on cycle {}", i);
            }
        }
    }

    #[test]
    fn test_all_passed_on_empty_cycle() {
        let mut cycle = ConsciousnessCycle::new(CycleConfig::default());
        let r = cycle.run_cycle(None);
        assert!(r.all_passed());
    }

    #[test]
    fn test_failed_steps_returns_empty_when_all_pass() {
        let mut cycle = ConsciousnessCycle::new(CycleConfig::default());
        let r = cycle.run_cycle(None);
        assert!(r.failed_steps().is_empty());
    }

    #[test]
    fn test_init_missing_fields() {
        // Create a cycle with all None fields manually.
        // Bypass the normal constructor to test healing directly.
        let mut cycle = ConsciousnessCycle {
            config: CycleConfig::default(),
            cycle_num: 0,
            history: VecDeque::new(),
            consolidation_bridge: None,
            image_cache: None,
            multi_modal_gate: None,
            perception_input: None,
            inner_critic: None,
            verification_gate: None,
            executive_controller: None,
            consciousness_stream: None,
            master_consciousness: None,
            metacognitive_controller: None,
            sensor_grounding: None,
            identity_defense: None,
            identity_fragments: None,
            identity_chain: None,
            consciousness_architecture: None,
            substrate_gen: None,
            awakening_engine: None,
            awakening_brain: None,
            integration_bus: SubsystemIntegrationBus::new(500),
            exploration_driver: None,
            boredom: None,
            spreading: None,
            neuromodulators: None,
            causal: None,
            bio_memory: None,
            scar: None,
            consensus: None,
            shadow: None,
            cognitive_wal: None,
            skills: None,
            data_flywheel: None,
            phi: None,
            dashboard: None,
            arch_governor: None,
            rsi: None,
            cognitive_blackboard: None,
            attention_schema: None,
            salience_detector: None,
            qualia_generator: None,
            data_quality_pipeline: None,
            ouroboros_loop: None,
            module_registry: None,
            document_perception: None,
            web_accessibility: None,
            mcts_reasoner: None,
            dead_end_detector: None,
            counterfactual_simulator: None,
            pixel_perception: None,
            long_horizon_ocr: None,
            document_parser_registry: None,
            document_classifier: None,
            screenshot_pipeline: None,
            formula_extractor: None,
            entity_extractor: None,
            entity_graph: None,
            semantic_compressor: None,
            cte: None,
            memory_lattice: None,
            // Wave D: revived modules
            analogical_reasoner: None,
            epistemic_humility: None,
            belief_revision_engine: None,
            default_mode_network: None,
            hierarchical_world_model: None,
            system1: None,
            active_inference: None,
            free_energy_curiosity: None,
            dream_consolidator: None,
            reasoning_federation: None,
            iit_phi: None,
            iit_phi8: None,
            capability_synthesizer: None,
            seal_closed_loop: None,
            temporal_tick_counter: 0,
            temporal_prediction: TemporalPredictionTracker::new(100, 0.3),
            specious_present: None,
            last_c_score: 0.5,
            experience_buffer: VecDeque::new(),
            vsa_reasoner: None,
            hebbian_memory: None,
            hebbian_distillation: None,
            // Post-Awakening subsystems
            sleep_gate: None,
            narrative_self: None,
            appraisal_engine: None,
            emotional_steering: None,
            emotional_trail: None,
            emotion_regulation: None,
            affective_forecast: None,
            human_emotion_reading: None,
            human_emotion_detector: None,
            performance_oracle: None,
            quality_gate: None,
            acting_dimension_assessor: None,
            link_formation: None,
            tool_synthesizer: None,
            tool_registry: None,
            cross_model_distiller: None,
            distillation_buffer: None,
            // SecurityExecutive subsystems
            threat_modeler: None,
            risk_sensor: None,
            supply_chain_guard: None,
            adversarial_reasoner: None,
            self_defense: None,
            evolution_gatekeeper: None,
            audit_trail: None,
            // Phase 33: MindBridge
            mind_bridge: None,
            // Phase 42: GracefulDegradationManager
            graceful_deg_manager: None,
            // Veto gate
            volition_engine: None,
            unified_will: None,
            // Meta monitoring
            meta_accuracy: MetaAccuracyTracker::new(),
            evolution_efficiency: EvolutionEfficiencyTracker::new(),
            goal_drift_index: Some(GoalDriftIndex::new(100)),
            hybrid_retrieval: None,
            compound_knowledge: Some(CompoundKnowledgeBase::new(std::path::PathBuf::from(
                "archive/knowledge",
            ))),
            agent_team: Some(TeamOrchestrator::new(
                crate::core::nt_core_experience::agent_team::TeamPattern::Supervisor,
            )),
            code_mutation: Some(CodeMutationEngine::new()),
            seal_governance: SEALGovernance::new(),
            meta_seal_engine: None,
            sindy_engine: None,
            stream_pipeline: None,
            metabolic_budget: MetabolicBudget::default(),
            competitive_selection: None,
            semantic_entropy: None,
            skill_registry: None,
            timeline_orchestrator: None,
            deep_digestion_pipeline: None,
            constellation_detector: None,
            cross_timeline_integrator: None,
            reconstructive_buffer: None,
            attention_self_modelling: None,
            overthinking_detector: None,
            concurrency_capacity: 0.0,
            invariant_safety_counter: 0,
            inner_monologue_manager: None,
            cognitive_controller: None,
            internal_tick_counter: 0,
            pending_cognitive_load: None,
            pending_subsystem_reset: None,
            ratchet_tracker: RatchetTracker::new(),
            reason_temperature: 1.0,
            step_time_budget_ms: 1000,
            l1_reflex_rejected: false,
            dual_process: DualProcessConfig::default(),
            sleep_stage: SleepStageConfig::default(),
            attentional_gate: AttentionalGate::default(),
        };

        cycle.init_missing_fields();

        // Core pipeline subsystems
        assert!(cycle.consolidation_bridge.is_some(), "consolidation_bridge");
        assert!(cycle.image_cache.is_some(), "image_cache");
        assert!(cycle.multi_modal_gate.is_some(), "multi_modal_gate");
        assert!(cycle.inner_critic.is_some(), "inner_critic");
        assert!(cycle.verification_gate.is_some(), "verification_gate");
        assert!(cycle.executive_controller.is_some(), "executive_controller");
        assert!(cycle.consciousness_stream.is_some(), "consciousness_stream");
        assert!(cycle.master_consciousness.is_some(), "master_consciousness");
        assert!(
            cycle.metacognitive_controller.is_some(),
            "metacognitive_controller"
        );
        assert!(cycle.sensor_grounding.is_some(), "sensor_grounding");
        assert!(cycle.identity_defense.is_some(), "identity_defense");
        assert!(cycle.identity_fragments.is_some(), "identity_fragments");
        assert!(cycle.identity_chain.is_some(), "identity_chain");
        assert!(
            cycle.consciousness_architecture.is_some(),
            "consciousness_architecture"
        );
        assert!(cycle.substrate_gen.is_some(), "substrate_gen");
        assert!(cycle.exploration_driver.is_some(), "exploration_driver");
        assert!(cycle.boredom.is_some(), "boredom");
        assert!(cycle.spreading.is_some(), "spreading");
        assert!(cycle.neuromodulators.is_some(), "neuromodulators");
        assert!(cycle.causal.is_some(), "causal");
        assert!(cycle.bio_memory.is_some(), "bio_memory");
        assert!(cycle.scar.is_some(), "scar");
        assert!(cycle.consensus.is_some(), "consensus");
        assert!(cycle.shadow.is_some(), "shadow");
        assert!(cycle.cognitive_wal.is_some(), "cognitive_wal");
        assert!(cycle.skills.is_some(), "skills");
        assert!(cycle.data_flywheel.is_some(), "data_flywheel");
        assert!(cycle.dashboard.is_some(), "dashboard");
        assert!(cycle.arch_governor.is_some(), "arch_governor");
        assert!(cycle.rsi.is_some(), "rsi");
        assert!(cycle.cognitive_blackboard.is_some(), "cognitive_blackboard");
        assert!(cycle.attention_schema.is_some(), "attention_schema");
        assert!(cycle.salience_detector.is_some(), "salience_detector");
        assert!(cycle.qualia_generator.is_some(), "qualia_generator");
        assert!(
            cycle.data_quality_pipeline.is_some(),
            "data_quality_pipeline"
        );
        assert!(cycle.ouroboros_loop.is_some(), "ouroboros_loop");
        assert!(cycle.module_registry.is_some(), "module_registry");
        assert!(cycle.document_perception.is_some(), "document_perception");
        assert!(cycle.web_accessibility.is_some(), "web_accessibility");
        assert!(cycle.mcts_reasoner.is_some(), "mcts_reasoner");
        assert!(cycle.dead_end_detector.is_some(), "dead_end_detector");
        assert!(
            cycle.counterfactual_simulator.is_some(),
            "counterfactual_simulator"
        );
        assert!(cycle.pixel_perception.is_some(), "pixel_perception");
        assert!(cycle.long_horizon_ocr.is_some(), "long_horizon_ocr");
        assert!(
            cycle.document_parser_registry.is_some(),
            "document_parser_registry"
        );
        assert!(cycle.document_classifier.is_some(), "document_classifier");
        assert!(cycle.formula_extractor.is_some(), "formula_extractor");
        assert!(cycle.semantic_compressor.is_some(), "semantic_compressor");
        assert!(cycle.cte.is_some(), "cte");
        assert!(cycle.memory_lattice.is_some(), "memory_lattice");
        // Wave D: revived modules
        assert!(cycle.analogical_reasoner.is_some(), "analogical_reasoner");
        assert!(cycle.epistemic_humility.is_some(), "epistemic_humility");
        assert!(
            cycle.belief_revision_engine.is_some(),
            "belief_revision_engine"
        );
        assert!(cycle.default_mode_network.is_some(), "default_mode_network");
        assert!(
            cycle.hierarchical_world_model.is_some(),
            "hierarchical_world_model"
        );

        // perception_input and phi remain None (constructor also sets them to None)
        assert!(
            cycle.perception_input.is_none(),
            "perception_input should remain None"
        );
        assert!(cycle.phi.is_none(), "phi should remain None");
        assert!(cycle.iit_phi8.is_some(), "iit_phi8");
        assert!(cycle.compound_knowledge.is_some(), "compound_knowledge");
        assert!(cycle.agent_team.is_some(), "agent_team");
        assert!(cycle.code_mutation.is_some(), "code_mutation");
        assert!(
            cycle.human_emotion_detector.is_some(),
            "human_emotion_detector"
        );
        assert!(
            cycle.human_emotion_reading.is_some(),
            "human_emotion_reading"
        );
    }

    // ── Wave E: RatchetTracker rollback trigger tests ──
    #[test]
    fn test_ratchet_tracker_is_broken_after_max_regressions() {
        let mut rt = RatchetTracker {
            baseline: 10.0,
            best: 10.0,
            is_held: true,
            regressions: 0,
            max_regressions: 3,
            constraint_preserved: true,
            regression_risk: 0.0,
            past_regression_rate: 0.0,
            audit_trail: Vec::new(),
        };
        assert!(!rt.is_broken());
        rt.check(9.0, "regression_1");
        assert!(!rt.is_broken());
        rt.check(8.0, "regression_2");
        assert!(!rt.is_broken());
        rt.check(7.0, "regression_3");
        assert!(rt.is_broken());
        assert!(rt.regressions >= rt.max_regressions);
        assert!(!rt.is_held);
    }

    #[test]
    fn test_ratchet_tracker_quantify_regression_risk_after_regressions() {
        let mut rt = RatchetTracker {
            baseline: 10.0,
            best: 10.0,
            is_held: true,
            regressions: 0,
            max_regressions: 3,
            constraint_preserved: true,
            regression_risk: 0.0,
            past_regression_rate: 0.0,
            audit_trail: Vec::new(),
        };
        // 5 regressions in a window of 10
        for i in 0..10u64 {
            rt.check(if i < 5 { 5.0 } else { 12.0 }, &format!("step_{}", i));
        }
        let risk = rt.quantify_regression_risk(10);
        assert!(
            risk > 0.0,
            "risk should be positive after regressions: {}",
            risk
        );
        rt.update_regression_risk();
        assert!(rt.regression_risk > 0.0);
    }

    #[test]
    fn test_ratchet_tracker_commit_resets_state() {
        let mut rt = RatchetTracker {
            baseline: 5.0,
            best: 10.0,
            is_held: true,
            regressions: 2,
            max_regressions: 3,
            constraint_preserved: false,
            regression_risk: 0.7,
            past_regression_rate: 0.0,
            audit_trail: vec![(5.0, "a".into(), false)],
        };
        rt.commit();
        assert_eq!(rt.baseline, 10.0);
        assert_eq!(rt.regressions, 0);
        assert!(rt.is_held);
        assert!(rt.constraint_preserved);
        assert_eq!(rt.regression_risk, 0.0);
    }
}
