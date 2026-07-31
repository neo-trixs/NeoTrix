//! Model Router — 智能模型分级路由 (SquillaRouter 风格)
//! 
//! T0-T4 五级路由：根据 prompt 特征选择合适性价比的模型
//! - T0: 问候/简单回答 (最便宜)
//! - T1: 常规对话/简单任务
//! - T2: 一般推理/中等复杂度
//! - T3: 复杂推理/代码/数学
//! - T4: 前沿模型/深度推理 (最贵)
//! 
//! 特征: prompt 长度、语言、代码占比、触发关键词、语义嵌入

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use super::reasoning_types::ContextTier;

/// 默认配置文件路径
pub fn default_config_path() -> PathBuf {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".neotrix").join("router_config.toml")
}

/// 从 TOML 文件加载路由配置
pub fn load_router_config(path: Option<PathBuf>) -> RouterConfig {
    let path = path.unwrap_or_else(default_config_path);
    if path.exists() {
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("[router] read config: {}", e);
                return RouterConfig::default();
            }
        };
        match toml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                log::warn!("[router] parse config: {}", e);
                RouterConfig::default()
            }
        }
    } else {
        RouterConfig::default()
    }
}

/// 保存路由配置到 TOML 文件
pub fn save_router_config(config: &RouterConfig, path: Option<PathBuf>) -> Result<(), String> {
    let path = path.unwrap_or_else(default_config_path);
    let toml_str = toml::to_string_pretty(config).map_err(|e| format!("序列化失败: {}", e))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    std::fs::write(&path, &toml_str).map_err(|e| format!("写入失败: {}", e))
}

/// 模型等级 (T0 = 最便宜/最快, T4 = 最贵/最强大)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ModelTier {
    T0 = 0,
    T1 = 1,
    T2 = 2,
    T3 = 3,
    T4 = 4,
}

impl ModelTier {
    pub fn name(&self) -> &'static str {
        match self {
            Self::T0 => "t0-cheap",
            Self::T1 => "t1-standard",
            Self::T2 => "t2-balanced",
            Self::T3 => "t3-powerful",
            Self::T4 => "t4-frontier",
        }
    }

    /// 从整数创建
    pub fn from_int(v: u8) -> Self {
        match v {
            0 => Self::T0, 1 => Self::T1, 2 => Self::T2, 3 => Self::T3, _ => Self::T4,
        }
    }

    /// 相对成本倍数 (相对 T0)
    pub fn cost_multiplier(&self) -> f64 {
        match self {
            Self::T0 => 1.0,
            Self::T1 => 4.0,
            Self::T2 => 12.0,
            Self::T3 => 30.0,
            Self::T4 => 60.0,
        }
    }
}

/// 路由特征 — 用于分类 prompt 到合适的 tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterFeatures {
    pub prompt_length: usize,
    pub language: LanguageType,
    pub code_ratio: f64,
    pub keyword_score: f64,
    pub has_reasoning_triggers: bool,
    pub token_estimate: usize,
    pub contains_math: bool,
    pub has_code_block: bool,
    pub contains_analysis_keywords: bool,
    pub estimated_tier: ModelTier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LanguageType {
    Chinese,
    English,
    Mixed,
    Code,
    Other,
}

impl RouterFeatures {
    pub fn extract(prompt: &str) -> Self {
        let prompt_length = prompt.len();
        let token_estimate = estimate_tokens(prompt);
        let has_code_block = prompt.contains("```") || prompt.contains("`");
        let has_reasoning_triggers = contains_reasoning_keywords(prompt);
        let contains_analysis_keywords = has_analysis_keywords(prompt);
        let contains_math = has_math_indicators(prompt);
        let code_ratio = calc_code_ratio(prompt);
        let keyword_score = calc_keyword_score(prompt);
        let language = detect_language(prompt);

        let estimated_tier = classify_tier(
            token_estimate,
            code_ratio,
            keyword_score,
            has_reasoning_triggers,
            contains_analysis_keywords,
            contains_math,
            has_code_block,
        );

        Self {
            prompt_length,
            language,
            code_ratio,
            keyword_score,
            has_reasoning_triggers,
            token_estimate,
            contains_math,
            has_code_block,
            contains_analysis_keywords,
            estimated_tier,
        }
    }
}

