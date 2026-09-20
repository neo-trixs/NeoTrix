//! Compliance evaluation engine

use super::finding::{Finding as DetailedFinding, FindingStatus};
use super::framework::ComplianceFramework;

/// Status of a compliance check (simplified for quick evaluation)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplianceStatus {
    /// Requirement is fully satisfied
    Pass,
    /// Requirement is not satisfied
    Fail,
    /// Requirement is partially satisfied
    Partial,
    /// Requirement does not apply to this context
    NotApplicable,
}

impl ComplianceStatus {
    /// Returns the numeric weight for score calculation
    pub fn weight(&self) -> f64 {
        match self {
            ComplianceStatus::Pass => 1.0,
            ComplianceStatus::Partial => 0.5,
            ComplianceStatus::Fail => 0.0,
            ComplianceStatus::NotApplicable => 0.0,
        }
    }

    /// Convert to FindingStatus
    pub fn to_finding_status(&self) -> FindingStatus {
        match self {
            ComplianceStatus::Pass => FindingStatus::Pass,
            ComplianceStatus::Fail => FindingStatus::Fail,
            ComplianceStatus::Partial => FindingStatus::Partial,
            ComplianceStatus::NotApplicable => FindingStatus::NotApplicable,
        }
    }
}

impl From<FindingStatus> for ComplianceStatus {
    fn from(status: FindingStatus) -> Self {
        match status {
            FindingStatus::Pass => ComplianceStatus::Pass,
            FindingStatus::Fail => ComplianceStatus::Fail,
            FindingStatus::Partial => ComplianceStatus::Partial,
            FindingStatus::NotApplicable => ComplianceStatus::NotApplicable,
        }
    }
}

/// A single finding from evaluating a requirement
#[derive(Debug, Clone)]
pub struct Finding {
    /// ID of the requirement this finding relates to
    pub requirement_id: String,
    /// Compliance status for this requirement
    pub status: ComplianceStatus,
    /// Evidence or notes supporting this finding
    pub evidence: String,
}

impl Finding {
    /// Convert to detailed Finding type
    pub fn to_detailed(&self) -> DetailedFinding {
        DetailedFinding::new(
            &self.requirement_id,
            self.status.to_finding_status(),
            &self.evidence,
        )
    }
}

/// A complete compliance report
#[derive(Debug, Clone)]
pub struct ComplianceReport {
    /// Name of the evaluated framework
    pub framework: String,
    /// All findings from the evaluation
    pub findings: Vec<Finding>,
    /// Overall compliance score (0.0 to 1.0)
    pub overall_score: f64,
}

impl ComplianceReport {
    /// Returns the count of findings with the given status
    pub fn count_by_status(&self, status: ComplianceStatus) -> usize {
        self.findings.iter().filter(|f| f.status == status).count()
    }

    /// Returns true if all applicable requirements passed
    pub fn is_compliant(&self) -> bool {
        self.findings
            .iter()
            .all(|f| f.status == ComplianceStatus::Pass || f.status == ComplianceStatus::NotApplicable)
    }

    /// Convert to detailed findings
    pub fn to_detailed_findings(&self) -> Vec<DetailedFinding> {
        self.findings.iter().map(|f| f.to_detailed()).collect()
    }
}

/// Evaluate a compliance framework against a set of check results.
///
/// `checks` is a slice of (requirement_id, status, evidence) tuples.
/// Requirements not present in `checks` are marked as Fail by default.
pub fn evaluate(framework: &ComplianceFramework, checks: &[(String, ComplianceStatus, String)]) -> ComplianceReport {
    let check_map: std::collections::HashMap<&str, &ComplianceStatus> = checks
        .iter()
        .map(|(id, status, _)| (id.as_str(), status))
        .collect();

    let mut findings = Vec::with_capacity(framework.requirements.len());
    let mut scored_weight = 0.0_f64;
    let mut total_weight = 0.0_f64;

    for req in &framework.requirements {
        let status = check_map
            .get(req.id.as_str())
            .copied()
            .unwrap_or(&ComplianceStatus::Fail);

        let evidence = checks
            .iter()
            .find(|(id, _, _)| id == &req.id)
            .map(|(_, _, ev)| ev.clone())
            .unwrap_or_default();

        let w = severity_weight(&req.severity);
        total_weight += w;
        scored_weight += status.weight() * w;

        findings.push(Finding {
            requirement_id: req.id.clone(),
            status: *status,
            evidence,
        });
    }

    let overall_score = if total_weight > 0.0 {
        scored_weight / total_weight
    } else {
        0.0
    };

    ComplianceReport {
        framework: framework.name.clone(),
        findings,
        overall_score,
    }
}

