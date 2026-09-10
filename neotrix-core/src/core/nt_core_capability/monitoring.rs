//! 能力监控和可观测性
//!
//! 提供能力调用的监控、日志、指标收集

use crate::core::nt_core_capability::*;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 监控配置
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// 启用日志
    pub enable_logging: bool,
    /// 启用指标
    pub enable_metrics: bool,
    /// 启用追踪
    pub enable_tracing: bool,
    /// 采样率
    pub sampling_rate: f64,
    /// 保留时间
    pub retention_period: Duration,
    /// 最大日志条目数
    pub max_log_entries: usize,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_logging: true,
            enable_metrics: true,
            enable_tracing: true,
            sampling_rate: 1.0,
            retention_period: Duration::from_secs(86400), // 24小时
            max_log_entries: 10000,
        }
    }
}

/// 日志级别
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// 监控日志条目
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// 时间戳
    pub timestamp: Instant,
    /// 级别
    pub level: LogLevel,
    /// 能力ID
    pub capability_id: String,
    /// 消息
    pub message: String,
    /// 持续时间
    pub duration_ms: Option<u64>,
    /// 元数据
    pub metadata: std::collections::HashMap<String, String>,
}

/// 能力指标
#[derive(Debug, Clone)]
pub struct CapabilityMetricsData {
    /// 调用次数
    pub call_count: u64,
    /// 成功次数
    pub success_count: u64,
    /// 失败次数
    pub failure_count: u64,
    /// 平均响应时间
    pub avg_response_time_ms: f64,
    /// 最大响应时间
    pub max_response_time_ms: u64,
    /// 最小响应时间
    pub min_response_time_ms: u64,
    /// P95响应时间
    pub p95_response_time_ms: u64,
    /// P99响应时间
    pub p99_response_time_ms: u64,
    /// 吞吐量 (QPS)
    pub throughput_qps: f64,
    /// 错误率
    pub error_rate: f64,
}

impl Default for CapabilityMetricsData {
    fn default() -> Self {
        Self {
            call_count: 0,
            success_count: 0,
            failure_count: 0,
            avg_response_time_ms: 0.0,
            max_response_time_ms: 0,
            min_response_time_ms: u64::MAX,
            p95_response_time_ms: 0,
            p99_response_time_ms: 0,
            throughput_qps: 0.0,
            error_rate: 0.0,
        }
    }
}

/// 监控收集器
pub struct MonitoringCollector {
    /// 配置
    config: MonitoringConfig,
    /// 日志条目
    logs: Vec<LogEntry>,
    /// 指标数据
    metrics: std::collections::HashMap<String, CapabilityMetricsData>,
    /// 响应时间历史
    response_times: std::collections::HashMap<String, Vec<u64>>,
    /// 开始时间
    start_time: Instant,
}

impl MonitoringCollector {
    /// 创建新的监控收集器
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            logs: Vec::new(),
            metrics: std::collections::HashMap::new(),
            response_times: std::collections::HashMap::new(),
            start_time: Instant::now(),
        }
    }

    /// 记录日志
    pub fn log(&mut self, entry: LogEntry) {
        if self.config.enable_logging && self.logs.len() < self.config.max_log_entries {
            self.logs.push(entry);
        }
    }

    /// 记录调用
    pub fn record_call(&mut self, capability_id: &str, success: bool, response_time_ms: u64) {
        let metrics = self
            .metrics
            .entry(capability_id.to_string())
            .or_insert_with(CapabilityMetricsData::default);

        metrics.call_count += 1;
        if success {
            metrics.success_count += 1;
        } else {
            metrics.failure_count += 1;
        }

        // 更新响应时间统计
        let times = self
            .response_times
            .entry(capability_id.to_string())
            .or_insert_with(Vec::new);
        times.push(response_time_ms);

        // 更新平均响应时间
        let total_time = metrics.avg_response_time_ms * (metrics.call_count - 1) as f64
            + response_time_ms as f64;
        metrics.avg_response_time_ms = total_time / metrics.call_count as f64;

        // 更新最大/最小响应时间
        metrics.max_response_time_ms = metrics.max_response_time_ms.max(response_time_ms);
        metrics.min_response_time_ms = metrics.min_response_time_ms.min(response_time_ms);

        // 更新错误率
        metrics.error_rate = metrics.failure_count as f64 / metrics.call_count as f64;

        // 更新吞吐量
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            metrics.throughput_qps = metrics.call_count as f64 / elapsed;
        }

        // 更新P95/P99响应时间
        if times.len() >= 20 {
            let mut sorted = times.clone();
            sorted.sort_unstable();
            let p95_index = (sorted.len() as f64 * 0.95) as usize;
            let p99_index = (sorted.len() as f64 * 0.99) as usize;
            metrics.p95_response_time_ms = sorted.get(p95_index).copied().unwrap_or(0);
            metrics.p99_response_time_ms = sorted.get(p99_index).copied().unwrap_or(0);
        }
    }

    /// 获取能力指标
    pub fn get_metrics(&self, capability_id: &str) -> Option<&CapabilityMetricsData> {
        self.metrics.get(capability_id)
    }

    /// 获取所有指标
    pub fn get_all_metrics(&self) -> &std::collections::HashMap<String, CapabilityMetricsData> {
        &self.metrics
    }

    /// 获取日志
    pub fn get_logs(&self, count: usize) -> Vec<&LogEntry> {
        self.logs.iter().rev().take(count).collect()
    }

    /// 获取指定能力的日志
    pub fn get_capability_logs(&self, capability_id: &str, count: usize) -> Vec<&LogEntry> {
        self.logs
            .iter()
            .filter(|l| l.capability_id == capability_id)
            .rev()
            .take(count)
            .collect()
    }

    /// 清理旧日志
    pub fn cleanup_logs(&mut self, max_age: Duration) {
        let cutoff = Instant::now() - max_age;
        self.logs.retain(|l| l.timestamp > cutoff);
    }

    /// 清理旧响应时间数据
    pub fn cleanup_response_times(&mut self, max_entries: usize) {
        for times in self.response_times.values_mut() {
            if times.len() > max_entries {
                times.drain(0..times.len() - max_entries);
            }
        }
    }

    /// 重置指标
    pub fn reset_metrics(&mut self, capability_id: &str) {
        self.metrics.remove(capability_id);
        self.response_times.remove(capability_id);
    }

    /// 重置所有指标
    pub fn reset_all_metrics(&mut self) {
        self.metrics.clear();
        self.response_times.clear();
    }

    /// 获取系统健康状态
    pub fn system_health(&self) -> SystemHealth {
        let total_calls: u64 = self.metrics.values().map(|m| m.call_count).sum();
        let total_failures: u64 = self.metrics.values().map(|m| m.failure_count).sum();
        let avg_error_rate = if total_calls > 0 {
            total_failures as f64 / total_calls as f64
        } else {
            0.0
        };

        let avg_response_time = if !self.metrics.is_empty() {
            self.metrics
                .values()
                .map(|m| m.avg_response_time_ms)
                .sum::<f64>()
                / self.metrics.len() as f64
        } else {
            0.0
        };

        SystemHealth {
            total_capabilities: self.metrics.len(),
            total_calls,
            total_failures,
            avg_error_rate,
            avg_response_time_ms: avg_response_time,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            log_count: self.logs.len(),
        }
    }
}

