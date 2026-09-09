//! ProviderMigrationRouter — 提供商迁移路由器
//!
//! 厂商抽象层 + 迁移路径 + 健康监控。
//! 支持在提供商之间无缝切换 (Runway, Veo, Wan, Hunyuan)。

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 提供商信息
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    /// 提供商 ID
    pub id: String,
    /// 提供商名
    pub name: String,
    /// 基础 URL
    pub base_url: String,
    /// API 版本
    pub api_version: String,
    /// 提供商状态
    pub status: ProviderStatus,
    /// 支持的模态
    pub supported_modalities: Vec<String>,
    /// 成本等级 (1-5)
    pub cost_tier: u8,
    /// 延迟等级 (1-5)
    pub latency_tier: u8,
    /// 质量等级 (1-5)
    pub quality_tier: u8,
    /// 最后健康检查时间
    pub last_health_check: Option<Instant>,
    /// 健康分数 (0-1)
    pub health_score: f64,
}

/// 提供商状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderStatus {
    Available,
    Degraded,
    Offline,
    Maintenance,
}

/// 迁移计划
#[derive(Debug, Clone)]
pub struct MigrationPlan {
    /// 源提供商
    pub source_provider: String,
    /// 目标提供商
    pub target_provider: String,
    /// 迁移原因
    pub reason: MigrationReason,
    /// 预计影响
    pub estimated_impact: MigrationImpact,
    /// 迁移步骤
    pub steps: Vec<MigrationStep>,
    /// 状态
    pub status: MigrationStatus,
}

/// 迁移原因
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationReason {
    /// 成本优化
    CostOptimization,
    /// 性能提升
    PerformanceImprovement,
    /// 可用性提升
    AvailabilityImprovement,
    /// 功能需求
    FeatureRequirement,
    /// 强制迁移 (提供商关闭)
    ForcedMigration,
}

/// 迁移影响
#[derive(Debug, Clone)]
pub struct MigrationImpact {
    /// 预计停机时间
    pub estimated_downtime: Duration,
    /// 预计成本变化
    pub cost_change_percent: f64,
    /// 预计延迟变化
    pub latency_change_ms: f64,
    /// 影响的作业数
    pub affected_jobs: u32,
}

/// 迁移步骤
#[derive(Debug, Clone)]
pub struct MigrationStep {
    /// 步骤名
    pub name: String,
    /// 步骤类型
    pub step_type: StepType,
    /// 依赖步骤
    pub dependencies: Vec<String>,
    /// 预计耗时
    pub estimated_duration: Duration,
    /// 状态
    pub status: StepStatus,
}

/// 步骤类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepType {
    /// 配置迁移
    ConfigMigration,
    /// 数据迁移
    DataMigration,
    /// 流量切换
    TrafficSwitch,
    /// 验证测试
    ValidationTest,
    /// 回滚计划
    RollbackPlan,
}

/// 步骤状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

/// 迁移状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationStatus {
    Planning,
    InProgress,
    Completed,
    Failed,
    RolledBack,
}

/// 提供商迁移路由器
pub struct ProviderMigrationRouter {
    /// 提供商列表
    providers: HashMap<String, ProviderInfo>,
    /// 迁移计划
    migration_plans: Vec<MigrationPlan>,
    /// 当前路由配置
    routing_config: RoutingConfig,
    /// 统计信息
    stats: MigrationStats,
}

/// 路由配置
#[derive(Debug, Clone)]
pub struct RoutingConfig {
    /// 默认提供商
    pub default_provider: String,
    /// 故障转移提供商
    pub fallback_providers: Vec<String>,
    /// 路由策略
    pub strategy: MigrationStrategy,
    /// 健康检查间隔
    pub health_check_interval: Duration,
    /// 自动故障转移
    pub auto_fallback: bool,
}

/// 迁移策略
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationStrategy {
    /// 成本优先
    CostFirst,
    /// 延迟优先
    LatencyFirst,
    /// 质量优先
    QualityFirst,
    /// 可用性优先
    AvailabilityFirst,
}

impl ProviderMigrationRouter {
    pub fn new(routing_config: RoutingConfig) -> Self {
        Self {
            providers: HashMap::new(),
            migration_plans: Vec::new(),
            routing_config,
            stats: MigrationStats::default(),
        }
    }

    /// 注册提供商
    pub fn register_provider(&mut self, provider: ProviderInfo) {
        self.providers.insert(provider.id.clone(), provider);
        self.stats.total_providers += 1;
    }

    /// 获取当前路由
    pub fn get_current_provider(&self) -> Option<&ProviderInfo> {
        self.providers.get(&self.routing_config.default_provider)
    }

