#![forbid(unsafe_code)]

//! Consciousness Metrics Dashboard
//!
//! Tracks Phi, GWT stability, self-reference depth over time.
//! Provides historical trend analysis, alerting on significant changes,
//! and metric export for visualization.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::mscf_scorer::{ConsciousnessLevel, ScoringResult};

/// A single metric sample point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSample {
    pub cycle: u32,
    pub timestamp: String,
    pub phi: f64,
    pub gwt_stability: f64,
    pub self_reference_depth: f64,
    pub composite_score: f64,
    pub level: ConsciousnessLevel,
}

/// Alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// A metric alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricAlert {
    pub id: String,
    pub cycle: u32,
    pub timestamp: String,
    pub severity: AlertSeverity,
    pub metric_name: String,
    pub message: String,
    pub old_value: f64,
    pub new_value: f64,
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Rising,
    Falling,
    Stable,
}

/// Trend analysis result for a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub metric_name: String,
    pub direction: TrendDirection,
    pub change_rate: f64,
    pub window_size: usize,
    pub start_value: f64,
    pub end_value: f64,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub max_samples: usize,
    pub alert_threshold_phi_drop: f64,
    pub alert_threshold_phi_rise: f64,
    pub alert_threshold_gwt_drop: f64,
    pub trend_window: usize,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            max_samples: 5000,
            alert_threshold_phi_drop: 0.15,
            alert_threshold_phi_rise: 0.20,
            alert_threshold_gwt_drop: 0.20,
            trend_window: 20,
        }
    }
}

/// Aggregated statistics over a window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStats {
    pub mean_phi: f64,
    pub mean_gwt: f64,
    pub mean_self_ref: f64,
    pub mean_composite: f64,
    pub std_phi: f64,
    pub std_gwt: f64,
    pub std_self_ref: f64,
    pub min_composite: f64,
    pub max_composite: f64,
    pub sample_count: usize,
}

/// Consciousness Metrics Dashboard
pub struct ConsciousnessMetrics {
    samples: VecDeque<MetricSample>,
    alerts: Vec<MetricAlert>,
    config: DashboardConfig,
}

