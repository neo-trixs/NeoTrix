pub mod agent;
pub mod content;
pub mod monetize;
pub mod pipeline;
pub mod publisher;
pub mod tracker;
pub mod video;

// ── 新增三大核心模块 ──
pub mod financial_abstraction;
pub mod knowledge_arbitrage;
pub mod wealth_model;

pub use agent::{AgentState, CycleResult, EarnAgent};
pub use content::{
    plan_for_video, ContentPlan, ContentPlanner, ContentTopic, StrategyConfig, VideoScene,
    VideoScript, VideoScriptPlanner,
};
pub use monetize::{AiToEarnBridge, AiToEarnConfig};
pub use pipeline::CycleReport;
pub use publisher::{
    default_registry, BrowserPublisher, CliPublisher, ContentMeta, ContentType, HttpPublisher,
    PublishResult, Publisher, PublisherRegistry,
};
pub use tracker::{EarnStats, EarningsRecord, RewardSignal};
pub use video::{
    synthesize_speech, EdgeTtsBackend, FfmpegRenderer, LocalMediaSource, MediaClip, MediaSource,
    PexelsSource, RenderConfig, TtsEngine, VideoPipeline,
};

// ── 新模块导出 ──
pub use financial_abstraction::{
    AbstractionLayer, DerivativeBook, FinancialAbstractionStack, LiquidityNetwork,
    SecuritizationPool, StrategyType, YieldStrategy,
};
pub use knowledge_arbitrage::{
    ArbitrageOpportunity, ArbitrageSourceType, AttentionAsset, EntityRelation, ExploitMethod,
    InformationSource, KnowledgeArbitrageEngine, KnowledgeEntity, KnowledgeGraph,
};
pub use wealth_model::{
    ActiveStrategy, ArbitrageStrategy, AssetClass, CapitalSource, CapitalStack, CompoundingTracker,
    InfoAdvantage, LeverageProfile, NetworkType, RegulatoryBarrier, WealthMechanism, WealthModel,
};

use crate::neotrix::nt_io_provider::factory::{create_provider, ProviderConfig};
use crate::neotrix::nt_io_provider::types::LlmProvider;

/// 轻量 LLM 引擎
pub struct LlmEngine {
    pub llm: Box<dyn LlmProvider>,
    pub runtime: tokio::runtime::Runtime,
    pub model: String,
}

impl LlmEngine {
    pub fn from_env() -> Self {
        let config = ProviderConfig::from_env();
        let model = config
            .model
            .clone()
            .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
        let llm = create_provider(config);
        let runtime =
            tokio::runtime::Runtime::new().expect("LlmEngine: failed to create tokio runtime");
        Self {
            llm,
            runtime,
            model,
        }
    }
}
