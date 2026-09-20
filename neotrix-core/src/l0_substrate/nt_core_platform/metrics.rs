#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
}

pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<String, MetricValue>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn increment_counter(&self, name: &str, value: u64) {
        let mut metrics = self.metrics.write().await;
        let entry = metrics.entry(name.to_string()).or_insert_with(|| MetricValue::Counter(0));
        if let MetricValue::Counter(ref mut count) = *entry {
            *count += value;
        }
    }

    pub async fn set_gauge(&self, name: &str, value: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.insert(name.to_string(), MetricValue::Gauge(value));
    }

    pub async fn record_histogram(&self, name: &str, value: f64) {
        let mut metrics = self.metrics.write().await;
        let entry = metrics.entry(name.to_string()).or_insert_with(|| MetricValue::Histogram(Vec::new()));
        if let MetricValue::Histogram(ref mut values) = *entry {
            values.push(value);
        }
    }

    pub async fn get(&self, name: &str) -> Option<MetricValue> {
        let metrics = self.metrics.read().await;
        metrics.get(name).cloned()
    }

    pub async fn snapshot(&self) -> HashMap<String, MetricValue> {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_type_serde_roundtrip() {
        let types = vec![MetricType::Counter, MetricType::Gauge, MetricType::Histogram];
        for t in types {
            let json = serde_json::to_string(&t).unwrap();
            let back: MetricType = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{:?}", t), format!("{:?}", back));
        }
    }

    #[test]
    fn metric_value_serde_roundtrip() {
        let values = vec![
            MetricValue::Counter(42),
            MetricValue::Gauge(3.14),
            MetricValue::Histogram(vec![1.0, 2.0, 3.0]),
        ];
        for v in values {
            let json = serde_json::to_string(&v).unwrap();
            let back: MetricValue = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{:?}", v), format!("{:?}", back));
        }
    }

    #[tokio::test]
    async fn metrics_collector_counter() {
        let mc = MetricsCollector::new();
        mc.increment_counter("test.counter", 5).await;
        mc.increment_counter("test.counter", 3).await;
        let val = mc.get("test.counter").await.unwrap();
        assert!(matches!(val, MetricValue::Counter(8)));
    }

    #[tokio::test]
    async fn metrics_collector_gauge() {
        let mc = MetricsCollector::new();
        mc.set_gauge("test.gauge", 42.5).await;
        let val = mc.get("test.gauge").await.unwrap();
        assert!(matches!(val, MetricValue::Gauge(v) if (v - 42.5).abs() < 1e-10));
    }

    #[tokio::test]
    async fn metrics_collector_histogram() {
        let mc = MetricsCollector::new();
        mc.record_histogram("test.hist", 1.0).await;
        mc.record_histogram("test.hist", 2.0).await;
        mc.record_histogram("test.hist", 3.0).await;
        let val = mc.get("test.hist").await.unwrap();
        if let MetricValue::Histogram(values) = val {
            assert_eq!(values, vec![1.0, 2.0, 3.0]);
        } else {
            panic!("expected Histogram");
        }
    }

    #[tokio::test]
    async fn metrics_collector_snapshot() {
        let mc = MetricsCollector::new();
        mc.increment_counter("a", 1).await;
        mc.set_gauge("b", 2.0).await;
        let snap = mc.snapshot().await;
        assert_eq!(snap.len(), 2);
    }

    #[tokio::test]
    async fn metrics_collector_get_none() {
        let mc = MetricsCollector::new();
        assert!(mc.get("nonexistent").await.is_none());
    }
}
