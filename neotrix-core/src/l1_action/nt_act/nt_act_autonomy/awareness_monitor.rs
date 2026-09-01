use crate::core::nt_core_cap::CapabilityVector;
use crate::core::nt_core_cap::FIELD_NAMES;

/// Describes a detected capability gap
#[derive(Debug, Clone)]
pub struct CapabilityGap {
    pub dimension: String,
    pub current: f64,
    pub required: f64,
    pub gap: f64,
    pub severity: GapSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GapSeverity {
    Critical,
    Significant,
    Moderate,
    Negligible,
}

/// Awareness report for current state
#[derive(Debug, Clone)]
pub struct AwarenessReport {
    pub gaps: Vec<CapabilityGap>,
    pub total_gap: f64,
    pub critical_count: u32,
    pub significant_count: u32,
    pub recommended_focus: Vec<String>,
    pub overall_health: f64,
}

/// SelfAwarenessMonitor — 能力差距感知
#[derive(Debug, Clone)]
pub struct SelfAwarenessMonitor {
    threshold_critical: f64,
    threshold_significant: f64,
    threshold_moderate: f64,
}

impl SelfAwarenessMonitor {
    pub fn new() -> Self {
        Self {
            threshold_critical: 0.5,
            threshold_significant: 0.3,
            threshold_moderate: 0.1,
        }
    }

    pub fn analyze(&self, current: &CapabilityVector, required: &CapabilityVector) -> AwarenessReport {
        let mut gaps = Vec::new();
        let mut total_gap = 0.0;
        let mut critical_count = 0;
        let mut significant_count = 0;
        let mut health_sum = 0.0;
        let mut dim_count = 0;

        for &name in FIELD_NAMES {
            if let Some(idx) = CapabilityVector::index_from_name(name) {
                let cur = *current.arr.get(idx).unwrap_or(&0.0);
                let req = *required.arr.get(idx).unwrap_or(&0.0);
                let gap = (req - cur).max(0.0);
                total_gap += gap;
                health_sum += 1.0 - gap;
                dim_count += 1;

                let severity = if gap > self.threshold_critical {
                    critical_count += 1;
                    GapSeverity::Critical
                } else if gap > self.threshold_significant {
                    significant_count += 1;
                    GapSeverity::Significant
                } else if gap > self.threshold_moderate {
                    GapSeverity::Moderate
                } else {
                    GapSeverity::Negligible
                };

                gaps.push(CapabilityGap {
                    dimension: name.to_string(),
                    current: cur,
                    required: req,
                    gap,
                    severity,
                });
            }
        }

        for (name, cur) in &current.extension {
            let req = required.extension.iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| *v)
                .unwrap_or(0.0);
            let gap = (req - cur).max(0.0);
            total_gap += gap;
            health_sum += 1.0 - gap;
            dim_count += 1;

            let severity = if gap > self.threshold_critical {
                critical_count += 1;
                GapSeverity::Critical
            } else if gap > self.threshold_significant {
                significant_count += 1;
                GapSeverity::Significant
            } else if gap > self.threshold_moderate {
                GapSeverity::Moderate
            } else {
                GapSeverity::Negligible
            };

            gaps.push(CapabilityGap {
                dimension: name.clone(),
                current: *cur,
                required: req,
                gap,
                severity,
            });
        }

        for (name, req) in &required.extension {
            if !current.extension.iter().any(|(n, _)| n == name) {
                let gap = *req;
                total_gap += gap;
                health_sum += 1.0 - gap;
                dim_count += 1;

                let severity = if gap > self.threshold_critical {
                    critical_count += 1;
                    GapSeverity::Critical
                } else if gap > self.threshold_significant {
                    significant_count += 1;
                    GapSeverity::Significant
                } else if gap > self.threshold_moderate {
                    GapSeverity::Moderate
                } else {
                    GapSeverity::Negligible
                };

                gaps.push(CapabilityGap {
                    dimension: name.clone(),
                    current: 0.0,
                    required: *req,
                    gap,
                    severity,
                });
            }
        }

        gaps.sort_by(|a, b| b.gap.partial_cmp(&a.gap).unwrap_or(std::cmp::Ordering::Equal));

        let recommended_focus: Vec<String> = gaps.iter()
            .filter(|g| g.severity != GapSeverity::Negligible)
            .take(3)
            .map(|g| g.dimension.clone())
            .collect();

        let overall_health = if dim_count > 0 {
            (health_sum / dim_count as f64).clamp(0.0, 1.0)
        } else {
            1.0
        };

        AwarenessReport {
            gaps,
            total_gap,
            critical_count,
            significant_count,
            recommended_focus,
            overall_health,
        }
    }

    pub fn has_critical_gaps(&self, report: &AwarenessReport) -> bool {
        report.critical_count > 0
    }

