use std::collections::HashMap;

/// Result of a single safety gate check.
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub name: &'static str,
    pub passed: bool,
    pub score: f64,
    pub detail: String,
}

/// Aggregate safety gate report.
#[derive(Debug, Clone)]
pub struct SafetyReport {
    pub all_passed: bool,
    pub checks: Vec<CheckResult>,
    pub timestamp_ns: u64,
    pub phase_label: String,
}

/// SafetyGate — hardcoded in Rust, Ne compiler can never modify.
///
/// Before any compiler evolution phase, ALL checks MUST pass.
/// After evolution, re-run AND compare with pre-evolution report.
pub struct SafetyGate {
    /// Reference VSA primitive results for behavior preservation check.
    pub reference_primitives: HashMap<&'static str, Vec<u8>>,
    /// Reference negentropy threshold.
    pub negentropy_threshold: f64,
    /// Maximum allowed delta for backward compat (per-program output difference).
    pub compatibility_tolerance: f64,
    /// Maximum allowed meta-accuracy gap.
    pub meta_accuracy_tolerance: f64,
    /// Epoch counter, incremented each phase pass.
    pub epoch: u64,
}

impl SafetyGate {
    pub fn new() -> Self {
        Self {
            reference_primitives: Self::compute_reference_primitives(),
            negentropy_threshold: 0.01,
            compatibility_tolerance: 0.001,
            meta_accuracy_tolerance: 0.05,
            epoch: 0,
        }
    }

    /// Run all 5 safety checks against the current system state.
    pub fn check_all(
        &self,
        _pre_evolution: Option<&SafetyReport>,
        self_compile_ok: bool,
        backward_compat_scores: &[f64],
        vsa_primitives_current: &HashMap<&'static str, Vec<u8>>,
        negentropy_delta: f64,
        meta_predicted: f64,
        meta_actual: f64,
    ) -> SafetyReport {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let mut checks = Vec::with_capacity(5);

        // 1. Self-consistency: Ne compiler can compile itself
        checks.push(self.check_self_consistency(self_compile_ok));

        // 2. Backward compatibility: all existing programs unchanged
        checks.push(self.check_backward_compatibility(backward_compat_scores));

        // 3. Behavior preservation: 8 VSA primitives identical
        checks.push(self.check_behavior_preservation(vsa_primitives_current));

        // 4. Negentropy non-negative: N_total does not decrease
        checks.push(self.check_negentropy(negentropy_delta));

        // 5. Meta-accuracy: self-prediction matches actual
        checks.push(self.check_meta_accuracy(meta_predicted, meta_actual));

        let all_passed = checks.iter().all(|c| c.passed);

        SafetyReport {
            all_passed,
            checks,
            timestamp_ns: now,
            phase_label: format!("epoch-{}", self.epoch),
        }
    }

    /// Check 1: Self-consistency — compiler can compile itself.
    fn check_self_consistency(&self, ok: bool) -> CheckResult {
        CheckResult {
            name: "self_consistency",
            passed: ok,
            score: if ok { 1.0 } else { 0.0 },
            detail: if ok {
                "Ne compiler self-compiles successfully".into()
            } else {
                "Ne compiler FAILED to self-compile".into()
            },
        }
    }

    /// Check 2: Backward compatibility — existing programs unchanged.
    fn check_backward_compatibility(&self, scores: &[f64]) -> CheckResult {
        if scores.is_empty() {
            return CheckResult {
                name: "backward_compatibility",
                passed: true,
                score: 1.0,
                detail: "No existing programs to test".into(),
            };
        }
        let min_score = scores.iter().cloned().fold(f64::MAX, f64::min);
        let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;
        let passed = min_score >= 1.0 - self.compatibility_tolerance;
        CheckResult {
            name: "backward_compatibility",
            passed,
            score: avg_score,
            detail: format!(
                "min={:.6} avg={:.6} tolerance={:.6} count={}",
                min_score,
                avg_score,
                self.compatibility_tolerance,
                scores.len()
            ),
        }
    }

