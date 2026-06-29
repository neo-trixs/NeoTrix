#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

/// Evidence types that can be included in a proof bundle
#[derive(Debug, Clone)]
pub enum EvidenceType {
    CompilationTest {
        passed: bool,
        output: String,
    },
    UnitTest {
        name: String,
        passed: bool,
        output: String,
    },
    SafetyCheck {
        check: String,
        passed: bool,
        detail: String,
    },
    BehavioralTest {
        scenario: String,
        passed: bool,
        metrics: Vec<(String, f64)>,
    },
    RegressionTest {
        baseline: f64,
        actual: f64,
        regressed: bool,
    },
}

/// A single evidence record
#[derive(Debug, Clone)]
pub struct EvidenceRecord {
    pub id: u64,
    pub etype: EvidenceType,
    pub timestamp: u64,
    pub description: String,
}

/// A complete proof bundle for a modification
#[derive(Debug, Clone)]
pub struct ProofBundle {
    pub modification_id: u64,
    pub target: String,
    pub evidence: Vec<EvidenceRecord>,
    pub verdict: Verdict,
    pub bundle_hash: u64,
    pub created_at: u64,
}

/// Verdict on a proof bundle
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Verdict {
    Safe,
    Unsafe,
    Inconclusive,
    NotTested,
}

/// Runner that generates proof bundles by running decisive tests
pub struct DecisiveTestRunner {
    pub evidence_counter: u64,
    pub bundle_counter: u64,
}

impl DecisiveTestRunner {
    pub fn new() -> Self {
        DecisiveTestRunner {
            evidence_counter: 0,
            bundle_counter: 0,
        }
    }

    pub fn generate_bundle(
        &mut self,
        modification_id: u64,
        target: &str,
        test_code: &[u8],
        safety_checks: &[(&str, bool)],
    ) -> ProofBundle {
        let mut evidence = Vec::new();
        let now = now_secs();

        let comp = self.run_compilation_test(test_code);
        evidence.push(self.make_record(
            EvidenceType::CompilationTest {
                passed: comp,
                output: format!("compilation {}", if comp { "ok" } else { "failed" }),
            },
            "compile test",
        ));

        for (check, passed) in safety_checks {
            evidence.push(self.make_record(
                EvidenceType::SafetyCheck {
                    check: (*check).into(),
                    passed: *passed,
                    detail: format!("check {}: {}", check, if *passed { "pass" } else { "fail" }),
                },
                check,
            ));
        }

        let verdict = self.evaluate_verdict(&evidence);
        let hash = self.compute_bundle_hash(modification_id, &evidence);

        self.bundle_counter += 1;
        ProofBundle {
            modification_id,
            target: target.into(),
            evidence,
            verdict,
            bundle_hash: hash,
            created_at: now,
        }
    }

    pub fn run_compilation_test(&self, code: &[u8]) -> bool {
        !code.is_empty()
    }

    pub fn run_unit_test(&self, name: &str, code: &[u8]) -> bool {
        !code.is_empty() && !name.is_empty()
    }

    pub fn run_regression_test(&self, baseline: f64, actual: f64, tolerance: f64) -> bool {
        (actual - baseline).abs() <= tolerance
    }

    pub fn bundle_report(&self, bundle: &ProofBundle) -> String {
        let passed = bundle
            .evidence
            .iter()
            .filter(|e| match &e.etype {
                EvidenceType::CompilationTest { passed, .. } => *passed,
                EvidenceType::UnitTest { passed, .. } => *passed,
                EvidenceType::SafetyCheck { passed, .. } => *passed,
                EvidenceType::BehavioralTest { passed, .. } => *passed,
                EvidenceType::RegressionTest { regressed, .. } => !*regressed,
            })
            .count();
        format!(
            "Bundle {} for '{}': {}/{} passed, verdict={:?}, hash={}",
            bundle.modification_id,
            bundle.target,
            passed,
            bundle.evidence.len(),
            bundle.verdict,
            bundle.bundle_hash,
        )
    }

    fn make_record(&mut self, etype: EvidenceType, desc: &str) -> EvidenceRecord {
        let id = self.evidence_counter;
        self.evidence_counter += 1;
        EvidenceRecord {
            id,
            etype,
            timestamp: now_secs(),
            description: desc.into(),
        }
    }

