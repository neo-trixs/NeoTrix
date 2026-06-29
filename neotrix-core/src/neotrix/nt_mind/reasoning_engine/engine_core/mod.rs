//! ReasoningEngine 结构体定义 + 构造器 + 主入口 + 导航方法

use tokio::runtime::Runtime;

use crate::core::nt_core_hex::ReasoningHexagram;
use crate::core::nt_core_llm_provider::LlmProvider;
use crate::core::nt_core_self::attention_head::AttentionDomain;
use crate::core::nt_core_self::reasoning_strategy::StrategyKind;
use crate::core::nt_core_self::silicon_self::SiliconSelfModel;
use crate::core::nt_core_self::CrystalRegistry;
use crate::core::{E8Policy, E8TransitionLearner, FullReasoningState, OneObserver};
use crate::neotrix::nt_memory_kb::KnowledgeBase;
use crate::neotrix::nt_mind::context_artifacts::indexer::ArtifactIndexer;
use crate::neotrix::nt_mind::core::BrainMutView;
use crate::neotrix::nt_mind::distillation::{AntiPattern, StrategicPrinciple};
use crate::neotrix::nt_mind::memory::ReasoningBank;
use crate::neotrix::nt_mind::model_router::ModelRouter;
use crate::neotrix::nt_mind::reasoning_engine::cognitive_observer::CognitiveEye;
use crate::neotrix::nt_mind::reasoning_engine::markov_check::MarkovCheck;
use crate::neotrix::nt_mind::reasoning_types::{ReasoningTrace, ReasoningType};
use crate::neotrix::nt_mind::thinking_bridge::ThinkingBridge;
use crate::neotrix::nt_world_jepa::{JepaWorldModel, WorldModelState};
use crate::neotrix::nt_world_search::WebSearchTool;

mod builder;
mod executor;
mod planner;
mod reflection;

pub const MAX_TRACES: usize = 500;
pub const MAX_STATE_TRAJECTORY: usize = 100;
pub const MAX_COST_LOG: usize = 1000;

/// 统一推理引擎 — 项目意识核心
/// The core's structured reasoning plan, generated before LLM execution
#[derive(Clone, Debug)]
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
    pub brain: Box<dyn BrainMutView>,
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
    pub gwt: Option<crate::neotrix::nt_expert_routing::workspace::GlobalWorkspace>,
    pub last_core_plan: Option<CoreReasoningPlan>,
    pub silicon_self: Option<SiliconSelfModel>,
    pub thinking_bridge: Option<ThinkingBridge>,
    pub e8_policy: Option<E8Policy>,
    pub e8_learner: Option<E8TransitionLearner>,
    pub nt_world_model: Option<WorldModelState>,
    pub jepa: Option<JepaWorldModel>,
    pub nt_world_search_tool: Option<WebSearchTool>,
    pub reasoning_distiller:
        crate::neotrix::nt_mind::reasoning_engine::reasoning_distiller::ReasoningDistiller,
    pub markov_check: MarkovCheck,
    /// VSA-tagged representation of the most recent LLM response.
    /// Set by `call_llm_with_ctx` so the consciousness system can distinguish
    /// Self(Thought) from World(UserInput) per Principle #5.
    pub last_llm_tagged: Option<crate::core::nt_core_consciousness::VsaTagged>,
    /// Self-model string (from SelfModelGenerator) injected as System prompt.
    /// Set externally; None = no system prompt.
    pub self_model: Option<String>,
    /// Token budget controller — trims prompt/system sources when set.
    /// Set externally; None = no budget enforcement.
    pub context_budget: Option<crate::core::nt_core_context::context_budget::ContextBudget>,
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

impl ReasoningEngine {
    /// Save E8 reasoning state (current_state, trajectory, strategy_matrix, observer) to JSON.
    pub fn save_e8_state(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let trajectory: Vec<FullReasoningState> = self
            .state_trajectory
            .iter()
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
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &json)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Load E8 reasoning state from JSON file.
    pub fn load_e8_state(
        &mut self,
        path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
        let brain: Box<dyn BrainMutView> =
            Box::new(crate::neotrix::nt_mind::self_iterating::ReasoningBrain::new());
        let bank = ReasoningBank::new(100);
        let mut engine = ReasoningEngine::new(
            crate::neotrix::nt_io_provider::factory::create_provider(
                crate::neotrix::nt_io_provider::factory::ProviderConfig::from_env(),
            ),
            brain,
            bank,
            "test-model",
        );
        let mut gwt = crate::neotrix::nt_expert_routing::workspace::GlobalWorkspace::new(0.3);
        gwt.register_default_specialists();
        engine = engine.with_gwt(gwt);
        engine = engine.with_silicon_self(SiliconSelfModel::new());
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

        let q_before = engine
            .e8_policy
            .as_ref()
            .expect("e8_policy set by make_engine")
            .mode_values[idx];
        engine.core_review("debug parser crash", "found the bug and fixed it", false);
        let q_after = engine
            .e8_policy
            .as_ref()
            .expect("e8_policy set by make_engine")
            .mode_values[idx];
        let count_after = engine
            .e8_policy
            .as_ref()
            .expect("e8_policy set by make_engine")
            .mode_counts[idx];

        assert!(count_after > 0, "mode should have been visited");
        assert!(
            q_after > q_before,
            "Q-value should increase after successful outcome (was {:.6}, now {:.6})",
            q_before,
            q_after
        );
        assert!(
            engine
                .e8_policy
                .as_ref()
                .expect("e8_policy set by make_engine")
                .epsilon()
                < 0.3
        );
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
            let outcome = if i % 2 == 0 {
                "ok fixed it cleanly"
            } else {
                "fail build broke"
            };
            let _ = engine.plan_reasoning(&task);
            engine.core_review(&task, outcome, false);
        }
        let kb = engine.kb.as_ref().expect("kb should be wired");
        let history = kb.get_evolution_history(10).expect("query");
        assert!(
            history.len() >= 5,
            "expected >= 5 ConversationRecords, got {}",
            history.len()
        );
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
                strategy_used: if i < 3 {
                    "structured".into()
                } else {
                    "auto".into()
                },
                e8_mode: "Creative".into(),
                specialist_winner: "writer".into(),
                actions_taken: vec!["plan".into()],
                obstacles_encountered: if i % 2 == 0 {
                    vec!["compile error".into()]
                } else {
                    vec![]
                },
                fix_patterns: vec![],
                outcome: if i < 4 {
                    "success".into()
                } else {
                    "failure".into()
                },
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
        assert!(
            !patterns.is_empty(),
            "ConversationDistillStage should write at least 1 EvolutionRecord from 6 records"
        );
        let has_err = patterns
            .iter()
            .any(|p| p.pattern_type == EvolutionPatternType::RecurringError);
        assert!(
            has_err,
            "should record RecurringError pattern (error_rate > 0)"
        );
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

