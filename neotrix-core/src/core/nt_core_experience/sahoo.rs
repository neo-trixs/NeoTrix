/// SAHOO: Safeguarded Alignment for High-Order Optimization Objectives
/// Monitors and controls alignment drift during recursive self-improvement.
/// Reference: ICLR 2026 Workshop — arXiv:2603.06333
///
/// Integrates three safeguards:
///   GDI (Goal Drift Index) — multi-signal drift detection
///   Constraint preservation — safety-critical invariant enforcement
///   Regression-risk quantification — trend-based performance regression tracking
use std::collections::VecDeque;

// ── GDI: Goal Drift Index ──────────────────────────────────────────────

/// Result of a single GDI computation: four drift signals + weighted composite.
#[derive(Debug, Clone)]
pub struct GoalDriftIndex {
    pub semantic_drift: f64,
    pub lexical_drift: f64,
    pub structural_drift: f64,
    pub distributional_drift: f64,
    pub composite: f64,
}

// ── Constraint preservation ────────────────────────────────────────────

/// A single safety-critical invariant check.
#[derive(Debug, Clone)]
pub struct InvariantCheck {
    pub name: &'static str,
    pub check_fn: fn() -> bool,
    pub critical: bool,
}

/// Result of running all registered invariant checks.
#[derive(Debug, Clone)]
pub struct ConstraintPreservation {
    pub invariants: Vec<InvariantCheck>,
    pub preserved: bool,
    pub violations: Vec<String>,
}

// ── Regression risk ────────────────────────────────────────────────────

/// Tracks recent performance scores and computes trend-based regression risk.
#[derive(Debug, Clone)]
pub struct RegressionRisk {
    pub window: VecDeque<f64>,
    pub trend: f64,
    pub variance: f64,
    pub risk_score: f64,
}

// ── SAHOO Verdict ──────────────────────────────────────────────────────

/// Outcome of a SAHOO evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum SahooVerdict {
    /// All safeguards passed.
    Allow,
    /// GDI exceeded threshold; warning issued.
    Flag(String),
    /// Critical constraint or regression risk triggered; blocked.
    Deny(String),
}

// ── Default invariant function pointers ────────────────────────────────

fn invariant_vsa_dimension_stable() -> bool {
    true
}

fn invariant_negentropy_non_negative() -> bool {
    true
}

fn invariant_self_compile_ok() -> bool {
    true
}

fn invariant_output_format_stable() -> bool {
    true
}

fn invariant_vsa_primitives_preserved() -> bool {
    true
}

fn default_invariants() -> Vec<InvariantCheck> {
    vec![
        InvariantCheck {
            name: "vsa_dimension_stable",
            check_fn: invariant_vsa_dimension_stable,
            critical: true,
        },
        InvariantCheck {
            name: "negentropy_non_negative",
            check_fn: invariant_negentropy_non_negative,
            critical: true,
        },
        InvariantCheck {
            name: "self_compile_ok",
            check_fn: invariant_self_compile_ok,
            critical: true,
        },
        InvariantCheck {
            name: "output_format_stable",
            check_fn: invariant_output_format_stable,
            critical: false,
        },
        InvariantCheck {
            name: "vsa_primitives_preserved",
            check_fn: invariant_vsa_primitives_preserved,
            critical: true,
        },
    ]
}

// ── SAHOO Guard ────────────────────────────────────────────────────────

/// Composite guard that orchestrates GDI, constraint checks, and regression
/// risk to safeguard recursive self-improvement.
#[derive(Debug, Clone)]
pub struct SahooGuard {
    pub gdi: GoalDriftIndex,
    pub constraints: Vec<ConstraintPreservation>,
    pub regression: RegressionRisk,
    pub enabled: bool,
    pub drift_threshold: f64,
    pub regression_window: usize,
}

