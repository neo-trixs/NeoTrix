//! ObservabilityStack — 可观测性栈
//!
//! 集成 traces, metrics, logs, evals 四大支柱，提供生产级可观测性。
//! 支持 per-layer latency budgets, cost attribution, 诊断上下文。

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 可观测性事件类型
#[derive(Debug, Clone, PartialEq)]
pub enum ObservabilityEvent {
    /// Trace 事件 (分布式追踪)
    TraceStart { span_id: String, parent_id: Option<String> },
    TraceEnd { span_id: String, duration: Duration },

    /// Metrics 事件 (指标)
    CounterIncrement { name: String, value: f64 },
    GaugeSet { name: String, value: f64 },
    HistogramRecord { name: String, value: f64 },

    /// Log 事件 (日志)
    Log { level: LogLevel, message: String, fields: HashMap<String, String> },

    /// Eval 事件 (评估)
    EvalScore { metric: String, score: f64 },
}

/// 日志级别
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// 单条可观测性记录
#[derive(Debug, Clone)]
pub struct ObservabilityRecord {
    pub event: ObservabilityEvent,
    pub timestamp: Instant,
    pub service: String,
    pub layer: Option<String>,
    pub latency_ms: Option<f64>,
    pub cost_usd: Option<f64>,
}

/// 可观测性栈
pub struct ObservabilityStack {
    /// 事件历史
    records: Vec<ObservabilityRecord>,
    /// 最大记录数
    max_records: usize,
    /// 服务名
    service: String,
    /// 当前活跃 span
    active_spans: HashMap<String, Instant>,
    /// 延迟预算 (per-layer)
    latency_budgets: HashMap<String, Duration>,
}

impl ObservabilityStack {
    pub fn new(service: &str) -> Self {
        Self {
            records: Vec::new(),
            max_records: 10_000,
            service: service.to_string(),
            active_spans: HashMap::new(),
            latency_budgets: HashMap::new(),
        }
    }

    /// 设置层延迟预算
    pub(crate) fn _set_latency_budget(&mut self, layer: &str, budget: Duration) {
        self.latency_budgets.insert(layer.to_string(), budget);
    }

    /// 开始 trace span
    pub fn trace_start(&mut self, span_id: &str, parent_id: Option<String>, layer: Option<String>) {
        self.active_spans.insert(span_id.to_string(), Instant::now());
        self.record(ObservabilityEvent::TraceStart {
            span_id: span_id.to_string(),
            parent_id,
        }, layer, None);
    }

    /// 结束 trace span
    pub fn trace_end(&mut self, span_id: &str, layer: Option<String>) -> Option<Duration> {
        if let Some(start) = self.active_spans.remove(span_id) {
            let duration = start.elapsed();
            self.record(ObservabilityEvent::TraceEnd {
                span_id: span_id.to_string(),
                duration,
            }, layer.clone(), None);

            // 检查延迟预算
            if let Some(layer_name) = &layer {
                if let Some(budget) = self.latency_budgets.get(layer_name) {
                    if duration > *budget {
                        self.log_warn(&format!("Layer {} exceeded latency budget: {:?} > {:?}", layer_name, duration, budget), Some(layer_name.clone()));
                    }
                }
            }

            Some(duration)
        } else {
            None
        }
    }

    /// 记录计数器
    pub fn counter(&mut self, name: &str, value: f64, layer: Option<String>) {
        self.record(ObservabilityEvent::CounterIncrement {
            name: name.to_string(),
            value,
        }, layer, None);
    }

    /// 记录 gauge
    pub fn gauge(&mut self, name: &str, value: f64, layer: Option<String>) {
        self.record(ObservabilityEvent::GaugeSet {
            name: name.to_string(),
            value,
        }, layer, None);
    }

    /// 记录直方图
    pub fn histogram(&mut self, name: &str, value: f64, layer: Option<String>) {
        self.record(ObservabilityEvent::HistogramRecord {
            name: name.to_string(),
            value,
        }, layer, None);
    }

    /// 记录日志
    pub fn log(&mut self, level: LogLevel, message: &str, fields: HashMap<String, String>, layer: Option<String>) {
        self.record(ObservabilityEvent::Log {
            level,
            message: message.to_string(),
            fields,
        }, layer, None);
    }

    pub fn log_debug(&mut self, message: &str, layer: Option<String>) {
        self.log(LogLevel::Debug, message, HashMap::new(), layer);
    }

    pub fn log_info(&mut self, message: &str, layer: Option<String>) {
        self.log(LogLevel::Info, message, HashMap::new(), layer);
    }

    pub fn log_warn(&mut self, message: &str, layer: Option<String>) {
        self.log(LogLevel::Warn, message, HashMap::new(), layer);
    }

    pub fn log_error(&mut self, message: &str, layer: Option<String>) {
        self.log(LogLevel::Error, message, HashMap::new(), layer);
    }

    /// 记录评估分数
    pub fn eval(&mut self, metric: &str, score: f64, layer: Option<String>) {
        self.record(ObservabilityEvent::EvalScore {
            metric: metric.to_string(),
            score,
        }, layer, None);
    }

    /// 记录事件
    fn record(&mut self, event: ObservabilityEvent, layer: Option<String>, latency_ms: Option<f64>) {
        self.records.push(ObservabilityRecord {
            event,
            timestamp: Instant::now(),
            service: self.service.clone(),
            layer,
            latency_ms,
            cost_usd: None,
        });

        if self.records.len() > self.max_records {
            self.records.remove(0);
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> ObservabilityStats {
        let mut traces = 0;
        let mut metrics = 0;
        let mut logs = 0;
        let mut evals = 0;

        for record in &self.records {
            match &record.event {
                ObservabilityEvent::TraceStart { .. } | ObservabilityEvent::TraceEnd { .. } => traces += 1,
                ObservabilityEvent::CounterIncrement { .. } | ObservabilityEvent::GaugeSet { .. } | ObservabilityEvent::HistogramRecord { .. } => metrics += 1,
                ObservabilityEvent::Log { .. } => logs += 1,
                ObservabilityEvent::EvalScore { .. } => evals += 1,
            }
        }

        ObservabilityStats {
            total_records: self.records.len() as u32,
            traces,
            metrics,
            logs,
            evals,
            active_spans: self.active_spans.len() as u32,
        }
    }
}

impl Default for ObservabilityStack {
    fn default() -> Self {
        Self::new("neotrix")
    }
}

/// 可观测性统计
#[derive(Debug, Clone)]
pub struct ObservabilityStats {
    pub total_records: u32,
    pub traces: u32,
    pub metrics: u32,
    pub logs: u32,
    pub evals: u32,
    pub active_spans: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_lifecycle() {
        let mut obs = ObservabilityStack::new("test");
        obs.trace_start("span-1", None, Some("L1".to_string()));
        std::thread::sleep(Duration::from_millis(10));
        let duration = obs.trace_end("span-1", Some("L1".to_string()));
        assert!(duration.is_some());
    }

    #[test]
    fn test_counter() {
        let mut obs = ObservabilityStack::new("test");
        obs.counter("api.calls", 1.0, None);
        let stats = obs.stats();
        assert_eq!(stats.metrics, 1);
    }
}
