pub mod agent;
pub mod content;
pub mod publisher;
pub mod pipeline;
pub mod tracker;
pub mod monetize;
pub mod video;

// ── 新增三大核心模块 ──
pub mod wealth_model;
pub mod financial_abstraction;
pub mod knowledge_arbitrage;

pub use agent::{EarnAgent, AgentState, CycleResult};
pub use content::{
    ContentPlanner, ContentPlan, ContentTopic, StrategyConfig,
    VideoScript, VideoScene, VideoScriptPlanner, plan_for_video,
};
pub use publisher::{
    Publisher, PublisherRegistry, PublishResult, ContentMeta, ContentType,
    CliPublisher, HttpPublisher, BrowserPublisher, default_registry,
};
pub use tracker::{EarnStats, EarningsRecord, RewardSignal};
pub use pipeline::CycleReport;
pub use video::{
    TtsEngine, EdgeTtsBackend, synthesize_speech,
    MediaSource, PexelsSource, LocalMediaSource, MediaClip,
    FfmpegRenderer, RenderConfig, VideoPipeline,
};
pub use monetize::{AiToEarnBridge, AiToEarnConfig};

// ── 新模块导出 ──
pub use wealth_model::{
    WealthModel, WealthMechanism, CapitalStack, CapitalSource,
    CompoundingTracker, LeverageProfile, ActiveStrategy,
    NetworkType, InfoAdvantage, RegulatoryBarrier, AssetClass, ArbitrageStrategy,
};
pub use financial_abstraction::{
    FinancialAbstractionStack, AbstractionLayer, SecuritizationPool,
    LiquidityNetwork, DerivativeBook, YieldStrategy, StrategyType,
};
pub use knowledge_arbitrage::{
    KnowledgeArbitrageEngine, InformationSource, SourceType,
    ArbitrageOpportunity, ExploitMethod, AttentionAsset,
    KnowledgeGraph, KnowledgeEntity, EntityRelation,
};

use crate::neotrix::provider::factory::{ProviderConfig, create_provider};
use crate::neotrix::provider::types::LlmProvider;

/// 轻量 LLM 引擎
pub struct LlmEngine {
    pub llm: Box<dyn LlmProvider>,
    pub runtime: tokio::runtime::Runtime,
    pub model: String,
}

impl LlmEngine {
    pub fn from_env() -> Self {
        let config = ProviderConfig::from_env();
        let model = config.model.clone().unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
        let llm = create_provider(config);
        let runtime = tokio::runtime::Runtime::new().expect("LlmEngine: failed to create tokio runtime");
        Self { llm, runtime, model }
    }
}
