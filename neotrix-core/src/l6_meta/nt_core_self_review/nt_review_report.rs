//! Self-review report — SelfReviewReport/BlastRadius assembly plus count helper.
//! Pure move from `mod.rs` (God-file split) — zero behavior change.

use std::collections::HashMap;
use std::path::Path;

use super::nt_review_types::BlastRadiusReport;
use super::nt_review_types::BlastRisk;
use super::nt_review_types::SelfReviewGate;
use super::nt_review_types::SelfReviewReport;
use super::scanners::cached_rs_files;
use neotrix_types::shared::Severity;

// ─── moved: mod.rs:298-316 report ───
impl SelfReviewGate {
    pub fn report(&self) -> SelfReviewReport {
        let mut failed = 0usize;
        let mut warnings = 0usize;
        for f in &self.findings {
            match f.severity {
                Severity::Error => failed += 1,
                Severity::Warning => warnings += 1,
                Severity::Info => {}
                _ => {}
            }
        }
        let passed = self.findings.len().saturating_sub(failed + warnings);
        SelfReviewReport {
            findings: self.findings.clone(),
            passed,
            failed,
            warnings,
        }
    }
}

// ─── moved: mod.rs:318-350 blast_radius ───
impl SelfReviewGate {
    /// Compute blast-radius from current findings.
    pub fn blast_radius(&self) -> BlastRadiusReport {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let files_scanned = count_rs_files(&src_dir);
        let mut affected: Vec<String> = self.findings.iter().map(|f| f.file.clone()).collect();
        affected.sort();
        affected.dedup();
        let affected_files = affected.len();
        let module_crossings = self
            .findings
            .iter()
            .filter(|f| f.category == "layer_violation" || f.category == "cross_file_impact")
            .count();
        let risk = if self.findings.iter().any(|f| {
            f.severity == Severity::Error
                && (f.category == "panic_audit" || f.category == "layer_violation")
        }) {
            BlastRisk::Critical
        } else if self.findings.len() > 10 {
            BlastRisk::High
        } else if self.findings.len() > 3 {
            BlastRisk::Medium
        } else {
            BlastRisk::Low
        };
        BlastRadiusReport {
            files_scanned,
            affected_files,
            module_crossings,
            risk,
            domain_density: HashMap::new(),
        }
    }
}

// ─── moved: mod.rs:1650-1652 count_rs_files ───
fn count_rs_files(dir: &Path) -> usize {
    cached_rs_files(dir).len()
}
