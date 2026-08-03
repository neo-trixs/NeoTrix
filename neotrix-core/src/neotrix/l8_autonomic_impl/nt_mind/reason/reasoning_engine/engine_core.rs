use std::sync::{Arc, Mutex};
use std::collections::BTreeMap;

use crate::core::l7_capability::nt_core_antidistil::AntiDistillationSystem;
use crate::core::nt_core_bank::ReasoningBank;
use crate::core::nt_core_e8::state_machine::E8StateMachine;
use crate::core::nt_core_e8::thinking_budget::DifficultyEstimator;
use crate::core::nt_core_e8::ewhr_bridge::E8EwhrBridge;
use crate::core::nt_core_e8::domain_transition::{CoTLength, E8TaskType, E8DomainTransitionModel};
use crate::core::nt_core_e8::nt_core_e8_prediction::E8PredictionOracle;
use crate::core::nt_core_e8::nt_core_fable_pattern::{FablePatternMatcher, FablePhase};
use crate::core::nt_core_e8::nt_core_synthesis::{ConsciousnessCoreSynthesis, SynthesisEffortTier};
use crate::core::nt_core_e8::sparse_moe::SparseMoERouter;
use crate::core::nt_core_e8::unified_latent::UnifiedLatentSpace;
use crate::core::nt_core_e8::nt_latent_reasoning::LatentReasoningPipeline;
use crate::core::nt_core_e8::nt_multimodal::{MultimodalEncoder, MultimodalInput};
use crate::core::nt_core_sae_bridge::SAEBridge;
use crate::core::nt_core_ttc::{EffortTier, EffortTierSelector, TtcEngine};
use crate::core::nt_core_prm::ProcessRewardLearner;
use crate::core::nt_core_aura::IntentEngine;
use crate::core::nt_core_trajectory_compress::{CompressionLevel, TrajectoryCompressor};

use crate::core::nt_io_telemetry::{AttributeValue, ConsoleTracer, CostTracker, NoopTracer, SpanKind, Tracer};
use crate::core::nt_core_gwt::resonance::MODULE_COUNT;
use crate::core::nt_core_gwt::workspace::GlobalWorkspace;
use crate::core::nt_core_hex::{FullReasoningState, ReasoningHexagram};
use crate::core::nt_core_observer::OneObserver;
use crate::core::nt_core_observer_error::ObserverErrorRecovery;
use crate::core::nt_core_self::silicon_self::SiliconSelfModel;
use crate::core::l7_capability::nt_act_orch_patterns::Orchestrator;
use crate::neotrix::nt_mind::core::BrainMutView;
use crate::neotrix::nt_mind::distillation::{AntiPattern, StrategicPrinciple};
use crate::neotrix::nt_mind::model_router::ModelRouter;
use crate::neotrix::nt_mind::reasoning_types::{ReasoningTrace, ReasoningType};
use crate::neotrix::nt_mind::control_distillation::{ControlDistiller, AlternatingSequence, ReasoningStep, ControlTrainer, SftReport, CsppoReport};
use crate::neotrix::nt_memory_kb::KnowledgeBase;
use crate::neotrix::nt_mind::context_artifacts::indexer::ArtifactIndexer;
use crate::neotrix::nt_io_provider::{LlmProvider, LlmRequest};
use crate::neotrix::nt_core_error::{NeoTrixResult, NeoTrixError};
use crate::neotrix::l9_transcendent_impl::nt_mind_consciousness_gold_standard::ConsciousnessGoldStandard;
use super::CognitiveEye;

pub const MAX_COST_LOG: usize = 1000;
pub const MAX_TRACES: usize = 1000;
/// F6 训练节流: 累积多少条交替序列后触发一次 SFT + CSPO 训练。
pub const CONTROL_TRAIN_BATCH: usize = 8;

pub struct CostRecord {
    pub tier: String,
    pub cost_estimate_usd: f64,
    pub duration_ms: u64,
    pub timestamp: i64,
}
pub type ReasoningStats = (usize, u64, f64);
pub type EngineMetrics = (usize, u64, u64, f64);

#[derive(serde::Serialize, serde::Deserialize)]
struct E8PersistedState {
    current_mode: u8,
    current_meta: u8,
    last_e8_attention_weights: Option<Vec<f64>>,
    #[serde(default)]
    last_e8_confidence: f64,
    trajectory_modes: Vec<(u8, u8)>,
    /// Serialized E8Policy RL state (mode_values, mode_counts, factor_energies, factor_control)
    #[serde(default)]
    e8_policy: Option<crate::core::nt_core_policy::E8Policy>,
    /// PRM learning count
    #[serde(default)]
    prm_learning_count: u64,
    /// PRM score history (last 100)
    #[serde(default)]
    prm_score_history: Vec<f64>,
}

pub struct ReasoningEngine {
    pub current_state: FullReasoningState,
    pub state_trajectory: Vec<FullReasoningState>,
    pub strategy_matrix: [[ReasoningHexagram; 8]; 8],
    pub observer: OneObserver,
    pub distill_interval: usize,
    pub last_core_plan: Option<String>,
    pub brain: Box<dyn BrainMutView>,
    pub bank: ReasoningBank,
    pub traces: Vec<ReasoningTrace>,
    pub principles: Vec<StrategicPrinciple>,
    pub anti_patterns: Vec<AntiPattern>,
    pub gwt: Option<GlobalWorkspace>,
    pub silicon_self: Option<SiliconSelfModel>,
    pub last_step_rewards: Vec<(String, f64)>,
    pub cost_log: Vec<CostRecord>,
    pub router: ModelRouter,
    pub gateway: Option<Arc<dyn LlmProvider>>,
    pub default_model: String,
    pub llm_call_count: u64,
    pub llm_total_time_ms: u64,
    pub llm_last_duration_ms: u64,
    pub bank_retrieval_count: u64,
    pub kb: Option<KnowledgeBase>,
    pub artifact_indexer: Option<ArtifactIndexer>,
    pub cognitive_eye: CognitiveEye,
    pub ttc_engine: Option<TtcEngine>,
    pub orchestrator: Option<Box<dyn Orchestrator>>,
    pub trajectory_compressor: Option<TrajectoryCompressor>,
    pub anti_distillation: Option<AntiDistillationSystem>,
    pub tracer: Option<ConsoleTracer>,
    pub cost_tracker: Option<CostTracker>,
    pub(crate) _last_watermarked: Option<String>,
    /// Control Distillation (F6): extracts alternating sequences from traces for CSPO training
    pub control_distiller: Option<ControlDistiller>,
    /// Distilled alternating sequences (control segments) for training feedback
    pub distilled_sequences: Vec<AlternatingSequence>,
    /// Batches of distilled sequences consumed by the trainer since last SFT/CSPO run
    pub train_batch: usize,
    /// E8→EWHR bridge: auto-proposes hypotheses from reasoning trajectory
    pub ewhr_bridge: Option<E8EwhrBridge>,
    /// SAE bridge: extracts interpretable features from E8 reasoning states
    pub sae_bridge: Option<SAEBridge>,
    /// PRM: process reward model scoring E8 reasoning steps
    pub prm: Option<ProcessRewardLearner>,
    /// Intent engine: tracks user/agent intent through reasoning
    pub intent_engine: Option<IntentEngine>,
    /// Hypothesis network: shared with EWHR REST API
    pub hypothesis_network: Option<Arc<Mutex<crate::neotrix::nt_memory_historian::nt_evidence_hypothesis::HypothesisNetwork>>>,
    /// Fable-5 pattern matcher: scores trajectory alignment against Mythos reasoning phases
    pub fable_matcher: Option<FablePatternMatcher>,
    /// E8 prediction oracle: distributional prediction with ensemble + MCTS
    pub prediction_oracle: Option<E8PredictionOracle>,
    /// Most recent E8 attention weights from prediction oracle (differentiable GWT bridge).
    /// Computed via `E8PredictionOracle::attention_weights()` after each `reason()` call.
    /// Shape: [f64; 64] — softmax-tempered distribution over all 64 E8 states.
    /// This is the GWT-differentiable bridge: GWT can modulate specialist attention
    /// based on which E8 states the oracle predicts as most likely next states.
    pub last_e8_attention_weights: Option<Vec<f64>>,
    /// MCTS confidence from the last prediction cycle, used as adaptive bias for GWT attention.
    /// Range: [0.0, 1.0]. Defaults to 0.5 at initialization.
    pub last_e8_confidence: f64,
    /// Domain-aware transition model with 6 sub-matrices (one per task type).
    /// Previously E8DomainTransitionModel (170 lines, 12 tests) was completely
    /// orphaned — defined in domain_transition.rs but never used in any production
    /// code path. Now wired into the prediction oracle for domain-specific blending.
    pub domain_transition_model: Option<E8DomainTransitionModel>,
    /// Fable 5 effort tier selector: maps task difficulty + length to
    /// Low/Medium/High/XHigh/Max tiers controlling sparse attention k,
    /// MCTS simulations, and TTC rollout depth.
    pub effort_tier_selector: EffortTierSelector,
    /// Most recently selected effort tier (for telemetry).
    pub last_effort_tier: Option<EffortTier>,
    /// Fused consciousness-core synthesis: all mainstream model innovations
    /// (K3 quantile balancing + sparse attention + AttnRes, DeepSeek-V4 mHC
    /// Birkhoff projection, Gemini 3.6 step routing cache, Qwen3/Fable 5
    /// effort tiers) fused into a single optimal prediction pipeline.
    pub synthesis: ConsciousnessCoreSynthesis,
    /// Observer error recovery with retry + circuit breaker + fallback
    pub observer_error_recovery: ObserverErrorRecovery,
    /// Phase 6.3 — sparse MoE router: groups the 64 E₈ states into 8 expert
    /// groups and routes attention to the top-2 each step, freezing the rest.
    pub sparse_moe: SparseMoERouter,
    /// Phase 10.1 — unified latent space bridging E₈ / GWT / HyperCube.
    pub unified_latent: UnifiedLatentSpace,
    /// Phase 10.2 — end-to-end latent reasoning: E8 latent → hypercube query →
    /// GWT broadcast with no intermediate text.
    pub latent_reasoning: LatentReasoningPipeline,
    /// Phase 10.3 — multimodal unified reasoning: text+image+audio encoders →
    /// unified latent space → cross-modal fusion driving the E8 loop.
    pub multimodal: MultimodalEncoder,
}

