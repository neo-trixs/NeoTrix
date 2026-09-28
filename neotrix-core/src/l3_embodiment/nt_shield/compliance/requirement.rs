//! Compliance requirement definition
//!
//! Defines individual compliance requirements with verification methods

/// Severity level for compliance requirements
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Severity {
    /// Critical requirements - must be met for compliance
    Critical,
    /// High priority requirements - should be met
    High,
    /// Medium priority requirements - recommended
    Medium,
    /// Low priority requirements - optional improvements
    Low,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "Critical"),
            Severity::High => write!(f, "High"),
            Severity::Medium => write!(f, "Medium"),
            Severity::Low => write!(f, "Low"),
        }
    }
}

/// Verification method for a requirement
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum VerificationMethod {
    /// Automated test verification
    AutomatedTest,
    /// Manual code review
    CodeReview,
    /// Static analysis
    StaticAnalysis,
    /// Dynamic analysis
    DynamicAnalysis,
    /// Penetration testing
    PenTest,
    /// Configuration audit
    ConfigAudit,
    /// Documentation review
    DocReview,
    /// Custom verification method
    Custom(String),
}

impl std::fmt::Display for VerificationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationMethod::AutomatedTest => write!(f, "Automated Test"),
            VerificationMethod::CodeReview => write!(f, "Code Review"),
            VerificationMethod::StaticAnalysis => write!(f, "Static Analysis"),
            VerificationMethod::DynamicAnalysis => write!(f, "Dynamic Analysis"),
            VerificationMethod::PenTest => write!(f, "Penetration Test"),
            VerificationMethod::ConfigAudit => write!(f, "Configuration Audit"),
            VerificationMethod::DocReview => write!(f, "Documentation Review"),
            VerificationMethod::Custom(method) => write!(f, "Custom: {}", method),
        }
    }
}

/// A single compliance requirement
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Requirement {
    /// Unique identifier in format "v5-x.y.z"
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Detailed description of the requirement
    pub description: String,
    /// Severity level
    pub severity: Severity,
    /// How this requirement should be verified
    pub verification_method: VerificationMethod,
    /// Optional tags for categorization
    pub tags: Vec<String>,
    /// Optional reference to external standard (e.g., NIST, ISO)
    pub reference: Option<String>,
}

impl Requirement {
    /// Create a new requirement with minimal fields
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        severity: Severity,
        verification_method: VerificationMethod,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            severity,
            verification_method,
            tags: Vec::new(),
            reference: None,
        }
    }

    /// Add a tag to the requirement
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add a reference to an external standard
    pub fn with_reference(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }

    /// Validate the requirement ID format (v5-x.y.z)
    pub fn validate_id(&self) -> bool {
        let parts: Vec<&str> = self.id.split('-').collect();
        if parts.len() != 2 || parts[0] != "v5" {
            return false;
        }
        let version_parts: Vec<&str> = parts[1].split('.').collect();
        version_parts.len() == 3 && version_parts.iter().all(|p| p.parse::<u32>().is_ok())
    }
}

impl std::fmt::Display for Requirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} (Severity: {})",
            self.id, self.title, self.severity
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requirement_creation() {
        let req = Requirement::new(
            "v5-1.0.0",
            "Test Requirement",
            "Description",
            Severity::High,
            VerificationMethod::AutomatedTest,
        );

        assert_eq!(req.id, "v5-1.0.0");
        assert_eq!(req.title, "Test Requirement");
        assert_eq!(req.severity, Severity::High);
        assert!(req.validate_id());
    }

    #[test]
    fn test_requirement_id_validation() {
        let valid = Requirement::new(
            "v5-2.1.0",
            "Valid",
            "Desc",
            Severity::Medium,
            VerificationMethod::CodeReview,
        );
        assert!(valid.validate_id());

        let invalid = Requirement::new(
            "v4-1.0.0",
            "Invalid",
            "Desc",
            Severity::Low,
            VerificationMethod::StaticAnalysis,
        );
        assert!(!invalid.validate_id());

        let invalid2 = Requirement::new(
            "v5-1.0",
            "Invalid",
            "Desc",
            Severity::Low,
            VerificationMethod::StaticAnalysis,
        );
        assert!(!invalid2.validate_id());
    }

    #[test]
    fn test_severity_ordering() {
        // Ord is derived from the declaration order, so `Critical` sorts first
        // (`Critical < High < Medium < Low`). Nothing in the codebase sorts or
        // compares this enum — weighting goes through explicit matches — so the
        // assertion pins the derived order rather than a risk ranking.
        assert!(Severity::Critical < Severity::High);
        assert!(Severity::High < Severity::Medium);
        assert!(Severity::Medium < Severity::Low);
    }
}