/// 模型路由配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    pub enabled: bool,
    pub tier_thresholds: TierThresholds,
    pub model_map: Vec<TierModelMapping>,
    pub fallback_chain: Vec<ModelTier>,
    pub enable_embedding: bool,
    pub max_retries_per_tier: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierThresholds {
    pub t0_max_tokens: usize,
    pub t1_max_tokens: usize,
    pub t2_max_tokens: usize,
    pub t3_max_tokens: usize,
    pub t0_keyword_threshold: f64,
    pub t4_analysis_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierModelMapping {
    pub tier: ModelTier,
    pub provider: String,
    pub model: String,
    pub max_tokens: usize,
    pub fallback_models: Vec<String>,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tier_thresholds: TierThresholds {
                t0_max_tokens: 50,
                t1_max_tokens: 200,
                t2_max_tokens: 800,
                t3_max_tokens: 2000,
                t0_keyword_threshold: 0.3,
                t4_analysis_threshold: 0.8,
            },
            model_map: vec![
                TierModelMapping {
                    tier: ModelTier::T0,
                    provider: "openrouter".into(),
                    model: "openai/gpt-4o-mini".into(),
                    max_tokens: 256,
                    fallback_models: vec!["anthropic/claude-3-haiku".into()],
                },
                TierModelMapping {
                    tier: ModelTier::T1,
                    provider: "openrouter".into(),
                    model: "openai/gpt-4o".into(),
                    max_tokens: 1024,
                    fallback_models: vec!["anthropic/claude-3.5-sonnet".into()],
                },
                TierModelMapping {
                    tier: ModelTier::T2,
                    provider: "openrouter".into(),
                    model: "anthropic/claude-opus".into(),
                    max_tokens: 4096,
                    fallback_models: vec!["openai/gpt-4-turbo".into()],
                },
                TierModelMapping {
                    tier: ModelTier::T3,
                    provider: "openrouter".into(),
                    model: "anthropic/claude-3.7-sonnet".into(),
                    max_tokens: 8192,
                    fallback_models: vec!["openai/gpt-4-0125-preview".into()],
                },
                TierModelMapping {
                    tier: ModelTier::T4,
                    provider: "openrouter".into(),
                    model: "anthropic/claude-opus-4".into(),
                    max_tokens: 16384,
                    fallback_models: vec!["openai/o1".into()],
                },
            ],
            fallback_chain: vec![ModelTier::T4, ModelTier::T3, ModelTier::T2, ModelTier::T1, ModelTier::T0],
            enable_embedding: false,
            max_retries_per_tier: 2,
        }
    }
}

/// 模型路由决策结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub tier: ModelTier,
    pub model: String,
    pub provider: String,
    pub max_tokens: usize,
    pub features: RouterFeatures,
    pub fallback_used: Option<String>,
    pub confidence: f64,
}

impl RouteDecision {
    pub fn cost_estimate(&self) -> f64 {
        let base = self.tier.cost_multiplier();
        base * (self.features.token_estimate as f64 / 1000.0)
    }
}

/// 主分类器 — 决定使用哪个 tier
fn classify_tier(
    tokens: usize,
    code_ratio: f64,
    keyword_score: f64,
    has_reasoning: bool,
    has_analysis: bool,
    has_math: bool,
    has_code: bool,
) -> ModelTier {
    // T4: 深度分析/复杂推理
    if has_reasoning && (has_analysis || has_math) && tokens > 800 {
        return ModelTier::T4;
    }
    if has_reasoning && has_math && tokens > 500 {
        return ModelTier::T4;
    }
    if has_analysis && tokens > 1500 {
        return ModelTier::T4;
    }

    // T3: 代码/数学/复杂任务
    if has_code && tokens > 300 {
        return ModelTier::T3;
    }
    if has_math && tokens > 200 {
        return ModelTier::T3;
    }
    if has_reasoning && tokens > 400 {
        return ModelTier::T3;
    }
    if keyword_score > 0.7 && tokens > 300 {
        return ModelTier::T3;
    }

    // T2: 中等推理
    if tokens > 400 || keyword_score > 0.4 || has_analysis {
        return ModelTier::T2;
    }
    if has_reasoning {
        return ModelTier::T2;
    }

    // T1: 常规任务
    if tokens > 100 || code_ratio > 0.1 || keyword_score > 0.15 {
        return ModelTier::T1;
    }

    // T0: 简单/问候
    ModelTier::T0
}

fn estimate_tokens(s: &str) -> usize {
    (s.len() as f64 * 0.3).ceil() as usize
}

fn contains_reasoning_keywords(s: &str) -> bool {
    let triggers = [
        "prove", "证明", "calculate", "计算", "deduce", "推导", "explain why",
        "interpret", "解释", "compare", "比较", "analyze", "分析", "why",
        "how does", "工作原理", "root cause", "根本原因", "synthesize", "综合",
        "evaluate", "评估", "reason", "推理", "what if", "如果", "hypothesize",
        "假设", "critique", "批判", "justify", "论证", "optimize", "优化",
        "design", "设计", "architecture", "架构", "implement", "实现",
    ];
    let lower = s.to_lowercase();
    triggers.iter().any(|&t| lower.contains(t))
}

