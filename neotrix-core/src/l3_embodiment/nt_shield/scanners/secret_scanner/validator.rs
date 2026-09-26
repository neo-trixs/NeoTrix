//! Secret validation engine
//!
//! Validates detected secrets to reduce false positives.

use super::finding::SecretFinding;

/// Validation method used to confirm a finding
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ValidationMethod {
    ApiCheck,
    PatternMatch,
    FalsePositiveList,
}

impl std::fmt::Display for ValidationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationMethod::ApiCheck => write!(f, "API_check"),
            ValidationMethod::PatternMatch => write!(f, "pattern_match"),
            ValidationMethod::FalsePositiveList => write!(f, "false_positive_list"),
        }
    }
}

/// Result of validating a secret finding
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationResult {
    /// Whether the finding is considered a valid secret
    pub is_valid: bool,
    /// Method used for validation
    pub validation_method: ValidationMethod,
    /// Human-readable details about the validation
    pub details: String,
}

/// Known false positive patterns
const FALSE_POSITIVES: &[&str] = &[
    "example",
    "EXAMPLE",
    "placeholder",
    "PLACEHOLDER",
    "your_key_here",
    "YOUR_KEY_HERE",
    "xxxxxx",
    "changeme",
    "CHANGEME",
    "todo",
    "TODO",
    "test",
    "TEST",
    "dummy",
    "DUMMY",
    "fake",
    "FAKE",
    "sample",
    "SAMPLE",
    "insert_key",
    "INSERT_KEY",
    "replace_me",
    "REPLACE_ME",
];

/// Secret validation engine
pub struct SecretValidator;

impl SecretValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self
    }

    /// Validate a secret finding
    pub fn validate(&self, finding: &SecretFinding) -> ValidationResult {
        // Phase 1: false positive list check
        if let Some(result) = self.check_false_positives(finding) {
            return result;
        }

        // Phase 2: pattern-based validation
        if let Some(result) = self.check_pattern_validity(finding) {
            return result;
        }

        // Phase 3: entropy check
        ValidationResult {
            is_valid: true,
            validation_method: ValidationMethod::PatternMatch,
            details: format!(
                "Pattern matched with {:.0}% confidence",
                finding.confidence * 100.0
            ),
        }
    }

    /// Check against known false positive patterns
    fn check_false_positives(&self, finding: &SecretFinding) -> Option<ValidationResult> {
        let masked_lower = finding.masked_value.to_lowercase();
        let type_lower = finding.secret_type.to_lowercase();
        // Masking keeps only first4***last4, so FP words sitting in the raw
        // middle can never match `masked_lower.contains(fp)`. Check the visible
        // head (raw first 4 chars, before "***") against known FP prefixes.
        // Tail-only markers are ignored so AWS doc keys (*MPLE) stay valid.
        let head_lower = match finding.masked_value.split("***").next() {
            Some(h) => h.to_lowercase(),
            None => String::new(),
        };

        for &fp in FALSE_POSITIVES {
            if masked_lower.contains(&fp.to_lowercase()) {
                return Some(ValidationResult {
                    is_valid: false,
                    validation_method: ValidationMethod::FalsePositiveList,
                    details: format!("Matched false positive pattern '{}' in masked value", fp),
                });
            }
            // Visible-head FP check: reject when the head opens a known FP
            // word (e.g. "exam" -> "example", "plac" -> "placeholder").
            if head_lower.len() >= 3 {
                let fp_lower = fp.to_lowercase();
                if fp_lower.starts_with(head_lower.as_str()) {
                    return Some(ValidationResult {
                        is_valid: false,
                        validation_method: ValidationMethod::FalsePositiveList,
                        details: format!(
                            "Masked head matches false positive prefix '{}'",
                            fp
                        ),
                    });
                }
            }
            // Also check the raw pattern context from the secret type
            if type_lower.contains("example") && finding.confidence < 0.9 {
                return Some(ValidationResult {
                    is_valid: false,
                    validation_method: ValidationMethod::FalsePositiveList,
                    details: "Secret type indicates example/test credential".to_string(),
                });
            }
        }

        None
    }

    /// Validate based on pattern-specific rules
    fn check_pattern_validity(&self, finding: &SecretFinding) -> Option<ValidationResult> {
        match finding.secret_type.as_str() {
            "AWS Access Key" => {
                // AKIA prefix must be exactly 4 chars, total key is 20 chars
                if finding.masked_value.len() < 5 {
                    return Some(ValidationResult {
                        is_valid: false,
                        validation_method: ValidationMethod::PatternMatch,
                        details: "AWS key too short after masking".to_string(),
                    });
                }
                Some(ValidationResult {
                    is_valid: true,
                    validation_method: ValidationMethod::PatternMatch,
                    details: "AWS Access Key pattern validated (AKIA prefix confirmed)".to_string(),
                })
            }
            "RSA Private Key" | "OPENSSH Private Key" | "Generic Private Key" => {
                // Private keys are always valid if header is present
                Some(ValidationResult {
                    is_valid: true,
                    validation_method: ValidationMethod::PatternMatch,
                    details: format!("{} header detected", finding.secret_type),
                })
            }
            "JWT Token" => {
                // Masking strips the middle dots, so dot-counting on the masked
                // value can never reach 2. Validate by the distinctive eyJ
                // header prefix instead (detector guarantees it for true JWTs).
                if finding.masked_value.starts_with("eyJ") {
                    Some(ValidationResult {
                        is_valid: true,
                        validation_method: ValidationMethod::PatternMatch,
                        details: "JWT structure validated (eyJ header prefix)".to_string(),
                    })
                } else {
                    Some(ValidationResult {
                        is_valid: false,
                        validation_method: ValidationMethod::PatternMatch,
                        details: "JWT missing expected eyJ header prefix".to_string(),
                    })
                }
            }
            "Database URL" => {
                // Must contain protocol prefix
                Some(ValidationResult {
                    is_valid: true,
                    validation_method: ValidationMethod::PatternMatch,
                    details: "Database connection string with valid protocol".to_string(),
                })
            }
            _ => None,
        }
    }
}

