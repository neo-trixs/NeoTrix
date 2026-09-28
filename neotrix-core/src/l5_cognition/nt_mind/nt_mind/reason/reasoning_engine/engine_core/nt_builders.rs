use std::collections::{ HashMap};
use std::sync::{Arc, Mutex};

use crate::l5_cognition::nt_core::capability::nt_core_antidistil::AntiDistillationSystem;
use crate::l5_cognition::nt_core_aura::IntentEngine;
use crate::l1_action::nt_core_bank::ReasoningBank;
use crate::l2_perception::nt_core_e8::domain_transition::{ E8DomainTransitionModel};
use crate::l2_perception::nt_core_e8::ewhr_bridge::E8EwhrBridge;
use crate::l2_perception::nt_core_e8::nt_core_e8_prediction::E8PredictionOracle;
use crate::l2_perception::nt_core_e8::nt_core_fable_pattern::{FablePatternMatcher};
use crate::l2_perception::nt_core_e8::nt_core_synthesis::{ConsciousnessCoreSynthesis};
use crate::l2_perception::nt_core_e8::nt_latent_reasoning::LatentReasoningPipeline;
use crate::l2_perception::nt_core_e8::nt_latent_transformer::LatentReasoningTransformer;
use crate::l2_perception::nt_core_e8::nt_multimodal::{MultimodalEncoder};
use crate::l2_perception::nt_core_e8::sparse_moe::SparseMoERouter;
use crate::l2_perception::nt_core_e8::unified_latent::UnifiedLatentSpace;
use crate::l5_cognition::nt_core_prm::ProcessRewardLearner;
use crate::l5_cognition::nt_core_sae_bridge::SAEBridge;
use crate::l5_cognition::nt_core_trajectory_compress::{CompressionLevel, TrajectoryCompressor};
use crate::l5_cognition::nt_core_ttc::{EffortTier, EffortTierSelector, TtcEngine};
use crate::l5_cognition::nt_mind::nt_mind::knowledge::context_artifacts::indexer::ArtifactIndexer;

use crate::l5_cognition::nt_core::capability::nt_act_orch_patterns::Orchestrator;
use crate::l5_cognition::nt_core_gwt::workspace::GlobalWorkspace;
use crate::l0_substrate::nt_core_hex::{FullReasoningState, ReasoningHexagram};
use crate::l6_meta::nt_core_observer::OneObserver;
use crate::l6_meta::nt_core_observer_error::ObserverErrorRecovery;
use crate::l5_cognition::l1_facade::silicon_self::SiliconSelfModel;
use crate::l0_substrate::nt_core_span::{ ConsoleTracer, CostTracker,
};
use crate::l5_cognition::l1_facade::ConsciousnessGoldStandard;
use crate::l5_cognition::l1_facade::{KnowledgeBase};
use crate::l5_cognition::nt_mind::nt_mind::control_distillation::{
    AlternatingSequence, ControlDistiller,
};
use crate::l5_cognition::nt_mind::nt_mind::core::BrainMutView;
use crate::l5_cognition::nt_mind::nt_mind::distillation::{AntiPattern, StrategicPrinciple};
use crate::l5_cognition::nt_mind::nt_mind::reasoning_types::{ReasoningRecord};
use crate::l5_cognition::nt_mind::nt_mind::seal_core::model_router::ModelRouter;
use crate::l2_perception::nt_world::nt_world_jepa::JepaWorldModel;
use super::super::CognitiveEye;
use crate::l1_action::nt_io::nt_io_provider::{ LlmProvider};

pub const MAX_COST_LOG: usize = 1000;
pub const MAX_TRACES: usize = 1000;
/// F6 训练节流: 累积多少条交替序列后触发一次 SFT + CSPO 训练。
pub const CONTROL_TRAIN_BATCH: usize = 8;
/// P0-6 会话内 KB 检索缓存容量上限: 超限时整体清空 (防膨胀, 保最旧语义)。
pub const MAX_KB_CACHE_ENTRIES: usize = 32;
/// P0-6 注入预算封顶: build_context 检索上下文拼进 prompt 前的 token 估算上限。
pub const MAX_KB_INJECTION_TOKENS: usize = 512;

pub struct CostRecord {
    pub tier: String,
    pub cost_estimate_usd: f64,
    pub duration_ms: u64,
    pub timestamp: i64,
}
pub type ReasoningStats = (usize, u64, f64);
pub type EngineMetrics = (usize, u64, u64, f64);

