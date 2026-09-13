//! MultiRegionScheduler — 多区域调度器
//!
//! 跨区域路由 + 故障转移 + 负载均衡。
//! 支持多区域部署和数据复制。

use std::collections::HashMap;
use std::time::Duration;

/// 区域信息
#[derive(Debug, Clone)]
pub struct RegionInfo {
    /// 区域 ID
    pub id: String,
    /// 区域名
    pub name: String,
    /// 地理位置
    pub location: GeoLocation,
    /// 区域状态
    pub status: RegionStatus,
    /// 延迟 (ms)
    pub latency_ms: f64,
    /// 可用性 (%)
    pub availability_percent: f64,
    /// 成本等级 (1-5)
    pub cost_tier: u8,
    /// 当前负载
    pub current_load: f64,
    /// 最大容量
    pub max_capacity: u32,
    /// 当前使用量
    pub current_usage: u32,
    /// 支持的模态
    pub supported_modalities: Vec<String>,
}

/// 地理位置
#[derive(Debug, Clone)]
pub struct GeoLocation {
    /// 纬度
    pub latitude: f64,
    /// 经度
    pub longitude: f64,
    /// 区域代码
    pub region_code: String,
}

/// 区域状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionStatus {
    Available,
    Degraded,
    Offline,
    Maintenance,
}

/// 路由决策
#[derive(Debug, Clone)]
pub struct RegionalRoutingDecision {
    /// 目标区域
    pub target_region: String,
    /// 预计延迟
    pub estimated_latency_ms: f64,
    /// 路由分数
    pub score: f64,
    /// 备选区域
    pub fallback_regions: Vec<String>,
}

/// 调度策略
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionalStrategy {
    /// 延迟优先
    LatencyFirst,
    /// 成本优先
    CostFirst,
    /// 可用性优先
    AvailabilityFirst,
    /// 负载均衡
    LoadBalanced,
    /// 地理就近
    GeoProximity,
}

/// 多区域调度器
pub struct MultiRegionScheduler {
    /// 区域列表
    regions: HashMap<String, RegionInfo>,
    /// 调度策略
    strategy: RegionalStrategy,
    /// 故障转移配置
    #[allow(dead_code)]
    failover_config: FailoverConfig,
    /// 统计信息
    stats: RegionalStats,
}

/// 故障转移配置
#[derive(Debug, Clone)]
pub struct FailoverConfig {
    /// 是否启用自动故障转移
    pub auto_failover: bool,
    /// 健康检查间隔
    pub health_check_interval: Duration,
    /// 故障检测阈值
    pub failure_threshold: u32,
    /// 最大故障转移次数
    pub max_failovers: u32,
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            auto_failover: true,
            health_check_interval: Duration::from_secs(30),
            failure_threshold: 3,
            max_failovers: 5,
        }
    }
}

impl MultiRegionScheduler {
    pub fn new(strategy: RegionalStrategy) -> Self {
        Self {
            regions: HashMap::new(),
            strategy,
            failover_config: FailoverConfig::default(),
            stats: RegionalStats::default(),
        }
    }

    /// 注册区域
    pub fn register_region(&mut self, region: RegionInfo) {
        self.regions.insert(region.id.clone(), region);
        self.stats.total_regions += 1;
    }

    /// 路由请求
    pub fn route(&self, requirements: &RegionalRequirements) -> Option<RegionalRoutingDecision> {
        let mut candidates: Vec<_> = self.regions.values()
            .filter(|r| r.status == RegionStatus::Available)
            .filter(|r| requirements.modalities.iter().all(|m| r.supported_modalities.contains(m)))
            .collect();

        match self.strategy {
            RegionalStrategy::LatencyFirst => {
                candidates.sort_by(|a, b| a.latency_ms.partial_cmp(&b.latency_ms).unwrap_or(std::cmp::Ordering::Equal));
            }
            RegionalStrategy::CostFirst => {
                candidates.sort_by(|a, b| a.cost_tier.cmp(&b.cost_tier));
            }
            RegionalStrategy::AvailabilityFirst => {
                candidates.sort_by(|a, b| b.availability_percent.partial_cmp(&a.availability_percent).unwrap_or(std::cmp::Ordering::Equal));
            }
            RegionalStrategy::LoadBalanced => {
                candidates.sort_by(|a, b| a.current_load.partial_cmp(&b.current_load).unwrap_or(std::cmp::Ordering::Equal));
            }
            RegionalStrategy::GeoProximity => {
                // 按地理距离排序
                candidates.sort_by(|a, b| {
                    let dist_a = self.calculate_distance(&requirements.origin, &a.location);
                    let dist_b = self.calculate_distance(&requirements.origin, &b.location);
                    dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }

        candidates.first().map(|region| {
            let fallback_regions: Vec<_> = candidates.iter()
                .skip(1)
                .take(3)
                .map(|r| r.id.clone())
                .collect();

            RegionalRoutingDecision {
                target_region: region.id.clone(),
                estimated_latency_ms: region.latency_ms,
                score: region.availability_percent / 100.0,
                fallback_regions,
            }
        })
    }

    /// 计算地理距离
    fn calculate_distance(&self, origin: &GeoLocation, target: &GeoLocation) -> f64 {
        let dx = origin.latitude - target.latitude;
        let dy = origin.longitude - target.longitude;
        (dx * dx + dy * dy).sqrt()
    }

    /// 检查区域健康
    pub(crate) fn _check_region_health(&mut self, region_id: &str) -> Result<bool, String> {
        if let Some(_region) = self.regions.get(region_id) {
            Err("not wired: region health check not implemented".to_string())
        } else {
            Err(format!("region '{}' not found", region_id))
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> RegionalStats {
        self.stats.clone()
    }
}

impl Default for MultiRegionScheduler {
    fn default() -> Self {
        Self::new(RegionalStrategy::LatencyFirst)
    }
}

/// 区域需求
#[derive(Debug, Clone)]
pub struct RegionalRequirements {
    /// 所需模态
    pub modalities: Vec<String>,
    /// 请求来源位置
    pub origin: GeoLocation,
    /// 最大延迟 (ms)
    pub max_latency_ms: f64,
    /// 最小可用性 (%)
    pub min_availability: f64,
}

/// 区域统计
#[derive(Debug, Clone, Default)]
pub struct RegionalStats {
    pub total_regions: u32,
    pub total_routed: u32,
    pub total_failovers: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_to_region() {
        let mut scheduler = MultiRegionScheduler::default();
        scheduler.register_region(RegionInfo {
            id: "us-east-1".to_string(),
            name: "US East".to_string(),
            location: GeoLocation {
                latitude: 37.7749,
                longitude: -122.4194,
                region_code: "us-east-1".to_string(),
            },
            status: RegionStatus::Available,
            latency_ms: 50.0,
            availability_percent: 99.9,
            cost_tier: 3,
            current_load: 0.3,
            max_capacity: 1000,
            current_usage: 300,
            supported_modalities: vec!["text".to_string(), "image".to_string()],
        });

        let requirements = RegionalRequirements {
            modalities: vec!["text".to_string()],
            origin: GeoLocation {
                latitude: 37.7749,
                longitude: -122.4194,
                region_code: "us-west-2".to_string(),
            },
            max_latency_ms: 100.0,
            min_availability: 99.0,
        };

        let decision = scheduler.route(&requirements);
        assert!(decision.is_some());
    }
}
