use super::goal_contract::VerdictVector;
use crate::neotrix::nt_mind::core::CapabilityVector;
use crate::neotrix::nt_mind::self_edit::MicroEdit;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub enum VsiStepVerdict {
    Pass(f64),
    Fail(String, f64),
    Skip,
}

impl VsiStepVerdict {
    pub fn passed(&self) -> bool {
        matches!(self, VsiStepVerdict::Pass(_))
    }

    pub fn score(&self) -> f64 {
        match self {
            VsiStepVerdict::Pass(s) => *s,
            VsiStepVerdict::Fail(_, s) => *s,
            VsiStepVerdict::Skip => 1.0,
        }
    }
}

pub struct VsiVerifier {
    pub threshold: f64,
    pub history: VecDeque<(String, VsiStepVerdict)>,
    pub checkpoints: HashMap<String, serde_json::Value>,
}

impl VsiVerifier {
    pub fn new() -> Self {
        Self {
            threshold: 0.85,
            history: VecDeque::with_capacity(100),
            checkpoints: HashMap::new(),
        }
    }

    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            threshold,
            history: VecDeque::with_capacity(100),
            checkpoints: HashMap::new(),
        }
    }

    pub fn verify(&mut self, stage_name: &str, capability: &CapabilityVector) -> VsiStepVerdict {
        let before_key = format!("{}_before", stage_name);
        let after_snapshot = Self::take_capability_snapshot(capability);

        let verdict = if let Some(before_value) = self.checkpoints.get(&before_key) {
            let before_cap: CapabilityVector =
                serde_json::from_value(before_value.clone()).unwrap_or_default();
            let after_cap: CapabilityVector =
                serde_json::from_value(after_snapshot.clone()).unwrap_or_default();

            let change_magnitude = compute_change_magnitude(&before_cap, &after_cap);

            if change_magnitude < self.threshold {
                VsiStepVerdict::Pass(change_magnitude)
            } else {
                VsiStepVerdict::Fail(
                    format!(
                        "capability change {:.4} exceeds threshold {}",
                        change_magnitude, self.threshold
                    ),
                    change_magnitude,
                )
            }
        } else {
            VsiStepVerdict::Skip
        };

        self.checkpoints
            .insert(format!("{}_after", stage_name), after_snapshot);

        self.history
            .push_back((stage_name.to_string(), verdict.clone()));
        if self.history.len() > 100 {
            self.history.pop_front();
        }

        verdict
    }

    pub fn snapshot_before(&mut self, stage_name: &str, capability: &CapabilityVector) {
        let key = format!("{}_before", stage_name);
        self.checkpoints
            .insert(key, Self::take_capability_snapshot(capability));
        if self.checkpoints.len() > 200 {
            let keys: Vec<String> = self.checkpoints.keys().cloned().collect();
            for k in keys.iter().step_by(2).take(50) {
                self.checkpoints.remove(k);
            }
        }
    }

    fn take_capability_snapshot(cap: &CapabilityVector) -> serde_json::Value {
        serde_json::to_value(cap.clone()).unwrap_or_default()
    }

    pub fn report(&self) -> String {
        if self.history.is_empty() {
            return "VSI: no verifications recorded".to_string();
        }

        let total = self.history.len();
        let passes = self.history.iter().filter(|(_, v)| v.passed()).count();
        let fails: Vec<&(String, VsiStepVerdict)> = self
            .history
            .iter()
            .filter(|(_, v)| matches!(v, VsiStepVerdict::Fail(_, _)))
            .collect();
        let skips = self
            .history
            .iter()
            .filter(|(_, v)| matches!(v, VsiStepVerdict::Skip))
            .count();

        let avg_score = if total > 0 {
            self.history.iter().map(|(_, v)| v.score()).sum::<f64>() / total as f64
        } else {
            0.0
        };

        let fail_details: Vec<String> = fails
            .iter()
            .map(|(name, v)| {
                if let VsiStepVerdict::Fail(reason, score) = v {
                    format!("  FAIL {}: {} (score={:.4})", name, reason, score)
                } else {
                    String::new()
                }
            })
            .collect();

        format!(
            "VSI Verifier Report:\n  Total: {}, Pass: {}, Fail: {}, Skip: {}\n  Avg confidence: {:.4}, Threshold: {}\n{}",
            total,
            passes,
            fails.len(),
            skips,
            avg_score,
            self.threshold,
            fail_details.join("\n"),
        )
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
        self.checkpoints.clear();
    }
}

