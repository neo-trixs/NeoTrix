//! Secret scanner for detecting leaked credentials and sensitive values
//!
//! Pattern-based detection with validation to reduce false positives.

pub mod detector;
pub mod finding;
pub mod scanner;
pub mod validator;

// Re-exports
pub use detector::SecretDetector;
pub use finding::{SecretFinding, Severity};
pub use scanner::{ScanReport, SecretScanner};
pub use validator::{SecretValidator, ValidationMethod, ValidationResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_end_to_end_scan() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("config.env"),
            "aws_access_key_id = \"AKIAIOSFODNN7EXAMPLE\"\n\
             GITHUB_TOKEN=ghp_abcdefghijklmnopqrstuvwxyz123456\n\
             -----BEGIN RSA PRIVATE KEY-----\n\
             DATABASE_URL=postgres://admin:pass@db.example.com/prod\n\
             token = \"eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.abc123\"",
        )
        .unwrap();

        let scanner = SecretScanner::new();
        let report = scanner.scan_directory(dir.path(), &["env"]);

        assert_eq!(report.files_scanned, 1);
        assert!(report.secrets_found >= 4);
        assert!(report.by_type.len() >= 3);
        assert!(report.by_severity.contains_key("Critical"));
    }

    #[test]
    fn test_detector_validator_integration() {
        let detector = SecretDetector::new();
        let validator = SecretValidator::new();

        let content = "aws_access_key_id = \"AKIAIOSFODNN7EXAMPLE\"";
        let findings = detector.detect(content, "test.toml");

        for finding in &findings {
            let result = validator.validate(finding);
            // Real AWS key should pass validation
            if finding.secret_type == "AWS Access Key" {
                assert!(result.is_valid);
            }
        }
    }
}
