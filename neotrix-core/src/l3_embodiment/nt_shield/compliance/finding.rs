//! Compliance finding definition
//!
//! Records the result of evaluating a specific requirement

/// Status of a finding after evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FindingStatus {
    /// Requirement fully met
    Pass,
    /// Requirement not met
    Fail,
    /// Requirement partially met
    Partial,
    /// Requirement not applicable
    NotApplicable,
}

impl std::fmt::Display for FindingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FindingStatus::Pass => write!(f, "Pass"),
            FindingStatus::Fail => write!(f, "Fail"),
            FindingStatus::Partial => write!(f, "Partial"),
            FindingStatus::NotApplicable => write!(f, "N/A"),
        }
    }
}

/// A single compliance finding
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Finding {
    /// The requirement ID this finding relates to
    pub requirement_id: String,
    /// Status of the finding
    pub status: FindingStatus,
    /// Evidence supporting the finding
    pub evidence: String,
    /// Suggested remediation if failed
    pub remediation: Option<String>,
    /// Optional details about partial compliance
    pub details: Option<String>,
}

impl Finding {
    /// Create a new finding
    pub fn new(
        requirement_id: impl Into<String>,
        status: FindingStatus,
        evidence: impl Into<String>,
    ) -> Self {
        Self {
            requirement_id: requirement_id.into(),
            status,
            evidence: evidence.into(),
            remediation: None,
            details: None,
        }
    }

    /// Add remediation guidance
    pub fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = Some(remediation.into());
        self
    }

    /// Add additional details
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Check if this finding is a failure
    pub fn is_failure(&self) -> bool {
        self.status == FindingStatus::Fail
    }

    /// Check if this finding is a pass
    pub fn is_pass(&self) -> bool {
        self.status == FindingStatus::Pass
    }
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.requirement_id, self.status, self.evidence
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_creation() {
        let finding = Finding::new(
            "v5-1.0.0",
            FindingStatus::Pass,
            "All tests passing",
        );

        assert_eq!(finding.requirement_id, "v5-1.0.0");
        assert_eq!(finding.status, FindingStatus::Pass);
        assert!(finding.is_pass());
        assert!(!finding.is_failure());
    }

    #[test]
    fn test_finding_with_remediation() {
        let finding = Finding::new(
            "v5-2.0.0",
            FindingStatus::Fail,
            "Missing error handling"
        )
        .with_remediation("Add try-catch blocks around network calls")
        .with_details("Specifically in nt_shield_comm.rs");

        assert!(finding.is_failure());
        assert!(finding.remediation.is_some());
        assert!(finding.details.is_some());
    }

    #[test]
    fn test_finding_status_display() {
        assert_eq!(FindingStatus::Pass.to_string(), "Pass");
        assert_eq!(FindingStatus::Fail.to_string(), "Fail");
        assert_eq!(FindingStatus::Partial.to_string(), "Partial");
        assert_eq!(FindingStatus::NotApplicable.to_string(), "N/A");
    }
}