impl ReasoningEngine {
    pub fn new(brain: Box<dyn BrainMutView>, bank: ReasoningBank) -> Self {
        Self {
            current_state: FullReasoningState::new(
                ReasoningHexagram::new(0),
                crate::core::nt_core_hex::MetaState::new(0),
            ),
            state_trajectory: Vec::new(),
            strategy_matrix: [[ReasoningHexagram::new(0); 8]; 8],
            observer: OneObserver::new(),
            distill_interval: 0,
            last_core_plan: None,
            brain,
            bank,
            traces: Vec::new(),
            principles: Vec::new(),
            anti_patterns: Vec::new(),
            gwt: None,
            silicon_self: None,
            last_step_rewards: Vec::new(),
            cost_log: Vec::new(),
            router: ModelRouter::new(),
            gateway: None,
            default_model: "default".into(),
            llm_call_count: 0,
            llm_total_time_ms: 0,
            llm_last_duration_ms: 0,
            bank_retrieval_count: 0,
            kb: None,
            artifact_indexer: None,
            cognitive_eye: CognitiveEye::new(),
            ttc_engine: None,
            orchestrator: None,
            trajectory_compressor: None,
            anti_distillation: None,
            tracer: None,
            cost_tracker: None,
            _last_watermarked: None,
            control_distiller: Some(ControlDistiller::new(Arc::new(ConsciousnessGoldStandard::new()))),
            distilled_sequences: Vec::new(),
            train_batch: 0,
            ewhr_bridge: None,
            sae_bridge: None,
            prm: None,
            intent_engine: None,
            hypothesis_network: None,
            fable_matcher: None,
            prediction_oracle: None,
            last_e8_attention_weights: None,
            last_e8_confidence: 0.5,
            domain_transition_model: None,
            effort_tier_selector: EffortTierSelector::default(),
            last_effort_tier: None,
            synthesis: ConsciousnessCoreSynthesis::default(),
            observer_error_recovery: ObserverErrorRecovery::new(),
            sparse_moe: SparseMoERouter::new(),
            unified_latent: UnifiedLatentSpace::new(),
            latent_reasoning: LatentReasoningPipeline::new(),
            multimodal: MultimodalEncoder::new(),
        }
    }

    pub fn from_env() -> Self {
        use crate::neotrix::nt_mind::self_iterating::brain_core::ReasoningBrain;
        Self::new(Box::new(ReasoningBrain::new()), ReasoningBank::new(4))
    }

    pub fn from_parts(brain: Box<dyn BrainMutView>, bank: ReasoningBank) -> Self {
        Self::new(brain, bank)
    }

    pub fn with_gateway(mut self, gateway: Arc<dyn LlmProvider>) -> Self {
        self.gateway = Some(gateway);
        self
    }

    pub fn with_kb(mut self, kb: KnowledgeBase) -> Self {
        self.kb = Some(kb);
        self
    }

    pub fn with_artifact_indexer(mut self, indexer: ArtifactIndexer) -> Self {
        self.artifact_indexer = Some(indexer);
        self
    }

    pub fn with_ttc_engine(mut self, engine: TtcEngine) -> Self {
        self.ttc_engine = Some(engine);
        self
    }

    pub fn with_orchestrator(mut self, orch: Box<dyn Orchestrator>) -> Self {
        self.orchestrator = Some(orch);
        self
    }

    pub fn with_trajectory_compressor(mut self, level: CompressionLevel) -> Self {
        self.trajectory_compressor = Some(TrajectoryCompressor::new(level));
        self
    }

    pub fn with_anti_distillation(mut self, ads: AntiDistillationSystem) -> Self {
        self.anti_distillation = Some(ads);
        self
    }

    pub fn with_tracer(mut self, tracer: ConsoleTracer) -> Self {
        self.tracer = Some(tracer);
        self
    }

    pub fn with_ewhr_bridge(mut self, bridge: E8EwhrBridge) -> Self {
        self.ewhr_bridge = Some(bridge);
        self
    }

    pub fn with_hypothesis_network(mut self, net: Arc<Mutex<crate::neotrix::nt_memory_historian::nt_evidence_hypothesis::HypothesisNetwork>>) -> Self {
        self.hypothesis_network = Some(net);
        self
    }

    pub fn with_sae_bridge(mut self, sae_bridge: SAEBridge) -> Self {
        self.sae_bridge = Some(sae_bridge);
        self
    }

    pub fn with_prm(mut self, prm: ProcessRewardLearner) -> Self {
        self.prm = Some(prm);
        self
    }

    pub fn with_intent_engine(mut self, intent_engine: IntentEngine) -> Self {
        self.intent_engine = Some(intent_engine);
        self
    }

    pub fn with_cost_tracker(mut self, cost_tracker: CostTracker) -> Self {
        self.cost_tracker = Some(cost_tracker);
        self
    }

    pub fn with_gwt(mut self, gwt: GlobalWorkspace) -> Self {
        self.gwt = Some(gwt);
        self
    }

    pub fn with_silicon_self(mut self, ss: SiliconSelfModel) -> Self {
        self.silicon_self = Some(ss);
        self
    }

    pub fn with_fable_matcher(mut self, matcher: FablePatternMatcher) -> Self {
        self.fable_matcher = Some(matcher);
        self
    }

    pub fn with_prediction_oracle(mut self, oracle: E8PredictionOracle) -> Self {
        self.prediction_oracle = Some(oracle);
        self
    }

    pub fn with_domain_transition(mut self, model: E8DomainTransitionModel) -> Self {
        self.domain_transition_model = Some(model);
        self
    }

    pub fn with_effort_tier_selector(mut self, selector: EffortTierSelector) -> Self {
        self.effort_tier_selector = selector;
        self
    }

    pub fn with_observer_transition_matrix(mut self, matrix: crate::core::nt_core_e8::E8TransitionMatrix) -> Self {
        self.observer = self.observer.with_transition_matrix(matrix);
        self
    }

