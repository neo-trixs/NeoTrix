//! 资源预算管理模块 (通用)
//!
//! 管理 AI 生成任务的 Token、GPU、成本等资源
//! 适用于：所有 AI 生成和推理场景

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 资源定义
// ============================================================================

/// 资源类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// Token
    Token,
    /// GPU 显存
    GPUMemory,
    /// GPU 计算时间
    GPUCompute,
    /// API 调用次数
    APICalls,
    /// 存储空间
    Storage,
    /// 网络流量
    Network,
    /// 成本 (美元)
    CostUSD,
}

/// 预算周期
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BudgetPeriod {
    /// 每日
    Daily,
    /// 每周
    Weekly,
    /// 每月
    Monthly,
    /// 总量
    Total,
}

/// 资源配额
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    /// 资源类型
    pub resource_type: ResourceType,
    /// 预算周期
    pub period: BudgetPeriod,
    /// 配额上限
    pub limit: f64,
    /// 已使用量
    pub used: f64,
    /// 保留量 (预留给高优先级任务)
    pub reserved: f64,
}

impl ResourceQuota {
    /// 计算剩余可用量
    pub fn available(&self) -> f64 {
        (self.limit - self.used - self.reserved).max(0.0)
    }
    
    /// 检查是否有足够资源
    pub fn has_enough(&self, amount: f64) -> bool {
        self.available() >= amount
    }
    
    /// 获取使用率
    pub fn usage_rate(&self) -> f64 {
        if self.limit == 0.0 {
            0.0
        } else {
            self.used / self.limit
        }
    }
    
    /// 是否接近上限
    pub(crate) fn _is_near_limit(&self, threshold: f64) -> bool {
        self.usage_rate() >= threshold
    }
}

/// 资源预算配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// 资源配额列表
    pub quotas: Vec<ResourceQuota>,
    /// 告警阈值 (0.0-1.0)
    pub alert_threshold: f64,
    /// 硬限制阈值 (0.0-1.0)
    pub hard_limit_threshold: f64,
    /// 是否允许超限 (临时)
    pub allow_overdraft: bool,
    /// 超限惩罚系数
    pub overdraft_penalty: f64,
    /// 成本优化策略
    pub cost_optimization: CostOptimization,
}

/// 成本优化策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOptimization {
    /// 是否启用批处理优化
    pub enable_batch_optimization: bool,
    /// 是否启用缓存复用
    pub enable_cache_reuse: bool,
    /// 是否启用降级处理
    pub enable_degradation: bool,
    /// 最大降级级别
    pub max_degradation_level: u8,
}

/// 资源使用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// 任务ID
    pub task_id: String,
    /// 资源类型
    pub resource_type: ResourceType,
    /// 使用量
    pub amount: f64,
    /// 成本
    pub cost: f64,
    /// 时间戳
    pub timestamp: u64,
    /// 任务标签
    pub tags: Vec<String>,
}

/// 预算检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetCheckResult {
    /// 是否在预算内
    pub within_budget: bool,
    /// 是否触发告警
    pub alert_triggered: bool,
    /// 是否触发硬限制
    pub hard_limit_triggered: bool,
    /// 剩余资源
    pub remaining: f64,
    /// 使用率
    pub usage_rate: f64,
    /// 建议 (降级/延迟/拒绝)
    pub recommendation: BudgetRecommendation,
}

/// 预算建议
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BudgetRecommendation {
    /// 继续执行
    Continue,
    /// 降级执行
    Degraded,
    /// 延迟执行
    Delay,
    /// 拒绝执行
    Reject,
}

// ============================================================================
// 资源预算管理器
// ============================================================================

/// 资源预算管理器
/// 管理 AI 生成任务的资源分配和成本控制
pub struct ResourceBudgetManager {
    /// 预算配置
    config: BudgetConfig,
    /// 使用历史
    history: Vec<ResourceUsage>,
    /// 当前周期使用统计
    period_usage: HashMap<ResourceType, f64>,
}

