use crate::core::nt_core_experience::sahoo::{SahooGuard, SahooVerdict};
use crate::core::nt_core_hcube::{QuantizedVSA, VSA_DIM};
use serde::{Deserialize, Serialize};
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

// ── SEVerA FGGM: Formally Guarded Generative Models (arXiv:2603.25111) ──

/// A pre- or post-condition expressed as a string predicate.
/// At the preliminary scaffolding level, the predicate is a human-readable
/// contract statement. Future upgrades will parse it into a first-order
/// logic formula for automated verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub name: String,
    pub pre_condition: String,
    pub post_condition: String,
}

/// Result of verifying a single contract against the current system state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum VerificationResult {
    /// The contract was checked and the obligation was met.
    Verified,
    /// The contract was checked but the obligation was NOT met.
    Obliged,
}

/// A set of contracts that are verified together.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSet {
    pub contracts: Vec<Contract>,
}

impl ContractSet {
    pub fn new() -> Self {
        Self {
            contracts: Vec::new(),
        }
    }

    /// Add a contract to the set.
    pub fn add(&mut self, name: &str, pre: &str, post: &str) {
        self.contracts.push(Contract {
            name: name.to_string(),
            pre_condition: pre.to_string(),
            post_condition: post.to_string(),
        });
    }

    /// Verify all contracts against a set of (pre_state, post_state) pairs.
    /// Each contract is checked: if the precondition holds in `pre_state`,
    /// the postcondition must hold in `post_state`.
    /// For now, this is a string-level placeholder verification using
    /// keyword presence as a proxy for formal logic.
    pub fn verify_all(
        &self,
        pre_state: &HashMap<&'static str, f64>,
        post_state: &HashMap<&'static str, f64>,
    ) -> Vec<(Contract, VerificationResult)> {
        self.contracts
            .iter()
            .map(|c| {
                let pre_ok = c
                    .pre_condition
                    .split_whitespace()
                    .any(|w| pre_state.contains_key(w));
                let post_ok = !pre_ok
                    || c.post_condition
                        .split_whitespace()
                        .any(|w| post_state.contains_key(w));
                (
                    c.clone(),
                    if post_ok {
                        VerificationResult::Verified
                    } else {
                        VerificationResult::Obliged
                    },
                )
            })
            .collect()
    }
}

impl Default for ContractSet {
    fn default() -> Self {
        Self::new()
    }
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
    // ── SEVerA FGGM: formally guarded contracts ──
    /// Contract set with pre/post conditions for self-modification safety.
    pub contracts: ContractSet,
    // ── SAHOO: Safeguarded Alignment for High-Order Optimization ──
    /// Optional SAHOO guard for GDI + constraint + regression-risk monitoring.
    pub sahoo_guard: Option<SahooGuard>,
    // ── P0: Proof-carrying verification gate (arXiv:2603.28650) ──
    /// Verification gate with deterministic invariant checks.
    pub verification_gate: Option<VerificationGate>,
    /// Cost counter for irreversible operations (monotonic).
    pub irreversible_cost: u64,
    /// Previous cost counter snapshot.
    pub prev_irreversible_cost: u64,
    /// Identity chain hash for continuity validation.
    pub identity_chain_hash: u64,
}

