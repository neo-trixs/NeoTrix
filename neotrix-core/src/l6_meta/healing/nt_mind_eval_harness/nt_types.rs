//! 评测共享类型 (Model/Dataset/Point/Curve/Report/Error/Judge)。

use crate::l1_action::nt_io::nt_io_provider::{LlmError, LlmProvider};
use crate::l5_cognition::nt_core_ttc::EffortTier;
use crate::l6_meta::nt_repair::nt_mind_consciousness_gold_standard::ConsciousnessLevel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// 基线模型规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    pub name: String,
    pub provider_type: String, // "vllm" | "sglang" | "ollama" | "openai" | "anthropic" 等
    pub model_id: String,
    pub base_url: Option<String>,
    pub api_key_env: Option<String>,
    pub pricing_per_1m_in: f64,  // USD per 1M input tokens
    pub pricing_per_1m_out: f64, // USD per 1M output tokens
}

/// 数据集规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetSpec {
    pub name: String,
    pub queries: Vec<EvalQuery>,
    pub judge_model: String, // e.g. "qwen3-80b-instruct" (LLM-as-judge)
    pub judge_base_url: Option<String>,
    pub judge_api_key_env: Option<String>,
    pub golden_answers: Option<HashMap<String, String>>, // query_id -> golden answer
}

/// 单条评测查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalQuery {
    pub id: String,
    pub prompt: String,
    pub category: String, // "math" | "reasoning" | "coding" | "knowledge" | "rag" | "creative"
    pub difficulty: f64,  // 0.0~1.0
    pub expected_tokens: u32, // 预估合理输出长度
}

/// 单次评测结果 (model × budget 点)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalPoint {
    pub model_name: String,
    pub budget: u32,
    pub query_id: String,
    pub response: String,
    pub actual_tokens: u32,
    pub quality_score: f64, // 0.0~1.0 (LLM judge)
    pub judge_justification: String,
    pub latency_ms: u64,
    pub cost_usd: f64,
    pub consciousness_phi: Option<f64>, // 可选：金标 φ 检测
    pub consciousness_level: Option<ConsciousnessLevel>,
}

/// 质量-成本矩阵行 (单模型全预算)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelQualityCurve {
    pub model_name: String,
    pub points: Vec<EvalPoint>,
    // 插值后的连续曲线 (用于 AUDC 计算)
    pub interpolated_quality: Vec<f64>, // 对应 DEFAULT_BUDGET_GRID
}

/// Pareto 前沿点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoPoint {
    pub model_name: String,
    pub budget: u32,
    pub quality: f64,
    pub cost_usd: f64,
}

/// 评测报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalReport {
    pub timestamp: i64,
    pub dataset_name: String,
    pub curves: Vec<ModelQualityCurve>,
    pub audc_scores: HashMap<String, f64>,  // model -> AUDC
    pub qnc_scores: HashMap<String, f64>,   // model -> QNC
    pub peak_quality: HashMap<String, f64>, // model -> Peak Quality
    pub pareto_frontier: Vec<ParetoPoint>,
    pub galaxy_vs_baseline: HashMap<String, GalaxyComparison>,
    pub summary: String,
}

/// 大阵 vs 基线对比
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyComparison {
    pub baseline_model: String,
    pub galaxy_effort_tier: EffortTier,
    pub quality_delta: f64,     // galaxy - baseline (同预算)
    pub token_savings_pct: f64, // 同质量下 galaxy 省 token %
    pub audc_improvement: f64,
}

/// Judge 规格
#[derive(Clone)]
pub(crate) struct JudgeSpec {
    pub(crate) provider: Arc<dyn LlmProvider>,
    pub(crate) model_id: String,
}

/// 评测错误类型
#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("Provider error: {0}")]
    ProviderError(LlmError),
    #[error("Judge parse error: {0}")]
    JudgeParseError(String),
    #[error("Join error: {0}")]
    JoinError(String),
    #[error("Config error: {0}")]
    ConfigError(String),
    #[error("Concurrency semaphore closed")]
    ConcurrencyClosed,
}
