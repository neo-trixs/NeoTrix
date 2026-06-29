#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Verdict {
    pub passed: bool,
    pub score: f64,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoopVerifier {
    pub last_score: f64,
    pub total_verified: u64,
    pub total_passed: u64,
    pub last_verdict: Option<Verdict>,
    pub uat_gate: Option<crate::core::nt_core_loop::uat_gate::UatGate>,
}

impl LoopVerifier {
    pub fn new() -> Self {
        Self {
            last_score: 0.0,
            total_verified: 0,
            total_passed: 0,
            last_verdict: None,
            uat_gate: None,
        }
    }

    pub fn with_uat_gate(mut self, gate: crate::core::nt_core_loop::uat_gate::UatGate) -> Self {
        self.uat_gate = Some(gate);
        self
    }

    pub fn verify(&mut self, output_quality: f64, coherence: f64, handler_count: usize) -> Verdict {
        self.total_verified += 1;
        let score =
            output_quality * 0.4 + coherence * 0.4 + (handler_count as f64 / 60.0).min(1.0) * 0.2;
        self.last_score = score;
        let passed = score >= 0.3;

        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        if output_quality < 0.3 {
            issues.push("output quality below threshold".to_string());
            recommendations.push("increase reflection depth".to_string());
        }
        if coherence < 0.3 {
            issues.push("low coherence".to_string());
            recommendations.push("feed more context to specious present".to_string());
        }
        if handler_count < 10 {
            issues.push("low handler activation".to_string());
            recommendations.push("check handler registry for orphan methods".to_string());
        }

        if let Some(ref gate) = self.uat_gate {
            let rate = gate.pass_rate();
            if rate < 0.5 {
                issues.push("uat_pass_rate_below_threshold".to_string());
                recommendations.push("review test cards and fix failing scenarios".to_string());
            }
        }

        if passed {
            self.total_passed += 1;
        }

        let verdict = Verdict {
            passed,
            score,
            issues,
            recommendations,
        };
        self.last_verdict = Some(verdict.clone());
        verdict
    }

    pub fn pass_rate(&self) -> f64 {
        if self.total_verified == 0 {
            return 1.0;
        }
        self.total_passed as f64 / self.total_verified as f64
    }
}

impl Default for LoopVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifier_new() {
        let v = LoopVerifier::new();
        assert_eq!(v.total_verified, 0);
        assert_eq!(v.last_score, 0.0);
        assert_eq!(v.pass_rate(), 1.0);
    }

    #[test]
    fn test_verifier_verify_passed() {
        let mut v = LoopVerifier::new();
        let verdict = v.verify(0.8, 0.9, 40);
        assert!(verdict.passed);
        assert!(verdict.score > 0.3);
        assert!(verdict.issues.is_empty());
    }

    #[test]
    fn test_verifier_verify_failed_low_quality() {
        let mut v = LoopVerifier::new();
        let verdict = v.verify(0.1, 0.2, 3);
        assert!(!verdict.passed);
        assert!(verdict.score < 0.3);
        assert!(!verdict.issues.is_empty());
    }

    #[test]
    fn test_verifier_verify_issues() {
        let mut v = LoopVerifier::new();
        let verdict = v.verify(0.1, 0.9, 40);
        assert!(!verdict.issues.is_empty());
        assert!(verdict.issues.iter().any(|i| i.contains("quality")));
    }

    #[test]
    fn test_verifier_pass_rate_tracking() {
        let mut v = LoopVerifier::new();
        v.verify(0.8, 0.9, 40); // pass
        v.verify(0.1, 0.2, 3); // fail
        assert_eq!(v.total_verified, 2);
        assert_eq!(v.total_passed, 1);
        assert!((v.pass_rate() - 0.5).abs() < 0.01);
    }
}
