/// Multi-dimensional quality gate — routes content between System 1 (fast) and System 2 (slow).
///
/// # Background
/// D-Mem (arXiv 2603.18631): multi-dimensional quality gating achieves 96.7% accuracy
///   with 65% fewer tokens than full deliberation. Four quality checks determine
///   whether content needs slow-path processing.
/// Mirror Benchmark (arXiv 2604.19809) C4: only external architectural constraint
///   reduces Confident Failure Rate from 0.600 → 0.143 (76% reduction).
///   Self-knowledge alone (C2) produces no improvement (p>0.05).
///
/// # Operation
/// When content arrives at the GATE step:
///   1. Score relevance, faithfulness, completeness, uncertainty
///   2. If ALL dimensions ≥ threshold → fast-path (COMPETE → ACT directly)
///   3. If ANY dimension < threshold → slow-path (full REASON→JUDGE→VERIFY)
///
/// The threshold is NOT static — it adapts via EMA from recent pass/fail history.
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct QualityGate {
    /// Threshold for each quality dimension (0.0-1.0)
    pub relevance_threshold: f64,
    pub faithfulness_threshold: f64,
    pub completeness_threshold: f64,
    pub uncertainty_max: f64,

    /// Whether to use fast-path (System 1 shortcut)
    pub enable_fast_path: bool,

    /// Adaptive threshold via EMA
    pub adapt_enabled: bool,
    pub ema_alpha: f64,

    /// History for adaptive threshold
    history: VecDeque<bool>,
    max_history: usize,

    /// Current adapted thresholds
    pub current_relevance_threshold: f64,
    pub current_faithfulness_threshold: f64,
    pub current_completeness_threshold: f64,
    pub current_uncertainty_max: f64,

    /// Statistics
    pub total_checks: u64,
    pub fast_path_count: u64,
    pub slow_path_count: u64,
    pub gate_passed: u64,
    pub gate_failed: u64,
}

impl Default for QualityGate {
    fn default() -> Self {
        Self::new()
    }
}

impl QualityGate {
    pub fn new() -> Self {
        Self {
            relevance_threshold: 0.6,
            faithfulness_threshold: 0.5,
            completeness_threshold: 0.5,
            uncertainty_max: 0.4,

            enable_fast_path: true,
            adapt_enabled: true,
            ema_alpha: 0.1,

            history: VecDeque::with_capacity(100),
            max_history: 100,

            current_relevance_threshold: 0.6,
            current_faithfulness_threshold: 0.5,
            current_completeness_threshold: 0.5,
            current_uncertainty_max: 0.4,

            total_checks: 0,
            fast_path_count: 0,
            slow_path_count: 0,
            gate_passed: 0,
            gate_failed: 0,
        }
    }

    /// Evaluate content quality and determine whether to fast-path.
    /// Returns (should_fast_path, quality_scores)
    pub fn evaluate(
        &mut self,
        relevance: f64,
        faithfulness: f64,
        completeness: f64,
        uncertainty: f64,
    ) -> (bool, QualityScores) {
        self.total_checks += 1;

        let scores = QualityScores {
            relevance,
            faithfulness,
            completeness,
            uncertainty,
        };

        let pass = relevance >= self.current_relevance_threshold
            && faithfulness >= self.current_faithfulness_threshold
            && completeness >= self.current_completeness_threshold
            && uncertainty <= self.current_uncertainty_max;

        if pass {
            self.gate_passed += 1;
        } else {
            self.gate_failed += 1;
        }

        // Fast-path decision
        let take_fast = pass && self.enable_fast_path;
        if take_fast {
            self.fast_path_count += 1;
        } else {
            self.slow_path_count += 1;
        }

        // Record outcome for adaptive threshold
        self.history.push_back(pass);
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }

        if self.adapt_enabled {
            self.adapt_thresholds();
        }

