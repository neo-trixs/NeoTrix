//! Secret detection engine
//!
//! Pattern-based detector using regex for common secret types.

use regex::Regex;
use std::sync::LazyLock;

use super::finding::{SecretFinding, Severity};

/// A detected secret pattern with its metadata
struct SecretPattern {
    name: &'static str,
    regex: LazyLock<Regex>,
    severity: Severity,
    confidence: f64,
}

/// Compiled detection patterns
static PATTERNS: LazyLock<Vec<SecretPattern>> = LazyLock::new(|| {
    vec![
        SecretPattern {
            name: "AWS Access Key",
            regex: LazyLock::new(|| {
                Regex::new("(?i)(?:^|[^A-Za-z0-9/+=])(AKIA[0-9A-Z]{16})(?:[^A-Za-z0-9/+=]|$)")
                    .expect("invalid AWS key regex")
            }),
            severity: Severity::Critical,
            confidence: 0.95,
        },
        SecretPattern {
            name: "AWS Secret Key",
            regex: LazyLock::new(|| {
                Regex::new(
                    "(?i)(?:aws[_\\-]?secret[_\\-]?access[_\\-]?key|secret[_\\-]?key)\\s*[:=]\\s*['\"]?([A-Za-z0-9/+=]{40})['\"]?",
                )
                .expect("invalid AWS secret key regex")
            }),
            severity: Severity::Critical,
            confidence: 0.90,
        },
        SecretPattern {
            name: "GitHub Token",
            regex: LazyLock::new(|| {
                Regex::new(
                    "(?:ghp_[A-Za-z0-9]{36}|gho_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{82})",
                )
                .expect("invalid GitHub token regex")
            }),
            severity: Severity::Critical,
            confidence: 0.98,
        },
        SecretPattern {
            name: "OpenAI API Key",
            regex: LazyLock::new(|| {
                Regex::new("sk-[A-Za-z0-9]{20}T3BlbkFJ[A-Za-z0-9]{20}")
                    .expect("invalid OpenAI key regex")
            }),
            severity: Severity::Critical,
            confidence: 0.97,
        },
        SecretPattern {
            name: "Slack Token",
            regex: LazyLock::new(|| {
                Regex::new("xoxb-[0-9]{10,13}-[0-9]{10,13}-[a-zA-Z0-9]{24}")
                    .expect("invalid Slack token regex")
            }),
            severity: Severity::High,
            confidence: 0.95,
        },
        SecretPattern {
            name: "Slack Webhook",
            regex: LazyLock::new(|| {
                Regex::new(
                    "https://hooks\\.slack\\.com/services/T[A-Z0-9]{8}/B[A-Z0-9]{8}/[a-zA-Z0-9]{24}",
                )
                .expect("invalid Slack webhook regex")
            }),
            severity: Severity::High,
            confidence: 0.92,
        },
        SecretPattern {
            name: "RSA Private Key",
            regex: LazyLock::new(|| {
                Regex::new("-----BEGIN RSA PRIVATE KEY-----").expect("invalid RSA key regex")
            }),
            severity: Severity::Critical,
            confidence: 0.99,
        },
        SecretPattern {
            name: "OPENSSH Private Key",
            regex: LazyLock::new(|| {
                Regex::new("-----BEGIN OPENSSH PRIVATE KEY-----")
                    .expect("invalid OPENSSH key regex")
            }),
            severity: Severity::Critical,
            confidence: 0.99,
        },
        SecretPattern {
            name: "Generic Private Key",
            regex: LazyLock::new(|| {
                Regex::new("-----BEGIN (?:EC |DSA )?PRIVATE KEY-----")
                    .expect("invalid generic private key regex")
            }),
            severity: Severity::Critical,
            confidence: 0.98,
        },
        SecretPattern {
            name: "Database URL",
            regex: LazyLock::new(|| {
                Regex::new(
                    "(?:postgres|mysql|mongodb|redis|amqp|mssql)://[^\\s'\"<>{}|\\\\^`\\[\\]]{10,}",
                )
                .expect("invalid database URL regex")
            }),
            severity: Severity::High,
            confidence: 0.85,
        },
        SecretPattern {
            name: "JWT Token",
            regex: LazyLock::new(|| {
                Regex::new("eyJ[A-Za-z0-9_-]{10,}\\.eyJ[A-Za-z0-9_-]{10,}\\.[A-Za-z0-9_-]{10,}")
                    .expect("invalid JWT regex")
            }),
            severity: Severity::Medium,
            confidence: 0.80,
        },
        SecretPattern {
            name: "Generic API Key",
            regex: LazyLock::new(|| {
                Regex::new(
                    "(?i)(?:api[_\\-]?key|apikey|api[_\\-]?secret|secret[_\\-]?key|access[_\\-]?token|auth[_\\-]?token)\\s*[:=]\\s*['\"]([A-Za-z0-9_\\-]{20,60})['\"]",
                )
                .expect("invalid generic API key regex")
            }),
            severity: Severity::High,
            confidence: 0.75,
        },
    ]
});