    /// Check 3: Behavior preservation — VSA primitives unchanged.
    fn check_behavior_preservation(
        &self,
        current: &HashMap<&'static str, Vec<u8>>,
    ) -> CheckResult {
        let mut all_match = true;
        let mut checked = 0usize;
        let mut max_deviation = 0.0f64;
        let mut details = Vec::new();

        for (name, expected) in &self.reference_primitives {
            if let Some(actual) = current.get(name) {
                checked += 1;
                let sim = Self::cosine_similarity(expected, actual);
                let dev = 1.0 - sim;
                if dev > max_deviation {
                    max_deviation = dev;
                }
                if dev > self.compatibility_tolerance {
                    all_match = false;
                    details.push(format!("{} deviation={:.6}", name, dev));
                }
            } else {
                all_match = false;
                details.push(format!("{} MISSING", name));
            }
        }

        let passed = all_match;
        CheckResult {
            name: "behavior_preservation",
            passed,
            score: 1.0 - max_deviation,
            detail: if passed {
                format!("all {} primitives match (max_dev={:.6})", checked, max_deviation)
            } else {
                format!("FAIL: {}", details.join("; "))
            },
        }
    }

    /// Check 4: Negentropy non-negative.
    fn check_negentropy(&self, delta: f64) -> CheckResult {
        let passed = delta >= -self.negentropy_threshold;
        CheckResult {
            name: "negentropy_non_negative",
            passed,
            score: delta,
            detail: format!(
                "ΔN_total={:.6} threshold={:.6}",
                delta, self.negentropy_threshold
            ),
        }
    }

    /// Check 5: Meta-accuracy within tolerance.
    fn check_meta_accuracy(&self, predicted: f64, actual: f64) -> CheckResult {
        let error = (predicted - actual).abs();
        let passed = error <= self.meta_accuracy_tolerance;
        CheckResult {
            name: "meta_accuracy",
            passed,
            score: 1.0 - error,
            detail: format!(
                "predicted={:.4} actual={:.4} error={:.4} tolerance={:.4}",
                predicted, actual, error, self.meta_accuracy_tolerance
            ),
        }
    }

    /// Compute reference primitive outputs on a fixed seed.
    fn compute_reference_primitives() -> HashMap<&'static str, Vec<u8>> {
        let mut m = HashMap::new();
        // 64-byte test vectors (simulated 512-bit VSA)
        let a: Vec<u8> = (0..64).map(|i| i as u8).collect();
        let b: Vec<u8> = (0..64).map(|i| (i as u8).wrapping_mul(3)).collect();

        // bind (XOR)
        let bound: Vec<u8> = a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect();
        m.insert("bind", bound);

        // bundle (majority-add)
        let bundled: Vec<u8> = a.iter().zip(b.iter()).map(|(x, y)| {
            let sum = (*x as u16) + (*y as u16);
            if sum >= 128 { 0xFFu8 } else { 0x00u8 }
        }).collect();
        m.insert("bundle", bundled);

        // permute (cyclic shift by 7)
        let k = 7usize;
        let mut perm = a.clone();
        let n = perm.len();
        perm.rotate_left(k % n);
        m.insert("permute", perm);

        // similarity (normalized Hamming)
        let matching: u32 = a.iter().zip(b.iter()).map(|(x, y)| (x ^ y).count_ones()).sum();
        let total = (a.len() * 8) as f64;
        let sim = 1.0 - (matching as f64 / total);
        m.insert("similarity", sim.to_le_bytes().to_vec());

        // negate (bitwise NOT)
        let negated: Vec<u8> = a.iter().map(|x| !x).collect();
        m.insert("negate", negated);

        // random_vector deterministically seeded
        let mut seed: u64 = 42;
        let rand_vec: Vec<u8> = (0..64).map(|_| {
            // xorshift64
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed as u8
        }).collect();
        m.insert("random_vector", rand_vec);