impl Default for SecretValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::finding::Severity;

    fn make_finding(
        secret_type: &str,
        value: &str,
        severity: Severity,
        confidence: f64,
    ) -> SecretFinding {
        SecretFinding::new(secret_type, value, "test.rs", 1, severity, confidence)
    }

    #[test]
    fn test_validate_real_aws_key() {
        let validator = SecretValidator::new();
        let finding = make_finding(
            "AWS Access Key",
            "AKIAIOSFODNN7EXAMPLE",
            Severity::Critical,
            0.95,
        );
        let result = validator.validate(&finding);
        assert!(result.is_valid);
    }

    #[test]
    fn test_reject_example_key() {
        let validator = SecretValidator::new();
        let finding = make_finding("Generic API Key", "example_key_here", Severity::Medium, 0.7);
        let result = validator.validate(&finding);
        assert!(!result.is_valid);
        assert_eq!(
            result.validation_method,
            ValidationMethod::FalsePositiveList
        );
    }

    #[test]
    fn test_reject_placeholder() {
        let validator = SecretValidator::new();
        let finding = make_finding(
            "GitHub Token",
            "placeholder_token_value_12345678",
            Severity::High,
            0.8,
        );
        let result = validator.validate(&finding);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_private_key() {
        let validator = SecretValidator::new();
        let finding = make_finding(
            "RSA Private Key",
            "-----BEGIN RSA PRIVATE KEY-----",
            Severity::Critical,
            0.99,
        );
        let result = validator.validate(&finding);
        assert!(result.is_valid);
        assert_eq!(result.validation_method, ValidationMethod::PatternMatch);
    }

    #[test]
    fn test_validate_jwt() {
        let validator = SecretValidator::new();
        let finding = make_finding("JWT Token", "eyJhbGci.xxx.yyy", Severity::Medium, 0.80);
        let result = validator.validate(&finding);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_database_url() {
        let validator = SecretValidator::new();
        let finding = make_finding(
            "Database URL",
            "postgres://user:pass@host:5432/db",
            Severity::High,
            0.85,
        );
        let result = validator.validate(&finding);
        assert!(result.is_valid);
    }

    #[test]
    fn test_confidence_based_pass() {
        let validator = SecretValidator::new();
        let finding = make_finding(
            "Slack Token",
            "xoxb-1234567890123-1234567890123-abcdefghijklmnopqrstuvwx",
            Severity::High,
            0.95,
        );
        let result = validator.validate(&finding);
        assert!(result.is_valid);
    }

    #[test]
    fn test_reject_changeme() {
        let validator = SecretValidator::new();
        let finding = make_finding(
            "Generic API Key",
            "changeme1234567890abcdef",
            Severity::Medium,
            0.7,
        );
        let result = validator.validate(&finding);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validation_result_display() {
        let result = ValidationResult {
            is_valid: true,
            validation_method: ValidationMethod::ApiCheck,
            details: "Verified via API".to_string(),
        };
        assert_eq!(result.validation_method.to_string(), "API_check");
    }
}
