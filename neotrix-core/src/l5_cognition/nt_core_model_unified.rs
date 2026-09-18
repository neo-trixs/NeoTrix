// Unified Model Interface — Neotrix CLI通用模型适配框架
//
// Provides a unified interface for all external model adapters.
// Adapt any external model (OpenAI, Anthropic, Gemini, DeepSeek, Ollama, etc.)
// through this standardized contract.
//
// ## Design Rationale
// - **R-P79**: External tech absorption wired to production path
// - **A1**: Cost-aware routing — not all tasks need the strongest model
// - **P5**: Skill as reusable template — composable atoms with strict interfaces

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export canonical TaskType from neotrix-types (R-P42: single source of truth)
pub use neotrix_types::core::TaskType;

/// Model preferences — user preferences for model selection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelPreferences {
    /// Allowed model providers (empty = all allowed)
    pub allowed_providers: Vec<String>,
    /// Forbidden model providers
    pub forbidden_providers: Vec<String>,
    /// Required capabilities for the task
    pub required_capabilities: Vec<Capability>,
    /// Capabilities to avoid
    pub avoided_capabilities: Vec<Capability>,
    /// Maximum cost per query (USD)
    pub max_cost_per_query: Option<f64>,
    /// Latency tolerance in milliseconds
    pub latency_tolerance: u64,
    /// Quality priority (0.0 = cost priority, 1.0 = quality priority)
    pub quality_priority: f32,
}

/// 统一模型请求 — 所有外部模型的标准输入格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequest {
    /// 任务类型 — determines routing and model selection
    pub task_type: TaskType,
    /// 用户内容
    pub content: String,
    /// 对话上下文片段 (for context窗口管理)
    pub context_fragments: Vec<String>,
    /// 成本预算 (USD; None = use default/gateway budget)
    pub cost_budget: Option<f64>,
    /// 最大延迟 (ms; None = use default)
    pub latency_tolerance: Option<u64>,
    /// 模型偏好
    pub preferences: ModelPreferences,
    /// 请求唯一标识
    pub request_id: String,
    /// 元数据 (模型特定扩展)
    pub metadata: HashMap<String, String>,
}

/// 统一模型响应 — 所有外部模型的标准输出格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    /// 实际使用的模型ID
    pub model_used: String,
    /// 模型提供者
    pub provider: String,
    /// 输出内容
    pub output: String,
    /// 成本 (USD)
    pub cost: f64,
    /// 延迟 (ms)
    pub latency_ms: u64,
    /// 是否降级 (通过fallback chain)
    pub degraded: bool,
    /// 降级原因
    pub degradation_reason: Option<String>,
    /// 任务类型
    pub task_type: TaskType,
    /// Token使用情况
    pub tokens: TokenUsage,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// Token使用情况
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    /// 输入tokens
    pub input: usize,
    /// 输出tokens
    pub output: usize,
    /// 总tokens
    pub total: usize,
}

/// 能力 — 模型支持的功能集合
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Capability {
    /// 基础聊天
    Chat,
    /// 代码生成
    CodeGeneration,
    /// 数学推理
    Reasoning,
    /// 研究分析
    Research,
    /// 创意写作
    Creative,
    /// 多模态 (vision)
    Multimodal,
    /// 音频处理
    Audio,
    /// 长上下文
    LongContext(u32), // context window size
    /// 工具使用
    ToolUsage,
    /// 结构化输出
    StructuredOutput,
    /// 自我修正
    SelfCorrection,
    /// 自定义能力
    Custom(String),
}

impl Capability {
    /// Check if capability meets minimum requirement
    pub fn meets_minimum(&self, required: &[Capability]) -> bool {
        if required.is_empty() {
            return true;
        }
        required.iter().all(|r| match (self, r) {
            (Capability::LongContext(_), Capability::LongContext(_)) => true,
            (c, r) => format!("{:?}", c) == format!("{:?}", r),
        })
    }
}

