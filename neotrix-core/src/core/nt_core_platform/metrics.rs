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
