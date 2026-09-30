//! ObservabilitySubstrate — 可观测性基础设施
//!
//! 基于 logfire + OpenTelemetry。
//! - Span 分布式追踪
//! - Metric 指标
//! - LogEntry 结构化日志
//! - OTLP 导出格式

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub id: String,
    pub name: String,
    pub start_ms: u64,
    pub end_ms: Option<u64>,
    pub status: SpanStatus,
    pub attributes: HashMap<String, String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpanStatus {
    Ok,
    Error,
    Unset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub fields: HashMap<String, String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpExport {
    pub traces: Vec<Span>,
    pub metrics: Vec<Metric>,
    pub logs: Vec<LogEntry>,
}

pub struct ObservabilitySubstrate {
    pub spans: Vec<Span>,
    pub metrics: Vec<Metric>,
    pub logs: Vec<LogEntry>,
    pub active_spans: HashMap<String, usize>,
}

impl ObservabilitySubstrate {
    pub fn new() -> Self {
        Self { spans: Vec::new(), metrics: Vec::new(), logs: Vec::new(), active_spans: HashMap::new() }
    }

    pub fn start_span(&mut self, name: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let span = Span {
            id: id.clone(), name: name.to_string(), start_ms: now_ms(), end_ms: None,
            status: SpanStatus::Unset, attributes: HashMap::new(), parent_id: None,
        };
        self.active_spans.insert(id.clone(), self.spans.len());
        self.spans.push(span);
        id
    }

    pub fn end_span(&mut self, span_id: &str, status: SpanStatus) {
        if let Some(&idx) = self.active_spans.get(span_id) {
            self.spans[idx].end_ms = Some(now_ms());
            self.spans[idx].status = status;
            self.active_spans.remove(span_id);
        }
    }

    pub fn record_metric(&mut self, name: &str, value: f64, labels: HashMap<String, String>) {
        self.metrics.push(Metric { name: name.to_string(), value, labels, timestamp: now_ms() });
    }

    pub fn log(&mut self, level: LogLevel, message: &str, fields: HashMap<String, String>) {
        self.logs.push(LogEntry { level, message: message.to_string(), fields, timestamp: now_ms() });
    }

    pub fn export_otlp(&self) -> OtlpExport {
        OtlpExport {
            traces: self.spans.clone(),
            metrics: self.metrics.clone(),
            logs: self.logs.clone(),
        }
    }

    pub fn query(&self, filter: &str) -> Vec<&LogEntry> {
        self.logs.iter().filter(|l| l.message.contains(filter) || l.fields.values().any(|v| v.contains(filter))).collect()
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        (self.spans.len(), self.metrics.len(), self.logs.len())
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_lifecycle() {
        let mut obs = ObservabilitySubstrate::new();
        let id = obs.start_span("test_span");
        obs.end_span(&id, SpanStatus::Ok);
        assert!(obs.spans[0].end_ms.is_some());
    }

    #[test]
    fn test_metric_and_log() {
        let mut obs = ObservabilitySubstrate::new();
        obs.record_metric("latency", 42.0, HashMap::new());
        obs.log(LogLevel::Info, "test message", HashMap::new());
        let (s, m, l) = obs.stats();
        assert_eq!(s, 0);
        assert_eq!(m, 1);
        assert_eq!(l, 1);
    }

    #[test]
    fn test_query() {
        let mut obs = ObservabilitySubstrate::new();
        obs.log(LogLevel::Info, "error in module X", HashMap::new());
        obs.log(LogLevel::Info, "all good", HashMap::new());
        let results = obs.query("error");
        assert_eq!(results.len(), 1);
    }
}
