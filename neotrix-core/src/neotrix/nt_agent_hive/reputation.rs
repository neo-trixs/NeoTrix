use std::collections::HashMap;

use super::sva_gate::SvaFieldEvaluation;
use super::types::HiveId;

/// Tracks per-sub-hive reputation based on consecutive SVAF content quality.
///
/// MMP content-driven convergence principle:
///   - Content quality is evaluated independently of peer trust
///   - Reputation is a secondary signal that amplifies over repeated cycles
///   - A single low-quality CMB doesn't penalize; persistent low-quality does
///
/// Score range: 0.0 (untrusted) to 1.0 (highly trusted), starting at 0.5 (neutral).
#[derive(Debug, Clone)]
pub struct ReputationEntry {
    /// Current reputation score [0.0, 1.0], EMA-smoothed
    pub score: f64,
    /// Consecutive packets where at least one SVAF field ACCEPTed
    pub consecutive_high: u32,
    /// Consecutive packets where NO SVAF field ACCEPTed (content-empty)
    pub consecutive_low: u32,
    /// Total packets evaluated from this peer
    pub total_packets: u64,
    /// Total SVAF-accepted packets from this peer
    pub total_accepted: u64,
    /// Last update timestamp (ns)
    pub last_update_ns: u64,
}

impl Default for ReputationEntry {
    fn default() -> Self {
        ReputationEntry {
            score: 0.5,
            consecutive_high: 0,
            consecutive_low: 0,
            total_packets: 0,
            total_accepted: 0,
            last_update_ns: now_ns(),
        }
    }
}

/// Per-sub-hive reputation tracker using SVAF content evaluations.
///
/// Reputation adapts slowly over many cycles — it's a cumulative trust signal
/// that supplements (not replaces) content-driven convergence.
pub struct ReputationTracker {
    entries: HashMap<HiveId, ReputationEntry>,
    /// Learning rate for EMA score updates
    alpha: f64,
    /// Number of consecutive_low before score starts decaying
    grace_period: u32,
    /// Score penalty per consecutive_low beyond grace period
    decay_per_step: f64,
    /// Score bonus per consecutive_high
    bonus_per_step: f64,
}

impl ReputationTracker {
    pub fn new() -> Self {
        ReputationTracker {
            entries: HashMap::new(),
            alpha: 0.15,
            grace_period: 3,
            decay_per_step: 0.05,
            bonus_per_step: 0.03,
        }
    }

    /// Record SVAF evaluation results for a sub-hive's packet.
    ///
    /// Updates reputation based on whether the content was SVAF-accepted.
    /// One rejected packet doesn't matter; persistent low-quality streaks do.
    pub fn record_evaluation(
        &mut self,
        hive_id: HiveId,
        _evaluations: &[SvaFieldEvaluation],
        accepted: bool,
    ) {
        let alpha = self.alpha;
        let grace_period = self.grace_period;
        let bonus_per_step = self.bonus_per_step;
        let decay_per_step = self.decay_per_step;

        let entry = self.entry_mut(hive_id);
        entry.total_packets += 1;
        entry.last_update_ns = now_ns();

        if accepted {
            entry.total_accepted += 1;
            entry.consecutive_high += 1;
            entry.consecutive_low = 0;
            let raw =
                compute_raw_bonus(entry.consecutive_high, true, bonus_per_step, decay_per_step);
            entry.score = entry.score + alpha * (raw - entry.score);
        } else {
            entry.consecutive_low += 1;
            entry.consecutive_high = 0;
            if entry.consecutive_low > grace_period {
                let raw = compute_raw_bonus(
                    entry.consecutive_low - grace_period,
                    false,
                    bonus_per_step,
                    decay_per_step,
                );
                entry.score = entry.score + alpha * (raw - entry.score);
            }
        }

        entry.score = entry.score.clamp(0.0, 1.0);
    }

    /// Get a peer's reputation score.
    /// Returns 0.5 (neutral) for unknown peers (trust-no-one, benefit-of-doubt).
    pub fn reputation(&self, hive_id: &HiveId) -> f64 {
        self.entries.get(hive_id).map(|e| e.score).unwrap_or(0.5)
    }

    /// Get the full reputation entry for a peer.
    pub fn entry(&self, hive_id: &HiveId) -> Option<&ReputationEntry> {
        self.entries.get(hive_id)
    }

    /// Get acceptance rate for a peer (0.0-1.0).
    pub fn acceptance_rate(&self, hive_id: &HiveId) -> f64 {
        self.entries
            .get(hive_id)
            .map(|e| {
                if e.total_packets == 0 {
                    0.5
                } else {
                    e.total_accepted as f64 / e.total_packets as f64
                }
            })
            .unwrap_or(0.5)
    }

    /// Weight factor for pool scoring: reputation modulates the final score.
    /// Reputation at 0.5 = neutral multiplier of 1.0.
    /// Range: [0.5, 1.5] — never fully zeroes out (content-driven convergence still works).
    pub fn weight_factor(&self, hive_id: &HiveId) -> f64 {
        let rep = self.reputation(hive_id);
        // Map [0.0, 1.0] → [0.5, 1.5], neutral at 0.5
        0.5 + rep
    }

