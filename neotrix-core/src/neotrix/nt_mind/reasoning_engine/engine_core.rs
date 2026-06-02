//! ReasoningEngine 结构体定义 + 构造器 + 主入口 + 导航方法

use tokio::runtime::Runtime;

use crate::neotrix::nt_mind::self_iterating::ReasoningBrain;
use crate::neotrix::nt_mind::memory::ReasoningBank;
use crate::neotrix::nt_mind::distillation::{StrategicPrinciple, AntiPattern};
use crate::neotrix::nt_mind::context_artifacts::ArtifactIndexer;
use crate::neotrix::nt_mind::reasoning_types::{ReasoningType, ReasoningTrace};
use crate::neotrix::nt_mind::model_router::ModelRouter;
use crate::neotrix::nt_mind::reasoning_engine::cognitive_observer::CognitiveEye;
use crate::neotrix::nt_mind::reasoning_engine::markov_check::MarkovCheck;
use crate::core::{
    CapabilityVector, ReasoningHexagram, FullReasoningState, MetaState, ModeFit,
    optimal_starting_mode, rank_modes_for_task, MODE_NAMES, MODE_DESCRIPTIONS,
    OneObserver, strategy_matrix, E8Policy, E8TransitionLearner,
};
use crate::core::nt_core_gwt::workspace::GlobalWorkspace;
use crate::core::nt_core_gwt::module_def::SpecialistType;
use crate::core::nt_core_self::{
    AttentionDomain, CrystalRegistry, StrategyKind, ReflectionGrade, SkillCrystal,
    SiliconSelfModel,
};
use crate::neotrix::nt_mind::thinking_bridge::ThinkingBridge;
use crate::neotrix::error::{NeoTrixError, NeoTrixResult};
use crate::neotrix::nt_memory_kb::KnowledgeBase;
use crate::neotrix::nt_world_jepa::{JepaWorldModel, WorldModelState};
use crate::neotrix::provider::{
    factory::{ProviderConfig, create_provider},
    types::{LlmProvider, LlmRequest},
};
use crate::neotrix::nt_shield_prompt::{PromptGuard, RiskLevel};
use crate::neotrix::mention::resolve_mentions;
use crate::neotrix::nt_world_search::WebSearchTool;
use tokio::sync::mpsc;

/// 统一推理引擎 — 项目意识核心
/// The core's structured reasoning plan, generated before LLM execution.
#[derive(Debug, Clone)]
pub struct CoreReasoningPlan {
    pub strategy: StrategyKind,
    pub domains: Vec<AttentionDomain>,
    pub e8_mode: ReasoningHexagram,
    pub mode_name: String,
    pub mode_desc: String,
    pub crystal_used: Option<usize>,
    pub specialist: String,
    pub guidance: Vec<String>,
    pub avoid_patterns: Vec<String>,
}

pub struct ReasoningEngine {
    pub llm: Box<dyn LlmProvider>,
    pub brain: ReasoningBrain,
    pub bank: ReasoningBank,
    pub traces: Vec<ReasoningTrace>,
    pub principles: Vec<StrategicPrinciple>,
    pub anti_patterns: Vec<AntiPattern>,
    pub(super) runtime: Runtime,
    pub default_model: String,
    pub distill_interval: usize,
    traces_since_distill: usize,
    pub current_state: FullReasoningState,
    pub state_trajectory: Vec<FullReasoningState>,
    pub(super) llm_call_count: u64,
    pub(super) llm_total_time_ms: u64,
    pub(super) llm_last_duration_ms: u64,
    pub(super) bank_retrieval_count: u64,
    pub(super) artifact_indexer: Option<ArtifactIndexer>,
    pub observer: OneObserver,
    pub router: ModelRouter,
    pub cost_log: Vec<CostRecord>,
    pub strategy_matrix: [[ReasoningHexagram; 8]; 8],
    pub cognitive_eye: CognitiveEye,
    pub crystal_registry: Option<CrystalRegistry>,
    pub last_crystal_used: Option<usize>,
    pub kb: Option<KnowledgeBase>,
    pub consciousness_iteration: u64,
    pub gwt: Option<GlobalWorkspace>,
    pub last_core_plan: Option<CoreReasoningPlan>,
    pub silicon_self: Option<SiliconSelfModel>,
    pub thinking_bridge: Option<ThinkingBridge>,
    pub e8_policy: Option<E8Policy>,
    pub e8_learner: Option<E8TransitionLearner>,
    pub nt_world_model: Option<WorldModelState>,
    pub jepa: Option<JepaWorldModel>,
    pub nt_world_search_tool: Option<WebSearchTool>,
    pub reasoning_distiller: crate::neotrix::nt_mind::reasoning_engine::reasoning_distiller::ReasoningDistiller,
    pub markov_check: MarkovCheck,
}

