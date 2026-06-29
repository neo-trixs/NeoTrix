use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum CapacityMetric {
    MemoryUsage,
    EntryCount,
    AverageSimilarity,
    NoveltyRatio,
    RetrievalLatency,
    EvictionRate,
}

impl CapacityMetric {
    pub fn name(&self) -> &'static str {
        match self {
            CapacityMetric::MemoryUsage => "memory_usage",
            CapacityMetric::EntryCount => "entry_count",
            CapacityMetric::AverageSimilarity => "average_similarity",
            CapacityMetric::NoveltyRatio => "novelty_ratio",
            CapacityMetric::RetrievalLatency => "retrieval_latency",
            CapacityMetric::EvictionRate => "eviction_rate",
        }
    }

    pub fn unit(&self) -> &'static str {
        match self {
            CapacityMetric::MemoryUsage => "bytes",
            CapacityMetric::EntryCount => "count",
            CapacityMetric::AverageSimilarity => "ratio",
            CapacityMetric::NoveltyRatio => "ratio",
            CapacityMetric::RetrievalLatency => "ms",
            CapacityMetric::EvictionRate => "per_hour",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CapacitySample {
    pub timestamp: u64,
    pub metric: CapacityMetric,
    pub value: f64,
    pub total_capacity: f64,
}

#[derive(Debug, Clone)]
pub struct CapacityWarning {
    pub metric: CapacityMetric,
    pub current_value: f64,
    pub threshold: f64,
    pub severity: &'static str,
    pub message: String,
}

pub struct CapacityMonitor {
    samples: HashMap<CapacityMetric, Vec<CapacitySample>>,
    capacity_limits: HashMap<CapacityMetric, f64>,
    warning_thresholds: HashMap<CapacityMetric, f64>,
    critical_thresholds: HashMap<CapacityMetric, f64>,
    max_samples: usize,
}

impl CapacityMonitor {
    pub fn new() -> Self {
        CapacityMonitor {
            samples: HashMap::new(),
            capacity_limits: HashMap::new(),
            warning_thresholds: HashMap::new(),
            critical_thresholds: HashMap::new(),
            max_samples: 10000,
        }
    }

    pub fn set_limit(&mut self, metric: CapacityMetric, limit: f64) {
        self.capacity_limits.insert(metric, limit);
    }

    pub fn set_warning_threshold(&mut self, metric: CapacityMetric, threshold: f64) {
        self.warning_thresholds.insert(metric, threshold);
    }

    pub fn record(&mut self, metric: CapacityMetric, value: f64, total_capacity: f64) {
        let sample = CapacitySample {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metric: metric.clone(),
            value,
            total_capacity,
        };
        let entries = self.samples.entry(metric).or_default();
        entries.push(sample);
        if entries.len() > self.max_samples {
            entries.remove(0);
        }
    }

    pub fn utilization(&self, metric: &CapacityMetric) -> Option<f64> {
        let limit = self.capacity_limits.get(metric)?;
        let samples = self.samples.get(metric)?;
        let latest = samples.last()?;
        if *limit == 0.0 {
            return None;
        }
        Some(latest.value / *limit)
    }

    fn get_threshold(
        &self,
        thresholds: &HashMap<CapacityMetric, f64>,
        metric: &CapacityMetric,
        default: f64,
    ) -> f64 {
        thresholds.get(metric).copied().unwrap_or(default)
    }

    pub fn check_warnings(&self) -> Vec<CapacityWarning> {
        let mut warnings = Vec::new();
        for (metric, samples) in &self.samples {
            if let Some(latest) = samples.last() {
                let limit = self
                    .capacity_limits
                    .get(metric)
                    .copied()
                    .unwrap_or(f64::MAX);
                let warn_frac = self.get_threshold(&self.warning_thresholds, metric, 0.8);
                let crit_frac = self.get_threshold(&self.critical_thresholds, metric, 0.95);

                if limit == f64::MAX || limit == 0.0 {
                    continue;
                }

                let util = latest.value / limit;

                if util >= crit_frac {
                    warnings.push(CapacityWarning {
                        metric: metric.clone(),
                        current_value: latest.value,
                        threshold: limit * crit_frac,
                        severity: "critical",
                        message: format!(
                            "{} at {:.2} {} exceeds critical threshold ({:.1}% of limit)",
                            metric.name(),
                            latest.value,
                            metric.unit(),
                            util * 100.0
                        ),
                    });
                } else if util >= warn_frac {
                    warnings.push(CapacityWarning {
                        metric: metric.clone(),
                        current_value: latest.value,
                        threshold: limit * warn_frac,
                        severity: "warning",
                        message: format!(
                            "{} at {:.2} {} exceeds warning threshold ({:.1}% of limit)",
                            metric.name(),
                            latest.value,
                            metric.unit(),
                            util * 100.0
                        ),
                    });
                }
            }
        }
        warnings
    }

    pub fn is_near_capacity(&self, metric: &CapacityMetric) -> bool {
        match self.utilization(metric) {
            Some(util) => {
                let warn = self.get_threshold(&self.warning_thresholds, metric, 0.8);
                util > warn
            }
            None => false,
        }
    }

    pub fn is_over_capacity(&self, metric: &CapacityMetric) -> bool {
        match self.utilization(metric) {
            Some(util) => util > 1.0,
            None => false,
        }
    }

    pub fn inverted_u_score(&self, metric: &CapacityMetric, window: usize) -> Option<f64> {
        let slope = self.trend(metric, window)?;
        let curvature = self.second_derivative(metric, window)?;
        if slope > 0.0 && curvature < 0.0 {
            Some(slope.abs() * curvature.abs())
        } else {
            Some(-1.0)
        }
    }

    pub fn trend(&self, metric: &CapacityMetric, window: usize) -> Option<f64> {
        let samples = self.samples.get(metric)?;
        if samples.len() < 2 {
            return None;
        }
        let window = window.min(samples.len());
        let subset = &samples[samples.len() - window..];

        let n = subset.len() as f64;
        let sum_x: f64 = (0..subset.len()).map(|i| i as f64).sum();
        let sum_y: f64 = subset.iter().map(|s| s.value).sum();
        let sum_xy: f64 = subset
            .iter()
            .enumerate()
            .map(|(i, s)| i as f64 * s.value)
            .sum();
        let sum_xx: f64 = (0..subset.len()).map(|i| (i as f64) * (i as f64)).sum();

        let denominator = n * sum_xx - sum_x * sum_x;
        if denominator == 0.0 {
            return None;
        }

        Some((n * sum_xy - sum_x * sum_y) / denominator)
    }

    pub fn second_derivative(&self, metric: &CapacityMetric, window: usize) -> Option<f64> {
        let samples = self.samples.get(metric)?;
        if samples.len() < 3 {
            return None;
        }
        let window = window.min(samples.len());
        let subset = &samples[samples.len() - window..];

        let mut slopes = Vec::with_capacity(subset.len() - 1);
        for i in 1..subset.len() {
            slopes.push(subset[i].value - subset[i - 1].value);
        }

        let n = slopes.len() as f64;
        let sum_x: f64 = (0..slopes.len()).map(|i| i as f64).sum();
        let sum_y: f64 = slopes.iter().sum();
        let sum_xy: f64 = slopes.iter().enumerate().map(|(i, s)| i as f64 * s).sum();
        let sum_xx: f64 = (0..slopes.len()).map(|i| (i as f64) * (i as f64)).sum();

        let denominator = n * sum_xx - sum_x * sum_x;
        if denominator == 0.0 {
            return None;
        }

        Some((n * sum_xy - sum_x * sum_y) / denominator)
    }

    pub fn peak_proximity(&self, metric: &CapacityMetric) -> Option<f64> {
        let util = self.utilization(metric)?;
        let peak_at = 0.7;
        Some((util / peak_at).min(1.0))
    }

    pub fn optimal_capacity_range(&self, _metric: &CapacityMetric) -> Option<(f64, f64)> {
        Some((0.5, 0.7))
    }

    pub fn recent_history(&self, metric: &CapacityMetric, n: usize) -> Vec<&CapacitySample> {
        match self.samples.get(metric) {
            Some(samples) => {
                let len = samples.len();
                let start = if n >= len { 0 } else { len - n };
                samples[start..].iter().collect()
            }
            None => Vec::new(),
        }
    }
}

pub struct CapacityReport {
    pub warnings: Vec<CapacityWarning>,
    pub peak_proximities: Vec<(CapacityMetric, f64)>,
    pub over_capacity_metrics: Vec<CapacityMetric>,
    pub timestamp: u64,
}

impl CapacityReport {
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        parts.push(format!("CapacityReport at {}", self.timestamp));
        if self.warnings.is_empty() {
            parts.push("  Warnings: none".to_string());
        } else {
            parts.push(format!("  Warnings ({}):", self.warnings.len()));
            for w in &self.warnings {
                parts.push(format!(
                    "    [{}] {}: {}",
                    w.severity,
                    w.metric.name(),
                    w.message
                ));
            }
        }
        if self.peak_proximities.is_empty() {
            parts.push("  Peak proximities: none".to_string());
        } else {
            parts.push("  Peak proximities:".to_string());
            for (m, p) in &self.peak_proximities {
                parts.push(format!("    {}: {:.3}", m.name(), p));
            }
        }
        if self.over_capacity_metrics.is_empty() {
            parts.push("  Over capacity: none".to_string());
        } else {
            parts.push("  Over capacity:".to_string());
            for m in &self.over_capacity_metrics {
                parts.push(format!("    {}", m.name()));
            }
        }
        parts.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_recent_history() {
        let mut cm = CapacityMonitor::new();
        cm.set_limit(CapacityMetric::MemoryUsage, 1024.0);
        cm.record(CapacityMetric::MemoryUsage, 100.0, 1024.0);
        cm.record(CapacityMetric::MemoryUsage, 200.0, 1024.0);
        cm.record(CapacityMetric::MemoryUsage, 300.0, 1024.0);
        let hist = cm.recent_history(&CapacityMetric::MemoryUsage, 2);
        assert_eq!(hist.len(), 2);
        assert_eq!(hist[0].value, 200.0);
        assert_eq!(hist[1].value, 300.0);
    }

    #[test]
    fn test_utilization() {
        let mut cm = CapacityMonitor::new();
        cm.set_limit(CapacityMetric::MemoryUsage, 1000.0);
        cm.record(CapacityMetric::MemoryUsage, 250.0, 1000.0);
        let util = cm.utilization(&CapacityMetric::MemoryUsage).unwrap();
        assert!((util - 0.25).abs() < 1e-10);
    }

    #[test]
    fn test_utilization_no_limit() {
        let cm = CapacityMonitor::new();
        assert!(cm.utilization(&CapacityMetric::MemoryUsage).is_none());
    }

    #[test]
    fn test_check_warnings_no_warning() {
        let mut cm = CapacityMonitor::new();
        cm.set_limit(CapacityMetric::MemoryUsage, 1000.0);
        cm.record(CapacityMetric::MemoryUsage, 100.0, 1000.0);
        let warnings = cm.check_warnings();
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_check_warnings_warning() {
        let mut cm = CapacityMonitor::new();
        cm.set_limit(CapacityMetric::MemoryUsage, 1000.0);
        cm.record(CapacityMetric::MemoryUsage, 850.0, 1000.0);
        let warnings = cm.check_warnings();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].severity, "warning");
    }

    #[test]
    fn test_check_warnings_critical() {
        let mut cm = CapacityMonitor::new();
        cm.set_limit(CapacityMetric::MemoryUsage, 1000.0);
        cm.record(CapacityMetric::MemoryUsage, 960.0, 1000.0);
        let warnings = cm.check_warnings();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].severity, "critical");
    }

    #[test]
    fn test_is_near_capacity() {
        let mut cm = CapacityMonitor::new();
        cm.set_limit(CapacityMetric::MemoryUsage, 1000.0);
        cm.record(CapacityMetric::MemoryUsage, 850.0, 1000.0);
        assert!(cm.is_near_capacity(&CapacityMetric::MemoryUsage));
        cm.record(CapacityMetric::MemoryUsage, 100.0, 1000.0);
        assert!(!cm.is_near_capacity(&CapacityMetric::MemoryUsage));
    }

    #[test]
    fn test_is_over_capacity() {
        let mut cm = CapacityMonitor::new();
        cm.set_limit(CapacityMetric::MemoryUsage, 1000.0);
        cm.record(CapacityMetric::MemoryUsage, 1100.0, 1000.0);
        assert!(cm.is_over_capacity(&CapacityMetric::MemoryUsage));
        cm.record(CapacityMetric::MemoryUsage, 900.0, 1000.0);
        assert!(!cm.is_over_capacity(&CapacityMetric::MemoryUsage));
    }

    #[test]
    fn test_trend_increasing() {
        let mut cm = CapacityMonitor::new();
        cm.set_limit(CapacityMetric::MemoryUsage, 1000.0);
        for i in 0..10 {
            cm.record(CapacityMetric::MemoryUsage, 100.0 + i as f64 * 20.0, 1000.0);
        }
        let slope = cm.trend(&CapacityMetric::MemoryUsage, 10).unwrap();
        assert!(slope > 0.0);
    }

    #[test]
    fn test_trend_decreasing() {
        let mut cm = CapacityMonitor::new();
        cm.set_limit(CapacityMetric::MemoryUsage, 1000.0);
        for i in 0..10 {
            cm.record(CapacityMetric::MemoryUsage, 300.0 - i as f64 * 20.0, 1000.0);
        }
        let slope = cm.trend(&CapacityMetric::MemoryUsage, 10).unwrap();
        assert!(slope < 0.0);
    }

    #[test]
    fn test_trend_flat() {
        let mut cm = CapacityMonitor::new();
        cm.set_limit(CapacityMetric::MemoryUsage, 1000.0);
        for _ in 0..10 {
            cm.record(CapacityMetric::MemoryUsage, 500.0, 1000.0);
        }
        let slope = cm.trend(&CapacityMetric::MemoryUsage, 10).unwrap();
        assert!(slope.abs() < 1e-10);
    }

    #[test]
    fn test_trend_insufficient_data() {
        let cm = CapacityMonitor::new();
        assert!(cm.trend(&CapacityMetric::MemoryUsage, 5).is_none());
    }

    #[test]
    fn test_second_derivative() {
        let mut cm = CapacityMonitor::new();
        cm.set_limit(CapacityMetric::MemoryUsage, 1000.0);
        cm.record(CapacityMetric::MemoryUsage, 100.0, 1000.0);
        cm.record(CapacityMetric::MemoryUsage, 200.0, 1000.0);
        cm.record(CapacityMetric::MemoryUsage, 250.0, 1000.0);
        let curv = cm
            .second_derivative(&CapacityMetric::MemoryUsage, 3)
            .unwrap();
        assert!(curv < 0.0);
    }

    #[test]
    fn test_second_derivative_insufficient_data() {
        let mut cm = CapacityMonitor::new();
        cm.set_limit(CapacityMetric::MemoryUsage, 1000.0);
        cm.record(CapacityMetric::MemoryUsage, 100.0, 1000.0);
        assert!(cm
            .second_derivative(&CapacityMetric::MemoryUsage, 3)
            .is_none());
    }

    #[test]
    fn test_inverted_u_score() {
        let mut cm = CapacityMonitor::new();
        cm.set_limit(CapacityMetric::MemoryUsage, 1000.0);
        cm.record(CapacityMetric::MemoryUsage, 100.0, 1000.0);
        cm.record(CapacityMetric::MemoryUsage, 180.0, 1000.0);
        cm.record(CapacityMetric::MemoryUsage, 240.0, 1000.0);
        cm.record(CapacityMetric::MemoryUsage, 280.0, 1000.0);
        let score = cm
            .inverted_u_score(&CapacityMetric::MemoryUsage, 4)
            .unwrap();
        assert!(score > 0.0);
    }

    #[test]
    fn test_peak_proximity() {
        let mut cm = CapacityMonitor::new();
        cm.set_limit(CapacityMetric::MemoryUsage, 1000.0);
        cm.record(CapacityMetric::MemoryUsage, 0.0, 1000.0);
        let p0 = cm.peak_proximity(&CapacityMetric::MemoryUsage).unwrap();
        assert!((p0 - 0.0).abs() < 1e-10);
        cm.record(CapacityMetric::MemoryUsage, 350.0, 1000.0);
        let p1 = cm.peak_proximity(&CapacityMetric::MemoryUsage).unwrap();
        assert!((p1 - 0.5).abs() < 1e-10);
        cm.record(CapacityMetric::MemoryUsage, 700.0, 1000.0);
        let p2 = cm.peak_proximity(&CapacityMetric::MemoryUsage).unwrap();
        assert!((p2 - 1.0).abs() < 1e-10);
        cm.record(CapacityMetric::MemoryUsage, 1000.0, 1000.0);
        let p3 = cm.peak_proximity(&CapacityMetric::MemoryUsage).unwrap();
        assert!((p3 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_optimal_capacity_range() {
        let cm = CapacityMonitor::new();
        let range = cm
            .optimal_capacity_range(&CapacityMetric::MemoryUsage)
            .unwrap();
        assert!((range.0 - 0.5).abs() < 1e-10);
        assert!((range.1 - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_empty_state() {
        let cm = CapacityMonitor::new();
        assert!(cm.utilization(&CapacityMetric::MemoryUsage).is_none());
        assert!(cm.check_warnings().is_empty());
        assert!(!cm.is_near_capacity(&CapacityMetric::MemoryUsage));
        assert!(!cm.is_over_capacity(&CapacityMetric::MemoryUsage));
        assert!(cm
            .recent_history(&CapacityMetric::MemoryUsage, 5)
            .is_empty());
    }

    #[test]
    fn test_single_sample_edge_case() {
        let mut cm = CapacityMonitor::new();
        cm.set_limit(CapacityMetric::MemoryUsage, 1000.0);
        cm.record(CapacityMetric::MemoryUsage, 500.0, 1000.0);
        assert!(cm.trend(&CapacityMetric::MemoryUsage, 5).is_none());
        assert!(cm
            .second_derivative(&CapacityMetric::MemoryUsage, 5)
            .is_none());
        assert!(cm
            .inverted_u_score(&CapacityMetric::MemoryUsage, 5)
            .is_none());
    }

    #[test]
    fn test_max_samples_pruning() {
        let mut cm = CapacityMonitor::new();
        cm.max_samples = 3;
        cm.set_limit(CapacityMetric::MemoryUsage, 1000.0);
        for i in 0..10 {
            cm.record(CapacityMetric::MemoryUsage, i as f64 * 100.0, 1000.0);
        }
        let hist = cm.recent_history(&CapacityMetric::MemoryUsage, 10);
        assert_eq!(hist.len(), 3);
        assert_eq!(hist[0].value, 700.0);
        assert_eq!(hist[2].value, 900.0);
    }

    #[test]
    fn test_capacity_report_summary() {
        let report = CapacityReport {
            warnings: Vec::new(),
            peak_proximities: Vec::new(),
            over_capacity_metrics: Vec::new(),
            timestamp: 1000,
        };
        let s = report.summary();
        assert!(s.contains("CapacityReport at 1000"));
        assert!(s.contains("Warnings: none"));
    }

    #[test]
    fn test_capacity_metric_name_and_unit() {
        assert_eq!(CapacityMetric::MemoryUsage.name(), "memory_usage");
        assert_eq!(CapacityMetric::MemoryUsage.unit(), "bytes");
        assert_eq!(CapacityMetric::EntryCount.name(), "entry_count");
        assert_eq!(CapacityMetric::EntryCount.unit(), "count");
        assert_eq!(
            CapacityMetric::AverageSimilarity.name(),
            "average_similarity"
        );
        assert_eq!(CapacityMetric::AverageSimilarity.unit(), "ratio");
        assert_eq!(CapacityMetric::NoveltyRatio.name(), "novelty_ratio");
        assert_eq!(CapacityMetric::NoveltyRatio.unit(), "ratio");
        assert_eq!(CapacityMetric::RetrievalLatency.name(), "retrieval_latency");
        assert_eq!(CapacityMetric::RetrievalLatency.unit(), "ms");
        assert_eq!(CapacityMetric::EvictionRate.name(), "eviction_rate");
        assert_eq!(CapacityMetric::EvictionRate.unit(), "per_hour");
    }

    #[test]
    fn test_custom_threshold() {
        let mut cm = CapacityMonitor::new();
        cm.set_limit(CapacityMetric::MemoryUsage, 1000.0);
        cm.set_warning_threshold(CapacityMetric::MemoryUsage, 0.5);
        cm.record(CapacityMetric::MemoryUsage, 600.0, 1000.0);
        let warnings = cm.check_warnings();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].severity, "warning");
        assert!(cm.is_near_capacity(&CapacityMetric::MemoryUsage));
    }
}
