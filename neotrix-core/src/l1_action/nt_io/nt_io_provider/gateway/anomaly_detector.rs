use std::collections::{HashMap, VecDeque};
use std::sync::RwLock;
use std::time::Instant;

/// 异常检测器 — 基于统计方法检测 provider 异常
pub struct AnomalyDetector {
    windows: RwLock<HashMap<String, SlidingWindow>>,
    config: AnomalyConfig,
    alerts: RwLock<Vec<AnomalyAlert>>,
}

#[derive(Debug, Clone)]
pub struct AnomalyConfig {
    pub window_size: usize,
    pub z_score_threshold: f64,
    pub min_samples: usize,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            window_size: 100,
            z_score_threshold: 2.5,
            min_samples: 10,
        }
    }
}

#[derive(Debug, Clone)]
struct SlidingWindow {
    values: VecDeque<f64>,
    sum: f64,
    sum_sq: f64,
}

impl SlidingWindow {
    fn new(capacity: usize) -> Self {
        Self {
            values: VecDeque::with_capacity(capacity),
            sum: 0.0,
            sum_sq: 0.0,
        }
    }

    fn push(&mut self, value: f64) {
        if self.values.len() == self.values.capacity() {
            if let Some(old) = self.values.pop_front() {
                self.sum -= old;
                self.sum_sq -= old * old;
            }
        }
        self.values.push_back(value);
        self.sum += value;
        self.sum_sq += value * value;
    }

    fn mean(&self) -> f64 {
        if self.values.is_empty() {
            0.0
        } else {
            self.sum / self.values.len() as f64
        }
    }

    fn std_dev(&self) -> f64 {
        if self.values.len() < 2 {
            return 0.0;
        }
        let n = self.values.len() as f64;
        let variance = (self.sum_sq - (self.sum * self.sum) / n) / (n - 1.0);
        variance.sqrt()
    }

    fn z_score(&self, value: f64) -> f64 {
        let mean = self.mean();
        let std = self.std_dev();
        if std == 0.0 {
            0.0
        } else {
            (value - mean) / std
        }
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

#[derive(Debug, Clone)]
pub struct AnomalyAlert {
    pub provider: String,
    pub metric: String,
    pub value: f64,
    pub z_score: f64,
    pub threshold: f64,
    pub timestamp: Instant,
}

impl AnomalyDetector {
    pub fn new(config: AnomalyConfig) -> Self {
        Self {
            windows: RwLock::new(HashMap::new()),
            config,
            alerts: RwLock::new(Vec::new()),
        }
    }

    pub fn record_latency(&self, provider: &str, latency_ms: f64) -> Option<AnomalyAlert> {
        self.record_metric(provider, "latency", latency_ms)
    }

    pub fn record_error_rate(&self, provider: &str, rate: f64) -> Option<AnomalyAlert> {
        self.record_metric(provider, "error_rate", rate)
    }

    pub fn record_metric(
        &self,
        provider: &str,
        metric: &str,
        value: f64,
    ) -> Option<AnomalyAlert> {
        let key = format!("{}:{}", provider, metric);
        let mut windows = self.windows.write().unwrap();
        let window = windows
            .entry(key)
            .or_insert_with(|| SlidingWindow::new(self.config.window_size));

        let z = window.z_score(value);
        window.push(value);

        if window.len() >= self.config.min_samples && z.abs() > self.config.z_score_threshold {
            let alert = AnomalyAlert {
                provider: provider.to_string(),
                metric: metric.to_string(),
                value,
                z_score: z,
                threshold: self.config.z_score_threshold,
                timestamp: Instant::now(),
            };
            self.alerts.write().unwrap().push(alert.clone());
            return Some(alert);
        }
        None
    }

    pub fn is_anomalous(&self, provider: &str, metric: &str, value: f64) -> bool {
        let key = format!("{}:{}", provider, metric);
        let windows = self.windows.read().unwrap();
        match windows.get(&key) {
            Some(w) if w.len() >= self.config.min_samples => {
                w.z_score(value).abs() > self.config.z_score_threshold
            }
            _ => false,
        }
    }

    pub fn get_alerts(&self, provider: Option<&str>) -> Vec<AnomalyAlert> {
        let alerts = self.alerts.read().unwrap();
        match provider {
            Some(p) => alerts.iter().filter(|a| a.provider == p).cloned().collect(),
            None => alerts.clone(),
        }
    }

    pub fn clear_alerts(&self) {
        self.alerts.write().unwrap().clear();
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new(AnomalyConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_values_no_alert() {
        let detector = AnomalyDetector::default();
        for i in 0..20 {
            assert!(detector.record_latency("p1", 100.0 + i as f64).is_none());
        }
    }

    #[test]
    fn test_anomaly_detected() {
        let detector = AnomalyDetector::default();
        for _ in 0..20 {
            detector.record_latency("p1", 100.0);
        }
        let alert = detector.record_latency("p1", 500.0);
        assert!(alert.is_some());
        let a = alert.unwrap();
        assert!(a.z_score > 2.0);
    }

    #[test]
    fn test_sliding_window() {
        let mut w = SlidingWindow::new(5);
        for i in 0..10 {
            w.push(i as f64);
        }
        assert_eq!(w.len(), 5);
        assert_eq!(w.mean(), 7.0);
    }
}
