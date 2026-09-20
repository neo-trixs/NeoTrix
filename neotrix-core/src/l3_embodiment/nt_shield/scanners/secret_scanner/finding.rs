//! Secret finding definition
//!
//! Records detected secrets with masked values and severity classification.

/// Severity of a detected secret
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum Severity {
    Critical,
    High,
    Medium,
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

/// A single secret finding
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecretFinding {
    /// Type of secret detected (e.g. "AWS Access Key", "GitHub Token")
    pub secret_type: String,
    /// Masked representation: first 4 chars + *** + last 4 chars
    pub masked_value: String,
    /// Path to the file containing the secret
    pub file_path: String,
    /// Line number where the secret was found
    pub line_number: usize,
    /// Severity of the finding
    pub severity: Severity,
    /// Detection confidence (0.0 - 1.0)
    pub confidence: f64,
}

impl SecretFinding {
    /// Create a new secret finding with automatic masking
    pub fn new(
        secret_type: impl Into<String>,
        raw_value: &str,
        file_path: impl Into<String>,
        line_number: usize,
        severity: Severity,
        confidence: f64,
    ) -> Self {
        Self {
            secret_type: secret_type.into(),
            masked_value: Self::mask_value(raw_value),
            file_path: file_path.into(),
            line_number,
            severity,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }

    /// Mask a secret value: first 4 + *** + last 4
    fn mask_value(value: &str) -> String {
        let len = value.len();
        if len <= 8 {
            return "***".to_string();
        }
        let prefix = &value[..4];
        let suffix = &value[len - 4..];
        format!("{}***{}", prefix, suffix)
    }
}

impl std::fmt::Display for SecretFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} at {}:{} (confidence: {:.0}%)",
            self.severity,
            self.secret_type,
            self.file_path,
            self.line_number,
            self.confidence * 100.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_long_value() {
        let finding = SecretFinding::new(
            "AWS Key",
            "AKIAIOSFODNN7EXAMPLE",
            "config.rs",
            42,
            Severity::High,
            0.95,
        );
        assert_eq!(finding.masked_value, "AKIA***MPLE");
    }

    #[test]
    fn test_mask_short_value() {
        let finding = SecretFinding::new("Short", "abc", "test.rs", 1, Severity::Low, 0.5);
        assert_eq!(finding.masked_value, "***");
    }

    #[test]
    fn test_mask_exactly_8_chars() {
        let finding = SecretFinding::new("Eight", "12345678", "test.rs", 1, Severity::Low, 0.5);
        assert_eq!(finding.masked_value, "***");
    }

    #[test]
    fn test_mask_exactly_9_chars() {
        let finding = SecretFinding::new("Nine", "123456789", "test.rs", 1, Severity::Low, 0.5);
        assert_eq!(finding.masked_value, "1234***6789");
    }

    #[test]
    fn test_confidence_clamped() {
        let finding = SecretFinding::new(
            "Test",
            "secret1234value5678",
            "test.rs",
            1,
            Severity::Medium,
            1.5,
        );
        assert_eq!(finding.confidence, 1.0);

        let finding = SecretFinding::new(
            "Test",
            "secret1234value5678",
            "test.rs",
            1,
            Severity::Medium,
            -0.5,
        );
        assert_eq!(finding.confidence, 0.0);
    }

    #[test]
    fn test_display() {
        let finding = SecretFinding::new(
            "GitHub Token",
            "ghp_abcdef1234567890abcdef1234567890abcd",
            "deploy.sh",
            10,
            Severity::Critical,
            0.99,
        );
        let display = format!("{}", finding);
        assert!(display.contains("Critical"));
        assert!(display.contains("GitHub Token"));
        assert!(display.contains("deploy.sh:10"));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let finding = SecretFinding::new(
            "API Key",
            "sk-1234567890abcdef1234567890abcdef",
            "main.rs",
            5,
            Severity::High,
            0.9,
        );
        let json = serde_json::to_string(&finding).unwrap();
        let deserialized: SecretFinding = serde_json::from_str(&json).unwrap();
        assert_eq!(finding.secret_type, deserialized.secret_type);
        assert_eq!(finding.masked_value, deserialized.masked_value);
        assert_eq!(finding.severity, deserialized.severity);
    }
}