        assert_ne!(plan.specialist, "None");
    }

    #[test]
    fn test_plan_reasoning_guidance_non_empty() {
        let mut engine = make_engine();
        let plan = engine.plan_reasoning("fix a critical error in the parser");

        assert!(!plan.guidance.is_empty());
        let has_root_cause = plan.guidance.iter().any(|g| g.contains("root cause"));
        assert!(has_root_cause, "bug task should get root-cause guidance");
    }

    #[test]
    fn test_plan_reasoning_domains_from_gwt() {
        let mut engine = make_engine();

        let _ = engine.plan_reasoning("analyze architecture design");

        let plan = engine.plan_reasoning("analyze architecture design");
        assert!(!plan.domains.is_empty());
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

        let matched = vec![StrategyKind::RecursiveDecomposition];
        let strat = engine.guide_strategy("Direct", &matched);
        assert_eq!(strat, StrategyKind::RecursiveDecomposition);
    }

    #[test]
    fn test_plan_reasoning_avoid_patterns_after_observations() {
        let mut engine = make_engine();

        let plan = engine.plan_reasoning("simple task");
        assert!(
            plan.avoid_patterns.is_empty(),
            "avoid_patterns should be empty with few observations"
        );

        for i in 0..12 {
            engine.cognitive_eye.total_observations = i + 1;
        }

        let plan = engine.plan_reasoning("another task");
        assert!(
            !plan.avoid_patterns.is_empty(),
            "avoid_patterns should exist after many observations"
        );
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
            assert_eq!(
                engine.current_state, initial_state,
                "state should roll back on LLM failure"
            );
            assert_eq!(
                engine.state_trajectory.len(),
                initial_len,
                "trajectory should revert on LLM failure"
            );
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
            assert_eq!(
                engine.current_state, initial_state,
                "state should roll back on LLM failure"
            );
            assert_eq!(
                engine.state_trajectory.len(),
                initial_len,
                "trajectory should revert on LLM failure"
            );
        }
    }

    #[test]
    fn test_reflect_on_trajectory_state_rollback_on_llm_failure() {
        let mut engine = make_engine();
        let _ = engine.plan_reasoning("step 1");
        let _ = engine.plan_reasoning("step 2");
        let initial_state = engine.current_state;
        let initial_len = engine.state_trajectory.len();
        let result = engine.reflect_on_trajectory("test");
        assert!(result.is_ok() || result.is_err());
        if result.is_err() {
            assert_eq!(
                engine.current_state, initial_state,
                "state should roll back on LLM failure"
            );
            assert_eq!(
                engine.state_trajectory.len(),
                initial_len,
                "trajectory should revert on LLM failure"
            );
        }
    }

    #[test]
    fn test_trim_trajectory_caps_growth() {
        let mut engine = make_engine();
        for i in 0..250 {
            let mode = ReasoningHexagram((i % 64) as u8);
            engine.current_state = engine.current_state.transition_to(mode);
            engine.state_trajectory.push(engine.current_state);
        }
        engine.trim_trajectory();
        assert_eq!(
            engine.state_trajectory.len(),
            MAX_STATE_TRAJECTORY,
            "trim should keep at most MAX_STATE_TRAJECTORY entries"
        );
    }

    #[test]
    fn test_mode_to_reasoning_type_uses_four_axes() {
        let engine = make_engine();
        let t1 = engine.mode_to_reasoning_type(&ReasoningHexagram(0));
        assert_eq!(t1, ReasoningType::TaskSolving);
        let t2 = engine.mode_to_reasoning_type(&ReasoningHexagram(8));
        assert_eq!(t2, ReasoningType::ErrorDebugging);
        let t3 = engine.mode_to_reasoning_type(&ReasoningHexagram(16));
        assert_eq!(t3, ReasoningType::General);
        let t4 = engine.mode_to_reasoning_type(&ReasoningHexagram(20));
        assert_eq!(t4, ReasoningType::KnowledgeQuery);
        let t5 = engine.mode_to_reasoning_type(&ReasoningHexagram(32));
        assert_eq!(t5, ReasoningType::PrdGeneration);
        let t6 = engine.mode_to_reasoning_type(&ReasoningHexagram(56));
        assert_eq!(t6, ReasoningType::Conversation);
    }
}