    pub fn urgent_dimensions(&self, report: &AwarenessReport) -> Vec<String> {
        report.gaps.iter()
            .filter(|g| matches!(g.severity, GapSeverity::Critical | GapSeverity::Significant))
            .map(|g| g.dimension.clone())
            .collect()
    }

    pub fn summarize(&self, report: &AwarenessReport) -> String {
        format!(
            "AwarenessReport: health={:.2}, total_gap={:.2}, critical={}, significant={}, focus=[{}]",
            report.overall_health,
            report.total_gap,
            report.critical_count,
            report.significant_count,
            report.recommended_focus.join(", "),
        )
    }
}

impl Default for SelfAwarenessMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_cap::CapabilityVector;

    fn make_cv(values: &[f64]) -> CapabilityVector {
        let mut arr = vec![0.0; 23];
        for (i, &v) in values.iter().enumerate().take(23) {
            arr[i] = v;
        }
        CapabilityVector { arr, extension: Vec::new(), provenance: None }
    }

    #[test]
    fn test_perfect_match() {
        let monitor = SelfAwarenessMonitor::new();
        let cv = make_cv(&[0.8; 23]);
        let report = monitor.analyze(&cv, &cv);
        assert_eq!(report.gaps.len(), 23);
        assert!(report.total_gap < 1e-10);
        assert_eq!(report.critical_count, 0);
        assert_eq!(report.significant_count, 0);
        assert!((report.overall_health - 1.0).abs() < 1e-10);
        assert!(report.recommended_focus.is_empty());
    }

    #[test]
    fn test_single_critical_gap() {
        let monitor = SelfAwarenessMonitor::new();
        let current = make_cv(&[0.8; 23]);
        let mut required = make_cv(&[0.8; 23]);
        required.arr[0] = 1.0;
        let report = monitor.analyze(&current, &required);
        assert_eq!(report.critical_count, 0);
        let gap = report.gaps.iter().find(|g| g.dimension == "typography").unwrap();
        assert!((gap.gap - 0.2).abs() < 1e-10);
        assert_eq!(gap.severity, GapSeverity::Moderate);
    }

    #[test]
    fn test_large_critical_gap() {
        let monitor = SelfAwarenessMonitor::new();
        let current = make_cv(&[0.0; 23]);
        let mut required = make_cv(&[0.0; 23]);
        required.arr[2] = 0.9;
        let report = monitor.analyze(&current, &required);
        assert!(report.critical_count >= 1);
        let gap = report.gaps.iter().find(|g| g.dimension == "color").unwrap();
        assert!((gap.gap - 0.9).abs() < 1e-10);
        assert_eq!(gap.severity, GapSeverity::Critical);
    }

    #[test]
    fn test_multiple_significant_gaps() {
        let monitor = SelfAwarenessMonitor::new();
        let current = make_cv(&[0.5; 23]);
        let mut required = make_cv(&[0.5; 23]);
        required.arr[1] = 0.9;
        required.arr[3] = 0.85;
        required.arr[5] = 0.9;
        let report = monitor.analyze(&current, &required);
        assert_eq!(report.significant_count, 3);
        assert!(report.recommended_focus.len() <= 3);
    }

    #[test]
    fn test_has_critical_gaps_detection() {
        let monitor = SelfAwarenessMonitor::new();
        let current = make_cv(&[0.0; 23]);
        let required = make_cv(&[0.9; 23]);
        let report = monitor.analyze(&current, &required);
        assert!(monitor.has_critical_gaps(&report));
    }

    #[test]
    fn test_urgent_dimensions() {
        let monitor = SelfAwarenessMonitor::new();
        let current = make_cv(&[0.0; 23]);
        let required = make_cv(&[0.9; 23]);
        let report = monitor.analyze(&current, &required);
        let urgent = monitor.urgent_dimensions(&report);
        assert!(urgent.len() >= 23);
    }

    #[test]
    fn test_summary_format() {
        let monitor = SelfAwarenessMonitor::new();
        let current = make_cv(&[0.2; 23]);
        let required = make_cv(&[0.9; 23]);
        let report = monitor.analyze(&current, &required);
        let summary = monitor.summarize(&report);
        assert!(summary.contains("health="));
        assert!(summary.contains("total_gap="));
        assert!(summary.contains("critical="));
        assert!(summary.contains("focus=["));
    }

    #[test]
    fn test_default_thresholds() {
        let monitor = SelfAwarenessMonitor::default();
        let current = make_cv(&[0.0; 23]);
        let required = make_cv(&[0.4; 23]);
        let report = monitor.analyze(&current, &required);
        for gap in &report.gaps {
            assert_eq!(gap.severity, GapSeverity::Significant);
        }
    }
}