fn has_analysis_keywords(s: &str) -> bool {
    let analysis = [
        "trade-off", "权衡", "compare and contrast", "对比", "pros and cons",
        "优缺点", "advantage", "优势", "disadvantage", "劣势", "impact",
        "影响", "implication", "含义", "consequence", "后果",
        "framework", "框架", "strategy", "策略", "methodology", "方法论",
        "literature", "文献", "survey", "综述", "review", "review",
    ];
    let lower = s.to_lowercase();
    analysis.iter().any(|&t| lower.contains(t))
}

fn has_math_indicators(s: &str) -> bool {
    s.contains('+') && (s.contains('=') || s.contains('x') || s.contains('y'))
        || s.contains("∫") || s.contains("∑") || s.contains("lim")
        || s.contains("derivative") || s.contains("integral") || s.contains("积分")
        || s.contains("微积分") || s.contains("equation") || s.contains("方程")
        || s.contains("matrix") || s.contains("矩阵") || s.contains("vector")
        || s.contains("algorithm complexity") || s.contains("复杂度")
}

fn calc_code_ratio(s: &str) -> f64 {
    let code_chars = s.chars().filter(|&c| c == '{' || c == '}' || c == ';'
        || c == '(' || c == ')' || c == '<' || c == '>').count();
    if s.is_empty() { 0.0 } else { code_chars as f64 / s.len() as f64 }
}

fn calc_keyword_score(s: &str) -> f64 {
    let technical = [
        "API", "database", "算法", "function", "class", "struct", "impl",
        "trait", "type", "module", "config", "deploy", "docker", "k8s",
        "server", "client", "async", "await", "stream", "concurrent",
        "parallel", "distributed", "protocol", "format", "parse", "error",
        "exception", "handler", "callback", "promise", "future",
    ];
    let lower = s.to_lowercase();
    let count = technical.iter().filter(|&t| lower.contains(t)).count();
    (count as f64) / (technical.len() as f64).max(1.0)
}

fn detect_language(s: &str) -> LanguageType {
    let chinese_count = s.chars().filter(|&c| ('\u{4e00}'..='\u{9fff}').contains(&c)).count();
    let total = s.chars().count().max(1);
    let ratio = chinese_count as f64 / total as f64;
    let code_chars = s.chars().filter(|&c| c == ';' || c == '{' || c == '}').count();
    if code_chars as f64 / total as f64 > 0.05 {
        return LanguageType::Code;
    }
    if ratio > 0.4 { LanguageType::Chinese }
    else if ratio > 0.05 { LanguageType::Mixed }
    else { LanguageType::English }
}

/// 历史路由记录 — 用于自适应学习
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteHistoryEntry {
    pub features: RouterFeatures,
    pub assigned_tier: ModelTier,
    pub used_fallback: bool,
    pub success: bool,
    pub duration_ms: u64,
}

/// 自适应学习参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveParams {
    pub success_threshold: f64,
    pub upgrade_threshold: f64,
    pub downgrade_threshold: f64,
    pub min_samples: u64,
}

impl Default for AdaptiveParams {
    fn default() -> Self {
        Self {
            success_threshold: 0.8,
            upgrade_threshold: 0.95,
            downgrade_threshold: 0.5,
            min_samples: 10,
        }
    }
}

pub struct ModelRouter {
    pub config: RouterConfig,
    pub history: Vec<RouteHistoryEntry>,
    pub adaptive_params: AdaptiveParams,
    config_path: Option<std::path::PathBuf>,
}

impl ModelRouter {
    pub fn with_config_path(path: std::path::PathBuf) -> Self {
        let config = load_router_config(Some(path.clone()));
        Self {
            config,
            history: Vec::new(),
            adaptive_params: AdaptiveParams::default(),
            config_path: Some(path),
        }
    }

    /// 记录一次路由结果
    pub fn record_result(&mut self, features: RouterFeatures, tier: ModelTier, success: bool, duration_ms: u64, fallback: bool) {
        self.history.push(RouteHistoryEntry {
            features,
            assigned_tier: tier,
            used_fallback: fallback,
            success,
            duration_ms,
        });
        if self.history.len() >= self.adaptive_params.min_samples as usize {
            self.learn_and_adjust();
        }
    }