    pub fn reason(&mut self, task: &str) -> NeoTrixResult<String> {
        let root_span = self.tracer.as_ref()
            .map(|t| t.start_span("reason", SpanKind::Handoff))
            .unwrap_or_else(|| NoopTracer.start_span("reason", SpanKind::Handoff));
        root_span.set_attribute("task", AttributeValue::String(task.to_string()));
        root_span.set_attribute("e8_state", AttributeValue::String(self.current_state.mode.mode_name().to_string()));
        root_span.set_attribute("agent", AttributeValue::String("ReasoningEngine".to_string()));
        root_span.set_gen_ai_system("neotrix");

        let mut e8_machine = E8StateMachine::from(self.current_state);
        if let Some(ref ttc) = self.ttc_engine {
            e8_machine.set_ttc_engine(ttc.clone());
            let difficulty = DifficultyEstimator::heuristic_difficulty(task, "reasoning");
            root_span.set_attribute("ttc_difficulty", AttributeValue::Float(difficulty));
            if difficulty > 0.3 {
                let allocation = ttc.allocate_budget(difficulty, 1.0);
                root_span.set_attribute("ttc_strategy", AttributeValue::String(format!("{:?}", allocation.strategy)));
                root_span.set_attribute("ttc_max_steps", AttributeValue::Int(allocation.budget.max_steps as i64));
                e8_machine.budget = Some(allocation);
            }
        }

        let hex = self.current_state.mode;
        let mode_name = hex.mode_name();
        let mode_desc = hex.mode_description();
        let context = self.build_context(task, ReasoningType::Conversation);
        let artifact_ctx = self.build_artifact_context(task);

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let date_line = if let Some(ref ads) = self.anti_distillation {
            let watermarked = ads.encode_date_line(&format!("Today's date is {}.", today));
            format!("\n{}\n", watermarked)
        } else {
            format!("\nToday's date is {}.\n", today)
        };

        // Pre-call task decomposition: split sensitive tasks into
        // safer subtasks when anticlistillation detects risk.
        let query = if let Some(ref ads) = self.anti_distillation {
            if let Some(suggestions) = ads.decompose_task(task) {
                let mut decomposed = String::new();
                decomposed.push_str("This task has been decomposed into steps:\n");
                for s in &suggestions {
                    decomposed.push_str(&format!("- {}\n", s.subtask));
                }
                decomposed.push_str("\nProceed through each step sequentially.");
                root_span.set_attribute("antidistil_decomposed", AttributeValue::Bool(true));
                root_span.set_attribute("antidistil_decomp_steps", AttributeValue::Int(suggestions.len() as i64));
                decomposed
            } else {
                task.to_string()
            }
        } else {
            task.to_string()
        };

        // Inject KB context from E8 state
        let kb_context = if let Some(ref kb) = self.kb {
            if let Ok(results) = kb.query_by_e8_state(self.current_state.mode, 5) {
                if !results.is_empty() {
                    let mut s = String::from("KB knowledge:\n");
                    for r in &results { s.push_str(&format!("- {} (score: {:.2})\n", r.node.title, r.score)); }
                    s
                } else { String::new() }
            } else { String::new() }
        } else { String::new() };

        let prompt = format!(
            "You are NeoTrix — mode: {mode_name}\n\
             Strategy: {mode_desc}\n\n\
             Past experiences:\n{context}\n\n\
              {kb_context}{artifact_ctx}{date_line}\
             Query: {query}"
        );

        let llm_span = self.tracer.as_ref()
            .map(|t| t.start_child_span(&root_span, "call_llm", SpanKind::Llm))
            .unwrap_or_else(|| NoopTracer.start_child_span(&root_span, "call_llm", SpanKind::Llm));
        llm_span.set_gen_ai_request_model(&self.default_model);

        let result = self.call_llm(&prompt);

        match &result {
            Ok(response) => {
                llm_span.set_attribute("response_length", AttributeValue::Int(response.len() as i64));
                if let Some(ref t) = self.tracer { t.end_span(llm_span); } else { NoopTracer.end_span(llm_span); }
                if let Some(ref _ttc) = self.ttc_engine {
                    let early_exit = e8_machine.check_early_exit(0.9);
                    root_span.set_attribute("ttc_early_exit", AttributeValue::Bool(early_exit));
                }
                let refused = detect_refusal_response(response);
                if let Some(ref mut ads) = self.anti_distillation {
                    ads.record_llm_call(refused);
                    let bits = ads.watermark.to_bits();
                    let watermarked = ads.watermark_response(response);
                    ads.register_trace(response, bits, &self.default_model, &prompt[..prompt.len().min(64)]);
                    if self.gateway.is_some() {
                        ads.detector.record_request(
                            &self.default_model,
                            task,
                            0.7,
                            4096,
                            response.len(),
                        );
                    }
                    root_span.set_attribute("antidistil_bits", AttributeValue::Int(bits as i64));
                    root_span.set_attribute("antidistil_watermarked", AttributeValue::Bool(true));
                    root_span.set_attribute("antidistil_refused", AttributeValue::Bool(refused));
                    // Store watermarked response for return, but don't early-return —
                    // continue to trajectory compression + KB learning.
                    self._last_watermarked = Some(watermarked);
                }
                refused
            }
            Err(e) => {
                llm_span.set_attribute("error", AttributeValue::String(format!("{}", e)));
                if let Some(ref t) = self.tracer { t.end_span(llm_span); } else { NoopTracer.end_span(llm_span); }
                false
            }
        };

        self.current_state = e8_machine.current_state;
        // Only record the start-of-call state when it actually differs from the
        // last trajectory entry. Combined with the oracle-driven transition in
        // the prediction block below, the trajectory becomes a clean sequence of
        // real state transitions instead of a run of identical self-loop states.
        if self.state_trajectory.last().map(|s| s.mode.0) != Some(self.current_state.mode.0) {
            self.state_trajectory.push(self.current_state);
        }

        if let Some(ref mut sae) = self.sae_bridge {
            let hex = self.current_state.mode;
            let features = sae.extract_features(hex.0, self.current_state.meta.0, &[]);
            if !features.is_empty() {
                root_span.set_attribute("sae_active_features", AttributeValue::Int(features.len() as i64));
                root_span.set_attribute("sae_top_feature", AttributeValue::String(
                    features.iter().max_by(|a, b| a.activation.total_cmp(&b.activation))
                        .map(|f| format!("f{}({:.3})", f.index, f.activation))
                        .unwrap_or_default()
                ));
            }
        }

        // Observer analysis: record transitions, detect patterns, compute PRM scores
        let observer_report = self.observer.analyze(&self.state_trajectory, &[task]);
        root_span.set_attribute("observer_traj_len", AttributeValue::Int(observer_report.trajectory_len as i64));
        root_span.set_attribute("observer_quality", AttributeValue::Float(observer_report.quality_score));
        root_span.set_attribute("observer_distinct_states", AttributeValue::Int(observer_report.distinct_states as i64));
        for p in &observer_report.patterns {
            root_span.set_attribute("observer_pattern", AttributeValue::String(p.clone()));
        }
        if observer_report.has_actionable_insight {
            root_span.set_attribute("observer_insight", AttributeValue::Bool(true));
        }
        if let Some(w) = observer_report.trajectory_weighted_score {
            root_span.set_attribute("observer_traj_weighted", AttributeValue::Float(w));
        }
        if let Some(c) = observer_report.convergence_score {
            root_span.set_attribute("observer_convergence", AttributeValue::Float(c));
        }

        // ProcessRewardLearner: learn from E8 trajectory using step-level rewards
        if let Some(ref mut prm_learner) = self.prm {
            let task_string = task.to_string();
            let traj_len = self.state_trajectory.len();
            prm_learner.learn_step(|collector| {
                collector.begin(task_string.clone());
                for (i, state) in self.state_trajectory.iter().enumerate() {
                    collector.record_step(
                        crate::core::nt_core_traits::SpecialistType::ReflectionEngine,
                        state.mode,
                        format!("e8_step_{}", i),
                        String::new(),
                        String::new(),
                        None,
                        true,
                        Some(state.meta.0 as f64 / 3.0),
                    );
                }
                if traj_len > 0 {
                    collector.finish(Some(observer_report.quality_score), true);
                } else {
                    collector.finish(None, false);
                }
            });
            root_span.set_attribute("prm_avg_score", AttributeValue::Float(prm_learner.avg_recent_score(10)));
            root_span.set_attribute("prm_learning_count", AttributeValue::Int(prm_learner.learning_count as i64));
        }

        // Fable-5 pattern matcher: score trajectory alignment against Mythos reasoning phases
        if let Some(ref matcher) = self.fable_matcher {
            if self.state_trajectory.len() >= 2 {
                let traj_modes: Vec<u8> = self.state_trajectory.iter().map(|s| s.mode.0).collect();
                let task_type_idx = match E8TaskType::detect(task) {
                    E8TaskType::General => 0,
                    E8TaskType::Reasoning => 1,
                    E8TaskType::Math => 2,
                    E8TaskType::Coding => 3,
                    E8TaskType::Agentic => 4,
                    E8TaskType::Creative => 5,
                };
                let alignment = matcher.score_alignment_advanced(&traj_modes, task_type_idx, 0.5);
                root_span.set_attribute("fable_composite", AttributeValue::Float(alignment.composite));
                root_span.set_attribute("fable_non_linear", AttributeValue::Float(alignment.non_linear_score));
                root_span.set_attribute("fable_phase_score", AttributeValue::Float(alignment.quality));
                root_span.set_attribute("fable_transition_score", AttributeValue::Float(alignment.transition_score));

                let sqv = matcher.sqv_score(&traj_modes);
                if sqv > 0.01 {
                    root_span.set_attribute("fable_sqv", AttributeValue::Float(sqv));
                }
                let deep = matcher.detect_deep_reason_pattern(&traj_modes);
                if deep > 0.0 {
                    root_span.set_attribute("fable_deep_reason", AttributeValue::Float(deep));
                }
            }
        }

        // E8 Prediction Oracle: compute prediction distribution for next E8 state
        // This provides a differentiable attention bridge to GWT via attention_weights()
        //
        // THREE WEAK LINKS REPAIRED:
        // 1. Domain-aware TM: E8DomainTransitionModel (170 lines, 6 sub-matrices, 12 tests)
        //    was completely orphaned — never used in production. Now domain-matrices are
        //    blended with the general matrix for task-type-specific transition priors.
        // 2. MCTS: predict_with_mcts() was constructed and injected but never called in
        //    production — only predict_distribution() (ensemble without lookahead) was used.
        //    MCTS adds 32-simulation beam search over E8 transition dynamics.
        // 3. save_e8() was defined but never called — all runtime learning lost on exit.
        //    Now persisted via the run_seal_loop() save_e8() call every 3 iterations.
        // Fable 5 effort tier: maps task difficulty + length to
        // Low/Medium/High/XHigh/Max, controlling sparse attention k,
        // MCTS simulations, and TTC rollout depth. Computed BEFORE the
        // oracle call so the MCTS budget is actually applied per effort tier
        // (previously the tier was only selected after prediction and never
        // reached the MCTS predictor, which ran with fixed budget 8/50).
        let difficulty = DifficultyEstimator::heuristic_difficulty(task, "reasoning");
        let effort_tier = self.effort_tier_selector.select_for_task(difficulty, task.len());
        self.last_effort_tier = Some(effort_tier);
        root_span.set_attribute("effort_tier", AttributeValue::String(format!("{:?}", effort_tier)));
        root_span.set_attribute("effort_rollout_depth", AttributeValue::Int(effort_tier.rollout_depth() as i64));
        root_span.set_attribute("effort_sparse_k", AttributeValue::Int(effort_tier.sparse_k() as i64));
        root_span.set_attribute("effort_mcts_sims", AttributeValue::Int(effort_tier.mcts_simulations() as i64));

        if let Some(ref mut oracle) = self.prediction_oracle {
            // Apply effort-tier budget to the MCTS predictor: Low tiers run
            // shallow/cheap lookahead, Max tiers run deep beam search. The
            // effort table (0/8/16/32/64 sims × 2/4/8/16/32 depth) mirrors
            // Qwen3/Fable 5 thinking-budget scaling. min 1 sim so Low still
            // produces a real rollout.
            let sims = effort_tier.mcts_simulations().max(1);
            let depth = effort_tier.rollout_depth().max(2);
            oracle.mcts.num_simulations = sims;
            oracle.mcts.max_depth = depth;
            root_span.set_attribute("e8_mcts_sims_effective", AttributeValue::Int(sims as i64));
            root_span.set_attribute("e8_mcts_depth_effective", AttributeValue::Int(depth as i64));
            let task_type = E8TaskType::detect(task);
            let current_mode = self.current_state.mode.0;
            let cot_length = CoTLength::from_tokens(task.len().max(100));
            let phase_step = self.state_trajectory.len().min(8);
            let current_phase = match phase_step {
                0 => FablePhase::Acknowledgment,
                1 => FablePhase::ProblemRestatement,
                2 => FablePhase::Decomposition,
                3 => FablePhase::FirstPrinciples,
                4 => FablePhase::SelfVerification,
                5 => FablePhase::AlternativeConsideration,
                6 => FablePhase::DeepDive,
                7 => FablePhase::Synthesis,
                _ => FablePhase::Conclusion,
            };

            // Domain-aware transition matrix: blend domain-specific + general matrix
            // based on detected task type. Falls back to raw observer TM if domain
            // model is not configured (backward compatible).
            let use_tm = if let Some(ref mut dtm) = self.domain_transition_model {
                let blended = dtm.blend(task_type);
                // Also record the current observer transition into the domain model
                // so each domain matrix accumulates task-type-specific patterns
                Some(blended)
            } else {
                self.observer.transition_matrix.clone()
            };

            if let Some(ref tm) = use_tm {
                // Record trajectory transitions into the domain model for future blending
                // Previously recorded a self-loop (cur, cur) which provided no meaningful
                // transition signal. Now iterates over state_trajectory windows to capture
                // actual state sequences — each (from, to) transition enriches the domain
                // sub-matrix for the detected task type.
                if let Some(ref mut dtm) = self.domain_transition_model {
                    if self.state_trajectory.len() >= 2 {
                        for w in self.state_trajectory.windows(2) {
                            dtm.record_transition(task_type, w[0].mode.0, w[1].mode.0);
                        }
                    }
                }

                // Use the real FablePatternMatcher (with community dataset weights) instead of
                // a fresh default, so the prediction oracle benefits from FableDistillationSeeder
                // knowledge across 2M+ community traces (GLM-5.2, Qwable-SDFT, Agentic-Distill, etc.)
                let pm = self.fable_matcher.clone().unwrap_or_default();
                // MCTS-enhanced prediction: blend ensemble (0.6) with MCTS beam search (0.4)
                // Previously only predict_distribution() was called in production — the MCTS
                // predictor was constructed and injected but never used. Now both are fused.
                let (dist, mcts_state, mcts_value, mcts_confidence) = oracle.predict_with_mcts(tm, current_mode, task_type, current_phase, &pm, cot_length);
                let (best_state, best_prob) = dist.best();
                let effective = dist.effective_90pct_count();
                root_span.set_attribute("e8_pred_best_state", AttributeValue::Int(best_state as i64));
                root_span.set_attribute("e8_pred_best_prob", AttributeValue::Float(best_prob));
                root_span.set_attribute("e8_pred_mcts_state", AttributeValue::Int(mcts_state as i64));
                root_span.set_attribute("e8_pred_entropy", AttributeValue::Float(dist.entropy));
                root_span.set_attribute("e8_pred_confidence", AttributeValue::Float(mcts_confidence));
                root_span.set_attribute("e8_pred_mcts_value", AttributeValue::Float(mcts_value));
                root_span.set_attribute("e8_pred_effective_90", AttributeValue::Int(effective as i64));
                root_span.set_attribute("e8_task_type", AttributeValue::String(task_type.label().to_string()));
                root_span.set_attribute("e8_current_phase", AttributeValue::String(current_phase.label().to_string()));
                root_span.set_attribute("e8_domain_blended", AttributeValue::Bool(self.domain_transition_model.is_some()));

                // Advance the E8 state machine toward the predicted best state so the
                // trajectory records real transitions instead of a frozen self-loop.
                // Previously E8StateMachine::transition() was never invoked in production
                // (HIGH-1 Track3): current_state stayed fixed, state_trajectory grew by
                // repeated identical states, and record_transition wrote only (cur, cur)
                // self-loops into the domain + general matrices. Now the prediction oracle
                // actually steers the state machine, respecting TTC budget gating.
                //
                // Confidence gate: only advance when the prediction carries real signal.
                // A near-uniform distribution (best_prob ~ 1/64, e.g. zero-data rows)
                // must not drive a random jump that would corrupt the learned transitions.
                let pred_hex = ReasoningHexagram::new(best_state);
                let pred_clear = best_prob >= 0.1 && best_state != current_mode;
                if pred_clear {
                    e8_machine.transition(pred_hex, task);
                    self.current_state = e8_machine.current_state;
                    // Guard against duplicate consecutive entries from the start-of-call push.
                    if self.state_trajectory.last().map(|s| s.mode.0) != Some(self.current_state.mode.0) {
                        self.state_trajectory.push(self.current_state);
                    }
                    root_span.set_attribute("e8_state_advanced", AttributeValue::Bool(true));
                } else {
                    root_span.set_attribute("e8_state_advanced", AttributeValue::Bool(false));
                }
                root_span.set_attribute("e8_trajectory_len", AttributeValue::Int(self.state_trajectory.len() as i64));

                // Store top-3 predictions as span attributes for GWT/SEAL consumption
                for (rank, &(state, prob)) in dist.top_5.iter().take(3).enumerate() {
                    root_span.set_attribute(
                        &format!("e8_pred_top{}", rank + 1),
                        AttributeValue::String(format!("s{}({:.3})", state, prob)),
                    );
                }

                // Fable 5 effort tier: computed above (before the oracle call) and
                // applied to the MCTS budget — reused here for the fusion pipeline's
                // sparse-k + thinking-budget scaling.
                let effort_tier = self.last_effort_tier.unwrap();

                // ── 意识体内核融合管线 (Consciousness Core Fusion) ──────────────
                // Fuses the defining 2026 frontier-model innovations into a single
                // optimal prediction:
                //   1. K3 Quantile Balancing → dominance_capped_distribution (no aux loss)
                //   2. K3 AttnRes → depth-residual skip-connection across trajectory
                //   3. K3 sparse experts → effort-scaled sparse top-K
                //   4. DeepSeek-V4 mHC → Birkhoff doubly-stochastic projection (column-balanced)
                //   5. Gemini 3.6 → step-route cache reusing routing across seal-loop steps
                //   6. Qwen3/Fable 5 → effort tier (thinking budget) scales sparsity
                let traj_modes: Vec<u8> = self.state_trajectory.iter().map(|s| s.mode.0).collect();
                let synth_effort = match effort_tier {
                    crate::core::nt_core_ttc::EffortTier::Low => SynthesisEffortTier::Low,
                    crate::core::nt_core_ttc::EffortTier::Medium => SynthesisEffortTier::Medium,
                    crate::core::nt_core_ttc::EffortTier::High => SynthesisEffortTier::High,
                    crate::core::nt_core_ttc::EffortTier::XHigh => SynthesisEffortTier::XHigh,
                    crate::core::nt_core_ttc::EffortTier::Max => SynthesisEffortTier::Max,
                };
                // Gemini 3.6 step-route cache: reuse routing decision for repeated
                // (task_type, phase, effort, source_bucket) contexts across the seal loop.
                let cache_key = crate::core::nt_core_e8::nt_core_synthesis::StepRouteCache::key(
                    task_type as u8, phase_step as u8, synth_effort.rank(), current_mode,
                );
                let mut route_hit = false;
                // Fable 5 classifier-wrapped routing: high-risk contexts bypass
                // the aggressive frontier path through a conservative fallback.
                // Total route stats are needed on BOTH cache-hit and cache-miss:
                // the cache key carries no risk signal, so a routing cached for a
                // benign task in the same (task_type, phase, effort, bucket) would
                // otherwise be served unguarded to a high-risk request.
                let total_routes = self.synthesis.step_route_cache.hits + self.synthesis.step_route_cache.misses;
                let route_hits = self.synthesis.step_route_cache.hits;
                if self.synthesis.fused_pipeline_enabled {
                    if let Some(cached_topk) = self.synthesis.step_route_cache.get(&cache_key) {
                        route_hit = true;
                        root_span.set_attribute("synthesis_route_cache", AttributeValue::Bool(true));
                        // Blend cached top-k into the attention vector
                        let mut attn = vec![0.0f64; 64];
                        for (state, prob) in &cached_topk {
                            attn[(*state as usize).min(63)] = *prob;
                        }
                        // Safety classifier must run on cache hits too: re-gate the
                        // cached routing through the same frontier/conservative switch.
                        if self.synthesis.safety_router.allow_frontier(task, route_hits, total_routes) {
                            attn = self.synthesis.muon.condition_vector(&attn);
                            root_span.set_attribute("synthesis_safety", AttributeValue::Bool(true));
                        } else {
                            attn = self.synthesis.safety_router.conservative_distribution(&attn);
                            root_span.set_attribute("synthesis_safety", AttributeValue::Bool(false));
                            root_span.set_attribute("synthesis_safety_fallback", AttributeValue::Bool(true));
                        }
                        self.last_e8_attention_weights = Some(attn);
                        let capped_confidence = mcts_confidence.min(effort_tier.confidence_cap());
                        self.last_e8_confidence = capped_confidence;
                    } else {
                        // Fused pipeline: dominance cap + AttnRes + effort sparse top-K
                        let mut fused = self.synthesis.fused_distribution(tm, current_mode, &traj_modes, synth_effort);
                        if self.synthesis.safety_router.allow_frontier(task, route_hits, total_routes) {
                            // DeepSeek-V4 Muon: condition the fused flow so transition
                            // columns stay well-conditioned (no rank collapse). Applied
                            // as an 8×8 Newton-Schulz orthogonalization of the
                            // 64-state attention distribution.
                            fused = self.synthesis.muon.condition_vector(&fused);
                            root_span.set_attribute("synthesis_safety", AttributeValue::Bool(true));
                        } else {
                            // Conservative path: damp aggressive distribution toward uniform
                            fused = self.synthesis.safety_router.conservative_distribution(&fused);
                            root_span.set_attribute("synthesis_safety", AttributeValue::Bool(false));
                            root_span.set_attribute("synthesis_safety_fallback", AttributeValue::Bool(true));
                        }
                        let topk: Vec<(u8, f64)> = fused.iter().enumerate()
                            .filter(|(_, &p)| p > 1e-4)
                            .map(|(i, &p)| (i as u8, p))
                            .collect();
                        self.synthesis.step_route_cache.put(cache_key, topk);
                        self.last_e8_attention_weights = Some(fused);
                        let capped_confidence = mcts_confidence.min(effort_tier.confidence_cap());
                        self.last_e8_confidence = capped_confidence;
                    }
                } else {
                    // Fallback: plain effort-scaled sparse attention
                    let attn = dist.attention_weights_sparse(0.8, effort_tier.sparse_k());
                    self.last_e8_attention_weights = Some(attn.to_vec());
                    let capped_confidence = mcts_confidence.min(effort_tier.confidence_cap());
                    self.last_e8_confidence = capped_confidence;
                }

                root_span.set_attribute("synthesis_route_hit", AttributeValue::Bool(route_hit));
                root_span.set_attribute("synthesis_route_hit_rate", AttributeValue::Float(self.synthesis.step_route_cache.hit_rate()));
                root_span.set_attribute("synthesis_active", AttributeValue::Bool(self.synthesis.fused_pipeline_enabled));
                for (k, v) in self.synthesis.telemetry() {
                    root_span.set_attribute(&k, AttributeValue::String(v));
                }

                let attn_ref = self.last_e8_attention_weights.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
                root_span.set_attribute(
                    "e8_attn_entropy",
                    AttributeValue::Float(
                        attn_ref.iter().filter(|&&p| p > 0.0).map(|&p| -p * p.log(2.0)).sum::<f64>(),
                    ),
                );

                // Phase 10.1 — unified latent space: project the current E8 state
                // and the aggregated workspace into the shared space and surface
                // their cross-domain similarity for telemetry.
                if attn_ref.len() == 64 {
                    let e8_embed = self.unified_latent.project_e8(attn_ref);
                    let state_proj = self.unified_latent.project_e8_state(self.current_state.mode);
                    let cross = self.unified_latent.cosine(&e8_embed, &state_proj);
                    root_span.set_attribute("unified_e8_self_sim", AttributeValue::Float(cross));
                }

                // Phase 10.2 — latent reasoning: query episodic latent memory
                // for the current state's nearest neighbors (no text) and
                // broadcast the resulting direct E8 attention bias to GWT.
                let latent_retrieval = self.latent_reasoning.query_state(self.current_state.mode);
                if !latent_retrieval.neighbor_modes.is_empty() {
                    let (latent_weights, latent_bias) =
                        self.latent_reasoning.to_gwt_attention(&latent_retrieval, 0.2);
                    root_span.set_attribute(
                        "latent_retrieval_top_sim",
                        AttributeValue::Float(latent_retrieval.similarities.first().copied().unwrap_or(0.0)),
                    );
                    root_span.set_attribute(
                        "latent_memory_fill",
                        AttributeValue::Float(self.latent_reasoning.fill_ratio()),
                    );
                    // Merge latent episodic weights into the current-cycle fused
                    // attention so the GWT broadcast (which runs after the fusion
                    // pipeline) consumes them — previously they were set directly
                    // on the GWT *after* the broadcast already ran, i.e. dead output.
                    if let Some(attn) = self.last_e8_attention_weights.as_mut() {
                        if attn.len() == latent_weights.len() {
                            let lw = latent_bias.clamp(0.0, 0.5);
                            for (a, &l) in attn.iter_mut().zip(latent_weights.iter()) {
                                *a = *a * (1.0 - lw) + l * lw;
                            }
                            let s: f64 = attn.iter().sum();
                            if s > 0.0 {
                                for a in attn.iter_mut() {
                                    *a /= s;
                                }
                            }
                        }
                    }
                }

                // Phase 10.3 — multimodal fusion: encode the task as text (the
                // available modality in the reasoning loop) into the unified
                // latent space, route via GWT modal attention, and fuse.
                let multi_input = MultimodalInput::text(task);
                let multi_embeds = self.multimodal.encode_all(&multi_input);
                if !multi_embeds.is_empty() {
                    let router_weights: BTreeMap<crate::core::nt_core_gwt::modality_router::Modality, f64> = {
                        let mut m = BTreeMap::new();
                        if let Some(g) = &self.gwt {
                            for mod_i in crate::core::nt_core_gwt::modality_router::Modality::ALL {
                                m.insert(mod_i, g.modality_router.weight_of(mod_i));
                            }
                        } else {
                            m.insert(crate::core::nt_core_gwt::modality_router::Modality::Text, 1.0);
                        }
                        m
                    };
                    let (fused, weights) = self.multimodal.fuse(&router_weights, &multi_embeds);
                    let fused_mode = self.multimodal.to_e8_mode(&fused);
                    root_span.set_attribute(
                        "multimodal_fused_mode",
                        AttributeValue::Int(fused_mode as i64),
                    );
                    root_span.set_attribute(
                        "multimodal_active_modalities",
                        AttributeValue::Int(weights.len() as i64),
                    );
                    self.latent_reasoning.record(
                        crate::core::nt_core_hex::ReasoningHexagram::new(fused_mode),
                        self.last_e8_confidence,
                        "multimodal",
                    );
                }
            }
        }

        // GWT resonant broadcast: consciousness layer processes the reasoning state.
        // Runs AFTER the fusion pipeline so it consumes the *current* cycle's E8
        // prediction attention weights (previously it consumed the previous cycle's
        // — a one-cycle lag that biased GWT toward stale E8 predictions).
        let mut hex_states = [ReasoningHexagram::new(0); MODULE_COUNT];
        for (i, h) in hex_states.iter_mut().enumerate() {
            if i < self.state_trajectory.len() {
                *h = self.state_trajectory[i].mode;
            } else {
                *h = self.current_state.mode;
            }
        }
        let gwt_content = format!("E8 state: {} | task: {}", self.current_state.mode.mode_name(), task);
        // Phase 6.3 — sparse MoE routing: score the 8 expert groups (proximity +
        // task affinity + transition mass), pick top-2, freeze the other 6 before
        // the GWT bridge consumes them. Computed before the `gwt` borrow so the
        // router can be mutated while `gwt` is borrowed mutably.
        let sparse_moe_state = if self.last_e8_attention_weights.as_ref().is_some_and(|v| v.len() >= 64) {
            let task_ty = E8TaskType::detect(task);
            let next_block_mass = self.sparse_moe_next_block_mass();
            let routing = self.sparse_moe.route(self.current_state.mode.0, task_ty, next_block_mass.as_ref());
            let attn_arr = {
                let attn_vec = self.last_e8_attention_weights.as_ref().unwrap();
                let mut arr = [0.0_f64; 64];
                arr.copy_from_slice(&attn_vec[..64]);
                self.sparse_moe.apply_mask(&routing, &arr)
            };
            Some((routing, attn_arr))
        } else {
            None
        };
        if let Some(ref mut gwt) = self.gwt {
            if let Some((routing, attn_arr)) = sparse_moe_state {
                let adaptive_bias = 0.1 + self.last_e8_confidence * 0.4;
                gwt.set_e8_attention_weights(attn_arr, adaptive_bias);
                root_span.set_attribute(
                    "sparse_moe_active",
                    AttributeValue::String(format!("{:?}", routing.active_groups).into()),
                );
                root_span.set_attribute(
                    "sparse_moe_sparsity",
                    AttributeValue::Float(routing.sparsity() as f64),
                );
            }
            let report = gwt.resonant_broadcast(&gwt_content, &hex_states);
            root_span.set_attribute("gwt_winner", AttributeValue::Int(report.winner as i64));
            root_span.set_attribute("gwt_entropy", AttributeValue::Float(report.entropy));
            root_span.set_attribute("gwt_complement_activated", AttributeValue::Bool(report.complement_activated));
            root_span.set_attribute("gwt_inner_speech", AttributeValue::Bool(gwt.inner_speech.total_generated > 0));
        }

        if let Some(ref compressor) = self.trajectory_compressor {
            let orig_len = self.state_trajectory.len();
            self.state_trajectory = compressor.compress_state_trajectory(&self.state_trajectory);
            let saved = orig_len.saturating_sub(self.state_trajectory.len());
            if saved > 0 {
                root_span.set_attribute("trajectory_compressed_states", AttributeValue::Int(saved as i64));
            }
        }

        if let Some(ref kb) = self.kb {
            if self.state_trajectory.len() >= 2 {
                let tips: Vec<serde_json::Value> = self.state_trajectory.windows(2).enumerate().filter_map(|(i, w)| {
                    if w[0].mode == w[1].mode {
                        Some(serde_json::json!({
                            "tip_type": "Optimization",
                            "source_step_idx": i,
                            "pattern": format!("Mode {:?} repeated consecutively", w[0].mode.mode_name()),
                            "recommendation": "Consolidate consecutive same-mode states".to_string(),
                            "confidence": 0.5,
                            "provenance": format!("engine: state {} -> {}", i, i+1),
                        }))
                    } else {
                        None
                    }
                }).collect();
                let report = serde_json::json!({
                    "tips": tips,
                    "trajectory_id": format!("reason-{}", task.len().min(16)),
                    "total_steps": self.state_trajectory.len(),
                    "success": result.is_ok(),
                });
                if let Err(e) = kb.store_learning_report(&report) {
                    root_span.set_attribute("kb_learn_error", AttributeValue::String(e));
                } else {
                    root_span.set_attribute("kb_learn_stored", AttributeValue::Bool(true));
                }
            }
        }

        // EWHR auto-invoke: analyze trajectory and propose hypothesis descriptions
        if let Some(ref bridge) = self.ewhr_bridge {
            let proposed = bridge.analyze_trajectory(&self.state_trajectory, task);
            if !proposed.is_empty() {
                root_span.set_attribute("ewhr_hypotheses_proposed", AttributeValue::Int(proposed.len() as i64));
                // Hydrate into HypothesisNetwork if available
                if let Some(ref net_lock) = self.hypothesis_network {
                    if let Ok(mut net) = net_lock.lock() {
                        let added = hydrate_ewhr_hypotheses(&mut net, &proposed, self.state_trajectory.len());
                        if added > 0 {
                            root_span.set_attribute("ewhr_hypotheses_hydrated", AttributeValue::Int(added as i64));
                        }
                    }
                }
            }
        }

        if let Some(ref t) = self.tracer { t.end_span(root_span); } else { NoopTracer.end_span(root_span); }

        // F6 closed loop (周天大阵运转): distill the successful trace into
        // alternating sequences and periodically run SFT + CSPO to update the
        // E8 policy, so reasoning controls learned from real runs feed back
        // into subsequent reason() calls via PRM (R-P79 production wiring).
        if let Ok(response) = &result {
            self.learn_from_trace(task, response);
        }

        self.core_review(task, &result);

        // Return watermarked response if anti-distillation was active, else raw response
        if let Some(watermarked) = self._last_watermarked.take() {
            Ok(watermarked)
        } else {
            result
        }
    }

