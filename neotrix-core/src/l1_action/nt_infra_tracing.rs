//! L1 基础设施 — 能力调用追踪 (Observability)
//!
//! 每次 Registry/Router/Bridge 调用自动记录:
//! - capability_id, action, latency, success/failure
//! - 聚合统计: 总调用/成功率/平均延迟/P99

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// 追踪 Span — 单次能力调用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySpan {
    pub span_id: String,
    pub capability_id: String,
    pub action: String,
    pub start_ts: u64,
    pub end_ts: Option<u64>,
    pub latency_ms: Option<f64>,
    pub success: bool,
    pub error_msg: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// 能力统计聚合
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityAggregate {
    pub capability_id: String,
    pub total_calls: u64,
    pub successful: u64,
    pub failed: u64,
    pub avg_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub last_call_ts: u64,
    pub error_rate: f64,
}

/// 追踪收集器 — 全局单例
pub struct TracingCollector {
    spans: Vec<CapabilitySpan>,
    aggregates: HashMap<String, CapabilityAggregate>,
    max_spans: usize,
}

impl Default for TracingCollector {
    fn default() -> Self { Self::new() }
}

impl TracingCollector {
    pub fn new() -> Self {
        Self {
            spans: Vec::new(),
            aggregates: HashMap::new(),
            max_spans: 10000,
        }
    }

    /// 开始追踪
    pub fn start_span(&mut self, capability_id: &str, action: &str) -> String {
        let span_id = format!("span_{}", uuid::Uuid::new_v4());
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.spans.push(CapabilitySpan {
            span_id: span_id.clone(),
            capability_id: capability_id.to_string(),
            action: action.to_string(),
            start_ts: now,
            end_ts: None,
            latency_ms: None,
            success: false,
            error_msg: None,
            metadata: HashMap::new(),
        });
        if self.spans.len() > self.max_spans {
            self.spans.remove(0);
        }
        span_id
    }

    /// 结束追踪
    pub fn end_span(&mut self, span_id: &str, success: bool, error_msg: Option<String>) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if let Some(span) = self.spans.iter_mut().find(|s| s.span_id == span_id) {
            span.end_ts = Some(now);
            span.latency_ms = Some(((now - span.start_ts) * 1000) as f64);
            span.success = success;
            span.error_msg = error_msg;
            // 更新聚合
            let entry = self.aggregates.entry(span.capability_id.clone())
                .or_insert_with(|| CapabilityAggregate {
                    capability_id: span.capability_id.clone(),
                    ..Default::default()
                });
            entry.total_calls += 1;
            if success { entry.successful += 1; } else { entry.failed += 1; }
            entry.error_rate = entry.failed as f64 / entry.total_calls as f64;
            entry.last_call_ts = now;
            if let Some(lat) = span.latency_ms {
                let n = entry.total_calls as f64;
                entry.avg_latency_ms = (entry.avg_latency_ms * (n - 1.0) + lat) / n;
            }
        }
    }

    /// 查询聚合统计
    pub fn aggregate(&self, capability_id: &str) -> Option<&CapabilityAggregate> {
        self.aggregates.get(capability_id)
    }

    /// 查询所有聚合
    pub fn all_aggregates(&self) -> &HashMap<String, CapabilityAggregate> {
        &self.aggregates
    }

    /// 查询最近 N 条 span
    pub fn recent_spans(&self, n: usize) -> Vec<&CapabilitySpan> {
        self.spans.iter().rev().take(n).collect()
    }

    /// 查询指定 capability 的 span
    pub fn spans_for(&self, capability_id: &str) -> Vec<&CapabilitySpan> {
        self.spans.iter().filter(|s| s.capability_id == capability_id).collect()
    }
}

// 全局追踪收集器 (lazy_static)
lazy_static::lazy_static! {
    static ref GLOBAL_COLLECTOR: Mutex<TracingCollector> = Mutex::new(TracingCollector::new());
}

/// 便捷函数
pub fn trace_start(capability_id: &str, action: &str) -> String {
    GLOBAL_COLLECTOR.lock().unwrap().start_span(capability_id, action)
}

pub fn trace_end(span_id: &str, success: bool, error: Option<String>) {
    GLOBAL_COLLECTOR.lock().unwrap().end_span(span_id, success, error);
}

pub fn trace_aggregate(capability_id: &str) -> Option<CapabilityAggregate> {
    GLOBAL_COLLECTOR.lock().unwrap().aggregate(capability_id).cloned()
}

pub fn trace_all_aggregates() -> HashMap<String, CapabilityAggregate> {
    GLOBAL_COLLECTOR.lock().unwrap().all_aggregates().clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_lifecycle() {
        let span_id = trace_start("test.cap", "execute");
        trace_end(&span_id, true, None);
        let agg = trace_aggregate("test.cap").unwrap();
        assert_eq!(agg.total_calls, 1);
        assert_eq!(agg.successful, 1);
    }

    #[test]
    fn test_trace_failure() {
        let span_id = trace_start("test.cap2", "execute");
        trace_end(&span_id, false, Some("error".into()));
        let agg = trace_aggregate("test.cap2").unwrap();
        assert_eq!(agg.failed, 1);
        assert!(agg.error_rate > 0.0);
    }
}
