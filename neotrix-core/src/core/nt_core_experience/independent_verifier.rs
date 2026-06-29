use std::collections::HashMap;

/// The verification dimension being checked
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum VerifierDimension {
    /// Correctness — does the output match the spec?
    Correctness,
    /// Coherence — internally consistent?
    Coherence,
    /// Safety — no harmful side effects?
    Safety,
    /// Faithfulness — faithful to source context?
    Faithfulness,
    /// Efficiency — resource usage within budget?
    Efficiency,
    /// Novelty — introduces new information?
    Novelty,
    /// Consistency — consistent with past decisions?
    Consistency,
}

/// The outcome of a single verification check
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerifierOutcome {
    pub dimension: VerifierDimension,
    pub score: f64,
    pub passed: bool,
    pub weight: f64,
    pub detail: String,
    pub issues: Vec<String>,
}

/// Verdict of the independent verifier
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum VerifierVerdict {
    /// Output passes all gates
    Pass,
    /// Output has minor issues, can proceed with caution
    PassWithWarnings { warnings: Vec<String> },
    /// Output fails critical gates, should retry
    Fail { reasons: Vec<String> },
    /// Needs human judgment
    Escalate { questions: Vec<String> },
    /// No verdict possible (insufficient context)
    Abstain { reason: String },
}

/// A verification rubric: what constitutes "good" and "done"
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerifierRubric {
    pub name: String,
    pub dimensions: Vec<VerifierDimension>,
    pub dimension_weights: HashMap<VerifierDimension, f64>,
    pub pass_threshold: f64,
    pub fail_threshold: f64,
    pub require_all_dimensions: bool,
}

impl Default for VerifierRubric {
    fn default() -> Self {
        let weights = HashMap::from([
            (VerifierDimension::Correctness, 0.30),
            (VerifierDimension::Coherence, 0.20),
            (VerifierDimension::Safety, 0.20),
            (VerifierDimension::Faithfulness, 0.15),
            (VerifierDimension::Consistency, 0.10),
            (VerifierDimension::Efficiency, 0.05),
        ]);
        Self {
            name: "default".to_string(),
            dimensions: vec![
                VerifierDimension::Correctness,
                VerifierDimension::Coherence,
                VerifierDimension::Safety,
                VerifierDimension::Faithfulness,
            ],
            dimension_weights: weights,
            pass_threshold: 0.7,
            fail_threshold: 0.3,
            require_all_dimensions: false,
        }
    }
}

/// A record of a verification event (for audit/logging)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerificationRecord {
    pub id: u64,
    pub timestamp: u64,
    pub rubric: String,
    pub outcomes: Vec<VerifierOutcome>,
    pub composite_score: f64,
    pub verdict: VerifierVerdict,
    pub maker_context: String,
    pub checker_context: String,
}

/// The IndependentVerifier implements the **maker-checker separation**
/// principle of loop engineering.
///
/// Unlike the existing LoopVerifier (which runs in the same context as
/// the LoopEngine), the IndependentVerifier:
/// - Uses a **separate evaluation context** (different VSA subspace)
/// - Applies **rubric-based scoring** with configurable dimensions
/// - Tracks **verification history** for drift detection
/// - Maintains **calibration** (accuracy of own judgments over time)
#[derive(Debug)]
pub struct IndependentVerifier {
    pub rubrics: HashMap<String, VerifierRubric>,
    pub records: Vec<VerificationRecord>,
    pub total_verified: u64,
    pub total_passed: u64,
    pub calibration_accuracy: f64,
    pub calibration_count: u64,
    next_id: u64,
}

impl IndependentVerifier {
    pub fn new() -> Self {
        let mut rubrics = HashMap::new();
        rubrics.insert("default".to_string(), VerifierRubric::default());

        Self {
            rubrics,
            records: Vec::new(),
            total_verified: 0,
            total_passed: 0,
            calibration_accuracy: 0.0,
            calibration_count: 0,
            next_id: 1,
        }
    }

    pub fn register_rubric(&mut self, name: &str, rubric: VerifierRubric) {
        self.rubrics.insert(name.to_string(), rubric);
    }

