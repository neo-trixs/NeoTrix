//! HealthChecker — 健康检查器
//!
//! 监控系统健康状态，提供 status endpoint，支持 per-layer 健康评分。
//! 集成 HeartbeatAggregator 信号，支持恢复检测和降级模式。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 组件健康状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// 单个健康检查结果
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
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
pub struct HealthChecker {
    /// 组件检查结果
    results: HashMap<String, HealthCheckResult>,
    /// 最大历史记录
    max_history: usize,
    /// 全局健康评分
    global_score: f64,
    /// 健康阈值
    healthy_threshold: f64,
    degraded_threshold: f64,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
            max_history: 100,
            global_score: 100.0,
            healthy_threshold: 80.0,
            degraded_threshold: 50.0,
        }
    }

    /// 记录健康检查结果
    pub fn record(&mut self, component: &str, status: HealthStatus, score: f64, latency_ms: f64, message: Option<String>) {
        self.results.insert(component.to_string(), HealthCheckResult {
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
            self.global_score = 100.0;
            return;
        }

        let total_score: f64 = self.results.values().map(|r| r.score).sum();
        self.global_score = total_score / self.results.len() as f64;
    }

    /// 获取全局健康状态
    pub fn global_status(&self) -> HealthStatus {
        if self.global_score >= self.healthy_threshold {
            HealthStatus::Healthy
        } else if self.global_score >= self.degraded_threshold {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        }
    }

    /// 获取全局健康评分
    pub fn global_score(&self) -> f64 {
        self.global_score
    }

    /// 获取指定组件状态
    pub fn component_status(&self, component: &str) -> Option<&HealthCheckResult> {
        self.results.get(component)
    }

    /// 获取所有组件状态
    pub fn all_results(&self) -> &HashMap<String, HealthCheckResult> {
        &self.results
    }

    /// 检查是否健康
    pub fn is_healthy(&self) -> bool {
        self.global_status() == HealthStatus::Healthy
    }

    /// 检查是否需要降级
    pub fn should_degrade(&self) -> bool {
        self.global_status() == HealthStatus::Degraded || self.global_status() == HealthStatus::Unhealthy
    }

    /// 获取统计信息
    pub fn stats(&self) -> HealthStats {
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

        HealthStats {
            total_components: self.results.len() as u32,
            healthy,
            degraded,
            unhealthy,
            unknown,
            global_score: self.global_score,
        }
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// 健康统计
#[derive(Debug, Clone)]
pub struct HealthStats {
    pub total_components: u32,
    pub healthy: u32,
    pub degraded: u32,
    pub unhealthy: u32,
    pub unknown: u32,
    pub global_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check() {
        let mut checker = HealthChecker::new();
        checker.record("nt_act", HealthStatus::Healthy, 95.0, 10.0, None);
        checker.record("nt_io", HealthStatus::Healthy, 90.0, 15.0, None);
        assert_eq!(checker.global_status(), HealthStatus::Healthy);
    }

    #[test]
    fn test_degraded_status() {
        let mut checker = HealthChecker::new();
        checker.record("nt_act", HealthStatus::Healthy, 95.0, 10.0, None);
        checker.record("nt_io", HealthStatus::Degraded, 60.0, 50.0, Some("slow".to_string()));
        assert_eq!(checker.global_status(), HealthStatus::Degraded);
    }
}