        (take_fast, scores)
    }

    /// Lightweight pre-check — fast rejection without full scoring.
    pub fn pre_check(&self, confidence: f64) -> bool {
        confidence >= self.current_relevance_threshold * 0.8
    }

    /// Adaptive threshold via EMA on recent pass/fail history.
    fn adapt_thresholds(&mut self) {
        if self.history.len() < 10 {
            return;
        }
        let recent_pass_rate: f64 = self
            .history
            .iter()
            .map(|&p| if p { 1.0 } else { 0.0 })
            .sum::<f64>()
            / self.history.len() as f64;

        // Target pass rate ~0.7 (not too strict, not too loose)
        let target = 0.7;
        let gap = target - recent_pass_rate;

        // Move thresholds toward tighter when failing too often, looser when passing too often
        let adjustment = self.ema_alpha * gap;
        self.current_relevance_threshold =
            (self.current_relevance_threshold - adjustment).clamp(0.2, 0.95);
        self.current_faithfulness_threshold =
            (self.current_faithfulness_threshold - adjustment).clamp(0.2, 0.95);
        self.current_completeness_threshold =
            (self.current_completeness_threshold - adjustment).clamp(0.2, 0.95);
        self.current_uncertainty_max = (self.current_uncertainty_max + adjustment).clamp(0.1, 0.8);
    }

    pub fn reset_stats(&mut self) {
        self.total_checks = 0;
        self.fast_path_count = 0;
        self.slow_path_count = 0;
        self.gate_passed = 0;
        self.gate_failed = 0;
        self.history.clear();
    }

    pub fn report(&self) -> GateReport {
        GateReport {
            current_relevance: self.current_relevance_threshold,
            current_faithfulness: self.current_faithfulness_threshold,
            current_completeness: self.current_completeness_threshold,
            current_uncertainty: self.current_uncertainty_max,
            total_checks: self.total_checks,
            fast_path_ratio: if self.total_checks > 0 {
                self.fast_path_count as f64 / self.total_checks as f64
            } else {
                0.0
            },
            pass_ratio: if self.total_checks > 0 {
                self.gate_passed as f64 / self.total_checks as f64
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct QualityScores {
    pub relevance: f64,
    pub faithfulness: f64,
    pub completeness: f64,
    pub uncertainty: f64,
}

impl QualityScores {
    pub fn combined(&self) -> f64 {
        (self.relevance + self.faithfulness + self.completeness + (1.0 - self.uncertainty)) / 4.0
    }
}

#[derive(Debug, Clone)]
pub struct GateReport {
    pub current_relevance: f64,
    pub current_faithfulness: f64,
    pub current_completeness: f64,
    pub current_uncertainty: f64,
    pub total_checks: u64,
    pub fast_path_ratio: f64,
    pub pass_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_quality_gate() {
        let g = QualityGate::new();
        assert_eq!(g.relevance_threshold, 0.6);
        assert!(g.enable_fast_path);
        assert!(g.adapt_enabled);
    }

    #[test]
    fn test_evaluate_fast_path() {
        let mut g = QualityGate::new();
        let (should_fast, scores) = g.evaluate(0.8, 0.7, 0.7, 0.2);
        assert!(should_fast);
        assert_eq!(g.fast_path_count, 1);
        assert_eq!(g.gate_passed, 1);
        assert!((scores.combined() - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_evaluate_slow_path_low_relevance() {
        let mut g = QualityGate::new();
        let (should_fast, _) = g.evaluate(0.3, 0.9, 0.9, 0.1);
        assert!(!should_fast);
        assert_eq!(g.slow_path_count, 1);
        assert_eq!(g.gate_failed, 1);
    }

    #[test]
    fn test_evaluate_slow_path_high_uncertainty() {
        let mut g = QualityGate::new();
        let (should_fast, _) = g.evaluate(0.8, 0.8, 0.8, 0.7);
        assert!(!should_fast);
        assert_eq!(g.slow_path_count, 1);
    }

    #[test]
    fn test_pre_check_rejects_low_confidence() {
        let g = QualityGate::new();
        assert!(!g.pre_check(0.1));
        assert!(g.pre_check(0.8));
    }

    #[test]
    fn test_adaptive_threshold_relaxes_on_too_many_failures() {
        let mut g = QualityGate::new();
        g.adapt_enabled = true;

        // Simulate 20 consecutive failures
        for _ in 0..20 {
            g.evaluate(0.3, 0.3, 0.3, 0.7);
        }

        // After 10+ failures, thresholds should have relaxed
        assert!(g.current_relevance_threshold < g.relevance_threshold);
        assert!(g.current_uncertainty_max > g.uncertainty_max);
    }

    #[test]
    fn test_adaptive_threshold_tightens_on_too_many_passes() {
        let mut g = QualityGate::new();
        g.adapt_enabled = true;

        // Simulate 20 consecutive passes
        for _ in 0..20 {
            g.evaluate(0.9, 0.9, 0.9, 0.1);
        }

        // Too many passes → thresholds tighten (increase)
        assert!(g.current_relevance_threshold > g.relevance_threshold + 0.001);
        assert!(g.gate_passed >= 20);
    }

    #[test]
    fn test_reset_stats() {
        let mut g = QualityGate::new();
        g.evaluate(0.8, 0.7, 0.7, 0.2);
        assert_eq!(g.total_checks, 1);
        g.reset_stats();
        assert_eq!(g.total_checks, 0);
        assert_eq!(g.fast_path_count, 0);
    }

    #[test]
    fn test_report_format() {
        let mut g = QualityGate::new();
        g.evaluate(0.8, 0.7, 0.7, 0.2);
        g.evaluate(0.3, 0.9, 0.9, 0.1);
        let report = g.report();
        assert_eq!(report.total_checks, 2);
        assert!((report.fast_path_ratio - 0.5).abs() < 0.01);
    }
}
