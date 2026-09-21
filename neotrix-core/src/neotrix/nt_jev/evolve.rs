//! JEV Evolve bridge — nightly EvalReport → evolution criteria scores.
//!
//! This is the L1→L4 wiring of the autonomous iteration loop:
//! ```text
//! audit trail → golden packs → eval::evaluate() → report_scores()
//!     → EvolvingEvaluator::evaluate_jev_report() → evolve_criteria()
//!     → absorber → new presets/thresholds → next nightly
//! ```
//! This module only translates scores (no dependency on `l6_meta`, keeping
//! the layering one-directional: L6 depends on JEV, never the reverse).
//! The criteria constructors live next to the consumer
//! (`EvolvingEvaluator::jev_criteria`).
//!
//! Key mapping (all higher-is-better, 0.0–1.0):
//! | criterion    | source                          | threshold rationale |
//! |--------------|---------------------------------|---------------------|
//! | accuracy     | report.accuracy                 | jevassert-style 0.8 |
//! | calibration  | 1 − ECE (max_ece 0.15 → t 0.85) | jevassert max_ece   |
//! | coverage     | coverage@precision-0.9          | abstention utility  |
//! | efficiency   | 100/(100+latency_ms)            | 43ms ≈ 0.7          |
//! | robustness   | 1 − Brier                       | proper-score guard  |

use std::collections::HashMap;

use super::eval::EvalReport;

/// Criterion name: classification accuracy.
pub const JEV_CRITERION_ACCURACY: &str = "accuracy";
/// Criterion name: calibration (1 − ECE).
pub const JEV_CRITERION_CALIBRATION: &str = "calibration";
/// Criterion name: kept-coverage at 0.9 precision.
pub const JEV_CRITERION_COVERAGE: &str = "coverage";
/// Criterion name: latency-derived efficiency.
pub const JEV_CRITERION_EFFICIENCY: &str = "efficiency";
/// Criterion name: robustness (1 − Brier).
pub const JEV_CRITERION_ROBUSTNESS: &str = "robustness";

/// All criterion names in canonical order.
pub const JEV_CRITERIA: &[&str] = &[
    JEV_CRITERION_ACCURACY,
    JEV_CRITERION_CALIBRATION,
    JEV_CRITERION_COVERAGE,
    JEV_CRITERION_EFFICIENCY,
    JEV_CRITERION_ROBUSTNESS,
];

/// Clamp a score into [0, 1] without `.clamp()` (repo lint prefers max/min).
fn unit(x: f64) -> f64 {
    x.max(0.0).min(1.0)
}

/// Translate an [`EvalReport`] into criterion scores for
/// `EvolvingEvaluator::evaluate` (keyed by [`JEV_CRITERIA`]).
pub fn report_scores(report: &EvalReport) -> HashMap<String, f64> {
    let mut m = HashMap::new();
    m.insert(JEV_CRITERION_ACCURACY.to_string(), unit(report.accuracy));
    m.insert(
        JEV_CRITERION_CALIBRATION.to_string(),
        unit(1.0 - report.ece),
    );
    m.insert(
        JEV_CRITERION_COVERAGE.to_string(),
        unit(report.coverage_at_p90),
    );
    m.insert(
        JEV_CRITERION_EFFICIENCY.to_string(),
        unit(100.0 / (100.0 + report.mean_latency_ms.max(0.0))),
    );
    m.insert(
        JEV_CRITERION_ROBUSTNESS.to_string(),
        unit(1.0 - report.brier),
    );
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report() -> EvalReport {
        EvalReport {
            n: 100,
            accuracy: 0.9,
            brier: 0.1,
            ece: 0.05,
            coverage_at_p90: 0.7,
            mean_latency_ms: 25.0,
        }
    }

    #[test]
    fn test_report_scores_mapping() {
        let s = report_scores(&report());
        assert!((s[JEV_CRITERION_ACCURACY] - 0.9).abs() < 1e-12);
        assert!((s[JEV_CRITERION_CALIBRATION] - 0.95).abs() < 1e-12);
        assert!((s[JEV_CRITERION_COVERAGE] - 0.7).abs() < 1e-12);
        assert!((s[JEV_CRITERION_EFFICIENCY] - 100.0 / 125.0).abs() < 1e-12);
        assert!((s[JEV_CRITERION_ROBUSTNESS] - 0.9).abs() < 1e-12);
    }

    #[test]
    fn test_report_scores_keys_match_criteria() {
        let s = report_scores(&report());
        for k in JEV_CRITERIA {
            assert!(s.contains_key(*k), "missing {}", k);
        }
        assert_eq!(s.len(), JEV_CRITERIA.len());
    }

    #[test]
    fn test_report_scores_empty_report() {
        let r = EvalReport {
            n: 0,
            accuracy: 0.0,
            brier: 0.0,
            ece: 0.0,
            coverage_at_p90: 0.0,
            mean_latency_ms: 0.0,
        };
        let s = report_scores(&r);
        assert_eq!(s[JEV_CRITERION_ACCURACY], 0.0);
        // No evidence of miscalibration/error/latency → neutral-good.
        assert_eq!(s[JEV_CRITERION_CALIBRATION], 1.0);
        assert_eq!(s[JEV_CRITERION_EFFICIENCY], 1.0);
        assert_eq!(s[JEV_CRITERION_ROBUSTNESS], 1.0);
    }

    #[test]
    fn test_report_scores_clamped() {
        let mut r = report();
        r.ece = 1.5;
        r.brier = 2.0;
        r.mean_latency_ms = -5.0;
        let s = report_scores(&r);
        assert_eq!(s[JEV_CRITERION_CALIBRATION], 0.0);
        assert_eq!(s[JEV_CRITERION_ROBUSTNESS], 0.0);
        assert_eq!(s[JEV_CRITERION_EFFICIENCY], 1.0);
    }
}
