/// NT-CORE Heartbeat — 统一系统健康聚合器
///
/// 聚合所有子系统的健康状态，提供统一的健康检查接口。

use std::collections::HashMap;

/// 健康状态
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// 健康报告
#[derive(Debug, Clone)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub components: HashMap<String, ComponentHealth>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 组件健康状态
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
}

/// 健康聚合器
pub struct HeartbeatAggregator {
    components: HashMap<String, ComponentHealth>,
}

impl HeartbeatAggregator {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// 记录组件健康状态
    pub fn record(&mut self, name: &str, status: HealthStatus, message: Option<String>) {
        self.components.insert(
            name.to_string(),
            ComponentHealth {
                name: name.to_string(),
                status,
                message,
            },
        );
    }

    /// 生成健康报告
    pub fn report(&self) -> HealthReport {
        let overall_status = if self.components.values().all(|c| c.status == HealthStatus::Healthy) {
            HealthStatus::Healthy
        } else if self.components.values().any(|c| c.status == HealthStatus::Unhealthy) {
            HealthStatus::Unhealthy
        } else if self.components.values().any(|c| c.status == HealthStatus::Degraded) {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unknown
        };

        HealthReport {
            status: overall_status,
            components: self.components.clone(),
            timestamp: chrono::Utc::now(),
        }
    }
}
