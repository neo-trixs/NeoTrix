//! Secret scanner orchestrator
//!
//! Scans files and directories for secrets, producing aggregated reports.

use std::collections::HashMap;
use std::path::Path;

use super::detector::SecretDetector;
use super::finding::SecretFinding;
use super::validator::{SecretValidator, ValidationResult};

/// Aggregated scan report
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScanReport {
    /// Number of files scanned
    pub files_scanned: usize,
    /// Total number of secrets found
    pub secrets_found: usize,
    /// Findings grouped by secret type
    pub by_type: HashMap<String, usize>,
    /// Findings grouped by severity
    pub by_severity: HashMap<String, usize>,
    /// Individual findings
    pub findings: Vec<SecretFinding>,
    /// Validation results keyed by file:line
    pub validations: HashMap<String, ValidationResult>,
}

impl ScanReport {
    /// Create an empty report
    fn empty() -> Self {
        Self {
            files_scanned: 0,
            secrets_found: 0,
            by_type: HashMap::new(),
            by_severity: HashMap::new(),
            findings: Vec::new(),
            validations: HashMap::new(),
        }
    }

    /// Merge another report into this one
    fn merge(&mut self, other: ScanReport) {
        self.files_scanned += other.files_scanned;
        self.secrets_found += other.secrets_found;
        for (k, v) in other.by_type {
            *self.by_type.entry(k).or_insert(0) += v;
        }
        for (k, v) in other.by_severity {
            *self.by_severity.entry(k).or_insert(0) += v;
        }
        self.findings.extend(other.findings);
        self.validations.extend(other.validations);
    }
}

/// Secret scanner that orchestrates detection and validation
pub struct SecretScanner {
    detector: SecretDetector,
    validator: SecretValidator,
}

impl SecretScanner {
    /// Create a new scanner with default detector and validator
    pub fn new() -> Self {
        Self {
            detector: SecretDetector::new(),
            validator: SecretValidator::new(),
        }
    }

    /// Create a scanner with custom detector
    pub fn with_detector(detector: SecretDetector) -> Self {
        Self {
            detector,
            validator: SecretValidator::new(),
        }
    }

    /// Scan a single file for secrets
    pub fn scan_file(&self, path: impl AsRef<Path>) -> Vec<SecretFinding> {
        let path = path.as_ref();
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        let file_path = path.to_string_lossy().to_string();
        self.detector.detect(&content, &file_path)
    }

    /// Scan a directory for secrets in files matching the given extensions
    pub fn scan_directory(&self, path: impl AsRef<Path>, extensions: &[&str]) -> ScanReport {
        let path = path.as_ref();
        let mut report = ScanReport::empty();

        let walker = match std::fs::read_dir(path) {
            Ok(w) => w,
            Err(_) => return report,
        };

        for entry in walker.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                // Recurse into subdirectories
                let sub_report = self.scan_directory(&entry_path, extensions);
                report.merge(sub_report);
                continue;
            }

            if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
                if extensions.iter().any(|e| *e == ext) {
                    report.files_scanned += 1;
                    let findings = self.scan_file(&entry_path);
                    let _file_key = entry_path.to_string_lossy().to_string();

                    for finding in findings {
                        // Validate each finding
                        let validation = self.validator.validate(&finding);
                        let val_key = format!("{}:{}", finding.file_path, finding.line_number);
                        report.validations.insert(val_key, validation);

                        // Aggregate counts
                        *report
                            .by_type
                            .entry(finding.secret_type.clone())
                            .or_insert(0) += 1;
                        *report
                            .by_severity
                            .entry(finding.severity.to_string())
                            .or_insert(0) += 1;

                        report.findings.push(finding);
                    }
                }
            }
        }

        report.secrets_found = report.findings.len();
        report
    }
}

impl Default for SecretScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_scan_file_with_secret() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "aws_access_key_id = \"AKIAIOSFODNN7EXAMPLE\"").unwrap();

        let scanner = SecretScanner::new();
        let findings = scanner.scan_file(tmp.path());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].secret_type, "AWS Access Key");
    }

    #[test]
    fn test_scan_file_clean() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "let x = 42;").unwrap();

        let scanner = SecretScanner::new();
        let findings = scanner.scan_file(tmp.path());
        assert!(findings.is_empty());
    }

    #[test]
    fn test_scan_directory() {
        let dir = tempfile::tempdir().unwrap();
        let file1 = dir.path().join("config.toml");
        let file2 = dir.path().join("main.rs");

        std::fs::write(&file1, "aws_access_key_id = \"AKIAIOSFODNN7EXAMPLE\"").unwrap();
        std::fs::write(&file2, "fn main() { println!(\"hello\"); }").unwrap();

        let scanner = SecretScanner::new();
        let report = scanner.scan_directory(dir.path(), &["toml", "rs"]);

        assert_eq!(report.files_scanned, 2);
        assert_eq!(report.secrets_found, 1);
        assert_eq!(report.findings[0].file_path, file1.to_string_lossy());
        assert!(report.by_type.contains_key("AWS Access Key"));
    }

    #[test]
    fn test_scan_directory_nonexistent() {
        let scanner = SecretScanner::new();
        let report = scanner.scan_directory("/nonexistent/path", &["rs"]);
        assert_eq!(report.files_scanned, 0);
        assert_eq!(report.secrets_found, 0);
    }

    #[test]
    fn test_scan_directory_with_subdirs() {
        let dir = tempfile::tempdir().unwrap();
        let subdir = dir.path().join("sub");
        std::fs::create_dir(&subdir).unwrap();

        std::fs::write(
            subdir.join("keys.env"),
            "GITHUB_TOKEN=ghp_abcdefghijklmnopqrstuvwxyz123456",
        )
        .unwrap();

        let scanner = SecretScanner::new();
        let report = scanner.scan_directory(dir.path(), &["env"]);

        assert_eq!(report.files_scanned, 1);
        assert_eq!(report.secrets_found, 1);
    }

    #[test]
    fn test_report_aggregation() {
        let dir = tempfile::tempdir().unwrap();

        // File with multiple secrets
        std::fs::write(
            dir.path().join("multi.txt"),
            "aws_access_key_id = \"AKIAIOSFODNN7EXAMPLE\"\n\
             -----BEGIN RSA PRIVATE KEY-----\n\
             DATABASE_URL=postgres://admin:pass@localhost/db",
        )
        .unwrap();

        // Second file with same type
        std::fs::write(
            dir.path().join("other.txt"),
            "aws_access_key_id = \"AKIAI44QH8DHBEXAMPLE\"",
        )
        .unwrap();

        let scanner = SecretScanner::new();
        let report = scanner.scan_directory(dir.path(), &["txt"]);

        assert_eq!(report.files_scanned, 2);
        assert!(report.secrets_found >= 3);
        // AWS type should appear twice
        assert_eq!(report.by_type.get("AWS Access Key"), Some(&2));
    }

    #[test]
    fn test_validation_in_report() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("test.txt"),
            "key = \"changeme1234567890abcdef\"",
        )
        .unwrap();

        let scanner = SecretScanner::new();
        let report = scanner.scan_directory(dir.path(), &["txt"]);

        // Should have validation results
        assert!(!report.validations.is_empty());
    }

    #[test]
    fn test_scan_file_nonexistent() {
        let scanner = SecretScanner::new();
        let findings = scanner.scan_file("/nonexistent/file.txt");
        assert!(findings.is_empty());
    }
}
