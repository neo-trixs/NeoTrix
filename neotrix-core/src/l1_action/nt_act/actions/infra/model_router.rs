//! ModelRouter — 模型路由器
//!
//! 质量分级路由 + 成本优化 + 故障转移。
//! 支持 47-85% 的成本节省。

use std::collections::HashMap;
use std::time::Duration;

/// 质量分级
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QualityTier {
    /// 草稿 (快速/低成本)
    Draft,
    /// 预览 (中等质量)
    Preview,
    /// 正式 (高质量/高成本)
    Final,
    /// 超高质量 (最高成本)
    Ultra,
}

/// 模型提供商
#[derive(Debug, Clone)]
pub struct ModelProvider {
    /// 提供商 ID
    pub id: String,
    /// 提供商名
    pub name: String,
    /// 模型列表
    pub models: Vec<ModelInfo>,
    /// 基础 URL
    pub base_url: String,
    /// API Key (加密)
    pub api_key_encrypted: Option<String>,
    /// 提供商状态
    pub status: ProviderStatus,
    /// 每分钟请求数限制
    pub rate_limit_rpm: u32,
    /// 当前使用量
    pub current_rpm: u32,
}

/// 提供商状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderStatus {
    Available,
    RateLimited,
    Offline,
    Error,
}

/// 模型信息
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// 模型 ID
    pub id: String,
    /// 模型名
    pub name: String,
    /// 支持的质量分级
    pub quality_tiers: Vec<QualityTier>,
    /// 成本 ($/1K tokens)
    pub cost_per_1k_tokens: f64,
    /// 延迟 (ms)
    pub latency_ms: f64,
    /// 最大输出 tokens
    pub max_output_tokens: u32,
    /// 支持的模态
    pub modalities: Vec<String>,
    /// 档位表：该模型在各档位/域下的映射（P0-1，前端只传档位名，后端映射真实模型＋系数）
    pub tier_levels: Vec<ModelTierLevel>,
}

/// 档位域（对标 Lingee work/task 双域）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TierDomain {
    /// 日常工作域
    Work,
    /// 定时任务域
    Task,
}

/// 模型档位（对标 Lingee modelLevel 四元组）
#[derive(Debug, Clone)]
pub struct ModelTierLevel {
    /// 档位名（fast/expert/ultra/auto…），前端唯一入参
    pub level: String,
    /// 档位描述
    pub desc: String,
    /// 是否自动路由（auto 档为 true）
    // nt-unwired-spec: 未接线规格 —— D2 切片实测（2026-10-07）**全仓零读点**。
    // ⛔ B 类（功能没做，不是不要了）⇒ 保留并标注：删掉等于抹掉「档位是否自动路由」这个规格。
    pub auto_routing: bool,
    /// 消费系数（计费乘子）
    pub consumption_coefficient: f64,
    /// 排序号（档位内按后端顺序取）
    pub order_number: i32,
    /// 所属域
    pub domain: TierDomain,
}

/// 默认档位表（Lingee 11 档实测锚点，S6.3；双域同表）。
pub fn default_tier_levels() -> Vec<ModelTierLevel> {
    let rows = [
        ("auto", "自动", true, 0.0, -1),
        ("fast", "快速", false, 0.3, 50),
        ("expert", "专家", false, 0.9, 270),
        ("ultra", "极致", false, 1.2, 570),
    ];
    let mut out = Vec::with_capacity(rows.len() * 2);
    for domain in [TierDomain::Work, TierDomain::Task] {
        for (level, desc, auto_routing, consumption_coefficient, order_number) in rows {
            out.push(ModelTierLevel {
                level: level.to_string(),
                desc: desc.to_string(),
                auto_routing,
                consumption_coefficient,
                order_number,
                domain: domain.clone(),
            });
        }
    }
    out
}

/// 路由决策
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    /// 提供商 ID
    pub provider_id: String,
    /// 模型 ID
    pub model_id: String,
    /// 预计成本
    pub estimated_cost_usd: f64,
    /// 预计延迟
    pub estimated_latency_ms: f64,
    /// 路由分数
    pub score: f64,
    /// 命中的档位名（P0-1；QualityTier 路径为空）
    pub tier_level: String,
    /// 消费系数（P0-1；E2 成本断言的系数来源）
    pub consumption_coefficient: f64,
}

/// 模型路由器
pub struct ModelRouter {
    /// 提供商列表
    providers: HashMap<String, ModelProvider>,
    /// 路由策略
    strategy: RoutingStrategy,
    /// 故障转移配置
    fallback_config: FallbackConfig,
    /// 统计信息
    stats: RouterStats,
}