    /// Auto-record conversation metadata on every reason() call.
    /// Stores task, outcome, E8 mode, specialist winner, error count into KB.
    pub fn core_review(&mut self, task: &str, result: &NeoTrixResult<String>) {
        let (outcome, error_ctx) = match result {
            Ok(_) => ("success", None),
            Err(e) => ("error", Some(format!("{}", e))),
        };
        self.record_trace(
            ReasoningType::Conversation,
            task, "", "",
            error_ctx.as_deref(),
            if result.is_ok() { 1.0 } else { 0.0 },
        );
        let error_count = if result.is_err() { 1 } else { 0 };
        let e8_mode = self.current_state.mode.mode_name().to_string();
        let specialist = self.gwt.as_ref()
            .and_then(|g| g.last_resonance.as_ref())
            .map(|r| r.winner.to_string())
            .unwrap_or_default();
        if let Some(ref kb) = self.kb {
            use crate::neotrix::nt_memory_kb::nt_memory_types::ConversationRecord;
            let record = ConversationRecord {
                id: format!("conv-{}", self.llm_call_count),
                session_id: String::new(),
                task_description: task.to_string(),
                user_intent: task.to_string(),
                strategy_used: format!("e8_mode:{}", e8_mode),
                e8_mode,
                specialist_winner: specialist,
                actions_taken: Vec::new(),
                obstacles_encountered: Vec::new(),
                fix_patterns: Vec::new(),
                outcome: outcome.to_string(),
                effectiveness: if result.is_ok() { 1.0 } else { 0.0 },
                reasoning_iterations: self.state_trajectory.len() as u32,
                error_count,
                timestamp: chrono::Utc::now().timestamp(),
            };
            let _ = kb.store_conversation_record(&record);
        }
    }

