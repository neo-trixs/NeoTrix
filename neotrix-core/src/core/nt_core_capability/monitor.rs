//! NeoTrix 能力监控仪表盘
//!
//! 实时监控所有能力的健康状态和性能指标

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::core::nt_core_capability::*;

/// 监控事件
#[derive(Debug, Clone)]
pub struct MonitorEvent {
    /// 事件时间
    pub timestamp: Instant,
    /// 能力ID
    pub capability_id: String,
    /// 事件类型
    pub event_type: EventType,
    /// 事件详情
    pub details: String,
}

/// 事件类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventType {
    /// 能力调用
    Call,
    /// 调用成功
    Success,
    /// 调用失败
    Failure,
    /// 延迟异常
    SlowResponse,
    /// 状态变更
    StateChange,
}

/// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// 调用次数
    pub call_count: u64,
    /// 成功次数
    pub success_count: u64,
    /// 失败次数
    pub failure_count: u64,
    /// 平均延迟 (ms)
    pub avg_latency_ms: f64,
    /// 最大延迟 (ms)
    pub max_latency_ms: u64,
    /// 最小延迟 (ms)
    pub min_latency_ms: u64,
    /// P95延迟 (ms)
    pub p95_latency_ms: u64,
    /// 成功率
    pub success_rate: f64,
    /// 吞吐量 (QPS)
    pub throughput_qps: f64,
}

/// 监控仪表盘
pub struct MonitorDashboard {
    /// 能力注册中心
    registry: Arc<CapabilityRegistry>,
    /// 事件日志
    events: Vec<MonitorEvent>,
    /// 性能指标
    metrics: HashMap<String, PerformanceMetrics>,
    /// 延迟历史
    latency_history: HashMap<String, Vec<u64>>,
    /// 最大事件数
    max_events: usize,
}

impl MonitorDashboard {
    /// 创建新的仪表盘
    pub fn new(registry: Arc<CapabilityRegistry>) -> Self {
        Self {
            registry,
            events: Vec::new(),
            metrics: HashMap::new(),
            latency_history: HashMap::new(),
            max_events: 1000,
        }
    }

    /// 记录事件
    pub fn record_event(&mut self, event: MonitorEvent) {
        if self.events.len() >= self.max_events {
            self.events.remove(0);
        }
        self.events.push(event.clone());

        // 更新指标
        let metrics = self.metrics.entry(event.capability_id.clone()).or_insert_with(|| PerformanceMetrics {
            call_count: 0,
            success_count: 0,
            failure_count: 0,
            avg_latency_ms: 0.0,
            max_latency_ms: 0,
            min_latency_ms: u64::MAX,
            p95_latency_ms: 0,
            success_rate: 1.0,
            throughput_qps: 0.0,
        });

        match event.event_type {
            EventType::Call => metrics.call_count += 1,
            EventType::Success => metrics.success_count += 1,
            EventType::Failure => metrics.failure_count += 1,
            _ => {}
        }

        // 更新成功率
        if metrics.call_count > 0 {
            metrics.success_rate = metrics.success_count as f64 / metrics.call_count as f64;
        }
    }

    /// 记录延迟
    pub fn record_latency(&mut self, capability_id: &str, latency_ms: u64) {
        let history = self.latency_history.entry(capability_id.to_string()).or_insert_with(Vec::new);
        history.push(latency_ms);

        // 保持最近1000个样本
        if history.len() > 1000 {
            history.remove(0);
        }

        // 更新指标
        if let Some(metrics) = self.metrics.get_mut(capability_id) {
            metrics.max_latency_ms = metrics.max_latency_ms.max(latency_ms);
            metrics.min_latency_ms = metrics.min_latency_ms.min(latency_ms);

            // 计算平均延迟
            let sum: u64 = history.iter().sum();
            metrics.avg_latency_ms = sum as f64 / history.len() as f64;

            // 计算P95延迟
            let mut sorted = history.clone();
            sorted.sort_unstable();
            let p95_index = (sorted.len() as f64 * 0.95) as usize;
            metrics.p95_latency_ms = sorted.get(p95_index).copied().unwrap_or(0);
        }
    }