/// Secret detection engine
pub struct SecretDetector;

impl SecretDetector {
    /// Create a new detector
    pub fn new() -> Self {
        Self
    }

    /// Detect secrets in content
    pub fn detect(&self, content: &str, file_path: &str) -> Vec<SecretFinding> {
        let mut findings = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let line_number = line_idx + 1;
            for pattern in PATTERNS.iter() {
                if pattern.regex.is_match(line) {
                    let captures = pattern.regex.captures(line);
                    let raw_value = captures
                        .and_then(|c| c.get(1))
                        .map(|m| m.as_str())
                        .unwrap_or_else(|| {
                            pattern
                                .regex
                                .find(line)
                                .map(|m| m.as_str())
                                .unwrap_or("unknown")
                        });

                    findings.push(SecretFinding::new(
                        pattern.name,
                        raw_value,
                        file_path,
                        line_number,
                        pattern.severity,
                        pattern.confidence,
                    ));
                }
            }
        }

        findings
    }

    /// Get the number of registered patterns
    pub fn pattern_count(&self) -> usize {
        PATTERNS.len()
    }
}

impl Default for SecretDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_aws_key() {
        let detector = SecretDetector::new();
        let content = "aws_access_key_id = \"AKIAIOSFODNN7EXAMPLE\"";
        let findings = detector.detect(content, "config.toml");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].secret_type, "AWS Access Key");
        assert_eq!(findings[0].severity, Severity::Critical);
        assert_eq!(findings[0].file_path, "config.toml");
        assert_eq!(findings[0].line_number, 1);
    }

    #[test]
    fn test_detect_github_token() {
        let detector = SecretDetector::new();
        let content = "GITHUB_TOKEN=ghp_abcdefghijklmnopqrstuvwxyz123456";
        let findings = detector.detect(content, ".env");
        assert!(findings.iter().any(|f| f.secret_type == "GitHub Token"));
    }

    #[test]
    fn test_detect_slack_token() {
        let detector = SecretDetector::new();
        let content = "SLACK_TOKEN=xoxb-1234567890123-1234567890123-abcdefghijklmnopqrstuvwx";
        let findings = detector.detect(content, "env.sh");
        assert!(findings.iter().any(|f| f.secret_type == "Slack Token"));
    }

    #[test]
    fn test_detect_private_key() {
        let detector = SecretDetector::new();
        let content = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...";
        let findings = detector.detect(content, "server.key");
        assert!(findings.iter().any(|f| f.secret_type == "RSA Private Key"));
    }

    #[test]
    fn test_detect_database_url() {
        let detector = SecretDetector::new();
        let content = "DATABASE_URL=postgres://user:pass@localhost:5432/mydb";
        let findings = detector.detect(content, "config.rs");
        assert!(findings.iter().any(|f| f.secret_type == "Database URL"));
    }

    #[test]
    fn test_detect_jwt_token() {
        let detector = SecretDetector::new();
        let content = "token = \"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U\"";
        let findings = detector.detect(content, "auth.rs");
        assert!(findings.iter().any(|f| f.secret_type == "JWT Token"));
    }

    #[test]
    fn test_detect_generic_api_key() {
        let detector = SecretDetector::new();
        let content = "api_key = \"sk-1234567890abcdef123456\"";
        let findings = detector.detect(content, "settings.yaml");
        assert!(findings.iter().any(|f| f.secret_type == "Generic API Key"));
    }

    #[test]
    fn test_no_false_positive_on_empty() {
        let detector = SecretDetector::new();
        let findings = detector.detect("", "empty.txt");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_multiple_secrets_in_file() {
        let detector = SecretDetector::new();
        let content = "\
aws_access_key_id = \"AKIAIOSFODNN7EXAMPLE\"
-----BEGIN RSA PRIVATE KEY-----
DATABASE_URL=postgres://admin:secret@db.example.com/prod
";
        let findings = detector.detect(content, "leaked.env");
        assert!(findings.len() >= 3);
    }

    #[test]
    fn test_line_numbers() {
        let detector = SecretDetector::new();
        let content = "line1\nline2\nline3\naws_access_key_id = \"AKIAIOSFODNN7EXAMPLE\"";
        let findings = detector.detect(content, "test.rs");
        assert_eq!(findings[0].line_number, 4);
    }

    #[test]
    fn test_pattern_count() {
        let detector = SecretDetector::new();
        assert!(detector.pattern_count() >= 10);
    }
}
