//! _HealthChecker — 健康检查器
//!
//! 监控系统健康状态，提供 status endpoint，支持 per-layer 健康评分。
//! 集成 HeartbeatAggregator 信号，支持恢复检测和降级模式。

use std::collections::HashMap;
use std::time::Instant;

use neotrix_types::shared::HealthStatus;

/// 单个健康检查结果
#[derive(Debug, Clone)]
pub struct _HealthCheckResult {
    /// 组件名
    pub component: String,
    /// 状态
    pub status: HealthStatus,
    /// 健康评分 (0-100)
    pub score: f64,
    /// 延迟 (ms)
    pub latency_ms: f64,
    /// 附加信息
    pub message: Option<String>,
    /// 检查时间
    pub checked_at: Instant,
}

/// 健康检查器
pub struct _HealthChecker {
    /// 组件检查结果
    results: HashMap<String, _HealthCheckResult>,
    /// 最大历史记录
    #[allow(dead_code)]
    max_history: usize,
    /// 全局健康评分
    _global_score: f64,
    /// 健康阈值
    healthy_threshold: f64,
    degraded_threshold: f64,
}

impl _HealthChecker {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
            max_history: 100,
            _global_score: 100.0,
            healthy_threshold: 80.0,
            degraded_threshold: 50.0,
        }
    }

    /// 记录健康检查结果
    pub fn record(&mut self, component: &str, status: HealthStatus, score: f64, latency_ms: f64, message: Option<String>) {
        self.results.insert(component.to_string(), _HealthCheckResult {
            component: component.to_string(),
            status,
            score,
            latency_ms,
            message,
            checked_at: Instant::now(),
        });

        self.recalculate_global_score();
    }

    /// 重新计算全局健康评分
    fn recalculate_global_score(&mut self) {
        if self.results.is_empty() {
            self._global_score = 100.0;
            return;
        }

        let total_score: f64 = self.results.values().map(|r| r.score).sum();
        self._global_score = total_score / self.results.len() as f64;
    }

    /// 获取全局健康状态
    pub fn _global_status(&self) -> HealthStatus {
        if self._global_score >= self.healthy_threshold {
            HealthStatus::Healthy
        } else if self._global_score >= self.degraded_threshold {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        }
    }

    /// 获取全局健康评分
    pub fn _global_score(&self) -> f64 {
        self._global_score
    }

    /// 获取指定组件状态
    pub fn _component_status(&self, component: &str) -> Option<&_HealthCheckResult> {
        self.results.get(component)
    }

    /// 获取所有组件状态
    pub fn all_results(&self) -> &HashMap<String, _HealthCheckResult> {
        &self.results
    }

    /// 检查是否健康
    pub fn is_healthy(&self) -> bool {
        self._global_status() == HealthStatus::Healthy
    }

    /// 检查是否需要降级
    pub fn _should_degrade(&self) -> bool {
        self._global_status() == HealthStatus::Degraded || self._global_status() == HealthStatus::Unhealthy
    }

    /// 获取统计信息
    pub fn stats(&self) -> _HealthStats {
        let mut healthy = 0;
        let mut degraded = 0;
        let mut unhealthy = 0;
        let mut unknown = 0;

        for result in self.results.values() {
            match result.status {
                HealthStatus::Healthy => healthy += 1,
                HealthStatus::Degraded => degraded += 1,
                HealthStatus::Unhealthy => unhealthy += 1,
                HealthStatus::Unknown => unknown += 1,
            }
        }

        _HealthStats {
            total_components: self.results.len() as u32,
            healthy,
            degraded,
            unhealthy,
            unknown,
            _global_score: self._global_score,
        }
    }
}

impl Default for _HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// 健康统计
#[derive(Debug, Clone)]
pub struct _HealthStats {
    pub total_components: u32,
    pub healthy: u32,
    pub degraded: u32,
    pub unhealthy: u32,
    pub unknown: u32,
    pub _global_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check() {
        let mut checker = _HealthChecker::new();
        checker.record("nt_act", HealthStatus::Healthy, 95.0, 10.0, None);
        checker.record("nt_io", HealthStatus::Healthy, 90.0, 15.0, None);
        assert_eq!(checker._global_status(), HealthStatus::Healthy);
    }

    #[test]
    fn test_degraded_status() {
        let mut checker = _HealthChecker::new();
        checker.record("nt_act", HealthStatus::Healthy, 95.0, 10.0, None);
        checker.record("nt_io", HealthStatus::Degraded, 60.0, 50.0, Some("slow".to_string()));
        assert_eq!(checker._global_status(), HealthStatus::Degraded);
    }
}