/// 系统健康状态
#[derive(Debug, Clone)]
pub struct SystemHealth {
    pub total_capabilities: usize,
    pub total_calls: u64,
    pub total_failures: u64,
    pub avg_error_rate: f64,
    pub avg_response_time_ms: f64,
    pub uptime_seconds: u64,
    pub log_count: usize,
}

/// 监控包装器
pub struct MonitoredCapability {
    /// 底层能力
    capability: Arc<dyn UnifiedCapability>,
    /// 监控收集器
    collector: Arc<std::sync::Mutex<MonitoringCollector>>,
}

impl MonitoredCapability {
    /// 创建监控包装器
    pub fn new(
        capability: Arc<dyn UnifiedCapability>,
        collector: Arc<std::sync::Mutex<MonitoringCollector>>,
    ) -> Self {
        Self {
            capability,
            collector,
        }
    }

    /// 带监控的执行
    pub fn execute_monitored(
        &self,
        input: CapabilityInput,
    ) -> Result<CapabilityOutput, CapabilityError> {
        let start = Instant::now();
        let result = self.capability.execute(input);
        let duration = start.elapsed().as_millis() as u64;

        // 记录日志
        let mut collector = self.collector.lock().unwrap();
        collector.log(LogEntry {
            timestamp: start,
            level: if result.is_ok() {
                LogLevel::Info
            } else {
                LogLevel::Error
            },
            capability_id: self.capability.meta().id.clone(),
            message: if result.is_ok() {
                "执行成功".into()
            } else {
                "执行失败".into()
            },
            duration_ms: Some(duration),
            metadata: std::collections::HashMap::new(),
        });

        // 记录调用
        collector.record_call(&self.capability.meta().id, result.is_ok(), duration);

        result
    }
}

impl UnifiedCapability for MonitoredCapability {
    fn meta(&self) -> CapabilityMeta {
        self.capability.meta()
    }

    fn health(&self) -> CapabilityHealth {
        self.capability.health()
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        self.execute_monitored(input)
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        self.capability.supports(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitoring_collector() {
        let config = MonitoringConfig::default();
        let mut collector = MonitoringCollector::new(config);

        collector.record_call("test", true, 100);
        collector.record_call("test", false, 200);

        let metrics = collector.get_metrics("test");
        assert!(metrics.is_some());
        let metrics = metrics.unwrap();
        assert_eq!(metrics.call_count, 2);
        assert_eq!(metrics.failure_count, 1);
    }

    #[test]
    fn system_health() {
        let config = MonitoringConfig::default();
        let mut collector = MonitoringCollector::new(config);

        collector.record_call("cap1", true, 100);
        collector.record_call("cap2", true, 150);

        let health = collector.system_health();
        assert_eq!(health.total_capabilities, 2);
        assert_eq!(health.total_calls, 2);
    }
}