    /// 选择最佳提供商
    pub fn select_provider(&self, requirements: &ProviderRequirements) -> Option<&ProviderInfo> {
        self.providers.values()
            .filter(|p| p.status == ProviderStatus::Available)
            .filter(|p| requirements.modalities.iter().all(|m| p.supported_modalities.contains(m)))
            .min_by(|a, b| {
                let score_a = self.calculate_provider_score(a, requirements);
                let score_b = self.calculate_provider_score(b, requirements);
                score_a.partial_cmp(&score_b).unwrap()
            })
    }

    /// 计算提供商分数
    fn calculate_provider_score(&self, provider: &ProviderInfo, _requirements: &ProviderRequirements) -> f64 {
        let cost_score = provider.cost_tier as f64 / 5.0;
        let latency_score = provider.latency_tier as f64 / 5.0;
        let quality_score = provider.quality_tier as f64 / 5.0;
        let health_score = provider.health_score;

        match self.routing_config.strategy {
            MigrationStrategy::CostFirst => cost_score * 0.6 + health_score * 0.4,
            MigrationStrategy::LatencyFirst => latency_score * 0.6 + health_score * 0.4,
            MigrationStrategy::QualityFirst => quality_score * 0.6 + health_score * 0.4,
            MigrationStrategy::AvailabilityFirst => health_score * 0.8 + cost_score * 0.2,
        }
    }

    /// 创建迁移计划
    pub fn create_migration_plan(&mut self, source: &str, target: &str, reason: MigrationReason) -> MigrationPlan {
        let plan = MigrationPlan {
            source_provider: source.to_string(),
            target_provider: target.to_string(),
            reason,
            estimated_impact: MigrationImpact {
                estimated_downtime: Duration::from_secs(300),
                cost_change_percent: 0.0,
                latency_change_ms: 0.0,
                affected_jobs: 0,
            },
            steps: vec![
                MigrationStep {
                    name: "配置迁移".to_string(),
                    step_type: StepType::ConfigMigration,
                    dependencies: vec![],
                    estimated_duration: Duration::from_secs(60),
                    status: StepStatus::Pending,
                },
                MigrationStep {
                    name: "流量切换".to_string(),
                    step_type: StepType::TrafficSwitch,
                    dependencies: vec!["配置迁移".to_string()],
                    estimated_duration: Duration::from_secs(30),
                    status: StepStatus::Pending,
                },
                MigrationStep {
                    name: "验证测试".to_string(),
                    step_type: StepType::ValidationTest,
                    dependencies: vec!["流量切换".to_string()],
                    estimated_duration: Duration::from_secs(120),
                    status: StepStatus::Pending,
                },
            ],
            status: MigrationStatus::Planning,
        };

        self.migration_plans.push(plan.clone());
        self.stats.total_migrations += 1;

        plan
    }

    /// 执行迁移
    pub fn execute_migration(&mut self, plan_id: usize) -> bool {
        if let Some(plan) = self.migration_plans.get_mut(plan_id) {
            plan.status = MigrationStatus::InProgress;
            // TODO: 实际执行迁移逻辑
            true
        } else {
            false
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> MigrationStats {
        self.stats.clone()
    }
}

impl Default for ProviderMigrationRouter {
    fn default() -> Self {
        Self::new(RoutingConfig {
            default_provider: "openai".to_string(),
            fallback_providers: vec!["anthropic".to_string(), "google".to_string()],
            strategy: MigrationStrategy::CostFirst,
            health_check_interval: Duration::from_secs(60),
            auto_fallback: true,
        })
    }
}

/// 提供商需求
#[derive(Debug, Clone)]
pub struct ProviderRequirements {
    /// 所需模态
    pub modalities: Vec<String>,
    /// 最大延迟 (ms)
    pub max_latency_ms: f64,
    /// 最大成本
    pub max_cost: f64,
    /// 最小质量
    pub min_quality: u8,
}

/// 迁移统计
#[derive(Debug, Clone, Default)]
pub struct MigrationStats {
    pub total_providers: u32,
    pub total_migrations: u32,
    pub total_fallbacks: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_provider() {
        let mut router = ProviderMigrationRouter::default();
        router.register_provider(ProviderInfo {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            base_url: "https://api.openai.com".to_string(),
            api_version: "v1".to_string(),
            status: ProviderStatus::Available,
            supported_modalities: vec!["text".to_string(), "image".to_string()],
            cost_tier: 3,
            latency_tier: 2,
            quality_tier: 5,
            last_health_check: None,
            health_score: 0.9,
        });

        let requirements = ProviderRequirements {
            modalities: vec!["text".to_string()],
            max_latency_ms: 1000.0,
            max_cost: 0.1,
            min_quality: 4,
        };

        let provider = router.select_provider(&requirements);
        assert!(provider.is_some());
    }
}