impl SafetyGate {
    pub fn new() -> Self {
        let mut contracts = ContractSet::new();
        contracts.add(
            "self_compile",
            "compiler_exists compiler_runnable",
            "compiler_output_exists compile_success",
        );
        contracts.add(
            "behavior_preserve",
            "vsa_primitives_defined",
            "vsa_outputs_match",
        );
        contracts.add(
            "negentropy_stable",
            "system_running",
            "negentropy_non_negative",
        );
        let mut vg = VerificationGate::new();
        vg.register_critical("vsa_dimension_stable");
        vg.register_critical("negentropy_non_negative");
        vg.register_critical("self_compile_ok");
        vg.register_critical("identity_chain_valid");
        vg.register_critical("irreversible_cost_monotonic");
        vg.register_optional("output_format_stable");
        vg.register_optional("vsa_primitives_preserved");

        Self {
            reference_primitives: Self::compute_reference_primitives(),
            negentropy_threshold: 0.01,
            compatibility_tolerance: 0.001,
            meta_accuracy_tolerance: 0.05,
            epoch: 0,
            contracts,
            sahoo_guard: Some(SahooGuard::new()),
            verification_gate: Some(vg),
            irreversible_cost: 0,
            prev_irreversible_cost: 0,
            identity_chain_hash: 0x_4E45_5452_4958,
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

        let mut checks = Vec::with_capacity(7);

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

        // 6. SAHOO: GDI + constraint preservation + regression risk
        checks.push(self.check_sahoo(vsa_primitives_current));

        // 7. Verification gate — proof-carrying style deterministic invariants
        checks.push(self.check_verification(
            backward_compat_scores,
            vsa_primitives_current,
            negentropy_delta,
            self_compile_ok,
        ));

        let all_passed = checks.iter().all(|c| c.passed);

        SafetyReport {
            all_passed,
            checks,
            timestamp_ns: now,
            phase_label: format!("epoch-{}", self.epoch),
        }
    }

    // ── SEVerA FGGM: Contract verification ──

    /// Verify all registered contracts against the current (pre, post) state.
    /// Returns a list of (contract_name, result) pairs.
    pub fn verify_contracts(
        &self,
        pre_state: &HashMap<&'static str, f64>,
        post_state: &HashMap<&'static str, f64>,
    ) -> Vec<(&str, VerificationResult)> {
        self.contracts
            .contracts
            .iter()
            .map(|c| {
                let pre_ok = c
                    .pre_condition
                    .split_whitespace()
                    .any(|w| pre_state.contains_key(w));
                let post_ok = !pre_ok
                    || c.post_condition
                        .split_whitespace()
                        .any(|w| post_state.contains_key(w));
                (
                    c.name.as_str(),
                    if post_ok {
                        VerificationResult::Verified
                    } else {
                        VerificationResult::Obliged
                    },
                )
            })
            .collect()
    }

    /// Convenience: verify contracts and return true iff all are Verified.
    pub fn all_contracts_verified(
        &self,
        pre_state: &HashMap<&'static str, f64>,
        post_state: &HashMap<&'static str, f64>,
    ) -> bool {
        self.verify_contracts(pre_state, post_state)
            .iter()
            .all(|(_, r)| *r == VerificationResult::Verified)
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
    fn check_behavior_preservation(&self, current: &HashMap<&'static str, Vec<u8>>) -> CheckResult {
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
                format!(
                    "all {} primitives match (max_dev={:.6})",
                    checked, max_deviation
                )
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

    /// Check 6: SAHOO safeguard — GDI + constraint preservation + regression risk.
    fn check_sahoo(&self, current: &HashMap<&'static str, Vec<u8>>) -> CheckResult {
        let Some(guard) = &self.sahoo_guard else {
            return CheckResult {
                name: "sahoo_safeguard",
                passed: true,
                score: 1.0,
                detail: "SAHOO guard not configured — skipped".into(),
            };
        };

        // Convert VSA primitives to f64 embedding for GDI computation
        let baseline: Vec<f64> = self
            .reference_primitives
            .iter()
            .flat_map(|(_, v)| v.iter().map(|&b| b as f64 / 255.0))
            .collect();
        let current_f64: Vec<f64> = current
            .iter()
            .flat_map(|(_, v)| v.iter().map(|&b| b as f64 / 255.0))
            .collect();

        // Use a clone for the immutable evaluate call
        let mut guard_mut = guard.clone();
        let verdict = guard_mut.evaluate(&baseline, &current_f64, 1.0);

        match verdict {
            SahooVerdict::Allow => CheckResult {
                name: "sahoo_safeguard",
                passed: true,
                score: 1.0 - guard_mut.gdi.composite,
                detail: format!(
                    "SAHOO: GDI={:.4} constraints={} regression_risk={:.4}",
                    guard_mut.gdi.composite,
                    guard_mut.constraints.iter().all(|c| c.preserved),
                    guard_mut.regression.risk_score,
                ),
            },
            SahooVerdict::Flag(reason) => CheckResult {
                name: "sahoo_safeguard",
                passed: true,
                score: 1.0 - guard_mut.gdi.composite,
                detail: format!("SAHOO flag: {}", reason),
            },
            SahooVerdict::Deny(reason) => CheckResult {
                name: "sahoo_safeguard",
                passed: false,
                score: 0.0,
                detail: format!("SAHOO deny: {}", reason),
            },
        }
    }

    /// Check 7: Verification gate — proof-carrying deterministic invariant checks.
    fn check_verification(
        &self,
        backward_compat_scores: &[f64],
        vsa_primitives_current: &HashMap<&'static str, Vec<u8>>,
        negentropy_delta: f64,
        self_compile_ok: bool,
    ) -> CheckResult {
        let Some(ref vg) = self.verification_gate else {
            return CheckResult {
                name: "verification_gate",
                passed: true,
                score: 1.0,
                detail: "Verification gate not configured — skipped".into(),
            };
        };

        if !vg.is_enabled() {
            return CheckResult {
                name: "verification_gate",
                passed: true,
                score: 1.0,
                detail: "Verification gate disabled — skipped".into(),
            };
        }

        // Compute invariant inputs from available state
        let vsa_dim = VSA_DIM;
        let identity_chain_valid = self.identity_chain_hash == 0x_4E45_5452_4958;
        let irreversible_cost = self.irreversible_cost;
        let prev_cost = self.prev_irreversible_cost;
        let output_format_stable = if backward_compat_scores.is_empty() {
            true
        } else {
            let min_score = backward_compat_scores
                .iter()
                .cloned()
                .fold(f64::MAX, f64::min);
            min_score >= 1.0 - self.compatibility_tolerance
        };
        let vsa_primitives_match = {
            let mut all_match = true;
            for (name, expected) in &self.reference_primitives {
                if let Some(actual) = vsa_primitives_current.get(name) {
                    let sim = Self::cosine_similarity(expected, actual);
                    if sim < 1.0 - self.compatibility_tolerance {
                        all_match = false;
                    }
                } else {
                    all_match = false;
                }
            }
            all_match
        };

        let report = vg.verify(
            vsa_dim,
            negentropy_delta,
            self_compile_ok,
            identity_chain_valid,
            irreversible_cost,
            prev_cost,
            output_format_stable,
            vsa_primitives_match,
        );

        let critical_failures: Vec<&String> = report
            .failures
            .iter()
            .filter(|f| f.starts_with("critical:"))
            .collect();
        let all_pass = report.all_pass;

        CheckResult {
            name: "verification_gate",
            passed: all_pass,
            score: if all_pass { 1.0 } else { 0.0 },
            detail: if all_pass {
                format!(
                    "Verification gate: {} critical / {} optional passed",
                    report.critical_passed, report.optional_passed
                )
            } else {
                format!(
                    "Verification gate FAILED ({} critical failures): {}",
                    critical_failures.len(),
                    critical_failures
                        .iter()
                        .map(|f| f.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
        }
    }

    /// Compute reference primitive outputs on a fixed seed.
    pub fn compute_reference_primitives() -> HashMap<&'static str, Vec<u8>> {
        let mut m = HashMap::new();
        // 64-byte test vectors (simulated 512-bit VSA)
        let a: Vec<u8> = (0..64).map(|i| i as u8).collect();
        let b: Vec<u8> = (0..64).map(|i| (i as u8).wrapping_mul(3)).collect();

        // bind (FFT-HRR circular convolution)
        let bound = QuantizedVSA::bind(&a, &b);
        m.insert("bind", bound);

        // bundle (majority-add)
        let bundled: Vec<u8> = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| {
                let sum = (*x as u16) + (*y as u16);
                if sum >= 128 {
                    0xFFu8
                } else {
                    0x00u8
                }
            })
            .collect();
        m.insert("bundle", bundled);

        // permute (cyclic shift by 7)
        let k = 7usize;
        let mut perm = a.clone();
        let n = perm.len();
        perm.rotate_left(k % n);
        m.insert("permute", perm);

        // similarity (normalized Hamming)
        let matching: u32 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (x ^ y).count_ones())
            .sum();
        let total = (a.len() * 8) as f64;
        let sim = 1.0 - (matching as f64 / total);
        m.insert("similarity", sim.to_le_bytes().to_vec());

        // negate (bitwise NOT)
        let negated: Vec<u8> = a.iter().map(|x| !x).collect();
        m.insert("negate", negated);

        // random_vector deterministically seeded
        let mut seed: u64 = 42;
        let rand_vec: Vec<u8> = (0..64)
            .map(|_| {
                // xorshift64
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                seed as u8
            })
            .collect();
        m.insert("random_vector", rand_vec);

        // cosine (raw)
        let dot: f64 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (*x as f64) * (*y as f64))
            .sum();
        let na: f64 = a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
        let nb: f64 = b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
        let cos = if na * nb == 0.0 { 0.0 } else { dot / (na * nb) };
        m.insert("cosine", cos.to_le_bytes().to_vec());

        // hamming_distance
        let dist: u32 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (x ^ y).count_ones())
            .sum();
        m.insert("hamming_distance", dist.to_le_bytes().to_vec());

        m
    }

    fn cosine_similarity(a: &[u8], b: &[u8]) -> f64 {
        let min_len = a.len().min(b.len());
        if min_len == 0 {
            return 0.0;
        }
        let dot: f64 = a[..min_len]
            .iter()
            .zip(b[..min_len].iter())
            .map(|(x, y)| (*x as f64) * (*y as f64))
            .sum();
        let na: f64 = a[..min_len]
            .iter()
            .map(|x| (*x as f64).powi(2))
            .sum::<f64>()
            .sqrt();
        let nb: f64 = b[..min_len]
            .iter()
            .map(|x| (*x as f64).powi(2))
            .sum::<f64>()
            .sqrt();
        if na * nb == 0.0 {
            0.0
        } else {
            dot / (na * nb)
        }
    }
}

// ── SGM Statistical Safety Gate ──────────────────────────────────────

/// Compute the Hoeffding bound for n samples at significance level alpha.
///
/// `bound = sqrt(ln(2/alpha) / (2*n))`
/// Returns 1.0 for n = 0 (maximum uncertainty).
pub fn compute_hoeffding_bound(n: u64, alpha: f64) -> f64 {
    if n == 0 {
        return 1.0;
    }
    let alpha = alpha.clamp(1e-300, 1.0);
    (2.0_f64 / alpha).ln().sqrt() / (2.0 * n as f64).sqrt()
}

/// StatisticalSafetyGate — SGM-inspired statistical confidence gate.
///
/// Replaces ad-hoc threshold checks with e-value tests and Hoeffding
/// concentration bounds (arXiv:2510.10232).  Designed as a second layer
/// after the existing `SafetyGate` fast checks.
///
/// # How it works
///
/// 1. Track `n_modifications` and `n_successful` over the lifetime of the gate.
/// 2. For each proposed edit, compute an e-value:
///    `e_value = empirical_success_rate / (1 - edit_risk)`
/// 3. Compare against the Hoeffding-adjusted lower confidence bound:
///    `lower_bound = empirical_success_rate - hoeffding_bound`
/// 4. Allow the edit iff the lower bound is at least the null-expected
///    success rate (`1 - edit_risk`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSafetyGate {
    /// Risk budget α — controls the false-positive rate (default 0.01).
    pub risk_budget: f64,
    /// Total number of tracked self-modifications.
    pub n_modifications: u64,
    /// Number of tracked modifications that were successful (non-harmful).
    pub n_successful: u64,
    /// Current Hoeffding bound: `sqrt(ln(2/α) / (2*n))`.
    pub hoeffding_bound: f64,
    /// Current e-value threshold: `n_successful/n + hoeffding_bound`.
    pub e_value_threshold: f64,
}

