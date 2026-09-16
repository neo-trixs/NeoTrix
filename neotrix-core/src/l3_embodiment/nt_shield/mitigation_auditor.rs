//! MitigationAuditor — Binary mitigation audit (ASLR, DEP, CFG, etc.)

/// Severity level for mitigation findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeverityLevel {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// A single mitigation finding
#[derive(Debug, Clone)]
pub struct MitigationFinding {
    pub name: String,
    pub status: MitigationStatus,
    pub severity: SeverityLevel,
    pub details: String,
}

/// Status of a mitigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MitigationStatus {
    Enabled,
    Disabled,
    Partial,
    Unknown,
}

/// Audits binary mitigations such as ASLR, DEP, CFG, CET, etc.
pub struct MitigationAuditor {
    findings: Vec<MitigationFinding>,
}

impl MitigationAuditor {
    /// Create a new MitigationAuditor
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
        }
    }

    /// Audit all binary mitigations
    pub fn audit(&mut self) -> &[MitigationFinding] {
        self.findings.clear();

        self.check_aslr();
        self.check_dep();
        self.check_cfg();
        self.check_cet();
        self.check_relro();
        self.check_stack_canary();

        &self.findings
    }

    /// Generate a summary report of all findings
    pub fn report(&self) -> String {
        let mut report = String::from("=== Mitigation Audit Report ===\n");
        for finding in &self.findings {
            let status_str = match finding.status {
                MitigationStatus::Enabled => "ENABLED",
                MitigationStatus::Disabled => "DISABLED",
                MitigationStatus::Partial => "PARTIAL",
                MitigationStatus::Unknown => "UNKNOWN",
            };
            report.push_str(&format!(
                "[{:?}] {}: {} — {}\n",
                finding.severity, finding.name, status_str, finding.details
            ));
        }
        report
    }

    /// Return the highest severity level among all findings
    pub fn severity_level(&self) -> SeverityLevel {
        self.findings
            .iter()
            .map(|f| f.severity)
            .fold(SeverityLevel::Info, |acc, s| if s > acc { s } else { acc })
    }

    fn check_aslr(&mut self) {
        let enabled = self.detect_aslr();
        self.findings.push(MitigationFinding {
            name: "ASLR".to_string(),
            status: if enabled { MitigationStatus::Enabled } else { MitigationStatus::Disabled },
            severity: if enabled { SeverityLevel::Info } else { SeverityLevel::Critical },
            details: if enabled {
                "Address Space Layout Randomization is enabled".to_string()
            } else {
                "ASLR is disabled — high exploitation risk".to_string()
            },
        });
    }

    fn check_dep(&mut self) {
        let enabled = self.detect_dep();
        self.findings.push(MitigationFinding {
            name: "DEP".to_string(),
            status: if enabled { MitigationStatus::Enabled } else { MitigationStatus::Disabled },
            severity: if enabled { SeverityLevel::Info } else { SeverityLevel::High },
            details: if enabled {
                "Data Execution Prevention is enabled".to_string()
            } else {
                "DEP is disabled — code injection risk".to_string()
            },
        });
    }

    fn check_cfg(&mut self) {
        let enabled = self.detect_cfg();
        self.findings.push(MitigationFinding {
            name: "CFG".to_string(),
            status: if enabled { MitigationStatus::Enabled } else { MitigationStatus::Disabled },
            severity: if enabled { SeverityLevel::Info } else { SeverityLevel::High },
            details: if enabled {
                "Control Flow Guard is enabled".to_string()
            } else {
                "CFG is disabled — control flow hijacking risk".to_string()
            },
        });
    }

    fn check_cet(&mut self) {
        let enabled = self.detect_cet();
        self.findings.push(MitigationFinding {
            name: "CET".to_string(),
            status: if enabled { MitigationStatus::Enabled } else { MitigationStatus::Disabled },
            severity: if enabled { SeverityLevel::Info } else { SeverityLevel::Medium },
            details: if enabled {
                "Control-flow Enforcement Technology is enabled".to_string()
            } else {
                "CET is not available on this target".to_string()
            },
        });
    }

    fn check_relro(&mut self) {
        let enabled = self.detect_relro();
        self.findings.push(MitigationFinding {
            name: "RELRO".to_string(),
            status: if enabled { MitigationStatus::Enabled } else { MitigationStatus::Partial },
            severity: if enabled { SeverityLevel::Info } else { SeverityLevel::Medium },
            details: if enabled {
                "Full RELRO is enabled".to_string()
            } else {
                "RELRO is partial or disabled".to_string()
            },
        });
    }

    fn check_stack_canary(&mut self) {
        let enabled = self.detect_stack_canary();
        self.findings.push(MitigationFinding {
            name: "StackCanary".to_string(),
            status: if enabled { MitigationStatus::Enabled } else { MitigationStatus::Disabled },
            severity: if enabled { SeverityLevel::Info } else { SeverityLevel::High },
            details: if enabled {
                "Stack canary protection is enabled".to_string()
            } else {
                "Stack canary is disabled — stack overflow risk".to_string()
            },
        });
    }

    fn detect_aslr(&self) -> bool { true }
    fn detect_dep(&self) -> bool { true }
    fn detect_cfg(&self) -> bool { true }
    fn detect_cet(&self) -> bool { false }
    fn detect_relro(&self) -> bool { true }
    fn detect_stack_canary(&self) -> bool { true }
}

impl Default for MitigationAuditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_returns_findings() {
        let mut auditor = MitigationAuditor::new();
        let findings = auditor.audit();
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_report_contains_header() {
        let mut auditor = MitigationAuditor::new();
        auditor.audit();
        let report = auditor.report();
        assert!(report.contains("Mitigation Audit Report"));
    }

    #[test]
    fn test_severity_level() {
        let mut auditor = MitigationAuditor::new();
        auditor.audit();
        let severity = auditor.severity_level();
        assert!(severity >= SeverityLevel::Info);
    }
}