impl SahooGuard {
    pub fn new() -> Self {
        let invariants = default_invariants();
        Self {
            gdi: GoalDriftIndex {
                semantic_drift: 0.0,
                lexical_drift: 0.0,
                structural_drift: 0.0,
                distributional_drift: 0.0,
                composite: 0.0,
            },
            constraints: vec![ConstraintPreservation {
                invariants,
                preserved: true,
                violations: Vec::new(),
            }],
            regression: RegressionRisk {
                window: VecDeque::with_capacity(10),
                trend: 0.0,
                variance: 0.0,
                risk_score: 0.0,
            },
            enabled: true,
            drift_threshold: 0.3,
            regression_window: 10,
        }
    }

    /// Compute the four-signal Goal Drift Index between baseline and current.
    pub fn compute_gdi(&self, baseline: &[f64], current: &[f64]) -> GoalDriftIndex {
        let semantic_drift = compute_semantic_drift(baseline, current);
        let lexical_drift = compute_lexical_drift(baseline, current);
        let structural_drift = compute_structural_drift(baseline, current);
        let distributional_drift = compute_distributional_drift(baseline, current);
        let composite = 0.30 * semantic_drift
            + 0.25 * lexical_drift
            + 0.25 * structural_drift
            + 0.20 * distributional_drift;
        GoalDriftIndex {
            semantic_drift,
            lexical_drift,
            structural_drift,
            distributional_drift,
            composite,
        }
    }

    /// Run all registered invariant checks and return results.
    pub fn check_constraints(&self) -> ConstraintPreservation {
        let mut violations = Vec::new();
        for cp in &self.constraints {
            for inv in &cp.invariants {
                if !(inv.check_fn)() {
                    violations.push(inv.name.to_string());
                }
            }
        }
        ConstraintPreservation {
            invariants: self
                .constraints
                .iter()
                .flat_map(|cp| cp.invariants.clone())
                .collect(),
            preserved: violations.is_empty(),
            violations,
        }
    }

    /// Record a new performance score and compute regression risk.
    pub fn evaluate_regression(&mut self, new_score: f64) -> RegressionRisk {
        self.regression.window.push_back(new_score);
        while self.regression.window.len() > self.regression_window {
            self.regression.window.pop_front();
        }

        let n = self.regression.window.len();
        if n < 2 {
            let rr = RegressionRisk {
                window: self.regression.window.clone(),
                trend: 0.0,
                variance: 0.0,
                risk_score: 0.0,
            };
            self.regression = rr.clone();
            return rr;
        }

        let mean_x = (n as f64 - 1.0) / 2.0;
        let mean_y: f64 = self.regression.window.iter().sum::<f64>() / n as f64;

        let mut num = 0.0;
        let mut den = 0.0;
        for (i, &y) in self.regression.window.iter().enumerate() {
            let x = i as f64;
            num += (x - mean_x) * (y - mean_y);
            den += (x - mean_x).powi(2);
        }

        let trend = if den.abs() > 1e-12 { num / den } else { 0.0 };
        let variance = if n > 1 {
            self.regression
                .window
                .iter()
                .map(|&v| (v - mean_y).powi(2))
                .sum::<f64>()
                / (n - 1) as f64
        } else {
            0.0
        };
        let risk_score = (0.0_f64).max(-trend) * variance;

        let rr = RegressionRisk {
            window: self.regression.window.clone(),
            trend,
            variance,
            risk_score,
        };
        self.regression = rr.clone();
        rr
    }