impl ResourceBudgetManager {
    /// 创建管理器
    pub fn new() -> Self {
        Self {
            config: BudgetConfig {
                quotas: vec![
                    ResourceQuota {
                        resource_type: ResourceType::Token,
                        period: BudgetPeriod::Daily,
                        limit: 1_000_000.0,
                        used: 0.0,
                        reserved: 0.0,
                    },
                    ResourceQuota {
                        resource_type: ResourceType::CostUSD,
                        period: BudgetPeriod::Monthly,
                        limit: 100.0,
                        used: 0.0,
                        reserved: 0.0,
                    },
                ],
                alert_threshold: 0.8,
                hard_limit_threshold: 0.95,
                allow_overdraft: false,
                overdraft_penalty: 1.5,
                cost_optimization: CostOptimization {
                    enable_batch_optimization: true,
                    enable_cache_reuse: true,
                    enable_degradation: true,
                    max_degradation_level: 2,
                },
            },
            history: vec![],
            period_usage: HashMap::new(),
        }
    }
    
    /// 使用配置创建
    pub fn with_config(config: BudgetConfig) -> Self {
        Self {
            config,
            history: vec![],
            period_usage: HashMap::new(),
        }
    }
    
    /// 检查预算
    pub fn check_budget(&self, resource_type: ResourceType, amount: f64) -> BudgetCheckResult {
        let quota = self.config.quotas.iter()
            .find(|q| q.resource_type == resource_type);
        
        match quota {
            Some(quota) => {
                let remaining = quota.available();
                let usage_rate = quota.usage_rate();
                let within_budget = remaining >= amount;
                let alert_triggered = usage_rate >= self.config.alert_threshold;
                let hard_limit_triggered = usage_rate >= self.config.hard_limit_threshold;
                
                let recommendation = if hard_limit_triggered {
                    BudgetRecommendation::Reject
                } else if alert_triggered {
                    BudgetRecommendation::Degraded
                } else if !within_budget && !self.config.allow_overdraft {
                    BudgetRecommendation::Delay
                } else {
                    BudgetRecommendation::Continue
                };
                
                BudgetCheckResult {
                    within_budget,
                    alert_triggered,
                    hard_limit_triggered,
                    remaining,
                    usage_rate,
                    recommendation,
                }
            }
            None => BudgetCheckResult {
                within_budget: true,
                alert_triggered: false,
                hard_limit_triggered: false,
                remaining: f64::MAX,
                usage_rate: 0.0,
                recommendation: BudgetRecommendation::Continue,
            },
        }
    }
    
    /// 记录资源使用
    pub fn record_usage(&mut self, usage: ResourceUsage) {
        // 更新配额使用量
        if let Some(quota) = self.config.quotas.iter_mut()
            .find(|q| q.resource_type == usage.resource_type) 
        {
            quota.used += usage.amount;
        }
        
        // 更新周期统计
        *self.period_usage.entry(usage.resource_type).or_insert(0.0) += usage.amount;
        
        self.history.push(usage);
    }
    
    /// 获取成本估算
    ///
    /// Uses `external_prices` when provided (model → $/1K tokens).
    /// Falls back to a **stale** built-in table that must be refreshed
    /// manually or replaced by a live pricing API call.
    ///
    /// Returns `None` if the model is unknown and no fallback exists.
    pub fn estimate_cost(
        &self,
        token_count: u64,
        model: &str,
        external_prices: Option<&HashMap<String, f64>>,
    ) -> Option<f64> {
        // 1. Check caller-supplied price map first (always fresh).
        if let Some(prices) = external_prices {
            if let Some(&cost_per_1k) = prices.get(model) {
                return Some((token_count as f64 / 1000.0) * cost_per_1k);
            }
        }

        // 2. Built-in fallback — STALE, will drift from real pricing.
        //    Refresh this table or provide `external_prices` to keep estimates accurate.
        #[allow(clippy::uninlined_format_args)]
        let cost_per_1k: f64 = match model {
            // OpenAI  (approximate, verify at https://openai.com/pricing)
            "gpt-4o" => 0.0025,
            "gpt-4o-mini" => 0.00015,
            "gpt-4-turbo" => 0.01,
            "gpt-4" => 0.03,
            "gpt-3.5-turbo" => 0.0005,
            // Anthropic (approximate)
            "claude-sonnet-4-20250514" => 0.003,
            "claude-3-5-sonnet-20241022" => 0.003,
            "claude-3-haiku-20240307" => 0.00025,
            "claude-3-opus-20240229" => 0.015,
            // Google (approximate)
            "gemini-2.0-flash" => 0.0001,
            "gemini-1.5-pro" => 0.00125,
            "gemini-1.5-flash" => 0.000075,
            // Local / free
            "ollama" | "local" => 0.0,
            // Unknown model — return None instead of a fake estimate.
            _ => return None,
        };

        Some((token_count as f64 / 1000.0) * cost_per_1k)
    }
    