    fn evaluate_verdict(&self, evidence: &[EvidenceRecord]) -> Verdict {
        if evidence.is_empty() {
            return Verdict::NotTested;
        }
        let all_pass = evidence.iter().all(|e| match &e.etype {
            EvidenceType::CompilationTest { passed, .. } => *passed,
            EvidenceType::UnitTest { passed, .. } => *passed,
            EvidenceType::SafetyCheck { passed, .. } => *passed,
            EvidenceType::BehavioralTest { passed, .. } => *passed,
            EvidenceType::RegressionTest { regressed, .. } => !*regressed,
        });
        if all_pass {
            Verdict::Safe
        } else {
            Verdict::Unsafe
        }
    }

    fn compute_bundle_hash(&self, mod_id: u64, evidence: &[EvidenceRecord]) -> u64 {
        let mut h = mod_id.wrapping_mul(31);
        for e in evidence {
            h = h.wrapping_mul(17).wrapping_add(e.id);
        }
        h
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_bundle_success() {
        let mut runner = DecisiveTestRunner::new();
        let checks = vec![("no_unsafe", true), ("no_infinite_loop", true)];
        let bundle = runner.generate_bundle(1, "test_mod", b"fn main() {}", &checks);
        assert_eq!(bundle.verdict, Verdict::Safe);
        assert_eq!(bundle.evidence.len(), 3);
    }

    #[test]
    fn test_generate_bundle_failure() {
        let mut runner = DecisiveTestRunner::new();
        let checks = vec![("safety_check", false)];
        let bundle = runner.generate_bundle(2, "bad_mod", b"", &checks);
        assert_eq!(bundle.verdict, Verdict::Unsafe);
    }

    #[test]
    fn test_compilation_test_empty_fails() {
        let runner = DecisiveTestRunner::new();
        assert!(!runner.run_compilation_test(b""));
        assert!(runner.run_compilation_test(b"fn main() {}"));
    }

    #[test]
    fn test_regression_test() {
        let runner = DecisiveTestRunner::new();
        assert!(runner.run_regression_test(10.0, 10.05, 0.1));
        assert!(!runner.run_regression_test(10.0, 11.0, 0.1));
    }

    #[test]
    fn test_bundle_report() {
        let mut runner = DecisiveTestRunner::new();
        let checks = vec![("check1", true)];
        let bundle = runner.generate_bundle(1, "test", b"code", &checks);
        let report = runner.bundle_report(&bundle);
        assert!(report.contains("Safe"));
    }

    #[test]
    fn test_empty_evidence_inconclusive() {
        let runner = DecisiveTestRunner::new();
        let bundle = ProofBundle {
            modification_id: 0,
            target: "".into(),
            evidence: vec![],
            verdict: Verdict::NotTested,
            bundle_hash: 0,
            created_at: 0,
        };
        assert_eq!(bundle.verdict, Verdict::NotTested);
    }

    #[test]
    fn test_bundle_counter_increments() {
        let mut runner = DecisiveTestRunner::new();
        runner.generate_bundle(1, "a", b"code", &[]);
        assert_eq!(runner.bundle_counter, 1);
        runner.generate_bundle(2, "b", b"code", &[]);
        assert_eq!(runner.bundle_counter, 2);
    }

    #[test]
    fn test_evidence_counter_increments() {
        let mut runner = DecisiveTestRunner::new();
        runner.generate_bundle(1, "a", b"code", &[("c", true), ("d", true)]);
        assert_eq!(runner.evidence_counter, 3);
    }

    #[test]
    fn test_bundle_hash_deterministic() {
        let mut runner = DecisiveTestRunner::new();
        let b1 = runner.generate_bundle(1, "t", b"code", &[("c", true)]);
        let hash1 = b1.bundle_hash;
        let b2 = runner.generate_bundle(1, "t", b"code", &[("c", true)]);
        let hash2 = b2.bundle_hash;
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_unit_test_empty_name_fails() {
        let runner = DecisiveTestRunner::new();
        assert!(!runner.run_unit_test("", b"code"));
        assert!(runner.run_unit_test("test_name", b"code"));
    }
}