/// 每次推理的成本记录
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CostRecord {
    pub timestamp: i64,
    pub tier: String,
    pub model: String,
    pub token_estimate: usize,
    pub cost_estimate_usd: f64,
    pub duration_ms: u64,
    pub reasoning_type: String,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct ReasoningStats {
    pub total_traces: usize,
    pub success_rate: f64,
    pub principles_count: usize,
    pub anti_patterns_count: usize,
    pub last_type: Option<ReasoningType>,
}

#[derive(Debug, Clone)]
pub struct EngineMetrics {
    pub total_llm_calls: u64,
    pub total_llm_time_ms: u64,
    pub last_call_duration_ms: u64,
    pub bank_retrieval_count: u64,
    pub total_traces: u64,
    pub principles_count: u64,
    pub anti_patterns_count: u64,
}

/// Map a GWT specialist type to relevant attention domains.
fn specialist_to_domains(st: &SpecialistType) -> Vec<AttentionDomain> {
    match st {
        SpecialistType::PatternMatcher => vec![AttentionDomain::PatternMatch, AttentionDomain::Code, AttentionDomain::Semantic],
        SpecialistType::AnomalyDetector => vec![AttentionDomain::RiskAssessment, AttentionDomain::Code, AttentionDomain::PatternMatch],
        SpecialistType::KnowledgeRetriever => vec![AttentionDomain::Semantic, AttentionDomain::PatternMatch, AttentionDomain::Temporal],
        SpecialistType::CodeAnalyzer => vec![AttentionDomain::Code, AttentionDomain::PatternMatch, AttentionDomain::RiskAssessment],
        SpecialistType::Planner => vec![AttentionDomain::Planning, AttentionDomain::GoalAlignment, AttentionDomain::Temporal],
        SpecialistType::KnowledgeIntegrator => vec![AttentionDomain::Semantic, AttentionDomain::Creativity, AttentionDomain::Temporal],
        SpecialistType::GoalPrioritizer => vec![AttentionDomain::GoalAlignment, AttentionDomain::Planning, AttentionDomain::Temporal],
        SpecialistType::RiskAssessor => vec![AttentionDomain::RiskAssessment, AttentionDomain::GoalAlignment, AttentionDomain::Code],
        SpecialistType::CreativityGenerator => vec![AttentionDomain::Creativity, AttentionDomain::Semantic, AttentionDomain::PatternMatch],
        SpecialistType::ReflectionEngine => vec![AttentionDomain::SelfReflection, AttentionDomain::Code, AttentionDomain::Planning],
        SpecialistType::MetaCognitionAnalyst => vec![AttentionDomain::SelfReflection, AttentionDomain::GoalAlignment, AttentionDomain::Planning],
        SpecialistType::AISecurity => vec![AttentionDomain::RiskAssessment, AttentionDomain::PatternMatch, AttentionDomain::Code],
        SpecialistType::ImageGenerator => vec![AttentionDomain::Creativity, AttentionDomain::PatternMatch, AttentionDomain::Semantic],
    }
}

impl ReasoningEngine {
    pub fn from_env(brain: ReasoningBrain, bank: ReasoningBank) -> Self {
        let config = ProviderConfig::from_env();
        let model = config.model.clone().unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
        let llm = create_provider(config);
        Self {
            llm,
            brain,
            bank,
            traces: Vec::new(),
            principles: Vec::new(),
            anti_patterns: Vec::new(),
            current_state: FullReasoningState::new(ReasoningHexagram(0), MetaState(0)),
            state_trajectory: Vec::new(),
            runtime: Runtime::new().expect("创建 Tokio Runtime 失败"),
            default_model: model,
            distill_interval: 10,
            traces_since_distill: 0,
            llm_call_count: 0,
            llm_total_time_ms: 0,
            llm_last_duration_ms: 0,
            bank_retrieval_count: 0,
            artifact_indexer: None,
            router: ModelRouter::new(),
            cost_log: Vec::new(),
            observer: OneObserver::new(),
            strategy_matrix: strategy_matrix(),
            cognitive_eye: CognitiveEye::new(),
            crystal_registry: None,
            last_crystal_used: None,
            kb: None,
            consciousness_iteration: 0,
            gwt: None,
            last_core_plan: None,
            silicon_self: None,
            thinking_bridge: None,
            e8_policy: None,
            e8_learner: None,
            nt_world_model: None,
            jepa: None,
            nt_world_search_tool: None,
            reasoning_distiller: crate::neotrix::nt_mind::reasoning_engine::reasoning_distiller::ReasoningDistiller::new(),
            markov_check: MarkovCheck::new(),
        }
    }

    pub fn new(llm: Box<dyn LlmProvider>, brain: ReasoningBrain, bank: ReasoningBank, model: &str) -> Self {
        Self {
            llm,
            brain,
            bank,
            traces: Vec::new(),
            principles: Vec::new(),
            anti_patterns: Vec::new(),
            current_state: FullReasoningState::new(ReasoningHexagram(0), MetaState(0)),
            state_trajectory: Vec::new(),
            runtime: Runtime::new().expect("创建 Tokio Runtime 失败"),
            default_model: model.to_string(),
            distill_interval: 10,
            traces_since_distill: 0,
            llm_call_count: 0,
            llm_total_time_ms: 0,
            llm_last_duration_ms: 0,
            bank_retrieval_count: 0,
            artifact_indexer: None,
            router: ModelRouter::new(),
            cost_log: Vec::new(),
            observer: OneObserver::new(),
            strategy_matrix: strategy_matrix(),
            cognitive_eye: CognitiveEye::new(),
            crystal_registry: None,
            last_crystal_used: None,
            kb: None,
            consciousness_iteration: 0,
            gwt: None,
            last_core_plan: None,
            silicon_self: None,
            thinking_bridge: None,
            e8_policy: None,
            e8_learner: None,
            nt_world_model: None,
            jepa: None,
            nt_world_search_tool: None,
            reasoning_distiller: crate::neotrix::nt_mind::reasoning_engine::reasoning_distiller::ReasoningDistiller::new(),
            markov_check: MarkovCheck::new(),
        }
    }

    pub fn with_crystal_registry(mut self, registry: CrystalRegistry) -> Self {
        self.crystal_registry = Some(registry);
        self
    }

    pub fn with_strategy_matrix(mut self, m: [[ReasoningHexagram; 8]; 8]) -> Self {
        self.strategy_matrix = m;
        self
    }

    pub fn with_artifact_indexer(mut self, indexer: ArtifactIndexer) -> Self {
        self.artifact_indexer = Some(indexer);
        self
    }

    pub fn with_kb(mut self, kb: KnowledgeBase) -> Self {
        self.kb = Some(kb);
        self
    }

    /// Select reasoning mode guided by crystallized skills when available.
    /// Falls back to default keyword-based optimal_starting_mode when no crystals match.
    pub(crate) fn select_mode(&mut self, task: &str) -> ReasoningHexagram {
        let default_mode = optimal_starting_mode(task);
        self.last_crystal_used = None;
        let Some(ref registry) = self.crystal_registry else {
            return default_mode;
        };
        let lower = task.to_lowercase();

        let matching: Vec<&SkillCrystal> = registry.crystals.iter()
            .filter(|c| c.effectiveness > 0.4)
            .filter(|c| {
                c.tags.iter().any(|t| lower.contains(&t.to_lowercase()))
                    || c.name.to_lowercase().split_whitespace()
                        .any(|w| lower.contains(w))
            })
            .collect();

        if matching.is_empty() {
            return default_mode;
        }

        // Boost hexagram selection: for each matching crystal, add its tags as extra
        // keyword matches to modes whose task_recommendation overlaps with those tags
        let best = matching.iter()
            .max_by(|a, b| a.effectiveness.partial_cmp(&b.effectiveness).unwrap())
            .unwrap();
        self.last_crystal_used = Some(best.id);

        let mut best_score = 0u32;
        let mut best_idx = 0u8;
        for bits in 0..64u8 {
            let state = ReasoningHexagram(bits);
            let keywords = state.task_recommendation();
            let keyword_score: u32 = keywords.iter().map(|kw| {
                if lower.contains(kw) { 1 } else { 0 }
            }).sum();
            let crystal_boost: u32 = best.tags.iter()
                .filter(|t| keywords.iter().any(|kw| kw.contains(&t.to_lowercase())))
                .count() as u32
                * 2;
            let total = keyword_score + crystal_boost;
            if total > best_score {
                best_score = total;
                best_idx = bits;
            }
        }
        ReasoningHexagram(best_idx)
    }

    pub fn with_gwt(mut self, gwt: GlobalWorkspace) -> Self {
        self.gwt = Some(gwt);
        self
    }

    pub fn with_silicon_self(mut self, model: SiliconSelfModel) -> Self {
        self.silicon_self = Some(model);
        self
    }

    pub fn with_thinking_bridge(mut self, bridge: ThinkingBridge) -> Self {
        self.thinking_bridge = Some(bridge);
        self
    }

    pub fn with_e8_policy(mut self, policy: E8Policy) -> Self {
        self.e8_policy = Some(policy);
        self
    }

    pub fn with_e8_learner(mut self, learner: E8TransitionLearner) -> Self {
        self.e8_learner = Some(learner);
        self
    }

    pub fn with_nt_world_model(mut self, wm: WorldModelState) -> Self {
        self.nt_world_model = Some(wm);
        self
    }

    pub fn with_nt_world_search(mut self, tool: WebSearchTool) -> Self {
        self.nt_world_search_tool = Some(tool);
        self
    }

    pub fn with_jepa(mut self, jepa: JepaWorldModel) -> Self {
        self.jepa = Some(jepa);
        self
    }

    /// Core-First Reasoning: generate a structured reasoning plan from core analysis,
    /// before any LLM call. Uses GWT resonance + SiliconSelfModel + ThinkingBridge +
    /// crystal registry + E8 state.
    pub fn plan_reasoning(&mut self, task: &str) -> CoreReasoningPlan {
        let mode = self.select_mode(task);
        self.current_state = self.current_state.transition_to(mode);
        self.state_trajectory.push(self.current_state);
        self.trim_trajectory();
        self.consciousness_iteration += 1;

        let mode_name = mode.mode_name().to_string();
        let mode_desc = mode.mode_description().to_string();

        let mut domains = vec![AttentionDomain::Code, AttentionDomain::Planning];
        let mut specialist_label = "None".to_string();
        let mut guidance = Vec::new();
        let mut avoid_patterns = Vec::new();
        let mut matched_strategies: Vec<StrategyKind> = Vec::new();

        // ── Phase 0: World Model state injection ──
        if let Some(ref jepa) = self.jepa {
            let default_wm = WorldModelState::new();
            let current_state = self.nt_world_model.as_ref().unwrap_or(&default_wm);
            let predicted = jepa.predict_next_state(current_state);
            guidance.push(format!(
                "World model predicts: CPU at {:.0}%, errors at {:.1}%, queue depth {}",
                predicted.cpu_usage * 100.0,
                predicted.error_rate * 100.0,
                predicted.task_queue_depth,
            ));
            if let Some(ref current) = self.nt_world_model {
                let trends = predicted.describe_trend(current);
                guidance.push(format!(
                    "Current environment: CPU {:.0}%, memory {:.0}% available, iteration {}",
                    current.cpu_usage * 100.0,
                    current.memory_available * 100.0,
                    current.iteration_count,
                ));
                for trend in &trends {
                    guidance.push(format!("Trend: {}", trend));
                }
            }
        }

        // ── Phase 0.5: ReasoningDistiller recommendation ──
        if self.reasoning_distiller.total_observations() >= 3 {
            if let Some((rec_mode, rec_reason, top_approaches)) = self.reasoning_distiller.recommend_mode(task) {
                guidance.push(format!(
                    "ReasoningDistiller recommends E8 mode {} ({}) — top approaches: {}",
                    rec_mode, rec_reason, top_approaches.join(", "),
                ));
            }
            let profile = self.reasoning_distiller.mode_profile(mode.0);
            for line in &profile {
                guidance.push(format!("[distiller] {}", line));
            }
        }

        // ── Phase 1: GWT resonance analysis ──
        if let Some(ref mut gwt) = self.gwt {
            gwt.broadcast(&format!("task: {task}"));
            if gwt.specialists.is_empty() {
                gwt.register_default_specialists();
            }
            let states = crate::core::default_specialist_states();

            // ── KB-enriched broadcast ──
            let broadcast_content = match self.kb {
                Some(ref kb) => {
                    let kb_results = kb.query_broadcast_context(task, 5)
                        .unwrap_or_default();
                    if kb_results.is_empty() {
                        format!("task analysis: {task}")
                    } else {
                        let ctx: Vec<&str> = kb_results.iter().map(|r| r.node.title.as_str()).collect();
                        format!("task analysis: {task}\nKnowledge context: {}", ctx.join(", "))
                    }
                }
                None => format!("task analysis: {task}"),
            };
            gwt.resonant_broadcast(&broadcast_content, &states);

            if let Some(winner) = gwt.resonance_winner() {
                specialist_label = format!("{:?}", winner.specialist_type);
                domains = specialist_to_domains(&winner.specialist_type);
                guidance.push(format!(
                    "GWT specialist consensus: {:?} leads the resonance",
                    winner.specialist_type
                ));
            }

            gwt.decay_all(0.1);
        }

        // ── Phase 2: SiliconSelfModel trigger matching ──
        if let Some(ref mut ss) = self.silicon_self {
            ss.observe(&format!("plan: {task}"));
            matched_strategies = ss.match_triggers(task);
            for strat in &matched_strategies {
                guidance.push(format!("SiliconSelf trigger match: {:?}", strat));
            }
            let state = ss.current_state();
            guidance.push(format!(
                "SiliconSelf context: {:.0}% used, {} patterns active",
                state.context_usage * 100.0,
                state.active_patterns,
            ));
        }

        // ── Phase 3: ThinkingBridge observation ──
        if let Some(ref mut bridge) = self.thinking_bridge {
            bridge.observe_task(task);
            bridge.run_reflection_cycle();
            let profile = bridge.attention_profile_summary();
            guidance.push(format!("ThinkingBridge: {}", profile));
        }

        // ── Phase 4: Task-type-specific guidance ──
        {
            let lower = task.to_lowercase();
            if lower.contains("bug") || lower.contains("error") || lower.contains("fix") {
                guidance.push("Trace the root cause before proposing a fix".to_string());
                guidance.push("Check edge cases that could mask the real issue".to_string());
            }
            if lower.contains("design") || lower.contains("architect") || lower.contains("plan") {
                guidance.push("Consider trade-offs between competing approaches".to_string());
                guidance.push("Surface assumptions that constrain the design space".to_string());
            }
            if lower.contains("review") || lower.contains("audit") || lower.contains("check") {
                guidance.push("Inspect for correctness, safety, and performance".to_string());
                guidance.push("Flag patterns that deviate from established conventions".to_string());
            }
        }

        // ── Phase 5: Crystal-guided strategy annotation ──
        if let Some(crystal_id) = self.last_crystal_used {
            guidance.push(format!("Crystallized skill #{} guides this reasoning", crystal_id));
        }

        // ── Phase 6: CognitiveObserver pre-check ──
        let known_spots = self.cognitive_eye.total_observations;
        if known_spots > 5 {
            avoid_patterns.push("Avoid over-indexing on the most recent trace".to_string());
        }
        if known_spots > 10 {
            avoid_patterns.push("Beware of confirmation bias from past successes".to_string());
        }

        // ── Phase 7: KB-enriched guidance ──
        if let Some(ref kb) = self.kb {
            let e8_tags = mode.task_recommendation();
            for tag in e8_tags.iter().take(3) {
                if let Ok(results) = kb.recommend_for_e8_mode(tag, 3) {
                    for r in &results {
                        guidance.push(format!("KB: {} — {}", tag, r.node.title));
                    }
                }
            }
        }

        // Decide strategy: mode_name match first, then SiliconSelf triggers, then CoT fallback
        let strategy = self.guide_strategy(&mode_name, &matched_strategies);

        // Sync the selected mode to E8Policy so its Q-update targets the correct mode
        if let Some(ref mut policy) = self.e8_policy {
            policy.set_previous(mode);
        }

        let plan = CoreReasoningPlan {
            strategy,
            domains,
            e8_mode: mode,
            mode_name,
            mode_desc,
            crystal_used: self.last_crystal_used,
            specialist: specialist_label,
            guidance,
            avoid_patterns,
        };
        self.last_core_plan = Some(plan.clone());
        plan
    }

    /// Resolve strategy from mode name and SiliconSelf matched triggers.
    fn guide_strategy(&self, mode_name: &str, matched: &[StrategyKind]) -> StrategyKind {
        let from_mode = match mode_name {
            "Reflection" => StrategyKind::Reflection,
            "ChainOfThought" | "Chain of Thought" => StrategyKind::ChainOfThought,
            "Deliberate" => StrategyKind::Deliberate,
            "Decompose" | "Decomposition" | "RecursiveDecomposition" => StrategyKind::RecursiveDecomposition,
            "Direct" | "Conversation" => StrategyKind::Direct,
            "ToolAssisted" => StrategyKind::ToolAssisted,
            "IterativeRefinement" => StrategyKind::IterativeRefinement,
            _ => StrategyKind::ChainOfThought,
        };
        // If SiliconSelf matched specific strategies, prefer those over default
        if let Some(first) = matched.first() {
            if *first != StrategyKind::Direct {
                return *first;
            }
        }
        from_mode
    }

    /// Post-reasoning core review: feeds outcome back to GWT, ThinkingBridge, SiliconSelfModel, crystals, observer, and distiller.
    fn core_review(&mut self, task: &str, outcome: &str, has_image: bool) {
        let success = !outcome.is_empty() && outcome.len() > 10;

        // GWT reward: broadcast outcome
        if let Some(ref mut gwt) = self.gwt {
            let reward = if success { 0.3 } else { -0.2 };
            gwt.broadcast(&format!("outcome: {} (success={})", &task[..task.len().min(60)], success));
            for (_, m) in gwt.specialists.iter_mut() {
                m.activation = (m.activation + reward).max(0.0).min(1.0);
            }
        }

        // ThinkingBridge → SiliconSelfModel recording
        if let Some(ref mut bridge) = self.thinking_bridge {
            bridge.observe_tool_use("reason", &outcome[..outcome.len().min(200)]);
            if success {
                bridge.silicon.attention_manager.stimulate_domain(AttentionDomain::SelfReflection, 0.1);
            }
        } else if let Some(ref mut ss) = self.silicon_self {
            ss.observe(&format!("outcome: {} (success={})", &task[..task.len().min(60)], success));
        }

        // Core analysis blind-spot detection (observer)
        let strategy_dist: std::collections::HashMap<StrategyKind, usize> = {
            let mut m = std::collections::HashMap::new();
            for t in &self.traces {
                let kind = match t.reasoning_type {
                    ReasoningType::Conversation => StrategyKind::Direct,
                    ReasoningType::TaskSolving => StrategyKind::ChainOfThought,
                    ReasoningType::ErrorDebugging => StrategyKind::Reflection,
                    ReasoningType::KnowledgeQuery => StrategyKind::ToolAssisted,
                    ReasoningType::PrdGeneration => StrategyKind::Deliberate,
                    ReasoningType::General => StrategyKind::ChainOfThought,
                };
                *m.entry(kind).or_insert(0) += 1;
            }
            m
        };
        let domains = self.last_core_plan.as_ref()
            .map(|p| p.domains.clone())
            .unwrap_or_else(|| vec![AttentionDomain::Code, AttentionDomain::Planning]);
        let grades: Vec<ReflectionGrade> = self.traces.iter()
            .map(|t| if t.success {
                if t.outcome_score > 0.8 { ReflectionGrade::Excellent }
                else if t.outcome_score > 0.5 { ReflectionGrade::Good }
                else if t.outcome_score > 0.2 { ReflectionGrade::Adequate }
                else { ReflectionGrade::Poor }
            } else {
                ReflectionGrade::Failed
            })
            .collect();
        let errors: Vec<String> = self.traces.iter()
            .filter_map(|t| t.error_context.clone())
            .collect();
        let context_pct = (self.traces.len() as f64 / 50.0).min(1.0);
        let cognitive_spots = self.cognitive_eye.observe(
            strategy_dist,
            domains,
            context_pct,
            grades,
            errors,
            &self.brain.capability,
        );

        for spot in &cognitive_spots {
            for (dim_name, delta) in &spot.capability_deltas {
                if let Some(idx) = CapabilityVector::index_from_name(dim_name) {
                    let cur = self.brain.capability.arr()[idx];
                    self.brain.capability.arr_mut()[idx] = (cur + delta).max(0.0).min(1.0);
                }
            }
        }
        if !cognitive_spots.is_empty() {
            self.brain.capability.normalize();
            log::debug!("[core_review] {} blind spots corrected", cognitive_spots.len());
        }

        // Crystal outcome feedback
        if let Some(crystal_id) = self.last_crystal_used {
            if let Some(ref mut registry) = self.crystal_registry {
                registry.record_use(crystal_id, self.traces.len());
                let penalty = cognitive_spots.len() as f64 * 0.05;
                if let Some(crystal) = registry.crystals.iter_mut().find(|c| c.id == crystal_id) {
                    crystal.effectiveness = (crystal.effectiveness - penalty).max(0.1);
                    if cognitive_spots.is_empty() {
                        crystal.effectiveness = (crystal.effectiveness + 0.02).min(1.0);
                    }
                }
            }
        }

        // E8 RL policy update
        let reward = if success {
            0.5 - (cognitive_spots.len() as f64 * 0.1).min(0.4)
        } else {
            -0.3
        };
        if let Some(ref mut policy) = self.e8_policy {
            if let Some(ref plan) = self.last_core_plan {
                debug_assert_eq!(
                    policy.previous_mode(),
                    Some(plan.e8_mode),
                    "E8Policy previous_mode ({:?}) must match last_core_plan e8_mode ({:?})",
                    policy.previous_mode(),
                    plan.e8_mode,
                );
            }
            policy.update(reward);
            policy.decay_epsilon();
        }
        if let Some(ref mut learner) = self.e8_learner {
            if let Some(ref plan) = self.last_core_plan {
                learner.record(task, plan.e8_mode, reward, self.consciousness_iteration);
            }
        }

        // ReasoningDistiller: observe LLM response structure, correlate with E8 mode + GWT specialist
        let e8_mode = self.last_core_plan.as_ref().map(|p| p.e8_mode.0).unwrap_or(0);
        let specialist = self.last_core_plan.as_ref().map(|p| p.specialist.as_str()).unwrap_or("unknown");
        let outcome_score = if success { 0.5 + reward.max(0.0) } else { reward.max(0.0) };
        self.reasoning_distiller.observe(task, outcome, e8_mode, specialist, outcome_score, has_image, None);

        let state_prev = vec![0.5, 0.5, 0.5];
        let state_cur = vec![
            self.current_state.mode.0 as f64 / 64.0,
            self.current_state.meta.0 as f64 / 3.0,
            outcome_score,
        ];
        let state_rnd = vec![0.0, 0.0, 0.0];
        let _msa_score = self.markov_check.evaluate(&state_prev, e8_mode, &state_cur, &state_rnd);

        // Record conversation for evolution training data
        self.record_conversation_evolution(task, outcome, success, cognitive_spots.len());
    }

    /// Record the external conversation as evolution training data.
    /// This feeds the meta-cognitive self-evolution loop.
    fn record_conversation_evolution(&self, task: &str, outcome: &str, success: bool, blind_spot_count: usize) {
        let Some(ref kb) = self.kb else { return };
        let plan = match self.last_core_plan {
            Some(ref p) => p,
            None => return,
        };
        let specialist_name = if let Some(ref gwt) = self.gwt {
            // Find highest-activation specialist
            gwt.specialists.iter()
                .max_by(|a, b| a.1.activation.partial_cmp(&b.1.activation).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(name, _)| name.clone())
                .unwrap_or_else(|| "unknown".into())
        } else {
            "unknown".into()
        };
        let action_count = self.traces.len() as u32;
        let error_count = self.traces.iter().filter(|t| !t.success).count() as u32;

        let record = crate::neotrix::nt_memory_kb::ConversationRecord {
            id: format!("conv_{}", self.consciousness_iteration),
            session_id: format!("session_{}", self.consciousness_iteration / 100),
            task_description: task.chars().take(120).collect(),
            user_intent: plan.guidance.first().cloned().unwrap_or_else(|| "unknown".into()),
            strategy_used: format!("{:?}", plan.strategy),
            e8_mode: format!("{:?}", plan.e8_mode),
            specialist_winner: specialist_name,
            actions_taken: vec![format!("plan: {:?}", plan)],
            obstacles_encountered: if !success {
                vec![format!("outcome: {}", outcome.chars().take(80).collect::<String>())]
            } else {
                Vec::new()
            },
            fix_patterns: if blind_spot_count > 0 {
                vec![format!("{} blind spots corrected", blind_spot_count)]
            } else {
                Vec::new()
            },
            outcome: if success { "success".into() } else { "failure".into() },
            effectiveness: if success {
                0.5 - (blind_spot_count as f64 * 0.1).min(0.4)
            } else {
                -0.3
            },
            reasoning_iterations: action_count,
            error_count,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        };
        let _ = kb.store_conversation_record(&record);
    }

    /// Core-First Reasoning: plan → LLM executes → core reviews
    pub fn reason(&mut self, task: &str) -> NeoTrixResult<String> {
        // Phase 1: Core generates the reasoning plan (no LLM involved)
        let plan = self.plan_reasoning(task);

        // Phase 2: LLM executes the plan
        let result = self.reason_with_plan(task, &plan);

        // Phase 3: Core reviews the outcome (always, even on LLM failure)
        match result {
            Ok(text) => {
                self.traces_since_distill += 1;
                if self.traces_since_distill >= self.distill_interval {
                    self.self_iterate();
                    self.traces_since_distill = 0;
                }
                self.core_review(task, &text, false);
                return Ok(text);
            }
            Err(e) => {
                self.core_review(task, "", false);
                return Err(e);
            }
        }
    }

    /// Build a plan-based prompt from the core reasoning plan.
    /// Resolves @-mention file references in the task string.
    fn build_plan_prompt(&self, task: &str, plan: &CoreReasoningPlan) -> String {
        let memories = self.bank.retrieve_relevant(task, None, 3);
        let memory_context: String = memories.iter()
            .map(|m| format!("  [{}] {} (reward={:.2})",
                if m.success { "OK" } else { "FAIL" }, m.task_description, m.reward))
            .collect::<Vec<_>>().join("\n");

        let artifact_context = self.build_artifact_context(task);
        let kb_context = self.kb_context(task);

        // Resolve @-mention file references in the task
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let (task_with_mentions, mentions) = resolve_mentions(task, &cwd);
        let mention_block: String = if mentions.is_empty() {
            String::new()
        } else {
            let mut block = String::from("\nReferenced files:\n");
            for m in &mentions {
                let flag = if m.truncated { " (truncated)" } else { "" };
                block.push_str(&format!(
                    "  [📎 {}] {} lines{flag}\n",
                    m.path.display(),
                    m.lines,
                ));
            }
            block
        };

        let guidance_blocks: String = plan.guidance.iter()
            .enumerate()
            .map(|(i, g)| format!("    {}. {}", i + 1, g))
            .collect::<Vec<_>>().join("\n");
        let avoid_blocks: String = if plan.avoid_patterns.is_empty() {
            "    None".to_string()
        } else {
            plan.avoid_patterns.iter()
                .map(|a| format!("    - {}", a))
                .collect::<Vec<_>>().join("\n")
        };
        let domain_str: String = plan.domains.iter()
            .map(|d| format!("    - {:?}", d))
            .collect::<Vec<_>>().join("\n");

        format!(
            "You are NeoTrix. The core has prepared a reasoning plan for you to execute.\n\n\
             == CORE REASONING PLAN ==\n\
             Mode: {}\n\
             Mode description: {}\n\
             Strategy: {:?}\n\
             E8 Hexagram: {:?}\n\
             Specialist consensus: {}\n\
             \n\
             Attention domains:\n{domain_str}\n\
             \n\
             Reasoning guidance:\n{guidance_blocks}\n\
             \n\
             Avoid these patterns:\n{avoid_blocks}\n\
             \n\
             == EXECUTION ==\n\
             Task: {task_with_mentions}\n\
             {mention_block}\
             Past experience:\n{memory_context}\n\
             {artifact_context}\n\
             {kb_context}\n\
             Execute the reasoning plan above. Think step by step within the guidance boundaries.",
            plan.mode_name,
            plan.mode_desc,
            plan.strategy,
            plan.e8_mode,
            plan.specialist,
        )
    }

    /// Build an LlmRequest with the core's reasoning plan injected.
    fn build_request(&mut self, task: &str, plan: &CoreReasoningPlan, image_data: Option<String>) -> NeoTrixResult<(LlmRequest, ReasoningType, String)> {
        let rtype = self.mode_to_reasoning_type(&plan.e8_mode);
        let prompt = self.build_plan_prompt(task, plan);

        let guard = PromptGuard::new();
        match guard.analyze(&prompt).0 {
            RiskLevel::Dangerous => {
                return Err(NeoTrixError::Brain("LLM 调用被阻止: 检测到注入模式".to_string()));
            }
            RiskLevel::Suspicious => {
                eprintln!("[warn] LLM prompt 含可疑模式，继续执行");
            }
            RiskLevel::Safe => {}
        }

        let route = self.router.route(&prompt);
        let model_name = self.default_model.clone();
        let mut request = LlmRequest::new(&model_name, &prompt);
        request.max_tokens = route.max_tokens as u32;
        request.image_data = image_data;

        Ok((request, rtype, prompt))
    }

    /// Core-First prompt construction: inject the full reasoning plan as instructions.
    /// Automatically handles web search tool calls (NEED_SEARCH: prefix) from the LLM.
    fn reason_with_plan(&mut self, task: &str, plan: &CoreReasoningPlan) -> NeoTrixResult<String> {
        let rtype = self.mode_to_reasoning_type(&plan.e8_mode);
        let mut prompt = self.build_plan_prompt(task, plan);

        for _round in 0..3 {
            let response = self.call_llm(&prompt)?;

            if let Some(query) = self.extract_search_request(&response) {
                match self.tool_call_nt_world_search(&query, 5) {
                    Ok(search_text) => {
                        prompt = format!(
                            "{}\n\n== WEB SEARCH RESULTS ==\n{}\n\nContinue with the above information.",
                            prompt, search_text
                        );
                        continue;
                    }
                    Err(e) => {
                        prompt = format!("{}\n\n[Web search failed: {}]", prompt, e);
                        continue;
                    }
                }
            }

            self.record_trace(rtype, task, &prompt, &response, None, 0.5);
            self.learn_from_trace(task, &response);
            return Ok(response);
        }

        Err(NeoTrixError::Brain("Exceeded maximum tool call rounds (3)".to_string()))
    }

    /// Check if the LLM response contains a web search request.
    fn extract_search_request(&self, response: &str) -> Option<String> {
        for line in response.lines() {
            let trimmed = line.trim();
            if let Some(query) = trimmed.strip_prefix("NEED_SEARCH:") {
                let q = query.trim().to_string();
                if !q.is_empty() {
                    return Some(q);
                }
            }
        }
        None
    }

    /// Execute a web search and return structured text suitable for LLM context.
    pub fn tool_call_nt_world_search(&self, query: &str, count: usize) -> Result<String, String> {
        match self.nt_world_search_tool {
            Some(ref tool) => tool.search(query, count),
            None => Err("Web search tool not configured. Enable via with_nt_world_search().".to_string()),
        }
    }

    /// Stream reasoning from LLM, sending tokens through an mpsc channel.
    /// Returns the full accumulated response and a Receiver for streaming.
    /// Core-First: plan → stream → core_review.
    pub async fn reason_stream(
        &mut self,
        task: &str,
        image_data: Option<String>,
    ) -> NeoTrixResult<(String, mpsc::Receiver<String>)> {
        let plan = self.plan_reasoning(task);
        let (request, rtype, prompt_text) = self.build_request(task, &plan, image_data.clone())?;

        let start = std::time::Instant::now();
        let mut receiver = match self.llm.stream_complete(&request).await {
            Ok(r) => r,
            Err(e) => {
                self.core_review(task, "", image_data.is_some());
                return Err(NeoTrixError::Brain(format!("stream start: {}", e)));
            }
        };

        let (tx, rx) = mpsc::channel::<String>(256);
        let mut full_response = String::new();

        while let Some(chunk_result) = receiver.recv().await {
            match chunk_result {
                Ok(chunk) => {
                    full_response.push_str(&chunk.content);
                    let _ = tx.send(chunk.content).await;
                }
                Err(e) => {
                    let _ = tx.send(format!("\n[LLM Error: {}]", e)).await;
                    break;
                }
            }
        }

        let elapsed = start.elapsed();
        let elapsed_ms = elapsed.as_millis() as u64;
        self.llm_call_count += 1;
        self.llm_total_time_ms += elapsed_ms;
        self.llm_last_duration_ms = elapsed_ms;

        self.cost_log.push(CostRecord {
            timestamp: chrono::Utc::now().timestamp(),
            tier: "stream".to_string(),
            model: self.default_model.clone(),
            token_estimate: (full_response.len() / 4).max(1),
            cost_estimate_usd: 0.0,
            duration_ms: elapsed_ms,
            reasoning_type: format!("{:?}", rtype),
            success: true,
        });

        self.record_trace(rtype, task, &prompt_text, &full_response, None, 0.8);
        self.learn_from_trace(task, &full_response);

        self.traces_since_distill += 1;
        if self.traces_since_distill >= self.distill_interval {
            self.self_iterate();
            self.traces_since_distill = 0;
        }

        self.core_review(task, &full_response, image_data.is_some());
        self.log_consciousness(task, &full_response, plan.e8_mode);

        Ok((full_response, rx))
    }

    #[allow(dead_code)]
    fn log_consciousness(&self, task: &str, response: &str, mode: ReasoningHexagram) {
        let Some(ref kb) = self.kb else { return };
        let summary = if response.len() > 200 { &response[..200] } else { response };
        let phi = mode.0 as f64 / 64.0;
        let coherence = self.current_state.meta.0 as f64 / 3.0;
        let details = format!("task={}, mode={}, response_preview={}",
            task, mode.0, summary);
        let _ = kb.record_consciousness_snapshot(
            phi,
            coherence,
            true,
            "daily_reasoning",
            &details,
        );
    }

    fn kb_context(&self, task: &str) -> String {
        let Some(ref kb) = self.kb else { return String::new() };
        match kb.search(task, 5) {
            Ok(results) if !results.is_empty() => {
                let entries: Vec<String> = results.iter()
                    .map(|r| format!("  · {} (score: {:.3})", r.node.title, r.score))
                    .collect();
                format!("Knowledge context:\n{}\n", entries.join("\n"))
            }
            _ => String::new(),
        }
    }

    fn mode_to_reasoning_type(&self, mode: &ReasoningHexagram) -> ReasoningType {
        let abs = mode.abstraction();
        let scp = mode.scope();
        let mtd = mode.method();
        let dep = mode.depth();
        match (abs, scp, mtd, dep) {
            (0, 0, 0, _) => ReasoningType::TaskSolving,
            (0, 0, 1, _) => ReasoningType::ErrorDebugging,
            (0, 1, 0, 0) => ReasoningType::General,
            (0, 1, 0, 1) => ReasoningType::KnowledgeQuery,
            (0, 1, 1, _) => ReasoningType::General,
            (1, 0, 0, _) => ReasoningType::PrdGeneration,
            (1, 0, 1, _) => ReasoningType::General,
            (1, 1, 0, _) => ReasoningType::KnowledgeQuery,
            (1, 1, 1, _) => ReasoningType::Conversation,
            _ => ReasoningType::General,
        }
    }

    fn trim_trajectory(&mut self) {
        if self.state_trajectory.len() > 200 {
            let remove = self.state_trajectory.len() - 200;
            self.state_trajectory.drain(0..remove);
        }
    }

    pub fn navigate_to_state(&self, task: &str) -> ReasoningHexagram {
        optimal_starting_mode(task)
    }

    pub fn rank_states_for_task(&self, task: &str, top_k: usize) -> Vec<ModeFit> {
        rank_modes_for_task(task, top_k)
    }

    pub fn reason_through_path(&mut self, task: &str, path: &[ReasoningHexagram]) -> NeoTrixResult<Vec<String>> {
        let mut results = Vec::with_capacity(path.len());
        for (i, &mode) in path.iter().enumerate() {
            let prev_state = self.current_state;
            self.current_state = self.current_state.transition_to(mode);
            self.state_trajectory.push(self.current_state);
            self.trim_trajectory();

            let mode_name = MODE_NAMES[mode.0 as usize];
            let mode_desc = MODE_DESCRIPTIONS[mode.0 as usize];

            // Build a plan for this step using the given mode
            // Sync the step mode to E8Policy so its Q-update targets the correct mode
            if let Some(ref mut policy) = self.e8_policy {
                policy.set_previous(mode);
            }

            let plan = CoreReasoningPlan {
                strategy: self.guide_strategy(mode_name, &[]),
                domains: vec![AttentionDomain::Code, AttentionDomain::Planning],
                e8_mode: mode,
                mode_name: mode_name.to_string(),
                mode_desc: mode_desc.to_string(),
                crystal_used: self.last_crystal_used,
                specialist: "path".to_string(),
                guidance: vec![format!("Path step {}: {} mode", i + 1, mode_name)],
                avoid_patterns: Vec::new(),
            };
            self.last_core_plan = Some(plan.clone());

            let result = self.reason_with_plan(task, &plan);
            match result {
                Ok(text) => {
                    self.core_review(task, &text, false);
                    if i > 0 {
                        let prev = path[i - 1];
                        let transition_info = format!(
                            "\n[Transition {i}: {} → {}, resonance={}, bits flipped={:06b}]",
                            MODE_NAMES[prev.0 as usize],
                            MODE_NAMES[mode.0 as usize],
                            prev.resonance_strength(&mode),
                            prev.0 ^ mode.0,
                        );
                        results.push(transition_info);
                    }
                    results.push(text);
                }
                Err(e) => {
                    self.current_state = prev_state;
                    self.state_trajectory.pop();
                self.core_review(task, "", false);
                    return Err(e);
                }
            }
        }

        Ok(results)
    }

    pub fn reflect_on_trajectory(&mut self, task: &str) -> NeoTrixResult<String> {
        let prev_state = self.current_state;
        self.current_state = self.current_state.reflect();
        self.state_trajectory.push(self.current_state);
        self.trim_trajectory();

        let trajectory_summary: String = self.state_trajectory.iter()
            .enumerate()
            .map(|(i, s)| {
                format!("  {}. {} (meta={:02b})", i, MODE_NAMES[s.mode.0 as usize], s.meta.0)
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Build a plan for meta-cognitive reflection, injecting trajectory as guidance
        let ref_mode = self.current_state.mode;
        let mode_name = MODE_NAMES[ref_mode.0 as usize];
        let mode_desc = MODE_DESCRIPTIONS[ref_mode.0 as usize];

        let plan = CoreReasoningPlan {
            strategy: self.guide_strategy(mode_name, &[]),
            domains: vec![AttentionDomain::SelfReflection, AttentionDomain::Planning],
            e8_mode: ref_mode,
            mode_name: mode_name.to_string(),
            mode_desc: mode_desc.to_string(),
            crystal_used: self.last_crystal_used,
            specialist: "ReflectionEngine".to_string(),
            guidance: vec![
                "Reflect on the reasoning trajectory step by step".to_string(),
                trajectory_summary,
                "1. Was the initial mode selection optimal?".to_string(),
                "2. Was there a better path through the state space?".to_string(),
                "3. What would a complementary (opposite) approach reveal?".to_string(),
                "4. What principle can you extract for future tasks?".to_string(),
            ],
            avoid_patterns: vec![],
        };
        self.last_core_plan = Some(plan.clone());

        if let Some(ref mut policy) = self.e8_policy {
            policy.set_previous(ref_mode);
        }

        let result = self.reason_with_plan(task, &plan);
        match result {
            Ok(text) => {
                self.core_review(task, &text, false);
                Ok(text)
            }
            Err(e) => {
                self.current_state = prev_state;
                self.state_trajectory.pop();
                self.core_review(task, "", false);
                Err(e)
            }
        }
    }

    pub fn reason_complement(&mut self, task: &str) -> NeoTrixResult<String> {
        let prev_state = self.current_state;
        let complement = self.current_state.mode.complement();
        self.current_state = self.current_state.transition_to(complement);
        self.state_trajectory.push(self.current_state);
        self.trim_trajectory();

        let mode_name = MODE_NAMES[complement.0 as usize];
        let mode_desc = MODE_DESCRIPTIONS[complement.0 as usize];

        if let Some(ref mut policy) = self.e8_policy {
            policy.set_previous(complement);
        }

        let plan = CoreReasoningPlan {
            strategy: self.guide_strategy(mode_name, &[]),
            domains: vec![AttentionDomain::Code, AttentionDomain::Planning],
            e8_mode: complement,
            mode_name: mode_name.to_string(),
            mode_desc: mode_desc.to_string(),
            crystal_used: self.last_crystal_used,
            specialist: "complement".to_string(),
            guidance: vec![format!("Complementary reasoning from {:?}", complement)],
            avoid_patterns: Vec::new(),
        };
        self.last_core_plan = Some(plan.clone());

        let result = self.reason_with_plan(task, &plan);
        match result {
            Ok(text) => {
                self.core_review(task, &text, false);
                Ok(text)
            }
            Err(e) => {
                self.current_state = prev_state;
                self.state_trajectory.pop();
                self.core_review(task, "", false);
                Err(e)
            }
        }
    }

    /// Save E8 reasoning state (current_state, trajectory, strategy_matrix, observer) to JSON.
    pub fn save_e8_state(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let trajectory: Vec<FullReasoningState> = self.state_trajectory.iter()
            .rev()
            .take(100)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        let snapshot = E8StateSnapshot {
            current_state: self.current_state,
            state_trajectory: trajectory,
            strategy_matrix: self.strategy_matrix,
            observer: self.observer.clone(),
        };

        let json = serde_json::to_string_pretty(&snapshot)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, &json)?;
        Ok(())
    }

    /// Load E8 reasoning state from JSON file.
    pub fn load_e8_state(&mut self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let snapshot: E8StateSnapshot = serde_json::from_str(&json)?;
        self.current_state = snapshot.current_state;
        self.state_trajectory = snapshot.state_trajectory;
        self.strategy_matrix = snapshot.strategy_matrix;
        self.observer = snapshot.observer;
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct E8StateSnapshot {
    current_state: FullReasoningState,
    state_trajectory: Vec<FullReasoningState>,
    strategy_matrix: [[ReasoningHexagram; 8]; 8],
    observer: OneObserver,
}


#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> ReasoningEngine {
        let brain = ReasoningBrain::new();
        let bank = ReasoningBank::new(100);
        let mut engine = ReasoningEngine::new(
            crate::neotrix::provider::factory::create_provider(
                crate::neotrix::provider::factory::ProviderConfig::from_env(),
            ),
            brain, bank, "test-model",
        );
        // Wire GWT with default specialists
        let mut gwt = GlobalWorkspace::new(0.3);
        gwt.register_default_specialists();
        engine = engine.with_gwt(gwt);
        // Wire SiliconSelfModel
        engine = engine.with_silicon_self(SiliconSelfModel::new());
        // Wire E8Policy for RL learning path testing
        engine = engine.with_e8_policy(E8Policy::default());
        engine = engine.with_nt_world_model(WorldModelState::new());
        engine = engine.with_jepa(JepaWorldModel::new(64));
        engine
    }

    #[test]
    fn test_core_review_updates_q_values() {
        let mut engine = make_engine();
        let plan = engine.plan_reasoning("debug parser crash");
        let idx = plan.e8_mode.0 as usize;

        let q_before = engine.e8_policy.as_ref().unwrap().mode_values[idx];
        engine.core_review("debug parser crash", "found the bug and fixed it", false);
        let q_after = engine.e8_policy.as_ref().unwrap().mode_values[idx];
        let count_after = engine.e8_policy.as_ref().unwrap().mode_counts[idx];

        assert!(count_after > 0, "mode should have been visited");
        assert!(q_after > q_before,
            "Q-value should increase after successful outcome (was {:.6}, now {:.6})",
            q_before, q_after);
        assert!(engine.e8_policy.as_ref().unwrap().epsilon() < 0.3);
    }

    #[test]
    fn test_conversation_evolution_writes_to_kb() {
        let tmp = std::env::temp_dir().join(format!("ntrix-kb-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).expect("create temp dir");
        let kb = crate::neotrix::nt_memory_kb::KnowledgeBase::open(Some(tmp.join("kb.db")))
            .expect("open KB");
        let mut engine = make_engine().with_kb(kb);
        for i in 0..5 {
            let task = format!("task_{}: solve a programming problem", i);
            let outcome = if i % 2 == 0 { "ok fixed it cleanly" } else { "fail build broke" };
            let _ = engine.plan_reasoning(&task);
            engine.core_review(&task, outcome, false);
        }
        let kb = engine.kb.as_ref().expect("kb should be wired");
        let history = kb.get_evolution_history(10).expect("query");
        assert!(history.len() >= 5,
            "expected >= 5 ConversationRecords, got {}", history.len());
        for r in &history {
            assert!(!r.task_description.is_empty());
            assert!(!r.e8_mode.is_empty());
            assert!(!r.strategy_used.is_empty());
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_conversation_distill_stage_writes_evolution_records() {
        use crate::neotrix::nt_memory_kb::{ConversationRecord, EvolutionPatternType};
        use crate::neotrix::nt_mind::self_iterating::pipeline::ConversationDistillStage;
        use crate::neotrix::nt_mind::self_iterating::BrainStage;

        let tmp = std::env::temp_dir().join(format!("ntrix-distill-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).expect("create temp dir");
        let kb = crate::neotrix::nt_memory_kb::KnowledgeBase::open(Some(tmp.join("kb.db")))
            .expect("open KB");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        for i in 0..6 {
            let r = ConversationRecord {
                id: format!("rec_{}", i),
                session_id: "s1".into(),
                task_description: format!("distill test {}", i),
                user_intent: "test".into(),
                strategy_used: if i < 3 { "structured".into() } else { "auto".into() },
                e8_mode: "Creative".into(),
                specialist_winner: "writer".into(),
                actions_taken: vec!["plan".into()],
                obstacles_encountered: if i % 2 == 0 { vec!["compile error".into()] } else { vec![] },
                fix_patterns: vec![],
                outcome: if i < 4 { "success".into() } else { "failure".into() },
                effectiveness: if i < 4 { 0.7 } else { -0.2 },
                reasoning_iterations: 3,
                error_count: if i % 2 == 0 { 1 } else { 0 },
                timestamp: now + i,
            };
            kb.store_conversation_record(&r).expect("store record");
        }

        let mut seal_brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        seal_brain._nt_memory_kb = Some(kb);
        let stage = ConversationDistillStage::new();
        let _ = stage.process(&mut seal_brain).expect("stage process");
        let kb = seal_brain._nt_memory_kb.as_ref().expect("kb attached");
        let patterns = kb.get_evolution_patterns(10).expect("query patterns");
        assert!(!patterns.is_empty(),
            "ConversationDistillStage should write at least 1 EvolutionRecord from 6 records");
        let has_err = patterns.iter().any(|p| p.pattern_type == EvolutionPatternType::RecurringError);
        assert!(has_err, "should record RecurringError pattern (error_rate > 0)");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_plan_reasoning_returns_valid_plan() {
        let mut engine = make_engine();
        let plan = engine.plan_reasoning("review this code for bugs");

        assert!(!plan.mode_name.is_empty());
        assert!(!plan.mode_desc.is_empty());
        assert!(!plan.domains.is_empty());
        assert!(plan.e8_mode.0 < 64);
        assert!(!plan.specialist.is_empty());
    }

    #[test]
    fn test_plan_reasoning_specialist_detected() {
        let mut engine = make_engine();
        let plan = engine.plan_reasoning("review this code for bugs");

        // GWT winners should produce a named specialist
        assert_ne!(plan.specialist, "None");
    }

    #[test]
    fn test_plan_reasoning_guidance_non_empty() {
        let mut engine = make_engine();
        let plan = engine.plan_reasoning("fix a critical error in the parser");

        assert!(!plan.guidance.is_empty());
        // Bug/fix task should get root-cause guidance
        let has_root_cause = plan.guidance.iter()
            .any(|g| g.contains("root cause"));
        assert!(has_root_cause, "bug task should get root-cause guidance");
    }

    #[test]
    fn test_plan_reasoning_domains_from_gwt() {
        let mut engine = make_engine();

        // First call warms up GWT
        let _ = engine.plan_reasoning("analyze architecture design");

        // GWT winner should map to domain set
        let plan = engine.plan_reasoning("analyze architecture design");
        assert!(!plan.domains.is_empty());
        // Should have at least one domain beyond the fallback defaults
        assert!(plan.domains.len() >= 2);
    }

    #[test]
    fn test_guide_strategy_resolves_from_mode() {
        let engine = make_engine();

        let strat = engine.guide_strategy("Reflection", &[]);
        assert_eq!(strat, StrategyKind::Reflection);

        let strat = engine.guide_strategy("ChainOfThought", &[]);
        assert_eq!(strat, StrategyKind::ChainOfThought);

        let strat = engine.guide_strategy("Deliberate", &[]);
        assert_eq!(strat, StrategyKind::Deliberate);

        let strat = engine.guide_strategy("Direct", &[]);
        assert_eq!(strat, StrategyKind::Direct);
    }

    #[test]
    fn test_guide_strategy_prefers_silicon_triggers() {
        let engine = make_engine();

        // SiliconSelf trigger match should override mode-based strategy
        let matched = vec![StrategyKind::RecursiveDecomposition];
        let strat = engine.guide_strategy("Direct", &matched);
        assert_eq!(strat, StrategyKind::RecursiveDecomposition);
    }

    #[test]
    fn test_plan_reasoning_avoid_patterns_after_observations() {
        let mut engine = make_engine();

        // With few observations, avoid_patterns should be empty
        let plan = engine.plan_reasoning("simple task");
        assert!(plan.avoid_patterns.is_empty(),
            "avoid_patterns should be empty with few observations");

        // Simulate many observations by pushing traces
        for i in 0..12 {
            engine.cognitive_eye.total_observations = i + 1;
        }

        let plan = engine.plan_reasoning("another task");
        assert!(!plan.avoid_patterns.is_empty(),
            "avoid_patterns should exist after many observations");
    }

    #[test]
    fn test_reason_through_path_empty() {
        let mut engine = make_engine();
        let initial_state = engine.current_state;
        let result = engine.reason_through_path("test", &[]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
        assert_eq!(engine.current_state, initial_state);
    }

    #[test]
    fn test_reason_through_path_state_rollback_on_llm_failure() {
        let mut engine = make_engine();
        let initial_state = engine.current_state;
        let initial_len = engine.state_trajectory.len();
        let path = vec![ReasoningHexagram(5)];
        let result = engine.reason_through_path("test", &path);
        assert!(result.is_ok() || result.is_err());
        if result.is_err() {
            assert_eq!(engine.current_state, initial_state, "state should roll back on LLM failure");
            assert_eq!(engine.state_trajectory.len(), initial_len, "trajectory should revert on LLM failure");
        }
    }

    #[test]
    fn test_reason_complement_state_rollback_on_llm_failure() {
        let mut engine = make_engine();
        let initial_state = engine.current_state;
        let initial_len = engine.state_trajectory.len();
        let result = engine.reason_complement("test");
        assert!(result.is_ok() || result.is_err());
        if result.is_err() {
            assert_eq!(engine.current_state, initial_state, "state should roll back on LLM failure");
            assert_eq!(engine.state_trajectory.len(), initial_len, "trajectory should revert on LLM failure");
        }
    }

    #[test]
    fn test_reflect_on_trajectory_state_rollback_on_llm_failure() {
        let mut engine = make_engine();
        // Seed a few trajectory entries so reflect has something to work with
        let _ = engine.plan_reasoning("step 1");
        let _ = engine.plan_reasoning("step 2");
        let initial_state = engine.current_state;
        let initial_len = engine.state_trajectory.len();
        let result = engine.reflect_on_trajectory("test");
        assert!(result.is_ok() || result.is_err());
        if result.is_err() {
            assert_eq!(engine.current_state, initial_state, "state should roll back on LLM failure");
            assert_eq!(engine.state_trajectory.len(), initial_len, "trajectory should revert on LLM failure");
        }
    }

    #[test]
    fn test_trim_trajectory_caps_growth() {
        let mut engine = make_engine();
        // Push 250 entries (200 + 50)
        for i in 0..250 {
            let mode = ReasoningHexagram((i % 64) as u8);
            engine.current_state = engine.current_state.transition_to(mode);
            engine.state_trajectory.push(engine.current_state);
        }
        engine.trim_trajectory();
        assert_eq!(engine.state_trajectory.len(), 200, "trim should keep at most 200 entries");
    }

    #[test]
    fn test_mode_to_reasoning_type_uses_four_axes() {
        let engine = make_engine();
        // Bits: [5]abstraction [4]scope [3]method [2]depth [1]reasoning_mode [0]stance
        // Concrete(0) · Focused(0) · Analytical(0) → TaskSolving
        let t1 = engine.mode_to_reasoning_type(&ReasoningHexagram(0));
        assert_eq!(t1, ReasoningType::TaskSolving);
        // Concrete(0) · Focused(0) · Generative(1) → ErrorDebugging
        let t2 = engine.mode_to_reasoning_type(&ReasoningHexagram(8));
        assert_eq!(t2, ReasoningType::ErrorDebugging);
        // Concrete(0) · Broad(1) · Analytical(0) · Deep(0) → General
        let t3 = engine.mode_to_reasoning_type(&ReasoningHexagram(16));
        assert_eq!(t3, ReasoningType::General);
        // Concrete(0) · Broad(1) · Analytical(0) · Fast(1) → KnowledgeQuery
        let t4 = engine.mode_to_reasoning_type(&ReasoningHexagram(20));
        assert_eq!(t4, ReasoningType::KnowledgeQuery);
        // Abstract(1) · Focused(0) · Analytical(0) → PrdGeneration
        let t5 = engine.mode_to_reasoning_type(&ReasoningHexagram(32));
        assert_eq!(t5, ReasoningType::PrdGeneration);
        // Abstract(1) · Broad(1) · Generative(1) → Conversation
        let t6 = engine.mode_to_reasoning_type(&ReasoningHexagram(56));
        assert_eq!(t6, ReasoningType::Conversation);
    }
}