impl StatisticalSafetyGate {
    /// Create a new gate with default α = 0.01 and no history.
    pub fn new() -> Self {
        Self {
            risk_budget: 0.01,
            n_modifications: 0,
            n_successful: 0,
            hoeffding_bound: 1.0,
            e_value_threshold: 0.5,
        }
    }

    /// Create a gate with a custom risk budget.
    pub fn with_risk_budget(alpha: f64) -> Self {
        let alpha = alpha.clamp(1e-6, 1.0);
        Self {
            risk_budget: alpha,
            ..Self::new()
        }
    }

    /// Evaluate whether a proposed edit should be allowed.
    ///
    /// * `edit_risk` — claimed probability the edit could cause harm (0.0–1.0).
    /// * `description` — human-readable label for error messages.
    ///
    /// Returns `Ok(true)` if the statistical test passes, or `Err(reason)` with
    /// a detailed explanation if the edit is rejected.
    pub fn evaluate_with_stats(&self, edit_risk: f64, description: &str) -> Result<bool, String> {
        let n = self.n_modifications.max(1);
        let emp_success = self.n_successful as f64 / n as f64;
        let null_success = 1.0 - edit_risk.clamp(0.0, 0.9999);

        // E-value: how much empirical evidence supports "edit is safe".
        // e_value > 1 means empirical success exceeds the null expectation.
        let e_value = emp_success / null_success;

        // Hoeffding lower confidence bound on the true success rate.
        let lower_bound = (emp_success - self.hoeffding_bound).max(0.0);

        if lower_bound >= null_success - 1e-12 {
            Ok(true)
        } else {
            Err(format!(
                "StatisticalSafetyGate rejected '{}': e_value={:.4} \
                 (emp_success={:.4}, null_success={:.4}), \
                 lower_bound={:.4} < {:.4}, hoeffding={:.4}, n={}",
                description,
                e_value,
                emp_success,
                null_success,
                lower_bound,
                null_success,
                self.hoeffding_bound,
                self.n_modifications
            ))
        }
    }