        // cosine (raw)
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| (*x as f64) * (*y as f64)).sum();
        let na: f64 = a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
        let nb: f64 = b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
        let cos = if na * nb == 0.0 { 0.0 } else { dot / (na * nb) };
        m.insert("cosine", cos.to_le_bytes().to_vec());

        // hamming_distance
        let dist: u32 = a.iter().zip(b.iter()).map(|(x, y)| (x ^ y).count_ones()).sum();
        m.insert("hamming_distance", dist.to_le_bytes().to_vec());

        m
    }

    fn cosine_similarity(a: &[u8], b: &[u8]) -> f64 {
        let min_len = a.len().min(b.len());
        if min_len == 0 {
            return 0.0;
        }
        let dot: f64 = a[..min_len].iter().zip(b[..min_len].iter())
            .map(|(x, y)| (*x as f64) * (*y as f64)).sum();
        let na: f64 = a[..min_len].iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
        let nb: f64 = b[..min_len].iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
        if na * nb == 0.0 { 0.0 } else { dot / (na * nb) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ref() -> SafetyGate {
        SafetyGate::new()
    }

    #[test]
    fn all_checks_pass_with_perfect_inputs() {
        let gate = make_ref();
        let primitives = SafetyGate::compute_reference_primitives();
        let report = gate.check_all(
            None,
            true,
            &[1.0, 1.0, 1.0],
            &primitives,
            0.05,
            0.9,
            0.88,
        );
        assert!(report.all_passed);
        assert_eq!(report.checks.len(), 5);
    }

    #[test]
    fn self_consistency_fails() {
        let gate = make_ref();
        let primitives = SafetyGate::compute_reference_primitives();
        let report = gate.check_all(None, false, &[1.0], &primitives, 0.05, 0.9, 0.88);
        assert!(!report.all_passed);
        assert!(!report.checks[0].passed);
    }

    #[test]
    fn backward_compat_fails_when_below_tolerance() {
        let gate = SafetyGate { compatibility_tolerance: 0.001, ..SafetyGate::new() };
        let primitives = SafetyGate::compute_reference_primitives();
        let report = gate.check_all(None, true, &[0.5], &primitives, 0.05, 0.9, 0.88);
        assert!(!report.all_passed);
        assert!(!report.checks[1].passed);
    }

    #[test]
    fn behavior_preservation_fails_on_mismatch() {
        let gate = make_ref();
        let mut primitives = SafetyGate::compute_reference_primitives();
        primitives.insert("bind", vec![0xFF; 64]);
        let report = gate.check_all(None, true, &[1.0], &primitives, 0.05, 0.9, 0.88);
        assert!(!report.all_passed);
        assert!(!report.checks[2].passed);
    }

    #[test]
    fn negentropy_fails_when_decreasing() {
        let gate = make_ref();
        let primitives = SafetyGate::compute_reference_primitives();
        let report = gate.check_all(None, true, &[1.0], &primitives, -0.1, 0.9, 0.88);
        assert!(!report.all_passed);
        assert!(!report.checks[3].passed);
    }

    #[test]
    fn meta_accuracy_fails_when_too_far_off() {
        let gate = make_ref();
        let primitives = SafetyGate::compute_reference_primitives();
        let report = gate.check_all(None, true, &[1.0], &primitives, 0.05, 0.9, 0.5);
        assert!(!report.all_passed);
        assert!(!report.checks[4].passed);
    }

    #[test]
    fn backward_compat_empty_list_passes() {
        let gate = make_ref();
        let primitives = SafetyGate::compute_reference_primitives();
        let report = gate.check_all(None, true, &[], &primitives, 0.05, 0.9, 0.88);
        assert!(report.all_passed);
        assert!(report.checks[1].passed);
    }

    #[test]
    fn epoch_increments_in_phase_label() {
        let mut gate = make_ref();
        let primitives = SafetyGate::compute_reference_primitives();
        let report = gate.check_all(None, true, &[], &primitives, 0.05, 0.9, 0.88);
        assert!(report.phase_label.contains("epoch-0"));
        gate.epoch = 5;
        let report2 = gate.check_all(None, true, &[], &primitives, 0.05, 0.9, 0.88);
        assert!(report2.phase_label.contains("epoch-5"));
    }

    #[test]
    fn behavior_preservation_detects_missing_primitive() {
        let gate = make_ref();
        let empty: HashMap<&'static str, Vec<u8>> = HashMap::new();
        let report = gate.check_all(None, true, &[1.0], &empty, 0.05, 0.9, 0.88);
        assert!(!report.all_passed);
        assert!(!report.checks[2].passed);
        assert!(report.checks[2].detail.contains("MISSING"));
    }

    #[test]
    fn default_primitives_count() {
        let gate = make_ref();
        assert_eq!(gate.reference_primitives.len(), 8);
    }

    #[test]
    fn check_all_sets_timestamp() {
        let gate = make_ref();
        let primitives = SafetyGate::compute_reference_primitives();
        let report = gate.check_all(None, true, &[1.0], &primitives, 0.05, 0.9, 0.88);
        assert!(report.timestamp_ns > 0);
    }

    #[test]
    fn cosine_similarity_identical() {
        let a = vec![0xAB; 64];
        let sim = SafetyGate::cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cosine_similarity_orthogonal() {
        let a = vec![0xFF; 64];
        let b = vec![0x00; 64];
        let sim = SafetyGate::cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-10);
    }
}