pub struct ReasoningEngine {
    pub current_state: FullReasoningState,
    pub state_trajectory: Vec<FullReasoningState>,
    pub strategy_matrix: [[ReasoningHexagram; 8]; 8],
    pub observer: OneObserver,
    pub distill_interval: usize,
    pub last_core_plan: Option<String>,
    pub brain: Box<dyn BrainMutView>,
    pub bank: ReasoningBank,
    pub traces: Vec<ReasoningRecord>,
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
    /// P0-6 会话内 KB 检索缓存: key 为 `s:{query 前缀}|{rtype}` (search 路径) 或
    /// `b:{task 前缀}` (ContextBuilder/回退路径), value 为已拼好的注入上下文,
    /// 避免同一 session 内相同/近似 query 每轮重搜重注入。容量上限 MAX_KB_CACHE_ENTRIES。
    pub kb_cache: Mutex<HashMap<String, String>>,
    /// P2-E2 相邻轮次注入去重: 上一轮已注入 KB 的 node id 集合。
    /// 新搜索若命中相同 node, 只注入增量差异 (新 node), 避免相邻轮次重复注入。
    pub last_kb_injected: Mutex<Vec<String>>,
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
    /// Verifier: grounded PRM verifier for step-level verification (Phase 2.2)
    pub verifier: Option<crate::l5_cognition::nt_core_prm::GroundedPrmVerifier>,
    /// ContextBuilder: KB/经验 → Kernel context 自动注入 (Phase 1.3)
    pub context_builder: Option<crate::l0_substrate::nt_core_answer_engine::ContextBuilder>,
    /// CoT Generator: Kernel 结构化推理 → 自然语言 CoT (Phase 2.1)
    pub cot_generator: Option<crate::l5_cognition::nt_core_cot_generator::DefaultCoTGenerator>,
    /// E8Policy: RL policy for reasoning mode selection (Phase 2.3)
    pub e8_policy: Option<crate::l5_cognition::nt_core_policy::E8Policy>,
    /// Intent engine: tracks user/agent intent through reasoning
    pub intent_engine: Option<IntentEngine>,
    /// Hypothesis network: shared with EWHR REST API
    pub hypothesis_network: Option<
        Arc<Mutex<crate::l4_emotion::nt_memory::nt_memory_historian::nt_evidence_hypothesis::HypothesisNetwork>>,
    >,
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
    /// Phase 10.1 — unified latent space bridging E₈ / GWT / HyperCube.
    pub unified_latent: UnifiedLatentSpace,
    /// Phase 10.2 — end-to-end latent reasoning: E8 latent → hypercube query →
    /// GWT broadcast with no intermediate text.
    pub latent_reasoning: LatentReasoningPipeline,
    /// Phase 6.3 — recursive latent reasoning transformer: iterates the fused
    /// attention vector in a continuous latent space, accumulating depth-scaling
    /// reward (Thinking Pixel §3.3). Wired into the reason hot path so the
    /// trajectory advances and reward folds into the observer's step feedback.
    pub latent_transformer: LatentReasoningTransformer,
    /// Phase 6.3 — sparse MoE router: scores E8 expert groups and keeps the
    /// top-2 active, masking the fused attention vector with mass conservation.
    pub sparse_moe: SparseMoERouter,
    /// Phase 10.3 — multimodal unified reasoning: text+image+audio encoders →
    /// unified latent space → cross-modal fusion driving the E8 loop.
    pub multimodal: MultimodalEncoder,
    /// JEPA world model — predict upcoming latent state from reasoning context,
    /// injected into the prompt as prior signal. Option: absent unless wired.
    pub jepa: Option<JepaWorldModel>,
}

impl ReasoningEngine {
    pub fn new(brain: Box<dyn BrainMutView>, bank: ReasoningBank) -> Self {
        Self {
            current_state: FullReasoningState::new(
                ReasoningHexagram::new(0),
                crate::l5_cognition::nt_core_hex::MetaState::new(0),
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
            kb_cache: Mutex::new(HashMap::new()),
            last_kb_injected: Mutex::new(Vec::new()),
            artifact_indexer: None,
            cognitive_eye: CognitiveEye::new(),
            ttc_engine: None,
            orchestrator: None,
            trajectory_compressor: None,
            anti_distillation: None,
            tracer: None,
            cost_tracker: None,
            _last_watermarked: None,
            control_distiller: Some(ControlDistiller::new(Arc::new(
                ConsciousnessGoldStandard::new(),
            ))),
            distilled_sequences: Vec::new(),
            train_batch: 0,
            ewhr_bridge: None,
            sae_bridge: None,
            prm: None,
            verifier: None,
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
            unified_latent: UnifiedLatentSpace::new(),
            latent_reasoning: LatentReasoningPipeline::new(),
            latent_transformer: LatentReasoningTransformer::new(),
            sparse_moe: SparseMoERouter::default(),
            multimodal: MultimodalEncoder::new(),
            jepa: None,
            context_builder: None,
            cot_generator: None,
            e8_policy: None,
        }
    }

    pub fn from_env() -> Self {
        use crate::l5_cognition::nt_mind::nt_mind::self_iterating::brain_core::ReasoningBrain;
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

    pub fn with_hypothesis_network(
        mut self,
        net: Arc<
            Mutex<crate::l4_emotion::nt_memory::nt_memory_historian::nt_evidence_hypothesis::HypothesisNetwork>,
        >,
    ) -> Self {
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

    pub(crate) fn _with_verifier(
        mut self,
        verifier: crate::l5_cognition::nt_core_prm::GroundedPrmVerifier,
    ) -> Self {
        self.verifier = Some(verifier);
        self
    }

    pub(crate) fn _with_context_builder(
        mut self,
        builder: crate::l0_substrate::nt_core_answer_engine::ContextBuilder,
    ) -> Self {
        self.context_builder = Some(builder);
        self
    }

    pub(crate) fn _with_cot_generator(
        mut self,
        generator: crate::l5_cognition::nt_core_cot_generator::DefaultCoTGenerator,
    ) -> Self {
        self.cot_generator = Some(generator);
        self
    }

    pub fn with_e8_policy(mut self, policy: crate::l5_cognition::nt_core_policy::E8Policy) -> Self {
        self.e8_policy = Some(policy);
        self
    }

    pub(crate) fn _with_intent_engine(mut self, intent_engine: IntentEngine) -> Self {
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

    pub(crate) fn _with_effort_tier_selector(mut self, selector: EffortTierSelector) -> Self {
        self.effort_tier_selector = selector;
        self
    }

    pub fn with_observer_transition_matrix(
        mut self,
        matrix: crate::l2_perception::nt_core_e8::E8TransitionMatrix,
    ) -> Self {
        self.observer = self.observer.with_transition_matrix(matrix);
        self
    }


    pub fn with_jepa(mut self, jepa: JepaWorldModel) -> Self {
        self.jepa = Some(jepa);
        self
    }
}