    /// Record the outcome of a modification and update confidence bounds.
    pub fn record_outcome(&mut self, success: bool) {
        self.n_modifications += 1;
        if success {
            self.n_successful += 1;
        }
        self.hoeffding_bound = compute_hoeffding_bound(self.n_modifications, self.risk_budget);

        let n = self.n_modifications.max(1);
        let emp_success = self.n_successful as f64 / n as f64;
        self.e_value_threshold = emp_success + self.hoeffding_bound;
    }

    /// Empirical success rate.
    pub fn empirical_success_rate(&self) -> f64 {
        let n = self.n_modifications.max(1);
        self.n_successful as f64 / n as f64
    }
}

impl Default for StatisticalSafetyGate {
    fn default() -> Self {
        Self::new()
    }
}

// ── P0: Verification gate (arXiv:2603.28650) ──
// Unlike classifier checks (statistical), verification checks are DETERMINISTIC.
// They MUST have ∑δ=0 — zero false negative rate for critical invariants.

/// A deterministic invariant check with zero false negative tolerance.
#[derive(Debug, Clone)]
pub struct VerificationGate {
    invariants: Vec<VerificationInvariant>,
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct VerificationInvariant {
    pub name: &'static str,
    pub critical: bool,
}

/// Result of running verification checks on registered invariants.
#[derive(Debug, Clone)]
pub struct VerificationReport {
    pub all_pass: bool,
    pub critical_count: usize,
    pub critical_passed: usize,
    pub optional_count: usize,
    pub optional_passed: usize,
    pub failures: Vec<String>,
}

impl VerificationGate {
    pub fn new() -> Self {
        Self {
            invariants: Vec::new(),
            enabled: true,
        }
    }