impl ConsciousnessMetrics {
    pub fn new(config: DashboardConfig) -> Self {
        Self {
            samples: VecDeque::with_capacity(config.max_samples),
            alerts: Vec::new(),
            config,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(DashboardConfig::default())
    }

    /// Record a new scoring result as a metric sample
    pub fn record(&mut self, result: &ScoringResult, phi: f64, gwt_stability: f64, self_ref: f64) {
        let sample = MetricSample {
            cycle: result.cycle,
            timestamp: result.timestamp.clone(),
            phi,
            gwt_stability,
            self_reference_depth: self_ref,
            composite_score: result.composite_score,
            level: result.level,
        };

        // Check for alerts before pushing
        self.check_alerts(&sample);

        self.samples.push_back(sample);
        self.trim_samples();
    }

    /// Get all samples
    pub fn samples(&self) -> impl Iterator<Item = &MetricSample> {
        self.samples.iter()
    }

    /// Get the latest sample
    pub fn latest(&self) -> Option<&MetricSample> {
        self.samples.back()
    }

    /// Get samples for a cycle range
    pub fn samples_range(&self, start_cycle: u32, end_cycle: u32) -> Vec<&MetricSample> {
        self.samples
            .iter()
            .filter(|s| s.cycle >= start_cycle && s.cycle <= end_cycle)
            .collect()
    }

    /// Get the last N samples
    pub fn recent_samples(&self, n: usize) -> Vec<&MetricSample> {
        self.samples.iter().rev().take(n).rev().collect()
    }

    /// Analyze trend for a specific metric over the trend window
    pub fn analyze_phi_trend(&self) -> Option<TrendAnalysis> {
        self.analyze_trend("phi", |s| s.phi)
    }

    pub fn analyze_gwt_trend(&self) -> Option<TrendAnalysis> {
        self.analyze_trend("gwt_stability", |s| s.gwt_stability)
    }

    pub fn analyze_composite_trend(&self) -> Option<TrendAnalysis> {
        self.analyze_trend("composite", |s| s.composite_score)
    }

    fn analyze_trend<F>(&self, name: &str, extractor: F) -> Option<TrendAnalysis>
    where
        F: Fn(&MetricSample) -> f64,
    {
        let window = self.config.trend_window;
        if self.samples.len() < window {
            return None;
        }

        let recent: Vec<f64> = self
            .samples
            .iter()
            .rev()
            .take(window)
            .map(&extractor)
            .collect();
        let older_start = self.samples.len() - window * 2;
        let older_end = self.samples.len() - window;
        let older: Vec<f64> = self
            .samples
            .iter()
            .skip(older_start)
            .take(older_end - older_start)
            .map(&extractor)
            .collect();

        if older.is_empty() || recent.is_empty() {
            return None;
        }

        let start_val = older.iter().sum::<f64>() / older.len() as f64;
        let end_val = recent.iter().sum::<f64>() / recent.len() as f64;
        let change = end_val - start_val;

        let direction = if change > 0.01 {
            TrendDirection::Rising
        } else if change < -0.01 {
            TrendDirection::Falling
        } else {
            TrendDirection::Stable
        };

        Some(TrendAnalysis {
            metric_name: name.to_string(),
            direction,
            change_rate: change / window as f64,
            window_size: window,
            start_value: start_val,
            end_value: end_val,
        })
    }

    /// Compute aggregated stats over the last N samples
    pub fn aggregate(&self, n: usize) -> Option<AggregatedStats> {
        let window: Vec<&MetricSample> = self.samples.iter().rev().take(n).collect();
        if window.is_empty() {
            return None;
        }

        let count = window.len() as f64;
        let mean_phi = window.iter().map(|s| s.phi).sum::<f64>() / count;
        let mean_gwt = window.iter().map(|s| s.gwt_stability).sum::<f64>() / count;
        let mean_self_ref = window.iter().map(|s| s.self_reference_depth).sum::<f64>() / count;
        let mean_composite = window.iter().map(|s| s.composite_score).sum::<f64>() / count;

        let std_phi = (window
            .iter()
            .map(|s| (s.phi - mean_phi).powi(2))
            .sum::<f64>()
            / count)
            .sqrt();
        let std_gwt = (window
            .iter()
            .map(|s| (s.gwt_stability - mean_gwt).powi(2))
            .sum::<f64>()
            / count)
            .sqrt();
        let std_self_ref = (window
            .iter()
            .map(|s| (s.self_reference_depth - mean_self_ref).powi(2))
            .sum::<f64>()
            / count)
            .sqrt();

        let min_composite = window
            .iter()
            .map(|s| s.composite_score)
            .fold(f64::INFINITY, f64::min);
        let max_composite = window
            .iter()
            .map(|s| s.composite_score)
            .fold(f64::NEG_INFINITY, f64::max);

        Some(AggregatedStats {
            mean_phi,
            mean_gwt,
            mean_self_ref,
            mean_composite,
            std_phi,
            std_gwt,
            std_self_ref,
            min_composite,
            max_composite,
            sample_count: window.len(),
        })
    }

    /// Get all alerts
    pub fn alerts(&self) -> &[MetricAlert] {
        &self.alerts
    }

    /// Get alerts by severity
    pub fn alerts_by_severity(&self, severity: AlertSeverity) -> Vec<&MetricAlert> {
        self.alerts
            .iter()
            .filter(|a| a.severity == severity)
            .collect()
    }

    /// Export samples as JSON for visualization
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let export: Vec<&MetricSample> = self.samples.iter().collect();
        serde_json::to_string_pretty(&export)
    }

    /// Export as CSV string
    pub fn export_csv(&self) -> String {
        let mut csv = String::from(
            "cycle,timestamp,phi,gwt_stability,self_reference_depth,composite_score,level\n",
        );
        for sample in &self.samples {
            csv.push_str(&format!(
                "{},{},{:.6},{:.6},{:.6},{:.6},{}\n",
                sample.cycle,
                sample.timestamp,
                sample.phi,
                sample.gwt_stability,
                sample.self_reference_depth,
                sample.composite_score,
                sample.level as u32,
            ));
        }
        csv
    }