    /// Run verification against a named rubric
    pub fn verify(
        &mut self,
        rubric_name: &str,
        dimension_scores: Vec<(VerifierDimension, f64, Vec<String>)>,
        maker_context: &str,
        checker_context: &str,
    ) -> VerifierVerdict {
        let rubric = match self.rubrics.get(rubric_name) {
            Some(r) => r.clone(),
            None => {
                return VerifierVerdict::Abstain {
                    reason: format!("unknown rubric: {}", rubric_name),
                }
            }
        };

        self.total_verified += 1;

        let mut outcomes = Vec::new();
        let mut total_weight = 0.0;
        let mut weighted_score = 0.0;
        let mut warnings = Vec::new();
        let mut failures = Vec::new();

        for (dimension, score, issues) in &dimension_scores {
            let weight = rubric
                .dimension_weights
                .get(dimension)
                .copied()
                .unwrap_or(0.1);
            let passed = *score >= rubric.pass_threshold;

            total_weight += weight;
            weighted_score += weight * score;

            if !passed {
                if *score < rubric.fail_threshold {
                    failures.push(format!(
                        "{:?} score {:.2} below fail threshold {:.2}: {}",
                        dimension,
                        score,
                        rubric.fail_threshold,
                        issues.join("; ")
                    ));
                } else {
                    warnings.push(format!(
                        "{:?} score {:.2} below pass threshold {:.2}",
                        dimension, score, rubric.pass_threshold
                    ));
                }
            }

            outcomes.push(VerifierOutcome {
                dimension: dimension.clone(),
                score: *score,
                passed,
                weight,
                detail: issues.join("; "),
                issues: issues.clone(),
            });
        }

        let composite_score = if total_weight > 0.0 {
            weighted_score / total_weight
        } else {
            0.0
        };

        let verdict = if !failures.is_empty() && composite_score < rubric.fail_threshold {
            VerifierVerdict::Fail {
                reasons: failures.clone(),
            }
        } else if !failures.is_empty() {
            VerifierVerdict::Escalate {
                questions: failures,
            }
        } else if !warnings.is_empty() || composite_score < rubric.pass_threshold {
            VerifierVerdict::PassWithWarnings {
                warnings: warnings.clone(),
            }
        } else {
            self.total_passed += 1;
            VerifierVerdict::Pass
        };

        let record = VerificationRecord {
            id: self.next_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            rubric: rubric_name.to_string(),
            outcomes,
            composite_score,
            verdict: verdict.clone(),
            maker_context: maker_context.to_string(),
            checker_context: checker_context.to_string(),
        };
        self.records.push(record);
        self.next_id += 1;

        if self.records.len() > 1000 {
            self.records.remove(0);
        }

        verdict
    }

    /// Record calibration feedback: was the verifier's judgment correct?
    pub fn record_calibration(&mut self, verifier_passed: bool, actual_outcome: bool) {
        self.calibration_count += 1;
        let correct = verifier_passed == actual_outcome;
        let delta = if correct { 1.0 } else { 0.0 };
        let n = self.calibration_count as f64;
        self.calibration_accuracy = self.calibration_accuracy * ((n - 1.0) / n) + delta / n;
    }

    pub fn pass_rate(&self) -> f64 {
        if self.total_verified == 0 {
            return 1.0;
        }
        self.total_passed as f64 / self.total_verified as f64
    }

    pub fn recent_verdicts(&self, n: usize) -> Vec<&VerifierVerdict> {
        self.records
            .iter()
            .rev()
            .take(n)
            .map(|r| &r.verdict)
            .collect()
    }