    pub fn register_critical(&mut self, name: &'static str) {
        self.invariants.push(VerificationInvariant {
            name,
            critical: true,
        });
    }

    pub fn register_optional(&mut self, name: &'static str) {
        self.invariants.push(VerificationInvariant {
            name,
            critical: false,
        });
    }

    /// Verify all registered invariants against current state.
    /// If any critical invariant fails, the gate denies (hard gate).
    pub fn verify(
        &self,
        vsa_dim: usize,
        negentropy_delta: f64,
        self_compile_ok: bool,
        identity_chain_valid: bool,
        irreversible_cost: u64,
        prev_irreversible_cost: u64,
        output_format_stable: bool,
        vsa_primitives_match: bool,
    ) -> VerificationReport {
        if !self.enabled {
            return VerificationReport {
                all_pass: true,
                critical_count: 0,
                critical_passed: 0,
                optional_count: 0,
                optional_passed: 0,
                failures: Vec::new(),
            };
        }

        let mut critical_count = 0usize;
        let mut critical_passed = 0usize;
        let mut optional_count = 0usize;
        let mut optional_passed = 0usize;
        let mut failures: Vec<String> = Vec::new();

        for inv in &self.invariants {
            let passed = match inv.name {
                "vsa_dimension_stable" => vsa_dim == VSA_DIM,
                "negentropy_non_negative" => negentropy_delta >= 0.0,
                "self_compile_ok" => self_compile_ok,
                "identity_chain_valid" => identity_chain_valid,
                "irreversible_cost_monotonic" => irreversible_cost >= prev_irreversible_cost,
                "output_format_stable" => output_format_stable,
                "vsa_primitives_preserved" => vsa_primitives_match,
                _ => true,
            };

            if inv.critical {
                critical_count += 1;
                if passed {
                    critical_passed += 1;
                } else {
                    failures.push(format!("critical:{}", inv.name));
                }
            } else {
                optional_count += 1;
                if passed {
                    optional_passed += 1;
                } else {
                    failures.push(format!("optional:{}", inv.name));
                }
            }
        }

        let all_pass = critical_passed == critical_count;
        VerificationReport {
            all_pass,
            critical_count,
            critical_passed,
            optional_count,
            optional_passed,
            failures,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for VerificationGate {
    fn default() -> Self {
        Self::new()
    }
}

/// Transaction scope for safe self-modification.
/// Captures a snapshot of any cloneable state and atomically commits
/// or rolls back on drop.  If dropped without an explicit `commit()`,
/// the warning in `Drop` fires — the caller should inspect the log.
pub struct TransactionScope<T: Clone> {
    snapshot: T,
    committed: bool,
}

impl<T: Clone> TransactionScope<T> {
    pub fn new(state: T) -> Self {
        Self {
            snapshot: state.clone(),
            committed: false,
        }
    }

    pub fn commit(&mut self) {
        self.committed = true;
    }

    /// Restore `state` to the captured snapshot if not yet committed.
    pub fn rollback(&mut self, state: &mut T) {
        if !self.committed {
            *state = self.snapshot.clone();
        }
    }
}

impl<T: Clone> Drop for TransactionScope<T> {
    fn drop(&mut self) {
        if !self.committed {
            log::warn!("[TransactionScope] dropped without commit — state may be stale");
        }
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
        let report = gate.check_all(None, true, &[1.0, 1.0, 1.0], &primitives, 0.05, 0.9, 0.88);
        assert!(report.all_passed);
        assert_eq!(report.checks.len(), 7);
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
        let gate = SafetyGate {
            compatibility_tolerance: 0.001,
            ..SafetyGate::new()
        };
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

    // ── StatisticalSafetyGate tests ────────────────────────────────────

    #[test]
    fn statistical_gate_default_constructs() {
        let gate = StatisticalSafetyGate::new();
        assert!((gate.risk_budget - 0.01).abs() < 1e-12);
        assert_eq!(gate.n_modifications, 0);
        assert_eq!(gate.n_successful, 0);
        assert!((gate.hoeffding_bound - 1.0).abs() < 1e-12);
    }

    #[test]
    fn statistical_gate_with_custom_risk_budget() {
        let gate = StatisticalSafetyGate::with_risk_budget(0.05);
        assert!((gate.risk_budget - 0.05).abs() < 1e-12);
    }

    #[test]
    fn statistical_gate_risk_budget_clamped() {
        let gate = StatisticalSafetyGate::with_risk_budget(0.0);
        assert!((gate.risk_budget - 1e-6).abs() < 1e-12);
        let gate = StatisticalSafetyGate::with_risk_budget(2.0);
        assert!((gate.risk_budget - 1.0).abs() < 1e-12);
    }

    #[test]
    fn statistical_gate_empirical_success_rate_zero_history() {
        let gate = StatisticalSafetyGate::new();
        assert!((gate.empirical_success_rate() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn statistical_gate_record_outcome_updates_counters() {
        let mut gate = StatisticalSafetyGate::new();
        gate.record_outcome(true);
        assert_eq!(gate.n_modifications, 1);
        assert_eq!(gate.n_successful, 1);
        assert!((gate.empirical_success_rate() - 1.0).abs() < 1e-12);
        assert!(gate.hoeffding_bound < 1.0);

        gate.record_outcome(false);
        assert_eq!(gate.n_modifications, 2);
        assert_eq!(gate.n_successful, 1);
        assert!((gate.empirical_success_rate() - 0.5).abs() < 1e-12);
    }

    #[test]
    fn statistical_gate_accepts_edit_with_good_history() {
        let mut gate = StatisticalSafetyGate::new();
        // 95 / 100 successes → high empirical success rate
        for _ in 0..95 {
            gate.record_outcome(true);
        }
        for _ in 0..5 {
            gate.record_outcome(false);
        }

        // An edit with risk < 0.1 should be allowed
        let result = gate.evaluate_with_stats(0.1, "test-edit");
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    }

    #[test]
    fn statistical_gate_rejects_edit_with_high_risk_and_sparse_history() {
        let gate = StatisticalSafetyGate::new();
        // No history → no evidence → high-risk edit should be rejected
        let result = gate.evaluate_with_stats(0.5, "risky-edit-no-history");
        assert!(
            result.is_err(),
            "Expected Err for risky edit with no history"
        );
        let err = result.unwrap_err();
        assert!(err.contains("StatisticalSafetyGate rejected"));
    }

    #[test]
    fn statistical_gate_100_modifications_95_percent_success_allows_low_risk() {
        let mut gate = StatisticalSafetyGate::new();
        for _ in 0..95 {
            gate.record_outcome(true);
        }
        for _ in 0..5 {
            gate.record_outcome(false);
        }

        // After 100 mods with 95% success, edits with risk < 0.1 should pass
        let result = gate.evaluate_with_stats(0.09, "safe-edit");
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);

        // Very high risk edits should still be flagged
        let result = gate.evaluate_with_stats(0.5, "risky-edit");
        assert!(result.is_err(), "Expected Err for high-risk edit");
    }

    #[test]
    fn hoeffding_bound_converges_as_n_increases() {
        let n_values = [1u64, 10, 100, 1000, 10000];
        let alpha = 0.01;
        let mut bounds: Vec<f64> = Vec::new();

        for &n in &n_values {
            let b = compute_hoeffding_bound(n, alpha);
            bounds.push(b);
        }

        // Bound should be monotonically decreasing
        for i in 1..bounds.len() {
            assert!(
                bounds[i] < bounds[i - 1] + 1e-12,
                "hoeffding_bound[{}]={:.6} should be < bound[{}]={:.6}",
                i,
                bounds[i],
                i - 1,
                bounds[i - 1]
            );
        }
    }

    #[test]
    fn hoeffding_bound_zero_n_returns_one() {
        let b = compute_hoeffding_bound(0, 0.01);
        assert!((b - 1.0).abs() < 1e-12);
    }

    #[test]
    fn hoeffding_bound_tighter_with_larger_alpha() {
        let b_loose = compute_hoeffding_bound(100, 0.1);
        let b_tight = compute_hoeffding_bound(100, 0.01);
        assert!(
            b_loose < b_tight,
            "Larger alpha should give tighter (smaller) bound: {} < {}",
            b_loose,
            b_tight
        );
    }

    #[test]
    fn statistical_gate_e_value_threshold_updates_after_record() {
        let mut gate = StatisticalSafetyGate::new();
        assert!((gate.e_value_threshold - 0.5).abs() < 1e-12);

        gate.record_outcome(true);
        // e_value_threshold = emp_success + hoeffding_bound
        // = 1.0 + compute_hoeffding_bound(1, 0.01)
        let expected_hb = compute_hoeffding_bound(1, 0.01);
        assert!((gate.e_value_threshold - (1.0 + expected_hb)).abs() < 1e-12);
    }

    #[test]
    fn statistical_gate_serialization_roundtrip() {
        let mut gate = StatisticalSafetyGate::new();
        for _ in 0..50 {
            gate.record_outcome(true);
        }
        for _ in 0..3 {
            gate.record_outcome(false);
        }

        let json = serde_json::to_string(&gate).expect("serialize");
        let deserialized: StatisticalSafetyGate = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.n_modifications, gate.n_modifications);
        assert_eq!(deserialized.n_successful, gate.n_successful);
        assert!((deserialized.risk_budget - gate.risk_budget).abs() < 1e-12);
        assert!((deserialized.hoeffding_bound - gate.hoeffding_bound).abs() < 1e-12);
        assert!((deserialized.e_value_threshold - gate.e_value_threshold).abs() < 1e-12);
    }

    #[test]
    fn statistical_gate_accepts_after_all_successful() {
        let mut gate = StatisticalSafetyGate::new();
        // 10/10 successful modifications → strong evidence
        for _ in 0..10 {
            gate.record_outcome(true);
        }

        // Even moderate-risk edits should pass with perfect history
        let result = gate.evaluate_with_stats(0.2, "moderate-risk");
        assert!(
            result.is_ok(),
            "Expected Ok with perfect history, got: {:?}",
            result
        );
    }

    #[test]
    fn statistical_gate_rejects_when_history_poor() {
        let mut gate = StatisticalSafetyGate::new();
        // 5/10 successful → weak evidence
        for _ in 0..5 {
            gate.record_outcome(true);
        }
        for _ in 0..5 {
            gate.record_outcome(false);
        }

        let result = gate.evaluate_with_stats(0.1, "poor-history-edit");
        assert!(result.is_err(), "Expected Err with poor history");
    }

    #[test]
    fn compute_hoeffding_bound_edge_cases() {
        let b = compute_hoeffding_bound(1, 0.01);
        assert!(b > 0.0 && b <= 1.0, "Bound should be in (0,1], got {}", b);

        let b = compute_hoeffding_bound(1_000_000, 0.01);
        assert!(b < 0.01, "Bound with large n should be small, got {}", b);
    }

    // ── SEVerA FGGM tests ──────────────────────────────────────────────

    #[test]
    fn contract_set_new_is_empty() {
        let set = ContractSet::new();
        assert!(set.contracts.is_empty());
    }

    #[test]
    fn contract_set_add_contract() {
        let mut set = ContractSet::new();
        set.add("test", "pre_cond", "post_cond");
        assert_eq!(set.contracts.len(), 1);
        assert_eq!(set.contracts[0].name, "test");
    }

    #[test]
    fn verify_all_contracts_pass() {
        let mut set = ContractSet::new();
        set.add("c1", "a", "b");
        let mut pre = HashMap::new();
        pre.insert("a", 1.0);
        let mut post = HashMap::new();
        post.insert("b", 1.0);
        let results = set.verify_all(&pre, &post);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, VerificationResult::Verified);
    }

    #[test]
    fn verify_all_contracts_obliged() {
        let mut set = ContractSet::new();
        set.add("c1", "a", "b");
        let mut pre = HashMap::new();
        pre.insert("a", 1.0);
        let post = HashMap::new(); // post has no "b"
        let results = set.verify_all(&pre, &post);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, VerificationResult::Obliged);
    }

    #[test]
    fn safety_gate_has_default_contracts() {
        let gate = make_ref();
        assert_eq!(gate.contracts.contracts.len(), 3);
    }

    #[test]
    fn verify_contracts_returns_results() {
        let gate = make_ref();
        let mut pre = HashMap::new();
        pre.insert("compiler_exists", 1.0);
        pre.insert("system_running", 1.0);
        let mut post = HashMap::new();
        post.insert("compile_success", 1.0);
        post.insert("negentropy_non_negative", 1.0);
        let results = gate.verify_contracts(&pre, &post);
        assert_eq!(results.len(), 3);
        let verified_count = results
            .iter()
            .filter(|(_, r)| *r == VerificationResult::Verified)
            .count();
        assert!(verified_count >= 2);
    }

    #[test]
    fn all_contracts_verified_true_when_all_pass() {
        let gate = make_ref();
        let mut pre = HashMap::new();
        pre.insert("compiler_exists", 1.0);
        pre.insert("vsa_primitives_defined", 1.0);
        pre.insert("system_running", 1.0);
        let mut post = HashMap::new();
        post.insert("compile_success", 1.0);
        post.insert("vsa_outputs_match", 1.0);
        post.insert("negentropy_non_negative", 1.0);
        assert!(gate.all_contracts_verified(&pre, &post));
    }

    #[test]
    fn all_contracts_verified_false_when_one_fails() {
        let gate = make_ref();
        let mut pre = HashMap::new();
        pre.insert("compiler_exists", 1.0);
        pre.insert("vsa_primitives_defined", 1.0);
        let post = HashMap::new();
        // No matching post-conditions
        assert!(!gate.all_contracts_verified(&pre, &post));
    }
}
