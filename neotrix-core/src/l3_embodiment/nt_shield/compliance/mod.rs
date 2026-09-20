//! Compliance framework and evaluation engine
//!
//! Provides standardized compliance frameworks (OWASP Top 10, ASVS) and
//! an evaluation engine to assess compliance status against those frameworks.

pub mod evaluator;
pub mod finding;
pub mod framework;
pub mod requirement;

// Re-exports from evaluator
pub use evaluator::{ComplianceReport, ComplianceStatus};

// Re-exports from finding
pub use finding::{Finding, FindingStatus};

// Re-exports from framework
pub use framework::{ComplianceFramework, Requirement as FrameworkRequirement};

// Re-exports from requirement
pub use requirement::{Requirement, Severity, VerificationMethod};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_re_exports_work() {
        let fw = ComplianceFramework::owasp_top10();
        assert_eq!(fw.requirements.len(), 10);

        let checks: Vec<(String, ComplianceStatus, String)> = vec![
            ("OWASP-A01".to_string(), ComplianceStatus::Pass, "OK".to_string()),
        ];
        let report = evaluator::evaluate(&fw, &checks);
        assert_eq!(report.framework, "OWASP Top 10");
    }

    #[test]
    fn test_end_to_end_owasp_evaluation() {
        let fw = ComplianceFramework::owasp_top10();
        let checks = vec![
            ("OWASP-A01".to_string(), ComplianceStatus::Pass, "Access control enforced".to_string()),
            ("OWASP-A02".to_string(), ComplianceStatus::Pass, "TLS 1.3 enabled".to_string()),
            ("OWASP-A03".to_string(), ComplianceStatus::Pass, "Input sanitized".to_string()),
            ("OWASP-A04".to_string(), ComplianceStatus::Partial, "Design reviewed partially".to_string()),
            ("OWASP-A05".to_string(), ComplianceStatus::Fail, "Default credentials found".to_string()),
        ];
        let report = evaluator::evaluate(&fw, &checks);
        assert!(report.overall_score > 0.0);
        assert!(report.overall_score < 1.0);
        assert_eq!(report.findings.len(), 10);
    }

    #[test]
    fn test_end_to_end_asvs_evaluation() {
        let fw = ComplianceFramework::asvs();
        let checks: Vec<(String, ComplianceStatus, String)> = fw
            .requirements
            .iter()
            .map(|r| (r.id.clone(), ComplianceStatus::Pass, "Verified".to_string()))
            .collect();
        let report = evaluator::evaluate(&fw, &checks);
        assert_eq!(report.overall_score, 1.0);
        assert!(report.is_compliant());
    }

    #[test]
    fn test_existing_finding_types() {
        let finding = Finding::new("REQ-1", FindingStatus::Pass, "Test evidence");
        assert_eq!(finding.requirement_id, "REQ-1");
        assert!(finding.is_pass());
    }

    #[test]
    fn test_existing_requirement_types() {
        let req = Requirement::new(
            "v5-1.0.0",
            "Test Requirement",
            "Description",
            Severity::High,
            VerificationMethod::AutomatedTest,
        );
        assert!(req.validate_id());
        assert_eq!(req.severity, Severity::High);
    }
}