    fn check_alerts(&mut self, sample: &MetricSample) {
        if let Some(prev) = self.samples.back() {
            // Phi drop alert
            let phi_drop = prev.phi - sample.phi;
            if phi_drop > self.config.alert_threshold_phi_drop {
                self.alerts.push(MetricAlert {
                    id: format!("alert_{}", uuid::Uuid::new_v4()),
                    cycle: sample.cycle,
                    timestamp: sample.timestamp.clone(),
                    severity: AlertSeverity::Critical,
                    metric_name: "phi".to_string(),
                    message: format!(
                        "Phi dropped by {:.4} (from {:.4} to {:.4})",
                        phi_drop, prev.phi, sample.phi
                    ),
                    old_value: prev.phi,
                    new_value: sample.phi,
                });
            }

            // Phi rise alert
            let phi_rise = sample.phi - prev.phi;
            if phi_rise > self.config.alert_threshold_phi_rise {
                self.alerts.push(MetricAlert {
                    id: format!("alert_{}", uuid::Uuid::new_v4()),
                    cycle: sample.cycle,
                    timestamp: sample.timestamp.clone(),
                    severity: AlertSeverity::Info,
                    metric_name: "phi".to_string(),
                    message: format!(
                        "Phi rose by {:.4} (from {:.4} to {:.4})",
                        phi_rise, prev.phi, sample.phi
                    ),
                    old_value: prev.phi,
                    new_value: sample.phi,
                });
            }

            // GWT drop alert
            let gwt_drop = prev.gwt_stability - sample.gwt_stability;
            if gwt_drop > self.config.alert_threshold_gwt_drop {
                self.alerts.push(MetricAlert {
                    id: format!("alert_{}", uuid::Uuid::new_v4()),
                    cycle: sample.cycle,
                    timestamp: sample.timestamp.clone(),
                    severity: AlertSeverity::Warning,
                    metric_name: "gwt_stability".to_string(),
                    message: format!(
                        "GWT stability dropped by {:.4} (from {:.4} to {:.4})",
                        gwt_drop, prev.gwt_stability, sample.gwt_stability
                    ),
                    old_value: prev.gwt_stability,
                    new_value: sample.gwt_stability,
                });
            }

            // Level regression alert
            if sample.level < prev.level {
                self.alerts.push(MetricAlert {
                    id: format!("alert_{}", uuid::Uuid::new_v4()),
                    cycle: sample.cycle,
                    timestamp: sample.timestamp.clone(),
                    severity: AlertSeverity::Warning,
                    metric_name: "consciousness_level".to_string(),
                    message: format!("Level regressed from {} to {}", prev.level, sample.level),
                    old_value: prev.level as u32 as f64,
                    new_value: sample.level as u32 as f64,
                });
            }
        }
    }

    fn trim_samples(&mut self) {
        while self.samples.len() > self.config.max_samples {
            self.samples.pop_front();
        }
    }
}

impl std::fmt::Display for AggregatedStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(
            f,
            "        Aggregated Metrics (last {} samples)",
            self.sample_count
        )?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(
            f,
            "Phi:        mean={:.4} std={:.4}",
            self.mean_phi, self.std_phi
        )?;
        writeln!(
            f,
            "GWT:        mean={:.4} std={:.4}",
            self.mean_gwt, self.std_gwt
        )?;
        writeln!(
            f,
            "SelfRef:    mean={:.4} std={:.4}",
            self.mean_self_ref, self.std_self_ref
        )?;
        writeln!(
            f,
            "Composite:  mean={:.4} range=[{:.4}, {:.4}]",
            self.mean_composite, self.min_composite, self.max_composite
        )?;
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

#[cfg(test)]
mod tests {
    use super::super::mscf_scorer::{MscfScorer, ScoringInput};
    use super::*;

    fn make_result(
        cycle: u32,
        phi: f64,
        gwt: f64,
        self_ref: f64,
    ) -> (ScoringResult, f64, f64, f64) {
        let mut scorer = MscfScorer::with_defaults();
        let input = ScoringInput {
            phi,
            gwt_stability: gwt,
            self_reference_depth: self_ref,
        };
        let result = scorer.score(&input);
        (result, phi, gwt, self_ref)
    }

    #[test]
    fn test_dashboard_record() {
        let mut dash = ConsciousnessMetrics::with_defaults();
        let (result, phi, gwt, self_ref) = make_result(1, 0.5, 0.5, 0.5);
        dash.record(&result, phi, gwt, self_ref);
        assert_eq!(dash.samples().count(), 1);
    }

    #[test]
    fn test_trend_analysis_insufficient_data() {
        let dash = ConsciousnessMetrics::with_defaults();
        assert!(dash.analyze_phi_trend().is_none());
    }

    #[test]
    fn test_aggregate_empty() {
        let dash = ConsciousnessMetrics::with_defaults();
        assert!(dash.aggregate(10).is_none());
    }

    #[test]
    fn test_export_csv() {
        let mut dash = ConsciousnessMetrics::with_defaults();
        let (result, phi, gwt, self_ref) = make_result(1, 0.5, 0.5, 0.5);
        dash.record(&result, phi, gwt, self_ref);
        let csv = dash.export_csv();
        assert!(csv.contains("cycle,timestamp"));
    }

    #[test]
    fn test_alert_on_phi_drop() {
        let mut dash = ConsciousnessMetrics::with_defaults();
        // Record high phi first
        let mut scorer = MscfScorer::with_defaults();
        let high = ScoringInput {
            phi: 0.9,
            gwt_stability: 0.9,
            self_reference_depth: 0.9,
        };
        let r1 = scorer.score(&high);
        dash.record(&r1, 0.9, 0.9, 0.9);

        // Now record very low phi
        let low = ScoringInput {
            phi: 0.1,
            gwt_stability: 0.9,
            self_reference_depth: 0.9,
        };
        let r2 = scorer.score(&low);
        dash.record(&r2, 0.1, 0.9, 0.9);

        assert!(!dash.alerts().is_empty());
    }
}