impl Default for VsiVerifier {
    fn default() -> Self {
        Self::new()
    }
}

fn compute_change_magnitude(before: &CapabilityVector, after: &CapabilityVector) -> f64 {
    let b_arr = before.arr();
    let a_arr = after.arr();
    let max_len = b_arr.len().min(a_arr.len());
    if max_len == 0 {
        return 0.0;
    }
    let sum_sq: f64 = b_arr[..max_len]
        .iter()
        .zip(a_arr[..max_len].iter())
        .map(|(b, a)| (a - b).powi(2))
        .sum();
    (sum_sq / max_len as f64).sqrt()
}

pub fn verify_edit_magnitudes(edits: &[MicroEdit], max_magnitude: f64) -> VsiStepVerdict {
    let mut max_seen = 0.0_f64;
    for edit in edits {
        let magnitude = match edit {
            MicroEdit::AdjustDimension(_, amount) => amount.abs(),
            MicroEdit::BatchAdjust(pairs) => {
                pairs.iter().map(|(_, a)| a.abs()).fold(0.0_f64, f64::max)
            }
            MicroEdit::AddedDimension(_, value) => *value,
            MicroEdit::ModifiedDimension(_, _, new) => *new,
            _ => continue,
        };
        if magnitude > max_seen {
            max_seen = magnitude;
        }
    }
    if max_seen <= max_magnitude {
        VsiStepVerdict::Pass(max_seen)
    } else {
        VsiStepVerdict::Fail(
            format!(
                "edit magnitude {:.4} exceeds max {:.4}",
                max_seen, max_magnitude
            ),
            max_seen,
        )
    }
}

pub fn verify_reward(reward: f64) -> VsiStepVerdict {
    if (-1.0..=1.0).contains(&reward) {
        VsiStepVerdict::Pass(reward.abs())
    } else {
        VsiStepVerdict::Fail(format!("reward {:.4} outside [-1, 1]", reward), reward)
    }
}

pub fn verify_budget(edit_count: usize, budget: usize) -> VsiStepVerdict {
    if edit_count <= budget {
        VsiStepVerdict::Pass(edit_count as f64 / budget.max(1) as f64)
    } else {
        VsiStepVerdict::Fail(
            format!("{} edits exceeds budget {}", edit_count, budget),
            edit_count as f64,
        )
    }
}