    /// Number of tracked peers.
    pub fn tracked_count(&self) -> usize {
        self.entries.len()
    }

    /// Internal: get or create mutable entry.
    fn entry_mut(&mut self, hive_id: HiveId) -> &mut ReputationEntry {
        self.entries.entry(hive_id).or_default()
    }
}

/// Compute the raw score target for a streak.
fn compute_raw_bonus(
    streak: u32,
    is_positive: bool,
    bonus_per_step: f64,
    decay_per_step: f64,
) -> f64 {
    let magnitude = (streak as f64).min(20.0);
    if is_positive {
        (0.5 + magnitude * bonus_per_step).min(1.0)
    } else {
        (0.5 - magnitude * decay_per_step).max(0.0)
    }
}

fn now_ns() -> u64 {
    crate::core::nt_core_time::unix_now_nanos()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hive::SvaField;

    fn dummy_evaluations(accepted: bool) -> Vec<SvaFieldEvaluation> {
        let field = SvaField::CapabilityDelta;
        if accepted {
            vec![SvaFieldEvaluation {
                field,
                raw_score: 0.9,
                decision: crate::core::nt_core_hive::SvaDecision::Accept,
            }]
        } else {
            vec![SvaFieldEvaluation {
                field,
                raw_score: 0.1,
                decision: crate::core::nt_core_hive::SvaDecision::Reject,
            }]
        }
    }

    #[test]
    fn test_new_peer_default_reputation() {
        let tracker = ReputationTracker::new();
        let id = HiveId::new(1);
        assert!((tracker.reputation(&id) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_accepted_improves_reputation() {
        let mut tracker = ReputationTracker::new();
        let id = HiveId::new(1);

        for _ in 0..5 {
            tracker.record_evaluation(id, &dummy_evaluations(true), true);
        }

        let rep = tracker.reputation(&id);
        assert!(
            rep > 0.5,
            "accepted packets should improve reputation, got {}",
            rep
        );
    }

    #[test]
    fn test_grace_period_tolerates_occasional_rejection() {
        let mut tracker = ReputationTracker::new();
        let id = HiveId::new(1);

        // 2 rejections within grace period should not significantly penalize
        for _ in 0..2 {
            tracker.record_evaluation(id, &dummy_evaluations(false), false);
        }

        let rep = tracker.reputation(&id);
        assert!(
            (rep - 0.5).abs() < 0.1,
            "grace period should tolerate rejections, got {}",
            rep
        );
    }

    #[test]
    fn test_persistent_low_quality_penalizes() {
        let mut tracker = ReputationTracker::new();
        let id = HiveId::new(1);

        // 10 rejections — beyond grace period
        for _ in 0..10 {
            tracker.record_evaluation(id, &dummy_evaluations(false), false);
        }

        let rep = tracker.reputation(&id);
        assert!(
            rep < 0.5,
            "persistent rejection should lower reputation, got {}",
            rep
        );
    }

    #[test]
    fn test_weight_factor_range() {
        let tracker = ReputationTracker::new();
        let id = HiveId::new(1);
        let wf = tracker.weight_factor(&id);
        assert!(
            wf >= 0.5 && wf <= 1.5,
            "weight factor should be in [0.5, 1.5], got {}",
            wf
        );
    }

    #[test]
    fn test_acceptance_rate() {
        let mut tracker = ReputationTracker::new();
        let id = HiveId::new(1);

        tracker.record_evaluation(id, &dummy_evaluations(true), true);
        tracker.record_evaluation(id, &dummy_evaluations(true), true);
        tracker.record_evaluation(id, &dummy_evaluations(false), false);

        let rate = tracker.acceptance_rate(&id);
        assert!(
            (rate - 2.0 / 3.0).abs() < 0.01,
            "acceptance rate should be ~0.667, got {}",
            rate
        );
    }

    #[test]
    fn test_multiple_peers_independent() {
        let mut tracker = ReputationTracker::new();
        let id_a = HiveId::new(1);
        let id_b = HiveId::new(2);

        for _ in 0..10 {
            tracker.record_evaluation(id_a, &dummy_evaluations(true), true);
        }
        for _ in 0..10 {
            tracker.record_evaluation(id_b, &dummy_evaluations(false), false);
        }

        let rep_a = tracker.reputation(&id_a);
        let rep_b = tracker.reputation(&id_b);
        assert!(rep_a > rep_b, "peer A should have higher reputation than B");
    }

    #[test]
    fn test_tracked_count() {
        let mut tracker = ReputationTracker::new();
        assert_eq!(tracker.tracked_count(), 0);
        tracker.record_evaluation(HiveId::new(1), &dummy_evaluations(true), true);
        tracker.record_evaluation(HiveId::new(2), &dummy_evaluations(true), true);
        assert_eq!(tracker.tracked_count(), 2);
    }
}