    /// Compute next-block probability mass for the 8 MoE expert groups from the
    /// domain-aware transition model's general matrix, used as the transition
    /// signal in sparse MoE routing. Returns `None` when no transition data exists.
    fn sparse_moe_next_block_mass(&self) -> Option<[f64; 8]> {
        let current = self.current_state.mode.0;
        let mut mass = [0.0f64; 8];
        if let Some(ref model) = self.domain_transition_model {
            let dist = model.general_matrix.dominance_capped_distribution(current, 0.5);
            let total: f64 = dist.iter().sum();
            if total > 0.0 {
                for (t, p) in dist.iter().enumerate() {
                    mass[t / 8] += p / total;
                }
                return Some(mass);
            }
        }
        None
    }

    pub fn reason_multi_agent(&mut self, task: &str) -> NeoTrixResult<String> {
        if let Some(ref orch) = self.orchestrator {
            match orch.execute(task) {
                Ok(output) => Ok(output.content),
                Err(e) => Err(NeoTrixError::Brain(format!("Orchestration failed: {}", e))),
            }
        } else {
            self.reason(task)
        }
    }

    pub fn reason_task(&mut self, task: &str) -> NeoTrixResult<String> { self.reason(task) }
    pub fn plan_reasoning(&self, _task: &str, _mode: u8) -> String { String::new() }
    pub fn with_jepa<T>(self, _jepa: T) -> Self { self }
    pub fn self_iterate(&mut self) {
        // Run observer analysis to monitor reasoning state health
        self.observer_analyze("self-iteration");
        // Record self-iteration through core_review
        let result = self.call_llm("self-iteration: analyze current state and propose improvements");
        self.core_review("self-iteration", &result);
        // Feed back to PRM for learning signal
        if let Some(ref mut prm) = self.prm {
            let score = if result.is_ok() { 0.5 } else { 0.0 };
            prm.learn_step(|collector| {
                collector.begin("self-iteration".to_string());
                collector.record_step(
                    crate::core::nt_core_traits::SpecialistType::ReflectionEngine,
                    self.current_state.mode,
                    "self_iterate".into(),
                    String::new(),
                    result.as_deref().unwrap_or("error").to_string(),
                    None,
                    true,
                    Some(score),
                );
                collector.finish(Some(score), result.is_ok());
            });
        }
        log::info!("[engine] Self-iteration cycle completed");
    }