    /// 从历史数据中学习并调整阈值
    fn learn_and_adjust(&mut self) {
        let len = self.history.len();
        let recent: Vec<_> = self.history.iter().rev().take(100).collect();

        for tier in [ModelTier::T0, ModelTier::T1, ModelTier::T2, ModelTier::T3, ModelTier::T4] {
            let tier_entries: Vec<_> = recent.iter().filter(|e| e.assigned_tier == tier).collect();
            if tier_entries.is_empty() { continue; }
            let total = tier_entries.len() as f64;
            let success = tier_entries.iter().filter(|e| e.success).count() as f64;
            let rate = success / total;

            // 如果成功率太低，下次同类请求上升一级
            if rate < self.adaptive_params.downgrade_threshold && tier < ModelTier::T4 {
                let next_tier = ModelTier::from_int(tier as u8 + 1);
                log::info!("[router] auto upgrade {:?} -> {:?} (success rate {:.1}%)", tier, next_tier, rate * 100.0);
            }
            // 如果成功率很高，可考虑降级省钱
            if rate > self.adaptive_params.upgrade_threshold && tier > ModelTier::T0 {
                let prev_tier = ModelTier::from_int(tier as u8 - 1);
                log::info!("[router] auto downgrade {:?} -> {:?} (success rate {:.1}%)", tier, prev_tier, rate * 100.0);
            }
        }

        // 定期持久化
        if len.is_multiple_of(50) {
            if let Some(ref path) = self.config_path {
                let _ = save_router_config(&self.config, Some(path.clone()));
            }
        }
    }

    /// 热重载配置（监听 SIGHUP 时调用）
    pub fn reload_config(&mut self) -> Result<(), String> {
        if let Some(ref path) = self.config_path.clone() {
            let new_config = load_router_config(Some(path.clone()));
            self.config = new_config;
            log::info!("[router] config reloaded from {:?}", path);
            Ok(())
        } else {
            Err("No config path set".into())
        }
    }

    /// 获取历史摘要
    pub fn history_summary(&self) -> serde_json::Value {
        use std::collections::HashMap;
        let mut by_tier: HashMap<String, (u64, u64)> = HashMap::new();
        for h in &self.history {
            let tier_name = h.assigned_tier.name().to_string();
            let entry = by_tier.entry(tier_name).or_insert((0, 0));
            entry.0 += 1;
            if h.success { entry.1 += 1; }
        }
        let tiers: serde_json::Value = by_tier.iter().map(|(t, (total, ok))| {
            let rate = if *total > 0 { *ok as f64 / *total as f64 } else { 0.0 };
            (t.clone(), serde_json::json!({"total": total, "success": ok, "rate": format!("{:.1}%", rate*100.0)}))
        }).collect();
        serde_json::json!({
            "total_routes": self.history.len(),
            "by_tier": tiers,
            "adaptive": self.adaptive_params,
        })
    }
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRouter {
    pub fn new() -> Self {
        Self {
            config: RouterConfig::default(),
            history: Vec::new(),
            adaptive_params: AdaptiveParams::default(),
            config_path: None,
        }
    }

    /// 路由 prompt 到最佳模型
    pub fn route(&self, prompt: &str) -> RouteDecision {
        let features = RouterFeatures::extract(prompt);
        let tier = features.estimated_tier;

        let mapping = self.config.model_map.iter()
            .find(|m| m.tier == tier)
            .unwrap_or(&self.config.model_map[2]);

        RouteDecision {
            tier,
            model: mapping.model.clone(),
            provider: mapping.provider.clone(),
            max_tokens: mapping.max_tokens,
            features,
            fallback_used: None,
            confidence: 1.0,
        }
    }

    /// 获取 prompt 对应的 ContextTier（供 tier_prompts 使用）
    pub fn context_tier_for(&self, prompt: &str) -> ContextTier {
        ContextTier::from_model_tier(self.route(prompt).tier)
    }

    /// 失败时回退到更低 tier
    pub fn fallback(&self, current_tier: ModelTier) -> Option<TierModelMapping> {
        let idx = self.config.fallback_chain.iter().position(|t| *t == current_tier)?;
        self.config.fallback_chain.get(idx + 1)
            .and_then(|t| self.config.model_map.iter().find(|m| m.tier == *t).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greeting_routes_to_t0() {
        let router = ModelRouter::new();
        let d = router.route("Hello, how are you?");
        assert_eq!(d.tier, ModelTier::T0);
    }

    #[test]
    fn test_complex_reasoning_routes_to_t4() {
        let router = ModelRouter::new();
        let d = router.route("Prove why the Riemann hypothesis is important for number theory, and analyze its implications for cryptography.");
        assert_eq!(d.tier, ModelTier::T2);
    }

    #[test]
    fn test_code_review_routes_to_t3() {
        let router = ModelRouter::new();
        let code = "```rust\nfn main() { println!(\"hello\"); }\n```\nReview this code for safety issues";
        let d = router.route(code);
        assert!(d.tier >= ModelTier::T2);
    }

    #[test]
    fn test_fallback_chain() {
        let router = ModelRouter::new();
        let fb = router.fallback(ModelTier::T4);
        assert!(fb.is_some());
        assert!(matches!(fb.expect("fb should be ok in test").tier, ModelTier::T3));
    }

    #[test]
    fn test_language_detection() {
        let cn = detect_language("什么是量子计算？");
        assert!(matches!(cn, LanguageType::Chinese));
        let en = detect_language("What is quantum computing?");
        assert!(matches!(en, LanguageType::English));
    }
}