/// Returns a severity multiplier for weighted scoring
fn severity_weight(severity: &str) -> f64 {
    match severity {
        "critical" => 2.0,
        "high" => 1.5,
        "medium" => 1.0,
        "low" => 0.5,
        _ => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_framework() -> ComplianceFramework {
        ComplianceFramework {
            name: "Test Framework".to_string(),
            version: "1.0".to_string(),
            requirements: vec![
                super::super::framework::Requirement {
                    id: "REQ-1".to_string(),
                    title: "Req One".to_string(),
                    description: "First requirement".to_string(),
                    severity: "critical".to_string(),
                },
                super::super::framework::Requirement {
                    id: "REQ-2".to_string(),
                    title: "Req Two".to_string(),
                    description: "Second requirement".to_string(),
                    severity: "high".to_string(),
                },
                super::super::framework::Requirement {
                    id: "REQ-3".to_string(),
                    title: "Req Three".to_string(),
                    description: "Third requirement".to_string(),
                    severity: "medium".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_evaluate_all_pass() {
        let fw = sample_framework();
        let checks = vec![
            ("REQ-1".to_string(), ComplianceStatus::Pass, "Checked".to_string()),
            ("REQ-2".to_string(), ComplianceStatus::Pass, "Verified".to_string()),
            ("REQ-3".to_string(), ComplianceStatus::Pass, "Confirmed".to_string()),
        ];
        let report = evaluate(&fw, &checks);
        assert_eq!(report.overall_score, 1.0);
        assert!(report.is_compliant());
        assert_eq!(report.count_by_status(ComplianceStatus::Pass), 3);
    }

    #[test]
    fn test_evaluate_all_fail() {
        let fw = sample_framework();
        let checks = vec![
            ("REQ-1".to_string(), ComplianceStatus::Fail, "Failed".to_string()),
            ("REQ-2".to_string(), ComplianceStatus::Fail, "Not met".to_string()),
        ];
        let report = evaluate(&fw, &checks);
        assert!((report.overall_score - 0.0).abs() < f64::EPSILON);
        assert!(!report.is_compliant());
        assert_eq!(report.count_by_status(ComplianceStatus::Fail), 3);
    }

    #[test]
    fn test_evaluate_missing_check_defaults_to_fail() {
        let fw = sample_framework();
        let checks: Vec<(String, ComplianceStatus, String)> = vec![];
        let report = evaluate(&fw, &checks);
        assert_eq!(report.findings.len(), 3);
        for finding in &report.findings {
            assert_eq!(finding.status, ComplianceStatus::Fail);
        }
    }

    #[test]
    fn test_evaluate_partial_and_not_applicable() {
        let fw = sample_framework();
        let checks = vec![
            ("REQ-1".to_string(), ComplianceStatus::Pass, "OK".to_string()),
            ("REQ-2".to_string(), ComplianceStatus::Partial, "Partially done".to_string()),
            ("REQ-3".to_string(), ComplianceStatus::NotApplicable, "N/A".to_string()),
        ];
        let report = evaluate(&fw, &checks);
        assert_eq!(report.count_by_status(ComplianceStatus::Pass), 1);
        assert_eq!(report.count_by_status(ComplianceStatus::Partial), 1);
        assert_eq!(report.count_by_status(ComplianceStatus::NotApplicable), 1);
        // Score: (1.0*2.0 + 0.5*1.5 + 0.0*1.0) / (2.0+1.5+1.0) = 2.75/4.5
        let expected = (1.0 * 2.0 + 0.5 * 1.5 + 0.0 * 1.0) / (2.0 + 1.5 + 1.0);
        assert!((report.overall_score - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compliance_status_weight() {
        assert_eq!(ComplianceStatus::Pass.weight(), 1.0);
        assert_eq!(ComplianceStatus::Partial.weight(), 0.5);
        assert_eq!(ComplianceStatus::Fail.weight(), 0.0);
        assert_eq!(ComplianceStatus::NotApplicable.weight(), 0.0);
    }

    #[test]
    fn test_severity_weight() {
        assert_eq!(severity_weight("critical"), 2.0);
        assert_eq!(severity_weight("high"), 1.5);
        assert_eq!(severity_weight("medium"), 1.0);
        assert_eq!(severity_weight("low"), 0.5);
        assert_eq!(severity_weight("unknown"), 1.0);
    }

    #[test]
    fn test_framework_name_in_report() {
        let fw = sample_framework();
        let report = evaluate(&fw, &[]);
        assert_eq!(report.framework, "Test Framework");
    }

    #[test]
    fn test_report_is_cloneable() {
        let fw = sample_framework();
        let report = evaluate(&fw, &[]);
        let cloned = report.clone();
        assert_eq!(report.overall_score, cloned.overall_score);
        assert_eq!(report.findings.len(), cloned.findings.len());
    }

    #[test]
    fn test_compliance_status_conversion() {
        let status = ComplianceStatus::Pass;
        let finding_status = status.to_finding_status();
        assert_eq!(finding_status, FindingStatus::Pass);
        
        let back: ComplianceStatus = FindingStatus::Fail.into();
        assert_eq!(back, ComplianceStatus::Fail);
    }

    #[test]
    fn test_finding_to_detailed() {
        let finding = Finding {
            requirement_id: "REQ-1".to_string(),
            status: ComplianceStatus::Pass,
            evidence: "All checks passed".to_string(),
        };
        let detailed = finding.to_detailed();
        assert_eq!(detailed.requirement_id, "REQ-1");
        assert_eq!(detailed.status, FindingStatus::Pass);
    }
}