pub fn verify_verdict_vector(verdict: &VerdictVector) -> VsiStepVerdict {
    let total = verdict.axes.len();
    if total == 0 {
        return VsiStepVerdict::Skip;
    }
    let passed = verdict.axes.iter().filter(|a| a.passed).count();
    let ratio = passed as f64 / total as f64;
    if ratio >= 0.5 {
        VsiStepVerdict::Pass(ratio)
    } else {
        VsiStepVerdict::Fail(format!("verdict {}/{} axes passed", passed, total), ratio)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_mind::core::CapabilityVector;
    use crate::neotrix::nt_mind::self_edit::MicroEdit;
    use crate::neotrix::nt_mind::self_iterating::goal_contract::{
        AxisSeverity, AxisVerdict, VerdictVector,
    };
    use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;

    #[test]
    fn test_vsi_verifier_pass_on_small_change() {
        let mut verifier = VsiVerifier::with_threshold(0.5);
        let mut brain = SelfIteratingBrain::new();
        let before = brain.brain.capability.clone();

        verifier.snapshot_before("test_stage", &brain.brain.capability);
        brain.brain.capability = before;
        let verdict = verifier.verify("test_stage", &brain.brain.capability);
        assert!(verdict.passed(), "zero change should pass");
        assert!(!matches!(verdict, VsiStepVerdict::Skip));
    }

    #[test]
    fn test_vsi_verifier_fail_on_large_change() {
        let mut verifier = VsiVerifier::with_threshold(0.1);
        let mut brain = SelfIteratingBrain::new();

        verifier.snapshot_before("test_stage", &brain.brain.capability);
        // Mutate capability significantly
        let mut cap = brain.brain.capability.clone();
        for v in cap.arr_mut().iter_mut().take(3) {
            *v = 0.9;
        }
        cap.normalize();
        brain.brain.capability = cap;

        let verdict = verifier.verify("test_stage", &brain.brain.capability);
        assert!(matches!(verdict, VsiStepVerdict::Fail(_, _)));
    }

    #[test]
    fn test_vsi_verifier_report_format() {
        let mut verifier = VsiVerifier::with_threshold(0.5);
        let mut brain = SelfIteratingBrain::new();
        let before = brain.brain.capability.clone();

        verifier.snapshot_before("stage_a", &brain.brain.capability);
        brain.brain.capability = before.clone();
        verifier.verify("stage_a", &brain.brain.capability);

        verifier.snapshot_before("stage_b", &brain.brain.capability);
        brain.brain.capability = before;
        verifier.verify("stage_b", &brain.brain.capability);

        let report = verifier.report();
        assert!(report.contains("VSI Verifier Report"));
        assert!(report.contains("Pass: 2"));
        assert!(report.contains("Total: 2"));
    }

    #[test]
    fn test_verify_edit_magnitudes_pass() {
        let edits = vec![
            MicroEdit::AdjustDimension("typography".into(), 0.1),
            MicroEdit::AdjustDimension("grid".into(), 0.2),
        ];
        let verdict = verify_edit_magnitudes(&edits, 0.5);
        assert!(verdict.passed());

        let large = vec![MicroEdit::AdjustDimension("test".into(), 2.0)];
        let fail = verify_edit_magnitudes(&large, 0.5);
        assert!(matches!(fail, VsiStepVerdict::Fail(_, _)));
    }

    #[test]
    fn test_verify_reward_bounds() {
        assert!(verify_reward(0.5).passed());
        assert!(verify_reward(1.0).passed());
        assert!(verify_reward(-1.0).passed());
        assert!(matches!(verify_reward(1.5), VsiStepVerdict::Fail(_, _)));
        assert!(matches!(verify_reward(-2.0), VsiStepVerdict::Fail(_, _)));
    }

    #[test]
    fn test_verify_budget() {
        assert!(verify_budget(5, 10).passed());
        assert!(matches!(verify_budget(10, 5), VsiStepVerdict::Fail(_, _)));
    }

    #[test]
    fn test_verify_verdict_vector() {
        let mut v = VerdictVector::new();
        assert!(matches!(verify_verdict_vector(&v), VsiStepVerdict::Skip));

        v.push(AxisVerdict {
            axis_name: "compile".into(),
            passed: true,
            details: "ok".into(),
            exit_code: Some(0),
            severity: AxisSeverity::Critical,
        });
        v.push(AxisVerdict {
            axis_name: "test".into(),
            passed: false,
            details: "fail".into(),
            exit_code: Some(1),
            severity: AxisSeverity::High,
        });
        let verdict = verify_verdict_vector(&v);
        assert!(verdict.passed(), "1/2 passes should be >= 0.5");
    }

    #[test]
    fn test_vsi_verifier_skip_on_no_snapshot() {
        let mut verifier = VsiVerifier::new();
        let brain = SelfIteratingBrain::new();
        let verdict = verifier.verify("unknown_stage", &brain.brain.capability);
        assert!(matches!(verdict, VsiStepVerdict::Skip));
    }

    #[test]
    fn test_compute_change_magnitude_identical() {
        let cap = CapabilityVector::default();
        let mag = compute_change_magnitude(&cap, &cap);
        assert!((mag - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_vsi_verifier_history_capped() {
        let mut verifier = VsiVerifier::with_threshold(100.0);
        let mut brain = SelfIteratingBrain::new();
        for i in 0..150 {
            let stage = format!("stage_{}", i);
            let before = brain.brain.capability.clone();
            verifier.snapshot_before(&stage, &brain.brain.capability);
            brain.brain.capability = before;
            verifier.verify(&stage, &brain.brain.capability);
        }
        assert!(verifier.history.len() <= 100);
    }
}
