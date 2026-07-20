
#[derive(Debug, Clone)]
pub struct AuditorFinding {
    pub file: String,
    pub category: String,
    pub severity: f64,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct MetaAuditor {
    pub total_audits: u64,
    pub total_findings: u64,
    pub total_false_positives: u64,
    pub findings_history: Vec<AuditorFinding>,
    pub false_positive_log: Vec<String>,
}

impl MetaAuditor {
    pub fn new() -> Self {
        Self {
            total_audits: 0,
            total_findings: 0,
            total_false_positives: 0,
            findings_history: Vec::new(),
            false_positive_log: Vec::new(),
        }
    }

    pub fn record_finding(&mut self, finding: AuditorFinding) {
        self.total_findings += 1;
        self.findings_history.push(finding);
    }

    pub fn record_false_positive(&mut self, description: &str) {
        self.total_false_positives += 1;
        self.false_positive_log.push(description.to_string());
    }

    pub fn accuracy(&self) -> f64 {
        if self.total_findings == 0 {
            return 1.0;
        }
        (self.total_findings.saturating_sub(self.total_false_positives)) as f64
            / self.total_findings as f64
    }

    pub fn false_positive_rate(&self) -> f64 {
        if self.total_findings == 0 {
            return 0.0;
        }
        self.total_false_positives as f64 / self.total_findings as f64
    }

    pub fn audit_report(&self) -> String {
        format!(
            "MetaAuditor: {} audits, {} findings ({} FP, {:.1}% accuracy)",
            self.total_audits,
            self.total_findings,
            self.total_false_positives,
            self.accuracy() * 100.0
        )
    }
}

impl Default for MetaAuditor {
    fn default() -> Self { Self::new() }
}

impl crate::core::nt_core_self_test::SelfTest for MetaAuditor {
    fn name(&self) -> &str { "meta_auditor" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if self.accuracy() < 0.0 || self.accuracy() > 1.0 {
            failures.push("meta_auditor: accuracy out of range".into());
        }
        if self.false_positive_rate() < 0.0 || self.false_positive_rate() > 1.0 {
            failures.push("meta_auditor: false_positive_rate out of range".into());
        }
        if failures.is_empty() { Ok(()) } else { Err(failures) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self_test::SelfTest;

    #[test]
    fn test_fresh_auditor() {
        let a = MetaAuditor::new();
        assert!((a.accuracy() - 1.0).abs() < 1e-9);
        assert!((a.false_positive_rate() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_findings_and_fps() {
        let mut a = MetaAuditor::new();
        a.record_finding(AuditorFinding {
            file: "test.rs".into(),
            category: "unsafe".into(),
            severity: 0.8,
            description: "found unsafe block".into(),
        });
        a.record_finding(AuditorFinding {
            file: "test2.rs".into(),
            category: "panic".into(),
            severity: 0.5,
            description: "found panic".into(),
        });
        a.record_false_positive("first finding was wrong");
        assert!((a.accuracy() - 0.5).abs() < 1e-9);
        assert!((a.false_positive_rate() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_report_string() {
        let a = MetaAuditor::new();
        let r = a.audit_report();
        assert!(r.contains("MetaAuditor"));
    }

    #[test]
    fn test_self_test() {
        let a = MetaAuditor::new();
        assert!(a.self_test().is_ok());
    }
}