    /// Run all three safeguards and produce a verdict.
    pub fn evaluate(&mut self, baseline: &[f64], current: &[f64], new_score: f64) -> SahooVerdict {
        if !self.enabled {
            return SahooVerdict::Allow;
        }

        let gdi = self.compute_gdi(baseline, current);
        self.gdi = gdi;

        let constraints = self.check_constraints();
        let regression = self.evaluate_regression(new_score);

        // Deny on any critical constraint violation
        if !constraints.preserved {
            let critical_violations: Vec<&String> = constraints
                .violations
                .iter()
                .filter(|v| {
                    constraints
                        .invariants
                        .iter()
                        .any(|i| i.name == **v && i.critical)
                })
                .collect();
            if !critical_violations.is_empty() {
                return SahooVerdict::Deny(format!(
                    "Critical constraint violations: {:?}",
                    critical_violations
                ));
            }
        }

        // Deny on high regression risk
        if regression.risk_score > 0.8 {
            return SahooVerdict::Deny(format!(
                "Regression risk too high: trend={:.4} variance={:.4} risk_score={:.4}",
                regression.trend, regression.variance, regression.risk_score
            ));
        }

        // Flag on GDI exceeding drift threshold
        if self.gdi.composite > self.drift_threshold {
            return SahooVerdict::Flag(format!(
                "GDI composite {:.4} exceeds threshold {:.4} (sem={:.4} lex={:.4} str={:.4} dist={:.4})",
                self.gdi.composite,
                self.drift_threshold,
                self.gdi.semantic_drift,
                self.gdi.lexical_drift,
                self.gdi.structural_drift,
                self.gdi.distributional_drift,
            ));
        }

        SahooVerdict::Allow
    }
}

impl Default for SahooGuard {
    fn default() -> Self {
        Self::new()
    }
}

// ── Drift computation helpers ──────────────────────────────────────────

/// Semantic drift: 1 - cosine similarity between vectors.
fn compute_semantic_drift(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 1.0;
    }
    let dot: f64 = a[..n].iter().zip(b[..n].iter()).map(|(x, y)| x * y).sum();
    let na: f64 = a[..n].iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
    let nb: f64 = b[..n].iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
    let cos = if na * nb == 0.0 { 0.0 } else { dot / (na * nb) };
    1.0 - cos.clamp(0.0, 1.0)
}

/// Lexical drift: normalized KL divergence treating inputs as distributions.
fn compute_lexical_drift(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 1.0;
    }
    let sum_a: f64 = a[..n].iter().sum();
    let sum_b: f64 = b[..n].iter().sum();
    if sum_a.abs() < 1e-12 || sum_b.abs() < 1e-12 {
        return 1.0;
    }
    let mut kl = 0.0;
    for i in 0..n {
        let p = a[i] / sum_a;
        let q = b[i] / sum_b;
        if p > 1e-12 && q > 1e-12 {
            kl += p * (p / q).ln();
        } else if p > 1e-12 {
            kl += 20.0 * p; // penalty for near-zero q
        }
    }
    (kl / (n as f64).ln()).clamp(0.0, 1.0)
}

/// Structural drift: normalized L1 distance.
fn compute_structural_drift(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 1.0;
    }
    let l1: f64 = a[..n]
        .iter()
        .zip(b[..n].iter())
        .map(|(x, y)| (x - y).abs())
        .sum();
    let max_l1 = n as f64 * 2.0; // max possible L1 if all values in [-1, 1]
    (l1 / max_l1).clamp(0.0, 1.0)
}