    /// 获取使用统计
    pub fn statistics(&self) -> BudgetStats {
        let total_cost: f64 = self.history.iter()
            .filter(|u| u.resource_type == ResourceType::CostUSD)
            .map(|u| u.cost)
            .sum();
        
        let total_tokens: f64 = self.history.iter()
            .filter(|u| u.resource_type == ResourceType::Token)
            .map(|u| u.amount)
            .sum();
        
        let total_tasks = self.history.iter()
            .map(|u| u.task_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .len();
        
        BudgetStats {
            total_cost_usd: total_cost,
            total_tokens: total_tokens as u64,
            total_tasks,
            avg_cost_per_task: if total_tasks > 0 { total_cost / total_tasks as f64 } else { 0.0 },
            avg_tokens_per_task: if total_tasks > 0 { total_tokens / total_tasks as f64 } else { 0.0 },
        }
    }
    
    /// 重置周期使用量
    pub(crate) fn _reset_period_usage(&mut self) {
        self.period_usage.clear();
        for quota in &mut self.config.quotas {
            quota.used = 0.0;
        }
    }
}

/// 预算统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStats {
    /// 总成本
    pub total_cost_usd: f64,
    /// 总 Token 数
    pub total_tokens: u64,
    /// 总任务数
    pub total_tasks: usize,
    /// 平均每任务成本
    pub avg_cost_per_task: f64,
    /// 平均每任务 Token 数
    pub avg_tokens_per_task: f64,
}

// ============================================================================
// 向后兼容别名
// ============================================================================