    /// 获取所有能力状态
    pub fn get_all_status(&self) -> Vec<CapabilityStatusReport> {
        self.registry.list_all().iter().map(|meta| {
            let health = self.registry.get(&meta.id)
                .map(|cap| cap.health())
                .unwrap_or(CapabilityHealth {
                    state: CapabilityState::Disabled,
                    success_rate: 0.0,
                    avg_latency_ms: 0,
                    last_called: None,
                    call_count: 0,
                });

            let metrics = self.metrics.get(&meta.id).cloned().unwrap_or(PerformanceMetrics {
                call_count: 0,
                success_count: 0,
                failure_count: 0,
                avg_latency_ms: 0.0,
                max_latency_ms: 0,
                min_latency_ms: 0,
                p95_latency_ms: 0,
                success_rate: 0.0,
                throughput_qps: 0.0,
            });

            CapabilityStatusReport {
                meta: meta.clone(),
                health,
                metrics,
            }
        }).collect()
    }

    /// 获取特定能力状态
    pub fn get_status(&self, capability_id: &str) -> Option<CapabilityStatusReport> {
        self.get_all_status().into_iter()
            .find(|s| s.meta.id == capability_id)
    }

    /// 获取最近事件
    pub fn recent_events(&self, count: usize) -> Vec<&MonitorEvent> {
        self.events.iter().rev().take(count).collect()
    }

    /// 获取事件统计
    pub fn event_stats(&self) -> EventStats {
        let mut stats = EventStats::default();
        for event in &self.events {
            match event.event_type {
                EventType::Call => stats.total_calls += 1,
                EventType::Success => stats.total_success += 1,
                EventType::Failure => stats.total_failures += 1,
                EventType::SlowResponse => stats.slow_responses += 1,
                _ => {}
            }
        }
        stats
    }

    /// 清理旧事件
    pub fn cleanup(&mut self, max_age: Duration) {
        let cutoff = Instant::now() - max_age;
        self.events.retain(|e| e.timestamp > cutoff);
    }
}

/// 能力状态报告
#[derive(Debug, Clone)]
pub struct CapabilityStatusReportReport {
    /// 元数据
    pub meta: CapabilityMeta,
    /// 健康状态
    pub health: CapabilityHealth,
    /// 性能指标
    pub metrics: PerformanceMetrics,
}

/// 事件统计
#[derive(Debug, Clone, Default)]
pub struct EventStats {
    /// 总调用次数
    pub total_calls: u64,
    /// 总成功次数
    pub total_success: u64,
    /// 总失败次数
    pub total_failures: u64,
    /// 慢响应次数
    pub slow_responses: u64,
}

impl EventStats {
    /// 成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_calls == 0 {
            0.0
        } else {
            self.total_success as f64 / self.total_calls as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_creation() {
        let registry = Arc::new(init_global_registry());
        let dashboard = MonitorDashboard::new(registry);
        assert!(dashboard.get_all_status().len() >= 3);
    }

    #[test]
    fn record_event() {
        let registry = Arc::new(init_global_registry());
        let mut dashboard = MonitorDashboard::new(registry);

        dashboard.record_event(MonitorEvent {
            timestamp: Instant::now(),
            capability_id: "test".into(),
            event_type: EventType::Call,
            details: "test call".into(),
        });

        let stats = dashboard.event_stats();
        assert_eq!(stats.total_calls, 1);
    }

    #[test]
    fn record_latency() {
        let registry = Arc::new(init_global_registry());
        let mut dashboard = MonitorDashboard::new(registry);

        dashboard.record_latency("test", 100);
        dashboard.record_latency("test", 200);

        let status = dashboard.get_status("test");
        assert!(status.is_some());
    }
}