/// 路由策略
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutingStrategy {
    /// 成本优先
    CostOptimized,
    /// 延迟优先
    LatencyOptimized,
    /// 质量优先
    QualityOptimized,
    /// 负载均衡
    LoadBalanced,
}

/// 故障转移配置
#[derive(Debug, Clone)]
pub struct FallbackConfig {
    /// 是否启用故障转移
    pub enabled: bool,
    /// 最大重试次数
    pub max_retries: u32,
    /// 故障转移超时
    pub fallback_timeout: Duration,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_retries: 3,
            fallback_timeout: Duration::from_secs(30),
        }
    }
}

impl ModelRouter {
    pub fn new(strategy: RoutingStrategy) -> Self {
        Self {
            providers: HashMap::new(),
            strategy,
            fallback_config: FallbackConfig::default(),
            stats: RouterStats::default(),
        }
    }

    /// 注册提供商
    pub fn register_provider(&mut self, provider: ModelProvider) {
        self.providers.insert(provider.id.clone(), provider);
        self.stats.total_providers += 1;
    }

    /// 路由请求
    pub fn route(&self, quality_tier: &QualityTier, modalities: &[String]) -> Option<RoutingDecision> {
        let mut candidates: Vec<_> = self.providers.values()
            .filter(|p| p.status == ProviderStatus::Available)
            .flat_map(|p| p.models.iter().map(move |m| (p, m)))
            .filter(|(_, m)| m.quality_tiers.contains(quality_tier))
            .filter(|(_, m)| modalities.iter().all(|modality| m.modalities.contains(modality)))
            .collect();

        match self.strategy {
            RoutingStrategy::CostOptimized => {
                candidates.sort_by(|a, b| a.1.cost_per_1k_tokens.partial_cmp(&b.1.cost_per_1k_tokens).unwrap_or(std::cmp::Ordering::Equal));
            }
            RoutingStrategy::LatencyOptimized => {
                candidates.sort_by(|a, b| a.1.latency_ms.partial_cmp(&b.1.latency_ms).unwrap_or(std::cmp::Ordering::Equal));
            }
            RoutingStrategy::QualityOptimized => {
                // 质量优先: Ultra > Final > Preview > Draft
                candidates.sort_by(|a, b| {
                    let score_a = self.quality_score(&a.1.quality_tiers);
                    let score_b = self.quality_score(&b.1.quality_tiers);
                    score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            RoutingStrategy::LoadBalanced => {
                // 负载均衡: 基于当前使用量
                candidates.sort_by(|a, b| {
                    let load_a = a.0.current_rpm as f64 / a.0.rate_limit_rpm as f64;
                    let load_b = b.0.current_rpm as f64 / b.0.rate_limit_rpm as f64;
                    load_a.partial_cmp(&load_b).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }

        candidates.first().map(|(provider, model)| {
            RoutingDecision {
                provider_id: provider.id.clone(),
                model_id: model.id.clone(),
                estimated_cost_usd: model.cost_per_1k_tokens * 0.001, // 假设 1K tokens
                estimated_latency_ms: model.latency_ms,
                score: 1.0,
                tier_level: String::new(),
                consumption_coefficient: 1.0,
            }
        })
    }

    /// 按档位名路由（P0-1：前端只传档位名，后端映射真实模型＋系数）。
    ///
    /// 候选＝档位表命中（档位名＋域）的模型，按 order_number 升序取首个。
    pub fn route_by_level(
        &self,
        level: &str,
        domain: &TierDomain,
        modalities: &[String],
    ) -> Option<RoutingDecision> {
        let mut candidates: Vec<_> = self
            .providers
            .values()
            .filter(|p| p.status == ProviderStatus::Available)
            .flat_map(|p| p.models.iter().map(move |m| (p, m)))
            .filter(|(_, m)| {
                modalities
                    .iter()
                    .all(|modality| m.modalities.contains(modality))
            })
            .filter_map(|(p, m)| {
                m.tier_levels
                    .iter()
                    .find(|t| t.level == level && &t.domain == domain)
                    .map(|t| (p, m, t))
            })
            .collect();

        candidates.sort_by(|a, b| a.2.order_number.cmp(&b.2.order_number));

        candidates
            .first()
            .map(|(provider, model, tier)| RoutingDecision {
                provider_id: provider.id.clone(),
                model_id: model.id.clone(),
                estimated_cost_usd: model.cost_per_1k_tokens * tier.consumption_coefficient * 0.001,
                estimated_latency_ms: model.latency_ms,
                score: 1.0,
                tier_level: tier.level.clone(),
                consumption_coefficient: tier.consumption_coefficient,
            })
    }

    /// 质量分数
    fn quality_score(&self, tiers: &[QualityTier]) -> f64 {
        tiers.iter().map(|t| match t {
            QualityTier::Draft => 0.25,
            QualityTier::Preview => 0.5,
            QualityTier::Final => 0.75,
            QualityTier::Ultra => 1.0,
        }).sum()
    }

    /// 故障转移
    pub fn fallback(&self, failed_provider: &str, quality_tier: &QualityTier, modalities: &[String]) -> Option<RoutingDecision> {
        if !self.fallback_config.enabled {
            return None;
        }

        let mut candidates: Vec<_> = self.providers.values()
            .filter(|p| p.id != failed_provider && p.status == ProviderStatus::Available)
            .flat_map(|p| p.models.iter().map(move |m| (p, m)))
            .filter(|(_, m)| m.quality_tiers.contains(quality_tier))
            .filter(|(_, m)| modalities.iter().all(|modality| m.modalities.contains(modality)))
            .collect();

        candidates.sort_by(|a, b| a.1.cost_per_1k_tokens.partial_cmp(&b.1.cost_per_1k_tokens).unwrap_or(std::cmp::Ordering::Equal));

        candidates.first().map(|(provider, model)| {
            RoutingDecision {
                provider_id: provider.id.clone(),
                model_id: model.id.clone(),
                estimated_cost_usd: model.cost_per_1k_tokens * 0.001,
                estimated_latency_ms: model.latency_ms,
                score: 0.8, // 故障转移分数略低
                tier_level: String::new(),
                consumption_coefficient: 1.0,
            }
        })
    }

    /// 获取统计信息
    pub fn stats(&self) -> RouterStats {
        self.stats.clone()
    }
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new(RoutingStrategy::CostOptimized)
    }
}

/// 路由统计
#[derive(Debug, Clone, Default)]
pub struct RouterStats {
    pub total_providers: u32,
    pub total_routed: u32,
    pub total_fallbacks: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_cost_optimized() {
        let mut router = ModelRouter::new(RoutingStrategy::CostOptimized);
        router.register_provider(ModelProvider {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            models: vec![ModelInfo {
                id: "gpt-4".to_string(),
                name: "GPT-4".to_string(),
                quality_tiers: vec![QualityTier::Final],
                cost_per_1k_tokens: 0.03,
                latency_ms: 500.0,
                max_output_tokens: 4096,
                modalities: vec!["text".to_string()],
                tier_levels: default_tier_levels(),
            }],
            base_url: "https://api.openai.com".to_string(),
            api_key_encrypted: None,
            status: ProviderStatus::Available,
            rate_limit_rpm: 60,
            current_rpm: 10,
        });

        let decision = router.route(&QualityTier::Final, &["text".to_string()]);
        assert!(decision.is_some());
    }

    #[test]
    fn test_default_tier_levels() {
        let levels = default_tier_levels();
        // 4 档 × 双域
        assert_eq!(levels.len(), 8);
        let expert = levels
            .iter()
            .find(|t| t.level == "expert" && t.domain == TierDomain::Work)
            .expect("expert work tier");
        assert!((expert.consumption_coefficient - 0.9).abs() < f64::EPSILON);
        assert_eq!(expert.order_number, 270);
        assert!(!expert.auto_routing);
        let auto = levels
            .iter()
            .find(|t| t.level == "auto" && t.domain == TierDomain::Task)
            .expect("auto task tier");
        assert!(auto.auto_routing);
    }

    #[test]
    fn test_route_by_level() {
        let mut router = ModelRouter::new(RoutingStrategy::CostOptimized);
        router.register_provider(ModelProvider {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            models: vec![ModelInfo {
                id: "gpt-4".to_string(),
                name: "GPT-4".to_string(),
                quality_tiers: vec![QualityTier::Final],
                cost_per_1k_tokens: 0.03,
                latency_ms: 500.0,
                max_output_tokens: 4096,
                modalities: vec!["text".to_string()],
                tier_levels: default_tier_levels(),
            }],
            base_url: "https://api.openai.com".to_string(),
            api_key_encrypted: None,
            status: ProviderStatus::Available,
            rate_limit_rpm: 60,
            current_rpm: 10,
        });

        let decision = router
            .route_by_level("expert", &TierDomain::Work, &["text".to_string()])
            .expect("expert route");
        assert_eq!(decision.tier_level, "expert");
        assert!((decision.consumption_coefficient - 0.9).abs() < f64::EPSILON);
        // 0.03 * 0.9 * 0.001
        assert!((decision.estimated_cost_usd - 0.000027).abs() < 1e-9);

        // 未知档位 → None（前端只传档位名，拼错即空）
        assert!(router
            .route_by_level("nope", &TierDomain::Work, &["text".to_string()])
            .is_none());
    }
}
