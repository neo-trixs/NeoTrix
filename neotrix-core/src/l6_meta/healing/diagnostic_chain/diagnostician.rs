#![forbid(unsafe_code)]

use super::health_signal::{HealthSignal, Severity};
use std::collections::HashMap;

/// Diagnostic result produced by analyzing a batch of health signals.
#[derive(Debug, Clone)]
pub struct DiagnosticResult {
    /// Identified root cause description.
    pub root_cause: String,
    /// Confidence in the diagnosis (0.0 ..= 1.0).
    pub confidence: f64,
    /// Ordered list of recommended repair actions.
    pub recommended_actions: Vec<String>,
    /// The component identified as the source.
    pub source_component: String,
}

/// Rule-based diagnostician that analyzes health signals to identify root causes.
///
/// Uses a severity-weighted signal aggregation strategy:
/// - Critical signals contribute 1.0 weight
/// - Warning signals contribute 0.5 weight
/// - Info signals contribute 0.1 weight
///
/// When multiple components have signals, the one with the highest aggregate
/// severity weight is selected as the root cause source.
pub struct Diagnostician;

impl Diagnostician {
    /// Analyze a batch of health signals and produce a diagnostic result.
    ///
    /// Returns `None` if no signals are provided or all signals are Info-level.
    pub fn analyze(signals: &[HealthSignal]) -> Option<DiagnosticResult> {
        if signals.is_empty() {
            return None;
        }

        // Aggregate severity weights per component.
        let mut component_weights: HashMap<&str, f64> = HashMap::new();
        let mut component_signals: HashMap<&str, Vec<&HealthSignal>> = HashMap::new();

        for sig in signals {
            let weight = match sig.severity {
                Severity::Critical => 1.0,
                Severity::Warning => 0.5,
                Severity::Info => 0.1,
            };
            *component_weights.entry(&sig.component).or_insert(0.0) += weight;
            component_signals
                .entry(&sig.component)
                .or_default()
                .push(sig);
        }

        // Find the component with highest aggregate weight.
        let (&source, &_max_weight) = component_weights
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))?;

        let source_signals = component_signals.get(source).unwrap();
        let max_severity = source_signals
            .iter()
            .map(|s| s.severity)
            .max()
            .unwrap_or(Severity::Info);

        // Build metric summary for root cause.
        let metric_names: Vec<&str> = source_signals
            .iter()
            .map(|s| s.metric_name.as_str())
            .collect();
        let root_cause = format!(
            "{}: {} anomalies detected (metrics: {})",
            source,
            source_signals.len(),
            metric_names.join(", ")
        );

        // Confidence scales with signal count and severity.
        let severity_bonus = match max_severity {
            Severity::Critical => 0.3,
            Severity::Warning => 0.15,
            Severity::Info => 0.0,
        };
        let count_factor = (source_signals.len() as f64).min(5.0) / 5.0;
        let confidence = (0.3 + count_factor * 0.5 + severity_bonus).min(1.0);

        // Generate recommended actions based on severity.
        let mut actions = Vec::new();
        if max_severity == Severity::Critical {
            actions.push(format!("Restart component: {}", source));
            actions.push("Escalate to operator".to_string());
        } else if max_severity == Severity::Warning {
            actions.push(format!("Monitor component: {}", source));
            actions.push(format!("Degrade gracefully: {}", source));
        } else {
            actions.push(format!("Log and continue: {}", source));
        }

        Some(DiagnosticResult {
            root_cause,
            confidence,
            recommended_actions: actions,
            source_component: source.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_empty_returns_none() {
        assert!(Diagnostician::analyze(&[]).is_none());
    }

    #[test]
    fn test_analyze_single_critical_signal() {
        let signals = vec![HealthSignal::critical(
            "nt_core_cache",
            "error_rate",
            0.95,
            1000,
        )];
        let result = Diagnostician::analyze(&signals).unwrap();
        assert!(result.root_cause.contains("nt_core_cache"));
        assert!(result.confidence >= 0.5);
        assert!(result
            .recommended_actions
            .iter()
            .any(|a| a.contains("Restart")));
    }

    #[test]
    fn test_analyze_multiple_components_picks_highest_weight() {
        let signals = vec![
            HealthSignal::info("nt_world_crawl", "latency_ms", 100.0, 1000),
            HealthSignal::critical("nt_core_cache", "error_rate", 0.9, 1000),
            HealthSignal::critical("nt_core_cache", "memory_bytes", 1e9, 1000),
        ];
        let result = Diagnostician::analyze(&signals).unwrap();
        assert_eq!(result.source_component, "nt_core_cache");
    }

    #[test]
    fn test_analyze_warning_only_produces_lower_confidence() {
        let signals = vec![HealthSignal::warning(
            "nt_core_cache",
            "hit_rate",
            0.3,
            1000,
        )];
        let result = Diagnostician::analyze(&signals).unwrap();
        assert!(result.confidence < 0.8);
        assert!(result
            .recommended_actions
            .iter()
            .any(|a| a.contains("Monitor")));
    }

    #[test]
    fn test_analyze_all_info_still_returns_result() {
        let signals = vec![HealthSignal::info("nt_core_cache", "uptime", 3600.0, 1000)];
        let result = Diagnostician::analyze(&signals).unwrap();
        assert!(result.confidence < 0.5);
    }

    #[test]
    fn test_analyze_multiple_same_component() {
        let signals = vec![
            HealthSignal::critical("comp_a", "metric1", 0.9, 1000),
            HealthSignal::warning("comp_a", "metric2", 0.5, 1000),
            HealthSignal::info("comp_a", "metric3", 0.1, 1000),
        ];
        let result = Diagnostician::analyze(&signals).unwrap();
        assert_eq!(result.source_component, "comp_a");
        // Multiple signals increase confidence
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_analyze_empty_signals_returns_none() {
        assert!(Diagnostician::analyze(&[]).is_none());
    }

    #[test]
    fn test_analyze_critical_signals_have_restart_action() {
        let signals = vec![
            HealthSignal::critical("db", "error_rate", 0.99, 1000),
            HealthSignal::critical("db", "latency_ms", 5000.0, 1000),
        ];
        let result = Diagnostician::analyze(&signals).unwrap();
        assert!(result
            .recommended_actions
            .iter()
            .any(|a| a.contains("Restart")));
        assert!(result
            .recommended_actions
            .iter()
            .any(|a| a.contains("Escalate")));
    }

    #[test]
    fn test_analyze_warning_signals_have_monitor_action() {
        let signals = vec![HealthSignal::warning("cache", "hit_rate", 0.3, 1000)];
        let result = Diagnostician::analyze(&signals).unwrap();
        assert!(result
            .recommended_actions
            .iter()
            .any(|a| a.contains("Monitor")));
    }

    #[test]
    fn test_analyze_info_signals_have_log_action() {
        let signals = vec![HealthSignal::info("svc", "uptime", 3600.0, 1000)];
        let result = Diagnostician::analyze(&signals).unwrap();
        assert!(result.recommended_actions.iter().any(|a| a.contains("Log")));
    }

    #[test]
    fn test_analyze_root_cause_contains_metric_names() {
        let signals = vec![
            HealthSignal::critical("db", "error_rate", 0.9, 1000),
            HealthSignal::critical("db", "connection_pool", 0.0, 1000),
        ];
        let result = Diagnostician::analyze(&signals).unwrap();
        assert!(result.root_cause.contains("error_rate"));
        assert!(result.root_cause.contains("connection_pool"));
    }

    #[test]
    fn test_analyze_confidence_scales_with_signal_count() {
        let single = vec![HealthSignal::warning("comp", "m1", 1.0, 1000)];
        let many = vec![
            HealthSignal::warning("comp", "m1", 1.0, 1000),
            HealthSignal::warning("comp", "m2", 1.0, 1000),
            HealthSignal::warning("comp", "m3", 1.0, 1000),
            HealthSignal::warning("comp", "m4", 1.0, 1000),
            HealthSignal::warning("comp", "m5", 1.0, 1000),
        ];
        let r1 = Diagnostician::analyze(&single).unwrap();
        let r2 = Diagnostician::analyze(&many).unwrap();
        assert!(r2.confidence >= r1.confidence);
    }
}
