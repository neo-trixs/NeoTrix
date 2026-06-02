//! 推理引擎类型定义（从 reasoning_engine.rs 拆分）
//!
//! 包含: CascadeConfig, CascadeResult, ReasoningType, ReasoningMethod,
//!       PerspectiveLens, ReasoningTrace, ReasoningStats

use serde::{Deserialize, Serialize};
use super::model_router::ModelTier;

/// Cascade 推理配置（来自 Wildfire SMoL 的多级联推理模式）
/// 类比：450M 模型做 fast classify，置信度低才升级到 full reason
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeConfig {
    pub enabled: bool,
    pub fast_max_tokens: u32,
    pub fast_context_size: u32,
    pub confidence_threshold: f64,
    pub deep_context_size: u32,
}

impl Default for CascadeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fast_max_tokens: 256,
            fast_context_size: 4096,
            confidence_threshold: 0.7,
            deep_context_size: 16384,
        }
    }
}

/// Cascade 推理结果
#[derive(Debug, Clone)]
pub struct CascadeResult {
    pub fast_response: String,
    pub escalated: bool,
    pub deep_response: Option<String>,
    pub confidence: f64,
    pub final_output: String,
}

/// 推理类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReasoningType {
    Conversation,
    TaskSolving,
    ErrorDebugging,
    KnowledgeQuery,
    General,
    PrdGeneration,
}

/// 推理方法（来自 qiaomu-heavyskill 的 8 种方法）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReasoningMethod {
    Direct,
    FirstPrinciples,
    Adversarial,
    EdgeCaseFocus,
    ConstraintPropagation,
    ReverseEngineering,
    HistoricalEmpirical,
    Analogical,
}

impl ReasoningMethod {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Direct, Self::FirstPrinciples, Self::Adversarial,
            Self::EdgeCaseFocus, Self::ConstraintPropagation,
            Self::ReverseEngineering, Self::HistoricalEmpirical, Self::Analogical,
        ]
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Direct => "最自然的方法（基线）",
            Self::FirstPrinciples => "分解为基本原则并重建",
            Self::Adversarial => "假设显而易见的答案是错误的",
            Self::EdgeCaseFocus => "从边界条件向内推理",
            Self::ConstraintPropagation => "从不可能的事情开始",
            Self::ReverseEngineering => "从期望的结果开始，逆向工作",
            Self::HistoricalEmpirical => "可比较的证据表明了什么",
            Self::Analogical => "映射到理解良好的领域",
        }
    }
}

/// 视角透镜（来自 qiaomu-heavyskill 的 8 种视角）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PerspectiveLens {
    Builder,
    Architect,
    Skeptic,
    User,
    Economist,
    Historian,
    Contrarian,
    Ethicist,
}

impl PerspectiveLens {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Builder, Self::Architect, Self::Skeptic, Self::User,
            Self::Economist, Self::Historian, Self::Contrarian, Self::Ethicist,
        ]
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Builder => "关心交付速度、实用性，害怕过度工程",
            Self::Architect => "关心长期可维护性、可扩展性，害怕技术债务",
            Self::Skeptic => "关心风险、失败模式、隐藏成本，害怕虚假自信",
            Self::User => "关心体验、清晰度、现实适用性，害怕理论解决方案",
            Self::Economist => "关心投资回报率、权衡、机会成本，害怕沉没成本陷阱",
            Self::Historian => "关心类似情况下以前发生过什么，害怕重蹈覆辙",
            Self::Contrarian => "关心每个人都在忽略什么，害怕群体思维",
            Self::Ethicist => "关心二阶效应谁会受到伤害，害怕狭隘优化",
        }
    }
}

/// 推理轨迹 — 每次推理的完整记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningTrace {
    pub id: String,
    pub reasoning_type: ReasoningType,
    pub reasoning_method: Option<ReasoningMethod>,
    pub perspective_lens: Option<PerspectiveLens>,
    pub task: String,
    pub prompt: String,
    pub llm_response: String,
    pub error_context: Option<String>,
    pub outcome_score: f64,
    pub success: bool,
    pub timestamp: i64,
}

/// Context tier based on LLM context window size (inspired by codegraph-rust)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContextTier {
    Small,
    Medium,
    Large,
    Massive,
}

impl ContextTier {
    /// 从 ModelTier 映射到 ContextTier
    pub fn from_model_tier(tier: ModelTier) -> Self {
        match tier {
            ModelTier::T0 => Self::Small,
            ModelTier::T1 => Self::Small,
            ModelTier::T2 => Self::Medium,
            ModelTier::T3 => Self::Large,
            ModelTier::T4 => Self::Massive,
        }
    }

    pub fn from_window(window_tokens: usize) -> Self {
        match window_tokens {
            0..=50_000 => Self::Small,
            50_001..=150_000 => Self::Medium,
            150_001..=500_000 => Self::Large,
            _ => Self::Massive,
        }
    }

    pub fn max_tool_calls(&self) -> usize {
        match self {
            Self::Small => 3,
            Self::Medium => 5,
            Self::Large => 6,
            Self::Massive => 8,
        }
    }

    pub fn max_search_results(&self) -> usize {
        match self {
            Self::Small => 10,
            Self::Medium => 25,
            Self::Large => 50,
            Self::Massive => 100,
        }
    }

    pub fn name(&self) -> &'static str {
        match self { Self::Small => "small", Self::Medium => "medium", Self::Large => "large", Self::Massive => "massive" }
    }
}

/// Context-aware execution limits
#[derive(Debug, Clone)]
pub struct ContextAwareLimits {
    pub tier: ContextTier,
    pub context_window: usize,
    pub max_tool_calls: usize,
    pub max_search_results: usize,
    pub safe_output_tokens: usize,
}

impl ContextAwareLimits {
    pub fn new(context_window: usize) -> Self {
        let tier = ContextTier::from_window(context_window);
        Self {
            tier,
            context_window,
            max_tool_calls: tier.max_tool_calls(),
            max_search_results: tier.max_search_results(),
            safe_output_tokens: (context_window as f64 * 0.85) as usize,
        }
    }
}
