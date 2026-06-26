/// HGM — Hierarchical Gradient Mapping (Branch CMP Metric)
///
/// Three sub-metrics fused into a single Coherence-Measured Progress score:
///   structural_delta (S): how much the pipeline structure changed  [0..1]
///   behavioral_divergence (B): how much output behavior diverged   [0..1]
///   coherence_score (C): whether changes harmonize with existing   [0..1]
///
/// cmp = C * (1 - sqrt(S * B))  — higher = better
///
/// This module is self-contained: snapshots use simple proxy values
/// (handler_count, negentropy, cycle) rather than depending on ConsciousnessIntegration.

/// Point-in-time snapshot using proxy values.
#[derive(Debug, Clone)]
pub struct HgmSnapshot {
    pub handler_count: usize,
    pub negentropy: f64,
    pub cycle: u64,
}

impl HgmSnapshot {
    pub fn capture(handler_count: usize, negentropy: f64, cycle: u64) -> Self {
        Self {
            handler_count,
            negentropy,
            cycle,
        }
    }
}

/// Three fused sub-metrics computed from before/after snapshots.
#[derive(Debug, Clone)]
pub struct HgmMetric {
    pub structural_delta: f64,
    pub behavioral_divergence: f64,
    pub coherence_score: f64,
}

impl HgmMetric {
    /// Compute all 3 sub-metrics from before/after snapshots.
    ///
    /// - structural_delta = min(1.0, |count_diff| / max_count)
    /// - behavioral_divergence = min(1.0, |Δnegentropy| / max(1.0, |before.negentropy|))
    /// - coherence_score = 1.0 - min(1.0, |Δcycle| / 1000.0)
    pub fn compute(before: &HgmSnapshot, after: &HgmSnapshot) -> Self {
        let structural_delta = Self::compute_structural_delta(before, after);
        let behavioral_divergence = Self::compute_behavioral_divergence(before, after);
        let coherence_score = Self::compute_coherence(before, after);

        Self {
            structural_delta,
            behavioral_divergence,
            coherence_score,
        }
    }

    /// Combined CMP score = coherence * (1.0 - sqrt(structural * behavioral)).
    /// Higher is better. Range: [0.0, 1.0]
    pub fn cmp_score(&self) -> f64 {
        let penalty = (self.structural_delta * self.behavioral_divergence).sqrt();
        (self.coherence_score * (1.0 - penalty)).clamp(0.0, 1.0)
    }

    /// A mutation is an improvement when cmp_score > 0.5
    pub fn is_improvement(&self) -> bool {
        self.cmp_score() > 0.5
    }

    /// structural_delta = min(1.0, |count_diff| / max(1, max_count))
    fn compute_structural_delta(before: &HgmSnapshot, after: &HgmSnapshot) -> f64 {
        let diff = (after.handler_count as i64 - before.handler_count as i64).unsigned_abs();
        let max_count = before.handler_count.max(after.handler_count).max(1);
        (diff as f64 / max_count as f64).min(1.0)
    }

    /// behavioral_divergence = min(1.0, |Δnegentropy| / max(1.0, |before.negentropy|))
    fn compute_behavioral_divergence(before: &HgmSnapshot, after: &HgmSnapshot) -> f64 {
        let diff = (after.negentropy - before.negentropy).abs();
        let denom = before.negentropy.abs().max(1.0);
        (diff / denom).min(1.0)
    }

    /// coherence_score = 1.0 - min(1.0, |Δcycle| / 1000.0)
    fn compute_coherence(before: &HgmSnapshot, after: &HgmSnapshot) -> f64 {
        let cycle_diff = (after.cycle as i64 - before.cycle as i64).unsigned_abs();
        (1.0 - (cycle_diff as f64 / 1000.0).min(1.0)).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hs(handler_count: usize, negentropy: f64, cycle: u64) -> HgmSnapshot {
        HgmSnapshot {
            handler_count,
            negentropy,
            cycle,
        }
    }

    #[test]
    fn test_identical_snapshots_perfect_score() {
        let s = hs(10, 0.75, 5);
        let m = HgmMetric::compute(&s, &s);
        assert!(m.structural_delta < 0.01);
        assert!(m.behavioral_divergence < 0.01);
        assert!((m.coherence_score - 1.0).abs() < 0.01);
        let cmp = m.cmp_score();
        assert!(cmp > 0.95, "identical → cmp_score near 1.0, got {}", cmp);
        assert!(m.is_improvement());
    }

    #[test]
    fn test_handler_count_change_lowers_score() {
        let before = hs(10, 0.75, 5);
        let after = hs(15, 0.75, 10);
        let m = HgmMetric::compute(&before, &after);
        assert!(m.structural_delta > 0.0);
        assert!(m.cmp_score() < 0.95);
    }

    #[test]
    fn test_large_cycle_gap_lowers_coherence() {
        let before = hs(10, 0.75, 0);
        let after = hs(10, 0.75, 500);
        let m = HgmMetric::compute(&before, &after);
        assert!(
            m.coherence_score < 0.6,
            "large cycle gap → coherence < 0.6, got {}",
            m.coherence_score
        );
        assert!(m.cmp_score() < 0.95);
    }

    #[test]
    fn test_all_metrics_in_range() {
        let before = hs(5, 0.6, 1);
        let after = hs(8, 0.4, 2);
        let m = HgmMetric::compute(&before, &after);
        for v in [
            m.structural_delta,
            m.behavioral_divergence,
            m.coherence_score,
            m.cmp_score(),
        ] {
            assert!((0.0..=1.0).contains(&v), "metric out of [0,1]: {}", v);
        }
    }

    #[test]
    fn test_zero_handlers_no_panic() {
        let before = hs(0, 0.5, 0);
        let after = hs(0, 0.5, 0);
        let m = HgmMetric::compute(&before, &after);
        assert!(m.structural_delta < 0.01);
    }

    #[test]
    fn test_is_improvement_threshold() {
        let s = hs(10, 0.75, 1);
        let m = HgmMetric::compute(&s, &s);
        assert!(m.is_improvement());
    }

    #[test]
    fn test_cmp_score_range() {
        let pairs = [
            (hs(0, 0.0, 0), hs(100, 1.0, 100)),
            (hs(100, 1.0, 100), hs(0, 0.0, 0)),
            (hs(50, 0.5, 50), hs(51, 0.51, 51)),
        ];
        for (b, a) in &pairs {
            let m = HgmMetric::compute(b, a);
            assert!(
                (0.0..=1.0).contains(&m.cmp_score()),
                "cmp_score out of range: {}",
                m.cmp_score()
            );
        }
    }

    #[test]
    fn test_behavioral_divergence_from_negentropy() {
        let before = hs(10, 0.75, 5);
        let after = hs(10, 0.25, 10);
        let m = HgmMetric::compute(&before, &after);
        // |0.25 - 0.75| / max(1.0, |0.75|) = 0.5 / 1.0 = 0.5
        assert!((m.behavioral_divergence - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_coherence_from_cycle_diff() {
        let before = hs(10, 0.75, 0);
        let after = hs(10, 0.75, 500);
        let m = HgmMetric::compute(&before, &after);
        // 1.0 - min(1.0, 500/1000) = 0.5
        assert!((m.coherence_score - 0.5).abs() < 0.01);
    }
}