/// Model metadata — used for cost-aware routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMeta {
    /// 模型成本权重 (每1K tokens USD; lower = cheaper)
    pub cost_weight: f64,
    /// 模型 tier (Fast/Balanced/Powerful)
    pub tier: ModelTier,
    /// 上下文窗口大小
    pub context_window: u32,
    /// 支持的能力
    pub capabilities: Vec<Capability>,
    /// 预估延迟 (ms)
    pub estimated_latency_ms: u64,
    /// 可靠性 (0.0 - 1.0)
    pub reliability: f32,
}

/// Model tier classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelTier {
    /// Fast, cheap, good for simple tasks
    Fast,
    /// Balanced cost/quality for everyday work
    Balanced,
    /// Most capable, expensive for complex tasks
    Powerful,
    /// Specialized for specific domains
    Specialized(String),
}

/// Token成本估算
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CostEstimate {
    /// 输入token成本 (USD)
    pub input_cost: f64,
    /// 输出token成本 (USD)
    pub output_cost: f64,
    /// 总成本 (USD)
    pub total_cost: f64,
    /// 预算剩余后的成本
    pub budget_remaining_after: f64,
}

/// GWT Salience Calculation — Cost-aware salience for attention routing
///
/// Formula: salience = importance / (cost_weight × tokens + ε) × latency_weight
///
/// Where:
/// - importance: task importance (0.0 - 1.0, from user preferences or context)
/// - cost_weight: model-specific cost per 1K tokens (from ModelMeta)
/// - tokens: estimated token count
/// - latency_weight: latency tolerance normalization factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalienceScore {
    /// 原始 salience 分数
    pub score: f64,
    /// 成本权重因子
    pub cost_factor: f64,
    /// 延迟因子
    pub latency_factor: f64,
    /// 任务重要性
    pub importance: f64,
}

impl SalienceScore {
    /// Calculate salience for GWT cost-aware routing
    ///
    /// # Formula
    /// `salience = importance / (cost_weight * tokens + 0.1) * latency_factor`
    ///
    /// The `0.1` epsilon prevents division by zero and slightly favors
    /// cheaper models for equally important tasks.
    ///
    /// # Example
    /// ```
    /// use neotrix_core::l5_cognition::nt_core_model_unified::*;
    ///
    /// let score = SalienceScore::calculate(
    ///     importance = 0.9,
    ///     cost_weight = 0.003,  // Claude Sonnet ~$3/1K
    ///     tokens = 1000,
    ///     latency_tolerance = 1000,  // 1 second
    /// );
    /// assert!(score.score > 0.0);
    /// ```
    pub fn calculate(
        importance: f64,
        cost_weight: f64,
        tokens: usize,
        latency_tolerance: u64,
    ) -> Self {
        // Normalize latency tolerance to weight (0.5 - 2.0 range)
        let latency_weight = match latency_tolerance {
            0..=100 => 0.5,      // Must be fast
            101..=500 => 0.8,    // Can wait a few seconds
            501..=2000 => 1.2,   // Can wait for complex processing
            2001..=10000 => 1.5, // Can wait for deep reasoning
            _ => 2.0,            // Can wait as long as needed
        };

        // Estimate cost factor
        let cost_factor = cost_weight * (tokens as f64 / 1000.0) + 0.1;

        let score = importance / cost_factor * latency_weight;

        Self {
            score,
            cost_factor,
            latency_factor: latency_weight,
            importance,
        }
    }

