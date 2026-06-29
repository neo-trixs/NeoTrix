use tokio::runtime::Runtime;

use crate::core::nt_core_self::{CrystalRegistry, SiliconSelfModel};
use crate::core::{
    strategy_matrix, E8Policy, E8TransitionLearner, FullReasoningState, MetaState, OneObserver,
    ReasoningHexagram,
};
use crate::neotrix::nt_expert_routing::workspace::GlobalWorkspace;
use crate::neotrix::nt_io_provider::factory::{create_provider, ProviderConfig};
use crate::neotrix::nt_io_provider::types::LlmProvider;
use crate::neotrix::nt_memory_kb::KnowledgeBase;
use crate::neotrix::nt_mind::context_artifacts::ArtifactIndexer;
use crate::neotrix::nt_mind::core::BrainMutView;
use crate::neotrix::nt_mind::memory::ReasoningBank;
use crate::neotrix::nt_mind::model_router::ModelRouter;
use crate::neotrix::nt_mind::reasoning_engine::cognitive_observer::CognitiveEye;
use crate::neotrix::nt_mind::reasoning_engine::markov_check::MarkovCheck;
use crate::neotrix::nt_mind::reasoning_engine::reasoning_distiller::ReasoningDistiller;
use crate::neotrix::nt_mind::thinking_bridge::ThinkingBridge;
use crate::neotrix::nt_world_jepa::{JepaWorldModel, WorldModelState};
use crate::neotrix::nt_world_search::WebSearchTool;

use super::ReasoningEngine;

impl ReasoningEngine {
    pub fn from_env(brain: Box<dyn BrainMutView>, bank: ReasoningBank) -> Self {
        let config = ProviderConfig::from_env();
        let model = config
            .model
            .clone()
            .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
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
            reasoning_distiller: ReasoningDistiller::new(),
            markov_check: MarkovCheck::new(),
            last_llm_tagged: None,
            self_model: None,
            context_budget: None,
        }
    }

    pub fn new(
        llm: Box<dyn LlmProvider>,
        brain: Box<dyn BrainMutView>,
        bank: ReasoningBank,
        model: &str,
    ) -> Self {
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
            reasoning_distiller: ReasoningDistiller::new(),
            markov_check: MarkovCheck::new(),
            last_llm_tagged: None,
            self_model: None,
            context_budget: None,
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

    pub fn with_self_model(mut self, model: String) -> Self {
        self.self_model = Some(model);
        self
    }
}
