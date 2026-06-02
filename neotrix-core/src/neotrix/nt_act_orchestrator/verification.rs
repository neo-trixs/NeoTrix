use crate::neotrix::nt_act_orchestrator::adversarial::{AdversarialVerifier, Perspective};
use crate::neotrix::nt_mind::self_iterating::pipeline::AutonomyLevel;

/// Post-execution verification stage
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VerificationStage {
    None,
    Basic,
    Adversarial,
}

/// Result of running verification
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub passed: bool,
    pub consensus_score: f64,
    pub perspectives: Vec<String>,
    pub issues: Vec<String>,
}

/// Components that can verify orchestration results
pub trait VerificationAware {
    fn verify_result(&self, input: &str, output: &str, stage: VerificationStage) -> VerificationResult;
}

/// Adversarial verification integration wrapping the existing AdversarialVerifier
pub struct AdversarialVerificationIntegration {
    pub verifier: AdversarialVerifier,
    pub min_autonomy: AutonomyLevel,
}

impl AdversarialVerificationIntegration {
    pub fn new() -> Self {
        Self {
            verifier: AdversarialVerifier::with_all_perspectives(),
            min_autonomy: AutonomyLevel::Full,
        }
    }

    pub fn with_verifier(verifier: AdversarialVerifier) -> Self {
        Self {
            verifier,
            min_autonomy: AutonomyLevel::Full,
        }
    }

    fn perspective_name(p: &Perspective) -> &'static str {
        match p {
            Perspective::Security => "Security",
            Perspective::Performance => "Performance",
            Perspective::Correctness => "Correctness",
            Perspective::Completeness => "Completeness",
            Perspective::EdgeCase => "EdgeCase",
        }
    }
}

impl Default for AdversarialVerificationIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl VerificationAware for AdversarialVerificationIntegration {
    fn verify_result(&self, input: &str, output: &str, stage: VerificationStage) -> VerificationResult {
        match stage {
            VerificationStage::None => VerificationResult {
                passed: true,
                consensus_score: 1.0,
                perspectives: vec![],
                issues: vec![],
            },
            VerificationStage::Basic => {
                let issues: Vec<String> = if output.trim().is_empty() {
                    vec!["Output is empty".to_string()]
                } else if output.len() < 10 {
                    vec!["Output is too short (less than 10 chars)".to_string()]
                } else {
                    vec![]
                };
                VerificationResult {
                    passed: issues.is_empty(),
                    consensus_score: if issues.is_empty() { 1.0 } else { 0.0 },
                    perspectives: vec!["Basic".to_string()],
                    issues,
                }
            }
            VerificationStage::Adversarial => {
                let findings = self.verifier.verify(input, output);
                let perspectives: Vec<String> = {
                    let mut seen: Vec<String> = Vec::new();
                    for finding in &findings {
                        let name = Self::perspective_name(&finding.perspective).to_string();
                        if !seen.contains(&name) {
                            seen.push(name);
                        }
                    }
                    if seen.is_empty() {
                        seen = vec![
                            "Correctness".to_string(),
                            "Completeness".to_string(),
                        ];
                    }
                    seen
                };
                let issues: Vec<String> = findings.iter().map(|f| f.finding.clone()).collect();
                let upheld_count = findings.iter().filter(|f| f.upheld).count();
                let total = findings.len();
                let consensus_score = if total == 0 { 1.0 } else { 1.0 - (upheld_count as f64 / total as f64) };
                VerificationResult {
                    passed: consensus_score >= 0.5 || total == 0,
                    consensus_score,
                    perspectives,
                    issues,
                }
            }
        }
    }
}

/// Helper: should verification run at given autonomy level
pub fn should_verify(autonomy: AutonomyLevel, stage: VerificationStage) -> bool {
    match stage {
        VerificationStage::None => false,
        VerificationStage::Basic => true,
        VerificationStage::Adversarial => autonomy == AutonomyLevel::Full,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_verification_passes_clean_output() {
        let integration = AdversarialVerificationIntegration::new();
        let result = integration.verify_result("test", "clean output with sufficient content", VerificationStage::Basic);
        assert!(result.passed, "basic check should pass clean output");
        assert_eq!(result.consensus_score, 1.0);
    }

    #[test]
    fn test_adversarial_verification_detects_issues() {
        let integration = AdversarialVerificationIntegration::new();
        let result = integration.verify_result(
            "auth",
            "The function takes a password and token, then executes an unsafe SQL query with a loop",
            VerificationStage::Adversarial,
        );
        assert!(!result.issues.is_empty(), "should detect nt_shield issues");
        assert!(!result.perspectives.is_empty(), "should report which perspectives flagged");
    }

    #[test]
    fn test_consensus_score_calculation() {
        let integration = AdversarialVerificationIntegration::new();
        let result = integration.verify_result("clean", "all good, no issues found", VerificationStage::Adversarial);
        assert!(result.consensus_score >= 0.0 && result.consensus_score <= 1.0, "score must be between 0 and 1");
    }

    #[test]
    fn test_adversarial_perspectives_non_empty() {
        let integration = AdversarialVerificationIntegration::new();
        let result = integration.verify_result("test", "some output content", VerificationStage::Adversarial);
        assert!(!result.perspectives.is_empty(), "adversarial should have perspective list");
    }

    #[test]
    fn test_verification_stage_none_skips_checking() {
        let integration = AdversarialVerificationIntegration::new();
        let result = integration.verify_result("", "", VerificationStage::None);
        assert!(result.passed, "None stage should always pass");
        assert_eq!(result.consensus_score, 1.0);
        assert!(result.issues.is_empty());
        assert!(result.perspectives.is_empty());
    }

    #[test]
    fn test_issues_list_for_failed_verification() {
        let integration = AdversarialVerificationIntegration::new();
        let result = integration.verify_result("test", "", VerificationStage::Basic);
        assert!(!result.passed, "empty output should fail basic verification");
        assert!(!result.issues.is_empty(), "should have at least one issue");
    }

    #[test]
    fn test_empty_input_handling() {
        let integration = AdversarialVerificationIntegration::new();
        let result = integration.verify_result("", "", VerificationStage::Adversarial);
        assert!(result.passed, "empty input should pass adversarial (no findings)");
        assert_eq!(result.consensus_score, 1.0);
    }

    #[test]
    fn test_high_consensus_score_passed() {
        let integration = AdversarialVerificationIntegration::new();
        let result = integration.verify_result("clean task", "perfect output without any risk indicators", VerificationStage::Adversarial);
        assert!(result.passed, "clean output should pass");
        assert!(result.consensus_score >= 0.5);
    }

    #[test]
    fn test_should_verify_autonomy_gating() {
        use AutonomyLevel::*;
        assert!(!should_verify(Proposal, VerificationStage::None));
        assert!(!should_verify(Full, VerificationStage::None));
        assert!(should_verify(Proposal, VerificationStage::Basic));
        assert!(should_verify(Bounded, VerificationStage::Basic));
        assert!(!should_verify(Proposal, VerificationStage::Adversarial));
        assert!(!should_verify(Bounded, VerificationStage::Adversarial));
        assert!(should_verify(Full, VerificationStage::Adversarial));
    }

    #[test]
    fn test_verification_result_types() {
        let vr = VerificationResult {
            passed: true,
            consensus_score: 0.95,
            perspectives: vec!["Correctness".into()],
            issues: vec![],
        };
        assert!(vr.passed);
        assert_eq!(vr.perspectives.len(), 1);
    }
}