    /// Compare two salience scores (higher = more salient)
    pub fn compare(&self, other: &Self) -> std::cmp::Ordering {
        self.score
            .partial_cmp(&other.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_salience_calculation() {
        // High importance task with cheap model
        let cheap = SalienceScore::calculate(0.8, 0.001, 1000, 500);

        // Low importance task with expensive model
        let expensive = SalienceScore::calculate(0.3, 0.03, 1000, 2000);

        // Cheap model should have higher salience for same importance
        assert!(
            cheap.score > expensive.score
                || (cheap.score == expensive.score && cheap.cost_factor < expensive.cost_factor)
        );
    }

    #[test]
    fn test_salience_importance_weight() {
        // Higher importance = higher salience (all else equal)
        let high = SalienceScore::calculate(1.0, 0.01, 500, 500);
        let low = SalienceScore::calculate(0.5, 0.01, 500, 500);
        assert!(high.score > low.score);
    }

    #[test]
    fn test_salience_cost_weight() {
        // Lower cost weight = higher salience (all else equal)
        let cheap = SalienceScore::calculate(0.5, 0.001, 1000, 500);
        let expensive = SalienceScore::calculate(0.5, 0.01, 1000, 500);
        assert!(cheap.score > expensive.score);
    }

    #[test]
    fn test_salience_tokens() {
        // More tokens = lower salience (all else equal), favors cheaper/short tasks
        let many_tokens = SalienceScore::calculate(0.5, 0.01, 4000, 500);
        let few_tokens = SalienceScore::calculate(0.5, 0.01, 100, 500);
        // More tokens should give lower salience (cheaper to do small tasks)
        assert!(many_tokens.score <= few_tokens.score);
    }

    #[test]
    fn test_model_meta_creation() {
        let meta = ModelMeta {
            cost_weight: 0.003,
            tier: ModelTier::Balanced,
            context_window: 200000,
            capabilities: vec![
                Capability::Chat,
                Capability::CodeGeneration,
                Capability::Reasoning,
            ],
            estimated_latency_ms: 800,
            reliability: 0.97,
        };

        assert_eq!(meta.cost_weight, 0.003);
        assert_eq!(meta.tier, ModelTier::Balanced);
        assert_eq!(meta.context_window, 200000);
        assert!(meta.capabilities.contains(&Capability::CodeGeneration));
    }

    #[test]
    fn test_model_request_creation() {
        let req = ModelRequest {
            task_type: TaskType::CodeGeneration,
            content: "implement a binary search".to_string(),
            context_fragments: vec!["sorted array".to_string()],
            cost_budget: Some(0.01),
            latency_tolerance: Some(2000),
            preferences: ModelPreferences {
                required_capabilities: vec![Capability::CodeGeneration],
                ..Default::default()
            },
            request_id: "req-001".to_string(),
            metadata: HashMap::new(),
        };

        assert_eq!(req.task_type, TaskType::CodeGeneration);
        assert_eq!(req.content, "implement a binary search");
        assert_eq!(req.cost_budget, Some(0.01));
        assert_eq!(req.latency_tolerance, Some(2000));
    }

    #[test]
    fn test_model_response_creation() {
        let resp = ModelResponse {
            model_used: "gpt-4o".to_string(),
            provider: "openai".to_string(),
            output: "Here's a binary search...".to_string(),
            cost: 0.005,
            latency_ms: 500,
            degraded: false,
            degradation_reason: None,
            task_type: TaskType::CodeGeneration,
            tokens: TokenUsage {
                input: 50,
                output: 20,
                total: 70,
            },
            metadata: HashMap::new(),
        };

        assert_eq!(resp.model_used, "gpt-4o");
        assert_eq!(resp.provider, "openai");
        assert_eq!(resp.cost, 0.005);
        assert_eq!(resp.latency_ms, 500);
        assert!(!resp.degraded);
    }

    #[test]
    fn test_capability_meets_minimum() {
        let cap = Capability::CodeGeneration;
        let required = vec![Capability::CodeGeneration];
        assert!(cap.meets_minimum(&required));

        let cap = Capability::Chat;
        assert!(!cap.meets_minimum(&required));
    }

    #[test]
    fn test_cost_estimate_creation() {
        let cost = CostEstimate {
            input_cost: 0.001,
            output_cost: 0.004,
            total_cost: 0.005,
            budget_remaining_after: 0.995,
        };

        assert_eq!(cost.total_cost, 0.005);
        assert_eq!(cost.input_cost, 0.001);
        assert_eq!(cost.output_cost, 0.004);
    }
}