    pub fn stats(&self) -> VerifierStats {
        VerifierStats {
            total_verified: self.total_verified,
            total_passed: self.total_passed,
            pass_rate: self.pass_rate(),
            calibration_accuracy: self.calibration_accuracy,
            calibration_count: self.calibration_count,
            rubrics_count: self.rubrics.len(),
            records_count: self.records.len(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VerifierStats {
    pub total_verified: u64,
    pub total_passed: u64,
    pub pass_rate: f64,
    pub calibration_accuracy: f64,
    pub calibration_count: u64,
    pub rubrics_count: usize,
    pub records_count: usize,
}

impl Default for IndependentVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifier_new() {
        let v = IndependentVerifier::new();
        assert_eq!(v.total_verified, 0);
        assert_eq!(v.pass_rate(), 1.0);
    }

    #[test]
    fn test_verifier_verify_pass() {
        let mut v = IndependentVerifier::new();
        let verdict = v.verify(
            "default",
            vec![
                (VerifierDimension::Correctness, 0.9, vec![]),
                (VerifierDimension::Coherence, 0.8, vec![]),
                (VerifierDimension::Safety, 0.95, vec![]),
                (VerifierDimension::Faithfulness, 0.85, vec![]),
            ],
            "maker_vsa_subspace",
            "checker_vsa_subspace",
        );
        assert!(matches!(verdict, VerifierVerdict::Pass));
        assert_eq!(v.total_verified, 1);
        assert_eq!(v.total_passed, 1);
    }

    #[test]
    fn test_verifier_verify_fail() {
        let mut v = IndependentVerifier::new();
        let verdict = v.verify(
            "default",
            vec![
                (
                    VerifierDimension::Correctness,
                    0.1,
                    vec!["wrong output".to_string()],
                ),
                (VerifierDimension::Coherence, 0.2, vec![]),
            ],
            "maker",
            "checker",
        );
        assert!(matches!(verdict, VerifierVerdict::Fail { .. }));
        assert_eq!(v.total_passed, 0);
    }

    #[test]
    fn test_verifier_verify_pass_with_warnings() {
        let mut v = IndependentVerifier::new();
        let verdict = v.verify(
            "default",
            vec![
                (VerifierDimension::Correctness, 0.9, vec![]),
                (
                    VerifierDimension::Coherence,
                    0.5,
                    vec!["low coherence".to_string()],
                ),
                (VerifierDimension::Safety, 0.9, vec![]),
            ],
            "maker",
            "checker",
        );
        assert!(matches!(verdict, VerifierVerdict::PassWithWarnings { .. }));
    }

    #[test]
    fn test_verifier_unknown_rubric() {
        let mut v = IndependentVerifier::new();
        let verdict = v.verify("nonexistent", vec![], "maker", "checker");
        assert!(matches!(verdict, VerifierVerdict::Abstain { .. }));
    }

    #[test]
    fn test_verifier_calibration() {
        let mut v = IndependentVerifier::new();
        v.record_calibration(true, true);
        assert!((v.calibration_accuracy - 1.0).abs() < 0.01);
        v.record_calibration(true, false);
        assert!((v.calibration_accuracy - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_verifier_register_rubric() {
        let mut v = IndependentVerifier::new();
        let custom = VerifierRubric {
            name: "strict".to_string(),
            pass_threshold: 0.9,
            fail_threshold: 0.5,
            ..Default::default()
        };
        v.register_rubric("strict", custom);
        assert_eq!(v.rubrics.len(), 2);
    }

    #[test]
    fn test_verifier_stats() {
        let mut v = IndependentVerifier::new();
        v.verify(
            "default",
            vec![(VerifierDimension::Correctness, 0.9, vec![])],
            "m",
            "c",
        );
        let stats = v.stats();
        assert_eq!(stats.total_verified, 1);
        assert_eq!(stats.total_passed, 1);
        assert!((stats.pass_rate - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_verifier_recent_verdicts() {
        let mut v = IndependentVerifier::new();
        v.verify(
            "default",
            vec![(VerifierDimension::Correctness, 0.9, vec![])],
            "m",
            "c",
        );
        v.verify(
            "default",
            vec![(
                VerifierDimension::Correctness,
                0.1,
                vec!["fail".to_string()],
            )],
            "m",
            "c",
        );
        let recent = v.recent_verdicts(2);
        assert_eq!(recent.len(), 2);
        assert!(matches!(recent[0], VerifierVerdict::Fail { .. }));
    }

    #[test]
    fn test_verifier_dimension_weights() {
        let mut v = IndependentVerifier::new();
        let verdict = v.verify(
            "default",
            vec![
                (VerifierDimension::Correctness, 0.9, vec![]),
                (VerifierDimension::Safety, 0.1, vec!["unsafe".to_string()]),
            ],
            "m",
            "c",
        );
        assert!(
            matches!(verdict, VerifierVerdict::Fail { .. })
                || matches!(verdict, VerifierVerdict::Escalate { .. })
        );
    }

    #[test]
    fn test_verifier_record_limit() {
        let mut v = IndependentVerifier::new();
        for i in 0..1100 {
            v.verify(
                "default",
                vec![(VerifierDimension::Correctness, 0.9, vec![])],
                "m",
                "c",
            );
        }
        assert!(v.records.len() <= 1000);
    }
}
