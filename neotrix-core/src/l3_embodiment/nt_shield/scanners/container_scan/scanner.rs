use std::collections::HashMap;

use super::misconfig::{DockerfileChecker, MisconfigCheck};
use super::sbom::{SbomEntry, SbomGenerator};
use super::vulnerability::{Severity, Vulnerability};

/// Aggregated scan report for a container image
#[derive(Debug, Clone)]
pub struct ScanReport {
    pub image_name: String,
    pub vulnerabilities: Vec<Vulnerability>,
    pub misconfigs: Vec<MisconfigCheck>,
    pub sbom_entries: Vec<SbomEntry>,
    pub risk_score: f64,
}

impl ScanReport {
    /// Compute a 0.0 - 10.0 risk score from findings
    pub fn compute_risk_score(vulns: &[Vulnerability], misconfigs: &[MisconfigCheck]) -> f64 {
        let mut score = 0.0_f64;

        for v in vulns {
            score += match v.severity {
                Severity::Critical => 3.0,
                Severity::High => 2.0,
                Severity::Medium => 1.0,
                Severity::Low => 0.3,
                Severity::Info => 0.05,
            };
        }

        for m in misconfigs {
            score += match m.severity {
                Severity::Critical => 2.5,
                Severity::High => 1.5,
                Severity::Medium => 0.8,
                Severity::Low => 0.2,
                Severity::Info => 0.05,
            };
        }

        (score * 10.0).round() / 10.0
    }
}

/// Container security scanner
pub struct ContainerScanner;

impl ContainerScanner {
    pub fn new() -> Self {
        Self
    }

    /// Simulated image scan — returns deterministic findings based on image name
    pub fn scan_image(image_name: &str) -> ScanReport {
        let vulnerabilities = Self::simulated_vulns(image_name);
        let risk_score = ScanReport::compute_risk_score(&vulnerabilities, &[]);
        ScanReport {
            image_name: image_name.to_string(),
            vulnerabilities,
            misconfigs: Vec::new(),
            sbom_entries: Vec::new(),
            risk_score,
        }
    }

    /// Scan a Dockerfile for misconfigurations
    pub fn scan_dockerfile(path: &str) -> Vec<MisconfigCheck> {
        DockerfileChecker::check(path)
    }

    /// Scan a Cargo.lock for known vulnerable packages (simulated)
    pub fn scan_lockfile(path: &str) -> Vec<Vulnerability> {
        let entries = SbomGenerator::generate_from_lock(path);
        let mut vulns = Vec::new();

        for entry in &entries {
            if let Some(v) = Self::check_known_vuln(&entry.package_name, &entry.version) {
                vulns.push(v);
            }
        }

        vulns
    }

    /// Produce a full scan report combining lockfile + Dockerfile analysis
    pub fn full_scan(
        image_name: &str,
        dockerfile_path: Option<&str>,
        lockfile_path: Option<&str>,
    ) -> ScanReport {
        let mut report = Self::scan_image(image_name);

        if let Some(df_path) = dockerfile_path {
            report.misconfigs = Self::scan_dockerfile(df_path);
        }

        if let Some(lf_path) = lockfile_path {
            report.vulnerabilities.extend(Self::scan_lockfile(lf_path));
        }

        report.risk_score =
            ScanReport::compute_risk_score(&report.vulnerabilities, &report.misconfigs);

        report
    }

    fn simulated_vulns(image_name: &str) -> Vec<Vulnerability> {
        let mut vulns = Vec::new();
        let lower = image_name.to_lowercase();

        if lower.contains("openssl") || lower.contains("ssl") {
            vulns.push(Vulnerability {
                id: "CVE-2024-0727".to_string(),
                severity: Severity::High,
                package_name: "openssl".to_string(),
                installed_version: "3.0.12".to_string(),
                fixed_version: Some("3.0.13".to_string()),
                description: "NULL dereference when processing PKCS12 data".to_string(),
                references: vec!["https://nvd.nist.gov/vuln/detail/CVE-2024-0727".to_string()],
            });
        }

        if lower.contains("python") || lower.contains("pip") {
            vulns.push(Vulnerability {
                id: "CVE-2024-6232".to_string(),
                severity: Severity::Critical,
                package_name: "python".to_string(),
                installed_version: "3.12.0".to_string(),
                fixed_version: Some("3.12.5".to_string()),
                description: "ReDoS in tarfile header parsing".to_string(),
                references: vec!["https://nvd.nist.gov/vuln/detail/CVE-2024-6232".to_string()],
            });
        }

        if lower.contains("node") || lower.contains("npm") {
            vulns.push(Vulnerability {
                id: "CVE-2024-22020".to_string(),
                severity: Severity::High,
                package_name: "node".to_string(),
                installed_version: "22.0.0".to_string(),
                fixed_version: Some("22.12.0".to_string()),
                description: "Network import restriction bypass".to_string(),
                references: vec!["https://nvd.nist.gov/vuln/detail/CVE-2024-22020".to_string()],
            });
        }

        vulns
    }

    fn check_known_vuln(package_name: &str, version: &str) -> Option<Vulnerability> {
        let vuln_db: HashMap<(&str, &str), (&str, Severity, &str, &str)> = HashMap::from([
            (
                ("openssl", "3.0.12"),
                (
                    "CVE-2024-0727",
                    Severity::High,
                    "NULL dereference in PKCS12",
                    "3.0.13",
                ),
            ),
            (
                ("tokio", "1.33.0"),
                (
                    "CVE-2023-22466",
                    Severity::Medium,
                    "Race condition in notify",
                    "1.33.1",
                ),
            ),
            (
                ("regex", "1.5.0"),
                (
                    "CVE-2022-24713",
                    Severity::High,
                    "Regex denial of service",
                    "1.5.5",
                ),
            ),
        ]);

        let key = (package_name, version);
        if let Some((id, severity, desc, fixed)) = vuln_db.get(&key) {
            Some(Vulnerability {
                id: id.to_string(),
                severity: *severity,
                package_name: package_name.to_string(),
                installed_version: version.to_string(),
                fixed_version: Some(fixed.to_string()),
                description: desc.to_string(),
                references: vec![],
            })
        } else {
            None
        }
    }
}

impl Default for ContainerScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_image_known() {
        let report = ContainerScanner::scan_image("python:3.12-slim");
        assert!(!report.vulnerabilities.is_empty());
        assert!(report.risk_score > 0.0);
    }

    #[test]
    fn test_scan_image_unknown() {
        let report = ContainerScanner::scan_image("alpine:3.19");
        assert!(report.vulnerabilities.is_empty());
        assert_eq!(report.risk_score, 0.0);
    }

    #[test]
    fn test_risk_score_calculation() {
        let vulns = vec![
            Vulnerability {
                id: "CVE-001".to_string(),
                severity: Severity::Critical,
                package_name: "a".to_string(),
                installed_version: "1.0".to_string(),
                fixed_version: None,
                description: String::new(),
                references: vec![],
            },
            Vulnerability {
                id: "CVE-002".to_string(),
                severity: Severity::Low,
                package_name: "b".to_string(),
                installed_version: "1.0".to_string(),
                fixed_version: None,
                description: String::new(),
                references: vec![],
            },
        ];
        let score = ScanReport::compute_risk_score(&vulns, &[]);
        assert!(score >= 3.3 && score <= 3.4);
    }

    #[test]
    fn test_full_scan_no_files() {
        let report = ContainerScanner::full_scan("nginx:latest", None, None);
        assert_eq!(report.image_name, "nginx:latest");
        assert!(report.misconfigs.is_empty());
        assert!(report.sbom_entries.is_empty());
    }
}