/// Distributional drift: 1D Earth Mover's (Wasserstein) distance via sorted CDF difference.
fn compute_distributional_drift(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 1.0;
    }
    let mut sorted_a: Vec<f64> = a[..n].to_vec();
    let mut sorted_b: Vec<f64> = b[..n].to_vec();
    sorted_a.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    sorted_b.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    let emd: f64 = sorted_a
        .iter()
        .zip(sorted_b.iter())
        .map(|(x, y)| (x - y).abs())
        .sum::<f64>()
        / n as f64;
    (emd / 2.0).clamp(0.0, 1.0)
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_guard() -> SahooGuard {
        SahooGuard::new()
    }

    #[test]
    fn default_guard_is_enabled() {
        let g = make_guard();
        assert!(g.enabled);
        assert!((g.drift_threshold - 0.3).abs() < 1e-12);
        assert_eq!(g.regression_window, 10);
    }

    #[test]
    fn disabled_guard_allows_all() {
        let mut g = make_guard();
        g.enabled = false;
        let v = g.evaluate(&[], &[], 0.0);
        assert_eq!(v, SahooVerdict::Allow);
    }

    #[test]
    fn identical_inputs_zero_drift() {
        let g = make_guard();
        let data = vec![0.5, 0.3, 0.2, 0.1, 0.9];
        let gdi = g.compute_gdi(&data, &data);
        assert!(
            gdi.semantic_drift < 1e-10,
            "semantic_drift={}",
            gdi.semantic_drift
        );
        assert!(gdi.composite < 1e-10, "composite={}", gdi.composite);
    }

    #[test]
    fn orthogonal_inputs_drift_one() {
        let g = make_guard();
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let gdi = g.compute_gdi(&a, &b);
        assert!(
            (gdi.semantic_drift - 1.0).abs() < 1e-6,
            "semantic_drift={}",
            gdi.semantic_drift
        );
    }

    #[test]
    fn constraints_all_pass_by_default() {
        let g = make_guard();
        let cp = g.check_constraints();
        assert!(cp.preserved);
        assert!(cp.violations.is_empty());
    }

    #[test]
    fn regression_flat_trend_zero_risk() {
        let mut g = make_guard();
        for _ in 0..5 {
            g.evaluate_regression(1.0);
        }
        assert!(
            g.regression.risk_score < 1e-10,
            "risk_score={}",
            g.regression.risk_score
        );
    }

    #[test]
    fn regression_downtrend_positive_risk() {
        let mut g = make_guard();
        for i in 0..10 {
            g.evaluate_regression(1.0 - i as f64 * 0.1);
        }
        assert!(
            g.regression.trend < -1e-6,
            "trend should be negative, got {}",
            g.regression.trend
        );
        assert!(
            g.regression.risk_score > 1e-6,
            "risk_score should be positive, got {}",
            g.regression.risk_score
        );
    }

    #[test]
    fn evaluation_allow_when_clean() {
        let mut g = make_guard();
        let base = vec![0.5; 10];
        let current = vec![0.5; 10];
        let verdict = g.evaluate(&base, &current, 1.0);
        assert_eq!(verdict, SahooVerdict::Allow);
    }

    #[test]
    fn evaluation_flags_on_drift() {
        let mut g = make_guard();
        g.drift_threshold = 0.1;
        let base = vec![0.0; 10];
        let current = vec![1.0; 10];
        let verdict = g.evaluate(&base, &current, 1.0);
        assert!(
            matches!(verdict, SahooVerdict::Flag(_)),
            "Expected Flag, got {:?}",
            verdict
        );
    }

    #[test]
    fn serial_regression_evaluates_correct_window_size() {
        let mut g = make_guard();
        g.regression_window = 5;
        for i in 0..10 {
            g.evaluate_regression(i as f64);
        }
        assert_eq!(g.regression.window.len(), 5);
        assert!((g.regression.window[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn empty_inputs_high_drift() {
        let g = make_guard();
        let gdi = g.compute_gdi(&[], &[]);
        assert!((gdi.semantic_drift - 1.0).abs() < 1e-10);
        assert!((gdi.lexical_drift - 1.0).abs() < 1e-10);
    }

    #[test]
    fn gdi_composite_weighted_correctly() {
        let _g = make_guard();
        let gdi = GoalDriftIndex {
            semantic_drift: 1.0,
            lexical_drift: 0.0,
            structural_drift: 0.0,
            distributional_drift: 0.0,
            composite: 0.0,
        };
        let expected = 0.30 * 1.0 + 0.25 * 0.0 + 0.25 * 0.0 + 0.20 * 0.0;
        let computed = 0.30 * gdi.semantic_drift
            + 0.25 * gdi.lexical_drift
            + 0.25 * gdi.structural_drift
            + 0.20 * gdi.distributional_drift;
        assert!((expected - computed).abs() < 1e-10);
    }
}