/// 成本管理器 (向后兼容别名)
pub type CostManager = ResourceBudgetManager;

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_within_limit_returns_continue() {
        let manager = ResourceBudgetManager::new();
        // Default token quota: 1M limit, 0 used → 100K request is within budget
        let result = manager.check_budget(ResourceType::Token, 100_000.0);
        assert!(result.within_budget);
        assert_eq!(result.recommendation, BudgetRecommendation::Continue);
    }

    #[test]
    fn test_budget_exceeded_returns_delay() {
        let config = BudgetConfig {
            quotas: vec![ResourceQuota {
                resource_type: ResourceType::Token,
                period: BudgetPeriod::Daily,
                limit: 1000.0,
                used: 800.0,  // 80% used
                reserved: 0.0,
            }],
            alert_threshold: 0.9,
            hard_limit_threshold: 0.95,
            allow_overdraft: false,
            overdraft_penalty: 1.5,
            cost_optimization: CostOptimization {
                enable_batch_optimization: false,
                enable_cache_reuse: false,
                enable_degradation: false,
                max_degradation_level: 0,
            },
        };
        let manager = ResourceBudgetManager::with_config(config);

        // 300 more tokens would exceed 1000 limit (800 + 300 = 1100 > 1000)
        let result = manager.check_budget(ResourceType::Token, 300.0);
        assert!(!result.within_budget);
        assert_eq!(result.recommendation, BudgetRecommendation::Delay);
    }

    #[test]
    fn test_alert_threshold_triggers_degraded() {
        let config = BudgetConfig {
            quotas: vec![ResourceQuota {
                resource_type: ResourceType::Token,
                period: BudgetPeriod::Daily,
                limit: 1000.0,
                used: 900.0,  // 90% used = alert_threshold
                reserved: 0.0,
            }],
            alert_threshold: 0.9,
            hard_limit_threshold: 0.95,
            allow_overdraft: false,
            overdraft_penalty: 1.5,
            cost_optimization: CostOptimization {
                enable_batch_optimization: false,
                enable_cache_reuse: false,
                enable_degradation: false,
                max_degradation_level: 0,
            },
        };
        let manager = ResourceBudgetManager::with_config(config);

        let result = manager.check_budget(ResourceType::Token, 1.0);
        assert!(result.within_budget);  // 99 remaining >= 1
        assert!(result.alert_triggered);
        assert_eq!(result.recommendation, BudgetRecommendation::Degraded);
    }

    #[test]
    fn test_hard_limit_triggers_reject() {
        let config = BudgetConfig {
            quotas: vec![ResourceQuota {
                resource_type: ResourceType::Token,
                period: BudgetPeriod::Daily,
                limit: 1000.0,
                used: 960.0,  // 96% used = hard_limit_threshold
                reserved: 0.0,
            }],
            alert_threshold: 0.8,
            hard_limit_threshold: 0.95,
            allow_overdraft: false,
            overdraft_penalty: 1.5,
            cost_optimization: CostOptimization {
                enable_batch_optimization: false,
                enable_cache_reuse: false,
                enable_degradation: false,
                max_degradation_level: 0,
            },
        };
        let manager = ResourceBudgetManager::with_config(config);

        let result = manager.check_budget(ResourceType::Token, 1.0);
        assert!(result.hard_limit_triggered);
        assert_eq!(result.recommendation, BudgetRecommendation::Reject);
    }

    #[test]
    fn test_usage_accumulation_affects_subsequent_checks() {
        let mut manager = ResourceBudgetManager::new();

        // First check: within budget
        let r1 = manager.check_budget(ResourceType::Token, 100_000.0);
        assert!(r1.within_budget);

        // Record heavy usage
        manager.record_usage(ResourceUsage {
            task_id: "heavy".into(),
            resource_type: ResourceType::Token,
            amount: 950_000.0,
            cost: 0.0,
            timestamp: 0,
            tags: vec![],
        });

        // Second check: now 950K used out of 1M → alert territory
        let r2 = manager.check_budget(ResourceType::Token, 100_000.0);
        assert!(r2.alert_triggered);
        assert_eq!(r2.recommendation, BudgetRecommendation::Degraded);
    }

    #[test]
    fn test_unknown_resource_type_returns_unlimited() {
        let manager = ResourceBudgetManager::new();
        // Network has no quota defined → should appear unlimited
        let result = manager.check_budget(ResourceType::Network, f64::MAX);
        assert!(result.within_budget);
        assert_eq!(result.remaining, f64::MAX);
        assert_eq!(result.recommendation, BudgetRecommendation::Continue);
    }

    #[test]
    fn test_record_usage_updates_statistics() {
        let mut manager = ResourceBudgetManager::new();

        manager.record_usage(ResourceUsage {
            task_id: "t1".into(),
            resource_type: ResourceType::Token,
            amount: 50_000.0,
            cost: 1.5,
            timestamp: 0,
            tags: vec![],
        });
        manager.record_usage(ResourceUsage {
            task_id: "t2".into(),
            resource_type: ResourceType::Token,
            amount: 30_000.0,
            cost: 0.9,
            timestamp: 0,
            tags: vec![],
        });

        let stats = manager.statistics();
        assert_eq!(stats.total_tokens, 80_000);
        assert_eq!(stats.total_tasks, 2);
        assert!((stats.total_cost_usd - 2.4).abs() < 0.001);
    }

    #[test]
    fn test_cost_estimation_known_model() {
        let manager = ResourceBudgetManager::new();
        let cost = manager.estimate_cost(1000, "gpt-4", None);
        assert!(cost.is_some());
        assert!(cost.unwrap() > 0.0);
    }

    #[test]
    fn test_cost_estimation_unknown_model_returns_none() {
        let manager = ResourceBudgetManager::new();
        let cost = manager.estimate_cost(1000, "unknown-model", None);
        assert!(cost.is_none());
    }

    #[test]
    fn test_cost_estimation_external_price_overrides_builtin() {
        let manager = ResourceBudgetManager::new();
        let mut prices = std::collections::HashMap::new();
        prices.insert("gpt-4".to_string(), 0.10);
        let cost = manager.estimate_cost(1000, "gpt-4", Some(&prices)).unwrap();
        // 1000 tokens * $0.10/1K = $0.10
        assert!((cost - 0.10).abs() < 0.001);
    }
}