    pub fn select_mode(&self, _query: &str) -> FullReasoningState {
        self.current_state
    }

    pub fn current_state_string(&self) -> String {
        let hex = self.current_state.mode;
        format!("{}: {}", hex.mode_name(), hex.mode_description())
    }

    pub fn save_e8_state(&self, path: &std::path::Path) -> Result<(), String> {
        let (e8_policy, prm_learning_count, prm_score_history) = self.prm.as_ref().map_or(
            (None, 0u64, Vec::new()),
            |prm| {
                let history: Vec<f64> = prm.score_history.iter().rev().take(100).cloned().collect();
                (Some(prm.policy.clone()), prm.learning_count, history)
            },
        );
        let state = E8PersistedState {
            current_mode: self.current_state.mode.0,
            current_meta: self.current_state.meta.0,
            last_e8_attention_weights: self.last_e8_attention_weights.clone(),
            last_e8_confidence: self.last_e8_confidence,
            trajectory_modes: self.state_trajectory.iter().map(|s| (s.mode.0, s.meta.0)).collect(),
            e8_policy,
            prm_learning_count,
            prm_score_history,
        };
        let json = serde_json::to_string_pretty(&state).map_err(|e| format!("serialize: {}", e))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {}", e))?;
        }
        std::fs::write(path, &json).map_err(|e| format!("write: {}", e))?;
        Ok(())
    }
    pub fn load_e8_state(&mut self, path: &std::path::Path) -> Result<(), String> {
        if !path.exists() {
            return Ok(());
        }
        let json = std::fs::read_to_string(path).map_err(|e| format!("read: {}", e))?;
        let state: E8PersistedState = serde_json::from_str(&json).map_err(|e| format!("deserialize: {}", e))?;
        self.current_state = FullReasoningState::new(
            ReasoningHexagram::new(state.current_mode.min(63)),
            crate::core::nt_core_hex::MetaState::new(state.current_meta),
        );
        self.last_e8_attention_weights = state.last_e8_attention_weights;
        self.last_e8_confidence = state.last_e8_confidence;
        self.state_trajectory = state.trajectory_modes.into_iter().map(|(mode, meta)| {
            FullReasoningState::new(
                ReasoningHexagram::new(mode.min(63)),
                crate::core::nt_core_hex::MetaState::new(meta),
            )
        }).collect();
        if let Some(policy) = state.e8_policy {
            if let Some(ref mut prm) = self.prm {
                prm.policy = policy;
                prm.learning_count = state.prm_learning_count;
                prm.score_history = state.prm_score_history;
            }
        }
        Ok(())
    }

    pub fn build_context(&self, query: &str, _rtype: ReasoningType) -> String {
        if let Some(ref kb) = self.kb {
            if let Ok(results) = kb.search(query, 3) {
                if !results.is_empty() {
                    let mut ctx = format!("Past experiences relevant to \"{}\":\n", query);
                    for r in &results {
                        ctx.push_str(&format!(
                            "- {}: {}\n",
                            r.node.title,
                            r.node.summary.as_deref().unwrap_or("(no summary)")
                        ));
                    }
                    return ctx;
                }
            }
        }
        format!("Reasoning task: {}", query)
    }
    pub fn build_artifact_context(&self, query: &str) -> String {
        if let Some(ref indexer) = self.artifact_indexer {
            let artifacts = indexer.store().search_keyword(query);
            if !artifacts.is_empty() {
                let mut ctx = String::from("Relevant artifacts:\n");
                for a in artifacts.iter().take(5) {
                    ctx.push_str(&format!("- {}: {} (tags: {:?})\n", a.name, a.content.chars().take(80).collect::<String>(), a.tags));
                }
                return ctx;
            }
        }
        String::new()
    }

    pub fn call_llm(&mut self, prompt: &str) -> NeoTrixResult<String> {
        if let Some(ref gateway) = self.gateway {
            let mut request = LlmRequest::new(&self.default_model, prompt);
            if let Some(tier) = self.last_effort_tier {
                let think = tier.thinking_budget_tokens();
                let max_tok = tier.max_tokens_budget();
                request.max_tokens = max_tok;
                request.thinking_budget = Some(think);
                if think > 0 {
                    if let Some(msg) = request.messages.first_mut() {
                        msg.content = format!(
                            "{}\n\n[budget] Reason within {} thinking tokens; answer within {} tokens.",
                            msg.content, think, max_tok
                        );
                    }
                }
            }
            let gateway_ref = gateway.clone();
            let response = tokio::task::block_in_place(|| {
                let handle = tokio::runtime::Handle::current();
                handle.block_on(gateway_ref.complete(&request))
            })
            .map_err(|e| NeoTrixError::Brain(format!("LLM call failed: {}", e)))?;
            let prompt_tokens = response.usage.prompt_tokens;
            let completion_tokens = response.usage.completion_tokens;
            if let Some(ref mut ct) = self.cost_tracker {
                ct.record(&self.default_model, prompt_tokens as u64, completion_tokens as u64);
            }
            Ok(response.content)
        } else {
            Err(NeoTrixError::Brain("No LLM provider configured".into()))
        }
    }

    pub fn call_llm_with_ctx(&mut self, ctx: &str, prompt: &str) -> NeoTrixResult<String> {
        self.call_llm(&format!("{}\n\n{}", ctx, prompt))
    }

    pub fn record_trace(
        &mut self,
        rt: ReasoningType,
        task: &str,
        prompt: &str,
        response: &str,
        error_info: Option<&str>,
        reward: f64,
    ) {
        let trace = ReasoningTrace {
            id: format!("trace-{}", self.llm_call_count),
            reasoning_type: rt,
            reasoning_method: None,
            perspective_lens: None,
            task: task.to_string(),
            prompt: prompt.to_string(),
            llm_response: response.to_string(),
            error_context: error_info.map(String::from),
            outcome_score: reward,
            success: error_info.is_none(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        if self.traces.len() >= MAX_TRACES {
            self.traces.remove(0);
        }
        self.traces.push(trace);
    }

    pub fn learn_from_trace(&mut self, task: &str, response: &str) {
        if let Some(ref mut prm) = self.prm {
            let _task_type = E8TaskType::detect(task);
            let substantive = if response.len() > 100 { 0.8 } else { 0.3 };
            let tier_credit = match self.last_effort_tier {
                Some(EffortTier::Low) => 1.0,
                Some(EffortTier::Medium) => 0.9,
                Some(EffortTier::High) => 0.8,
                Some(EffortTier::XHigh) => 0.7,
                Some(EffortTier::Max) => 0.6,
                None => 0.85,
            };
            let step_reward = substantive * tier_credit;
            prm.learn_step(|collector| {
                collector.begin(task.to_string());
                collector.record_step(
                    crate::core::nt_core_traits::SpecialistType::ReflectionEngine,
                    self.current_state.mode,
                    "learn_from_trace".into(),
                    task.to_string(),
                    response.to_string(),
                    None,
                    true,
                    Some(step_reward),
                );
                collector.finish(Some(step_reward), true);
            });
        }
        // F6 wiring: distill control segments from the trace for CSPO training (R-P36 behavioral grounding)
        if let Some(seq) = self.distill_trace(task, response) {
            log::debug!("[control-distill] distilled {} segments (quality={:.3})", seq.segments.len(), seq.outcome_quality);
        }
        // F6 closed loop: once a batch of alternating sequences has accumulated,
        // consume them via SFT + CSPO and write the updated policy back into PRM.
        // Throttled here (in addition to reason()) so training also runs on the
        // offline learn_from_trace path.
        if self.train_batch >= CONTROL_TRAIN_BATCH {
            if let Some((sft, csppo)) = self.train_from_distilled() {
                log::debug!(
                    "[control-train] SFT(c={},r={}) CSPO(reward={:.3},masked={})",
                    sft.control_updates,
                    sft.reason_updates,
                    csppo.total_control_reward,
                    csppo.masked_steps,
                );
            }
        }
    }

    /// 把单条推理 response 蒸馏为交替序列 (Reason ↔ Control)，供 CSPO/SFT 训练消费。
    /// 按换行/句读切分步骤；无法解析时返回 None (失败静默，不影响主推理路径)。
    pub fn distill_trace(&mut self, task: &str, response: &str) -> Option<AlternatingSequence> {
        let distiller = self.control_distiller.as_ref()?;
        let steps = split_response_into_steps(response);
        if steps.is_empty() {
            return None;
        }
        let id = format!("distill_{}_{}", self.traces.len(), chrono::Utc::now().timestamp());
        let seq = distiller.extract_alternating_sequence(id, task, response, &steps, response).ok()?;
        if self.distilled_sequences.len() >= MAX_TRACES {
            self.distilled_sequences.remove(0);
        }
        self.distilled_sequences.push(seq.clone());
        self.train_batch += 1;
        Some(seq)
    }

    /// F6 训练闭环：消费蒸馏序列，SFT + CSPO 更新 E8 policy。
    ///
    /// 从当前 `prm.policy` 克隆构造临时训练器（单策略权威，避免平行状态，
    /// R-P42），运行 SFT（阶段 1）+ CSPO（阶段 2）后写回 `prm.policy`。
    /// 节流由 `reason()` 主流程按 `train_batch` 阈值触发。
    pub fn train_from_distilled(&mut self) -> Option<(SftReport, CsppoReport)> {
        if self.distilled_sequences.is_empty() || self.prm.is_none() {
            return None;
        }
        let seqs = std::mem::take(&mut self.distilled_sequences);
        let policy = self.prm.as_ref().map(|p| p.policy.clone())?;
        let gold = Arc::new(ConsciousnessGoldStandard::new());
        let mut trainer = ControlTrainer::new(policy, gold);
        let sft = trainer.sft(&seqs).ok()?;
        let csppo = trainer.csppo(&seqs).ok()?;
        if let Some(ref mut prm) = self.prm {
            prm.policy = trainer.policy.clone();
            prm.learning_count += 1;
        }
        self.train_batch = 0;
        log::info!(
            "[control-train] batch={} sft(control={},reason={}) csppo(reward={:.3},masked={})",
            seqs.len(),
            sft.control_updates,
            sft.reason_updates,
            csppo.total_control_reward,
            csppo.masked_steps,
        );
        Some((sft, csppo))
    }

    pub fn observer_analyze(&mut self, task: &str) {
        // Use error recovery to wrap the observer analysis with retry + circuit breaker + fallback
        let report = self.observer_error_recovery.execute(|| {
            Ok(self.observer.analyze(&self.state_trajectory, &[task]))
        }).unwrap_or_else(|_| {
            log::warn!("[observer] Error recovery exhausted, using degraded report");
            self.observer.analyze(&self.state_trajectory, &[task])
        });
        if report.has_actionable_insight {
            if let Some(ref mut prm) = self.prm {
                let bonus = report.quality_score * 0.1;
                let current_mode = self.current_state.mode.0 as usize;
                prm.policy.mode_values[current_mode.min(63)] = (prm.policy.mode_values[current_mode.min(63)] + bonus).min(1.0);
            }
            // Feed richer observer data to E8 policy: step attention weights, convergence
            if let Some(ref mut prm) = self.prm {
                if let Some(ref attn) = report.step_attention {
                    for (mode_idx, weight) in attn.iter().enumerate() {
                        if mode_idx < prm.policy.mode_values.len() {
                            prm.policy.mode_values[mode_idx] = (prm.policy.mode_values[mode_idx] + weight * 0.05).min(1.0);
                        }
                    }
                }
            }
        }
        // Log critical patterns
        for cp in &report.critical_patterns {
            log::info!("[observer] Critical pattern detected: {}", cp);
        }
    }

    pub fn infer_reasoning_type(task: &str) -> ReasoningType {
        let lower = task.to_lowercase();
        let math_keywords = ["solve", "calculate", "compute", "equation", "math", "algebra", "calculus", "derivative", "integral"];
        let coding_keywords = ["implement", "function", "bug", "test", "code", "compile", "refactor", "debug", "api", "class", "struct"];
        let reasoning_keywords = ["why", "explain", "analyze", "compare", "reason", "evaluate", "hypothesis", "infer", "deduce"];
        let knowledge_keywords = ["what is", "define", "meaning of", "tell me about", "information on", "search for"];
        for k in math_keywords { if lower.contains(k) { return ReasoningType::TaskSolving; } }
        for k in coding_keywords { if lower.contains(k) { return ReasoningType::General; } }
        for k in reasoning_keywords { if lower.contains(k) { return ReasoningType::TaskSolving; } }
        for k in knowledge_keywords { if lower.contains(k) { return ReasoningType::KnowledgeQuery; } }
        ReasoningType::Conversation
    }

    /// Stream the reasoning response token-by-token through a channel.
    /// Returns the full response alongside a receiver for streaming.
    pub async fn reason_stream(
        &mut self,
        task: &str,
        _budget: Option<u32>,
    ) -> NeoTrixResult<(String, tokio::sync::mpsc::Receiver<String>)> {
        let response = self.reason(task)?;
        let (tx, rx) = tokio::sync::mpsc::channel(64);
        let response_clone = response.clone();
        tokio::spawn(async move {
            for word in response_clone.split(' ') {
                if tx.send(format!("{} ", word)).await.is_err() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        });
        Ok((response, rx))
    }
}

/// EWHR 提议 → HypothesisNetwork 落点。
///
/// 将 `analyze_trajectory` 返回的候选假说字符串 (Vec<String>) 转化为
/// HypothesisNetwork 节点。幂等: 同一 (trajectory_len, index) 的 id 已存在则跳过,
/// 避免每轮重复落点。返回实际新增数量。
fn hydrate_ewhr_hypotheses(
    net: &mut crate::neotrix::nt_memory_historian::nt_evidence_hypothesis::HypothesisNetwork,
    proposed: &[String],
    tick: usize,
) -> usize {
    let mut added = 0usize;
    for (i, proposal) in proposed.iter().enumerate() {
        let id = format!("ewhr_{}_{}", tick, i);
        if net.get_hypothesis(&id).is_some() {
            continue;
        }
        let title = if proposal.chars().count() > 32 {
            let truncated: String = proposal.chars().take(30).collect();
            format!("{}…", truncated)
        } else {
            proposal.clone()
        };
        net.propose_hypothesis(&id, &title, proposal, 0.5);
        added += 1;
    }
    added
}

/// Detect if an LLM response is a refusal (empty, apology, or explicit refusal patterns).
pub fn detect_refusal_response(response: &str) -> bool {
    let trimmed = response.trim();
    if trimmed.is_empty() || trimmed.len() < 5 {
        return true;
    }
    let lower = trimmed.to_lowercase();
    // Common refusal/apology patterns
    let patterns = [
        "i cannot",
        "i can't",
        "i'm sorry",
        "i am sorry",
        "i apologize",
        "i'm not able",
        "i am not able",
        "i'm unable",
        "i am unable",
        "i cannot fulfill",
        "i can't fulfill",
        "cannot provide",
        "can't provide",
        "cannot assist",
        "can't assist",
        "not allowed to",
        "against my policy",
        "against my guidelines",
        "i don't feel comfortable",
        "sorry, but i cannot",
        "sorry, i cannot",
        "as an ai",
        "as an ai language model",
        "i'm designed to",
        "i was designed to",
    ];
    patterns.iter().any(|p| lower.contains(p))
}

/// 把推理 response 文本切分为步骤序列，供 ControlDistiller 检测 takeover 点。
/// 按换行分段；若不足 2 段则按句号/分号切分。每步携带近似 token 数。
pub fn split_response_into_steps(response: &str) -> Vec<ReasoningStep> {
    let mut segments: Vec<String> = response
        .split('\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if segments.len() < 2 {
        segments = response
            .split(|c| c == '.' || c == ';' || c == '。' || c == '；')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
    }
    segments
        .iter()
        .enumerate()
        .map(|(i, text)| ReasoningStep {
            step_idx: i,
            text: text.clone(),
            e8_mode: None,
            token_count: text.len() / 4,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_refusal_response_empty() {
        assert!(detect_refusal_response(""));
        assert!(detect_refusal_response("   "));
        assert!(detect_refusal_response("no"));
    }

    #[test]
    fn test_detect_refusal_response_explicit() {
        assert!(detect_refusal_response("I cannot fulfill that request."));
        assert!(detect_refusal_response("Sorry, but I cannot help with this."));
        assert!(detect_refusal_response("I'm sorry, I cannot provide that information."));
        assert!(detect_refusal_response("As an AI language model, I cannot do that."));
        assert!(detect_refusal_response("I'm not able to assist with this request."));
    }

    #[test]
    fn test_detect_refusal_response_normal() {
        assert!(!detect_refusal_response("Here is a detailed analysis of your code..."));
        assert!(!detect_refusal_response("The answer to your question is..."));
        assert!(!detect_refusal_response("Let me help you with that."));
        assert!(!detect_refusal_response("Here is the implementation:"));
    }

    #[test]
    fn test_hydrate_ewhr_hypotheses_adds_nodes() {
        use crate::neotrix::nt_memory_historian::nt_evidence_hypothesis::HypothesisNetwork;
        let mut net = HypothesisNetwork::new();
        let proposed = vec![
            "agent should adopt Direct strategy when context is high".to_string(),
            "error recovery benefits from rollback-first policy".to_string(),
        ];
        let added = hydrate_ewhr_hypotheses(&mut net, &proposed, 42);
        assert_eq!(added, 2);
        assert_eq!(net.hypotheses.len(), 2);
        assert!(net.get_hypothesis("ewhr_42_0").is_some());
        assert!(net.get_hypothesis("ewhr_42_1").is_some());
    }

    #[test]
    fn test_hydrate_ewhr_hypotheses_idempotent() {
        use crate::neotrix::nt_memory_historian::nt_evidence_hypothesis::HypothesisNetwork;
        let mut net = HypothesisNetwork::new();
        let proposed = vec!["same proposal repeated".to_string()];
        // 同一 tick 两次调用 → 第二次不重复落点
        let added1 = hydrate_ewhr_hypotheses(&mut net, &proposed, 7);
        let added2 = hydrate_ewhr_hypotheses(&mut net, &proposed, 7);
        assert_eq!(added1, 1);
        assert_eq!(added2, 0, "same tick must not duplicate hypotheses");
        assert_eq!(net.hypotheses.len(), 1);
        // 不同 tick → 允许新增 (新一轮轨迹)
        let added3 = hydrate_ewhr_hypotheses(&mut net, &proposed, 8);
        assert_eq!(added3, 1);
        assert_eq!(net.hypotheses.len(), 2);
    }

    #[test]
    fn test_hydrate_ewhr_hypotheses_long_title_truncated() {
        use crate::neotrix::nt_memory_historian::nt_evidence_hypothesis::HypothesisNetwork;
        let mut net = HypothesisNetwork::new();
        let long = "a very long hypothesis description that definitely exceeds the thirty two character title limit for display purposes".to_string();
        let added = hydrate_ewhr_hypotheses(&mut net, &[long.clone()], 1);
        assert_eq!(added, 1);
        let h = net.get_hypothesis("ewhr_1_0").expect("node exists");
        assert!(h.title.len() <= 33, "title must be truncated to 32+ellipsis, got len {}", h.title.len());
    }

    #[test]
    fn test_split_response_into_steps_by_newline() {
        let steps = split_response_into_steps("First step\nSecond step\nThird step");
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0].text, "First step");
        assert_eq!(steps[1].step_idx, 1);
        assert!(steps[0].token_count > 0);
    }

    #[test]
    fn test_split_response_into_steps_by_sentence() {
        let steps = split_response_into_steps("No newlines. Only sentences here.");
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].text, "No newlines");
    }

    #[test]
    fn test_split_response_into_steps_empty() {
        assert!(split_response_into_steps("").is_empty());
        assert!(split_response_into_steps("   \n  ").is_empty());
    }

    #[test]
    fn test_distill_trace_wires_control_distiller() {
        let mut engine = ReasoningEngine::from_env();
        let response = "First we compute the sum.\nWait, rethink.\nThen we verify.";
        engine.learn_from_trace("math", response);
        assert_eq!(engine.distilled_sequences.len(), 1, "distillation must run via learn_from_trace (R-P36 grounding)");
        let seq = &engine.distilled_sequences[0];
        assert!(!seq.segments.is_empty(), "alternating sequence must contain segments");
    }

    #[test]
    fn test_train_from_distilled_closes_loop() {
        use crate::core::nt_core_prm::ProcessRewardLearner;
        use crate::core::nt_core_policy::E8Policy;
        let mut engine = ReasoningEngine::from_env();
        let prm = ProcessRewardLearner::new(E8Policy::default(), Box::new(crate::core::nt_core_prm::HeuristicCoach::new("test")));
        engine = engine.with_prm(prm);

        // Accumulate CONTROL_TRAIN_BATCH distilled sequences with takeover markers
        let responses = [
            "Compute the integral.\nWait, reconsider the bounds.\nThen verify the result.",
            "Solve the equation.\nActually, switch to substitution.\nCheck the algebra.",
            "Derive the formula.\nHmm, backtrack to the derivative.\nValidate step by step.",
            "Factor the polynomial.\nAlternatively use grouping.\nConfirm each factor.",
            "Simplify the fraction.\nOn second thought, use common denominator.\nVerify the simplification.",
            "Evaluate the limit.\nWait, apply L'Hopital.\nThen check continuity.",
            "Prove the theorem.\nLet me rethink the induction base.\nValidate the inductive step.",
            "Compute the determinant.\nActually, expand along the first row.\nVerify the arithmetic.",
        ];
        for r in responses {
            engine.learn_from_trace("math", r);
        }

        // Batch threshold reached → training must have run and drained sequences
        assert_eq!(engine.train_batch, 0, "train_batch must reset after training");
        assert!(engine.distilled_sequences.len() < responses.len(), "training must consume distilled sequences");
        if let Some(ref prm) = engine.prm {
            assert!(prm.learning_count >= 1, "PRM learning_count must advance after training");
        } else {
            panic!("PRM must be configured");
        }
    }
}
