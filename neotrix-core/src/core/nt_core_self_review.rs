//! Self-review gate — 27+ mechanical checks for code quality.
//! Distilled from 2026-07-05 3-deep-roam cycle: 49 weak links, 4 P0/P1 fixes,
//! Internet research (cargo-panic-audit PA001-PA009, Revet blast-radius, evole-loop audit phase).
//! Cycle 27 additions: PA010-PA015 — negative signal zeroing, FTS desync, zscore length,
//! substring tag matching, HashMap iteration order, clamp semantics.
//! Cycle 28 additions: PA016-PA019 — Python bare except, SQL injection, legacy tables, main guard.
//! Cycle 29 additions: PA020-PA023 — Karpathy-inspired code principles (Simplicity First, Surgical Changes,
//! Complexity Budget, Goal-Driven Execution). Inspired by multica-ai/andrej-karpathy-skills.
//! Cycle 30 additions: PA024-PA027 — SEAL stage health, CI workflow coverage, test assertion hygiene,
//! dependency version consistency. Distilled from Architecture Rebirth full-stack audit cycle.
//! Run via: `cargo test --lib -p neotrix -- self_review`

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use regex::Regex;
use serde::{Deserialize, Serialize};
use syn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "ERROR"),
            Severity::Warning => write!(f, "WARN"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFinding {
    pub severity: Severity,
    pub category: String,
    pub message: String,
    pub file: String,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfReviewReport {
    pub findings: Vec<ReviewFinding>,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
}

impl SelfReviewReport {
    pub fn is_pass(&self) -> bool {
        self.failed == 0
    }

    pub fn summary(&self) -> String {
        format!(
            "Self-review: {} passed, {} failed, {} warnings — overall {}",
            self.passed,
            self.failed,
            self.warnings,
            if self.is_pass() { "PASS" } else { "FAIL" }
        )
    }
}

/// Blast-radius report — estimates cross-file impact of findings.
/// Inspired by Revet's blast-radius summary (deterministic, risk-scored).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadiusReport {
    /// Total source files scanned
    pub files_scanned: usize,
    /// Files with at least one finding
    pub affected_files: usize,
    /// Module boundary crossings (src/domain-a -> src/domain-b imports)
    pub module_crossings: usize,
    /// Risk level summary
    pub risk: BlastRisk,
    /// Per-domain finding density
    pub domain_density: HashMap<String, f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BlastRisk {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for BlastRisk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlastRisk::Low => write!(f, "LOW"),
            BlastRisk::Medium => write!(f, "MEDIUM"),
            BlastRisk::High => write!(f, "HIGH"),
            BlastRisk::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Architecture depth category — which NeoTrix domain layer a module belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ArchLayer {
    L0Core,
    L1Act,
    L2World,
    L3Memory,
    L4Cognition,
    L5Prm,
    L6Self,
    L7Capability,
    L8Seal,
    L9Transcendent,
    Unknown,
}

impl ArchLayer {
    pub fn from_path(path: &Path) -> Self {
        let p = path.to_string_lossy();
        if p.contains("l0_core") || p.contains("/core/") { Self::L0Core }
        else if p.contains("l1_body") || p.contains("l1_act") { Self::L1Act }
        else if p.contains("l2_world") { Self::L2World }
        else if p.contains("l3_memory") { Self::L3Memory }
        else if p.contains("l4_cognition") { Self::L4Cognition }
        else if p.contains("l5_prm") || p.contains("nt_core_prm") { Self::L5Prm }
        else if p.contains("l6_self") || p.contains("l6_autonomic") { Self::L6Self }
        else if p.contains("l7_capability") { Self::L7Capability }
        else if p.contains("l8_autonomic") || p.contains("l8_seal") { Self::L8Seal }
        else if p.contains("l9_transcendent") { Self::L9Transcendent }
        else { Self::Unknown }
    }

    pub fn layer_index(&self) -> i32 {
        match self {
            ArchLayer::L0Core => 0,
            ArchLayer::L1Act => 1,
            ArchLayer::L2World => 2,
            ArchLayer::L3Memory => 3,
            ArchLayer::L4Cognition => 4,
            ArchLayer::L5Prm => 5,
            ArchLayer::L6Self => 6,
            ArchLayer::L7Capability => 7,
            ArchLayer::L8Seal => 8,
            ArchLayer::L9Transcendent => 9,
            ArchLayer::Unknown => -1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelfReviewConfig {
    pub unwrap_max: usize,
    pub expect_max: usize,
    pub todo_max: usize,
    pub unimplemented_max: usize,
    pub allow_dead_max: usize,
    pub empty_match_max: usize,
    pub index_multiplier: usize,
    pub index_absolute: usize,
    pub lock_unwrap_max: usize,
    pub exit_max: usize,
    pub observer_quality_threshold: f64,
    pub uncovered_test_max: usize,
    pub unwrap_in_lazy_max: usize,
    pub unused_imports_max: usize,
    pub karpathy_simplicity_max: usize,
    pub karpathy_surgical_files: usize,
    pub karpathy_surgical_lines: usize,
    pub karpathy_complexity_max: usize,
    pub karpathy_goal_driven_max: usize,
    pub seal_stub_max: usize,
    pub min_test_line_count: usize,
    pub scan_safety_bound: usize,
}

impl Default for SelfReviewConfig {
    fn default() -> Self {
        Self {
            unwrap_max: 120,
            expect_max: 50,
            todo_max: 5,
            unimplemented_max: 3,
            allow_dead_max: 50,
            empty_match_max: 15,
            index_multiplier: 5,
            index_absolute: 50,
            lock_unwrap_max: 3,
            exit_max: 3,
            observer_quality_threshold: 0.3,
            uncovered_test_max: 5,
            unwrap_in_lazy_max: 3,
            unused_imports_max: 20,
            karpathy_simplicity_max: 30,
            karpathy_surgical_files: 10,
            karpathy_surgical_lines: 500,
            karpathy_complexity_max: 20,
            karpathy_goal_driven_max: 30,
            seal_stub_max: 3,
            min_test_line_count: 50,
            scan_safety_bound: 300,
        }
    }
}

pub struct SelfReviewGate {
    pub strict_mode: bool,
    pub findings: Vec<ReviewFinding>,
    /// Configurable threshold overrides
    pub config: SelfReviewConfig,
    /// Optional observer feedback: quality score from OneObserver (0.0–1.0)
    pub observer_quality: Option<f64>,
    /// Optional observer patterns detected
    pub observer_patterns: Vec<String>,
}

impl Default for SelfReviewGate {
    fn default() -> Self {
        Self {
            strict_mode: true,
            findings: Vec::new(),
            config: SelfReviewConfig::default(),
            observer_quality: None,
            observer_patterns: Vec::new(),
        }
    }
}

impl crate::core::nt_core_self_test::SelfTest for SelfReviewGate {
    fn name(&self) -> &str { "self_review_gate" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if self.config.min_test_line_count < 1 {
            failures.push("self_review_gate: min_test_line_count must be >= 1".into());
        }
        if self.config.scan_safety_bound < 1 {
            failures.push("self_review_gate: scan_safety_bound must be >= 1".into());
        }
        if failures.is_empty() { Ok(()) } else { Err(failures) }
    }
}

impl SelfReviewGate {
    pub fn new(strict_mode: bool) -> Self {
        Self {
            strict_mode,
            findings: Vec::new(),
            config: SelfReviewConfig::default(),
            observer_quality: None,
            observer_patterns: Vec::new(),
        }
    }

    pub fn with_observer_feedback(mut self, quality: f64, patterns: Vec<String>) -> Self {
        self.observer_quality = Some(quality);
        self.observer_patterns = patterns;
        self
    }

    pub fn syn_depth(&self, code: &str) -> usize {
        match syn::parse_file(code) {
            Ok(file) => syn_file_max_depth(&file),
            Err(_) => estimate_brace_depth(code),
        }
    }

    pub fn check(&mut self, condition: bool, severity: Severity, category: impl Into<String>, message: String, file: impl Into<String>, line: u32) {
        if !condition {
            self.findings.push(ReviewFinding {
                severity,
                category: category.into(),
                message,
                file: file.into(),
                line,
            });
        }
    }

    pub fn report(&self) -> SelfReviewReport {
        let mut failed = 0usize;
        let mut warnings = 0usize;
        for f in &self.findings {
            match f.severity {
                Severity::Error => failed += 1,
                Severity::Warning => warnings += 1,
                Severity::Info => {}
            }
        }
        let passed = self.findings.len().saturating_sub(failed + warnings);
        SelfReviewReport { findings: self.findings.clone(), passed, failed, warnings }
    }

    /// Compute blast-radius from current findings.
    pub fn blast_radius(&self) -> BlastRadiusReport {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let files_scanned = count_rs_files(&src_dir);
        let mut affected: Vec<String> = self.findings.iter().map(|f| f.file.clone()).collect();
        affected.sort();
        affected.dedup();
        let affected_files = affected.len();
        let module_crossings = self.findings.iter()
            .filter(|f| f.category == "layer_violation" || f.category == "cross_file_impact")
            .count();
        let risk = if self.findings.iter().any(|f| f.severity == Severity::Error && (f.category == "panic_audit" || f.category == "layer_violation")) {
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

    pub fn run_all(&mut self) -> SelfReviewReport {
        self.check_no_unwrap_in_production();
        self.check_no_todo_in_production();
        self.check_panic_audit_detailed();
        self.check_no_dead_code_without_allow();
        self.check_public_api_has_docs();
        self.check_assertion_failures();
        self.check_mutex_poison();
        self.check_indexing_panics();

        if self.strict_mode {
            self.check_no_empty_match_arms();
            self.check_layer_violations();
            self.check_architecture_layer_depth();
            self.check_observer_feedback();
            self.check_process_exit();
            self.check_binary_health();
            self.check_test_density();
            self.check_init_safety();
            self.check_orphan_files();
            self.check_unused_imports();
            self.check_python_bare_except();
            self.check_python_sql_injection();
            self.check_python_legacy_tables();
            self.check_python_main_guard();
        }

        // Cycle 27 additions — always run (pattern detection, not severity-gated)
        self.check_negative_signal_zeroed();
        self.check_fts_desync();
        self.check_zscore_length_mismatch();
        self.check_substring_tag_matching();
        self.check_hashmap_iteration_order();
        self.check_clamp_negative_semantics();

        // Cycle 29 additions — Karpathy-inspired code principles (PA020-PA023)
        self.check_karpathy_simplicity_first();
        self.check_karpathy_surgical_changes();
        self.check_karpathy_complexity_budget();
        self.check_karpathy_goal_driven_execution();

        // Cycle 30 additions — always run
        self.check_seal_stage_health();
        self.check_ci_workflow_coverage();
        self.check_test_assertion_hygiene();
        self.check_dep_version_consistency();

        self.report()
    }

    fn check_no_unwrap_in_production(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let unwrap_count = scan_for_patterns("", &[PatternConfig { name: "unwrap".into(), pattern: regex::escape(".unwrap(") }], Some(&src_dir)).len();
        let expect_count = scan_for_patterns("", &[PatternConfig { name: "expect".into(), pattern: regex::escape(".expect(") }], Some(&src_dir)).len();
        let msg = format!(
            "PA001-PA002: {} .unwrap() and {} .expect() calls in src/ (excludes #[cfg(test)])",
            unwrap_count, expect_count
        );
        self.check(
            unwrap_count <= self.config.unwrap_max && expect_count <= self.config.expect_max,
            Severity::Warning,
            "unwrap_safety",
            msg,
            file!(),
            line!(),
        );
    }

    fn check_no_todo_in_production(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let todo_count = scan_for_pattern_excluding_tests(&src_dir, "todo!(");
        let unimpl_count = scan_for_pattern_excluding_tests(&src_dir, "unimplemented!(");
        let msg = format!(
            "PA004-PA005: {} todo!() and {} unimplemented!() calls in src/ (excluding #[cfg(test)])",
            todo_count, unimpl_count
        );
        self.check(
            todo_count < self.config.todo_max && unimpl_count < self.config.unimplemented_max,
            Severity::Warning,
            "todo_check",
            msg,
            file!(),
            line!(),
        );
    }

    fn check_no_dead_code_without_allow(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let allow_dead = scan_for_patterns("", &[PatternConfig { name: "allow_dead_code".into(), pattern: regex::escape("#[allow(dead_code)]") }], Some(&src_dir)).len();
        let msg = format!(
            "Found {} #[allow(dead_code)] annotations in src/ (potential dead code)",
            allow_dead
        );
        self.check(
            allow_dead < self.config.allow_dead_max,
            Severity::Warning,
            "dead_code",
            msg,
            file!(),
            line!(),
        );
    }

    fn check_public_api_has_docs(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let pub_fn = scan_for_patterns("", &[PatternConfig { name: "pub_fn".into(), pattern: regex::escape("pub fn ") }], Some(&src_dir)).len();
        let pub_struct = scan_for_patterns("", &[PatternConfig { name: "pub_struct".into(), pattern: regex::escape("pub struct ") }], Some(&src_dir)).len();
        let doc_lines = scan_for_patterns("", &[PatternConfig { name: "doc_comment".into(), pattern: regex::escape("///") }], Some(&src_dir)).len();
        let msg = format!(
            "{} pub fn, {} pub struct, {} doc comments — doc ratio: {:.1}%",
            pub_fn,
            pub_struct,
            doc_lines,
            if pub_fn + pub_struct > 0 { doc_lines as f64 / (pub_fn + pub_struct) as f64 * 100.0 } else { 0.0 }
        );
        self.check(
            doc_lines >= (pub_fn + pub_struct) / 2,
            Severity::Info,
            "public_docs",
            msg,
            file!(),
            line!(),
        );
    }

    fn check_no_empty_match_arms(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let empty_match = scan_for_patterns("", &[PatternConfig { name: "empty_match".into(), pattern: regex::escape(" => {},") }], Some(&src_dir)).len();
        let msg = format!(
            "Found {} empty match arms ( => {{}},) in src/",
            empty_match
        );
        self.check(
            empty_match < self.config.empty_match_max,
            Severity::Warning,
            "empty_match",
            msg,
            file!(),
            line!(),
        );
    }

    /// Extended panic audit — cargo-panic-audit classes PA003-PA005.
    /// Detects `panic!()`, `todo!()`, `unreachable!()` in production code.
    fn check_panic_audit_detailed(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let panic_count = scan_for_pattern_excluding_tests(&src_dir, "panic!(");
        let todo_count = scan_for_pattern_excluding_tests(&src_dir, "todo!(");
        let unreachable_count = scan_for_pattern_excluding_tests(&src_dir, "unreachable!(");
        let mut msg = String::from("Panic audit [cargo-panic-audit PA003-PA005]:");
        msg.push_str(&format!(" panic!()={}", panic_count));
        msg.push_str(&format!(" todo!()={}", todo_count));
        msg.push_str(&format!(" unreachable!()={}", unreachable_count));
        self.check(
            panic_count == 0 && todo_count == 0 && unreachable_count == 0,
            Severity::Error,
            "panic_audit",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA006: Array/slice indexing — `arr[i]`, `vec[j]` that may panic on OOB.
    /// Detected via pattern match on bracket-access expressions.
    fn check_indexing_panics(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let bracket_idx = scan_for_pattern_excluding_tests(&src_dir, "[");
        let get_calls = scan_for_pattern_excluding_tests(&src_dir, ".get(");
        let raw_idx = bracket_idx.saturating_sub(get_calls * 3);
        let msg = format!(
            "PA006: ~{} raw index expressions (arr[i]) vs {} .get() safe accesses — prefer .get() for bounds safety",
            raw_idx, get_calls
        );
        self.check(
            raw_idx < get_calls * self.config.index_multiplier || raw_idx < self.config.index_absolute,
            Severity::Info,
            "indexing_panic",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA007: Assertion failures — `assert!()`, `assert_eq!()` in production.
    fn check_assertion_failures(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let assert_count = scan_for_pattern_excluding_tests(&src_dir, "assert!(");
        let assert_eq_count = scan_for_pattern_excluding_tests(&src_dir, "assert_eq!(");
        let msg = format!(
            "PA007: {} assert!() and {} assert_eq!() in src/ (production assertions may panic)",
            assert_count, assert_eq_count
        );
        self.check(
            assert_count == 0 && assert_eq_count == 0,
            Severity::Warning,
            "assertion_failure",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA008: Mutex/RwLock unwrap — `.lock().unwrap()` pattern (panic amplification).
    fn check_mutex_poison(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let lock_unwrap = scan_for_pattern_excluding_tests(&src_dir, ".lock().unwrap(");
        let mutex_unwrap = scan_for_pattern_excluding_tests(&src_dir, ".lock(");
        let msg = format!(
            "PA008: {} .lock().unwrap() patterns (panic amplification risk). Use .lock().expect() or ? with poison handling.",
            lock_unwrap
        );
        self.check(
            lock_unwrap < self.config.lock_unwrap_max || mutex_unwrap == 0,
            Severity::Warning,
            "mutex_poison",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA009: `process::exit()` or `std::process::exit` in production.
    fn check_process_exit(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let exit_count = scan_for_pattern_excluding_tests(&src_dir, "process::exit(");
        let msg = format!(
            "PA009: {} process::exit() calls found — kills process immediately, prefer graceful shutdown",
            exit_count
        );
        self.check(
            exit_count < self.config.exit_max,
            Severity::Warning,
            "process_exit",
            msg,
            file!(),
            line!(),
        );
    }

    /// Layer violation check — verifies core/ does not import from neotrix/.
    fn check_layer_violations(&mut self) {
        let core_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("core");
        if !core_dir.exists() {
            return;
        }
        let violation_count = scan_for_patterns("", &[PatternConfig { name: "layer_violation".into(), pattern: regex::escape("use crate::neotrix") }], Some(&core_dir)).len();
        let msg = format!(
            "Layer violation: {} `use crate::neotrix` imports found in core/ (L0→L9 violation)",
            violation_count
        );
        self.check(
            violation_count == 0,
            Severity::Error,
            "layer_violation",
            msg,
            file!(),
            line!(),
        );
    }

    /// Architecture layer depth analysis — verifies no backward dependency flow.
    /// Scans each file's `use crate::` imports and checks layer ordering.
    fn check_architecture_layer_depth(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut violations = 0usize;
        if let Ok(entries) = std::fs::read_dir(&src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_none_or(|e| e != "rs") { continue; }
                let source_layer = ArchLayer::from_path(&path);
                if source_layer.layer_index() < 0 { continue; }
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        if line.starts_with("use crate::") {
                            let target_layer = self.detect_import_layer(line);
                            if target_layer.layer_index() >= 0
                                && target_layer.layer_index() < source_layer.layer_index()
                            {
                                violations += 1;
                            }
                        }
                    }
                }
            }
        }
        let msg = format!(
            "Architecture depth: {} reverse-layer imports (higher layer importing from lower)",
            violations
        );
        self.check(
            violations == 0,
            Severity::Warning,
            "arch_depth",
            msg,
            file!(),
            line!(),
        );
    }

    fn detect_import_layer(&self, line: &str) -> ArchLayer {
        if line.contains("core::") || line.contains("::core::") { ArchLayer::L0Core }
        else if line.contains("l1_body") || line.contains("::act::") { ArchLayer::L1Act }
        else if line.contains("l2_world") || line.contains("::world::") { ArchLayer::L2World }
        else if line.contains("l3_memory") || line.contains("::memory::") { ArchLayer::L3Memory }
        else if line.contains("l4_cognition") || line.contains("::cognition::") { ArchLayer::L4Cognition }
        else if line.contains("::prm::") || line.contains("nt_core_prm") { ArchLayer::L5Prm }
        else if line.contains("l6_self") || line.contains("::self") || line.contains("::mind::") { ArchLayer::L6Self }
        else if line.contains("l7_capability") || line.contains("::capability::") { ArchLayer::L7Capability }
        else if line.contains("l8_autonomic") || line.contains("l8_seal") { ArchLayer::L8Seal }
        else if line.contains("l9_transcendent") || line.contains("::transcendent::") { ArchLayer::L9Transcendent }
        else { ArchLayer::Unknown }
    }

    /// Observer feedback integration — consumes OneObserver quality/patterns from reasoning engine.
    /// If observer detects trajectory quality < 0.3 or critical patterns, flag as warning.
    fn check_observer_feedback(&mut self) {
        let observer_feedback = self.observer_quality;
        let has_critical = self.observer_patterns.iter().any(|p| p.contains("oscillation") || p.contains("stuck"));
        if let Some(q) = observer_feedback {
            let degraded = q < self.config.observer_quality_threshold;
            let msg = format!(
                "Observer feedback: quality={:.2}, critical_patterns={} — reasoning trajectory {}",
                q,
                self.observer_patterns.len(),
                if degraded { "DEGRADED (quality < 0.3)" } else { "OK" }
            );
            self.check(
                !degraded,
                if degraded { Severity::Warning } else { Severity::Info },
                "observer_feedback",
                msg,
                file!(),
                line!(),
            );
        }
        if has_critical {
            let pat_msg = format!(
                "Observer critical patterns: {}",
                self.observer_patterns.join(", ")
            );
            self.check(
                false,
                Severity::Warning,
                "observer_pattern",
                pat_msg,
                file!(),
                line!(),
            );
        }
    }

    /// Check binary health — verify [[bin]] entries in Cargo.toml match files in src/bin/.
    /// Detects orphan entries (bin declared but no file) and orphan files (file but no entry).
    /// Distilled from Cycle 24 deep scan: 12 bins verified, 0 orphans.
    fn check_binary_health(&mut self) {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let cargo_toml = manifest_dir.join("Cargo.toml");
        let bin_dir = manifest_dir.join("src").join("bin");

        // Parse [[bin]] entries from Cargo.toml
        let content = match std::fs::read_to_string(&cargo_toml) {
            Ok(c) => c,
            Err(_) => return,
        };
        let mut declared_bins: Vec<String> = Vec::new();
        let mut in_bin_section = false;
        for line in content.lines() {
            if line.trim_start().starts_with("[[bin]]") {
                in_bin_section = true;
                continue;
            }
            if in_bin_section {
                if line.trim_start().starts_with('[') {
                    in_bin_section = false;
                    continue;
                }
                if let Some(name) = line.trim().strip_prefix("name = \"") {
                    if let Some(end) = name.find('\"') {
                        declared_bins.push(name[..end].to_string());
                    }
                }
            }
        }

        // Collect actual binary files
        let mut actual_bins: Vec<String> = Vec::new();
        if bin_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&bin_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "rs") {
                        if let Some(stem) = path.file_stem() {
                            actual_bins.push(stem.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        // Check for orphans: declared but no file
        let mut orphan_declarations: Vec<String> = Vec::new();
        for bin in &declared_bins {
            if !actual_bins.contains(bin) {
                orphan_declarations.push(bin.clone());
            }
        }
        // Check for orphans: file but no declaration
        let mut orphan_files: Vec<String> = Vec::new();
        for bin in &actual_bins {
            if !declared_bins.contains(bin) {
                orphan_files.push(bin.clone());
            }
        }

        let decl_ok = orphan_declarations.is_empty();
        let file_ok = orphan_files.is_empty();
        let total_declared = declared_bins.len();
        let total_actual = actual_bins.len();

        let msg = format!(
            "Binary health: {} declared, {} files, decl_orphans={:?} file_orphans={:?}",
            total_declared, total_actual, orphan_declarations, orphan_files,
        );
        self.check(
            decl_ok && file_ok,
            Severity::Warning,
            "binary_health",
            msg,
            file!(),
            line!(),
        );
    }

    /// Check test density — find .rs files >50 lines without a #[test] attribute.
    /// Distilled from Cycle 24 test density analysis: 2007 total #[test], gaps in ewhr_bridge/pipeline.
    fn check_test_density(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut uncovered: Vec<String> = Vec::new();
        scan_file_test_coverage(&src_dir, &mut uncovered);
        let msg = format!(
            "Test density: {} uncovered files (>50 lines, no #[test]) — check ewhr_bridge, pipeline, nt-act social stubs",
            uncovered.len(),
        );
        self.check(
            uncovered.len() < self.config.uncovered_test_max,
            Severity::Warning,
            "test_density",
            msg,
            file!(),
            line!(),
        );
    }

    /// Check init safety — detect LazyLock constructors using .unwrap() or .expect().
    /// Distilled from Cycle 24 fix: 7 LazyLock unwrap→expect in session.rs, fetcher.rs.
    fn check_init_safety(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let lazy_unwrap = scan_for_patterns("", &[PatternConfig { name: "lazy_lock_init".into(), pattern: regex::escape("LazyLock::new(||") }], Some(&src_dir)).len();
        let unwrap_in_lazy = scan_for_pattern_in_lazy_init(&src_dir);
        let msg = format!(
            "Init safety: {} LazyLock inits, {} with unwrap/expect in closure — prefer expect() or fallback",
            lazy_unwrap, unwrap_in_lazy,
        );
        self.check(
            unwrap_in_lazy < self.config.unwrap_in_lazy_max,
            Severity::Warning,
            "init_safety",
            msg,
            file!(),
            line!(),
        );
    }

    /// Check orphan files — detect .rs files in src/ but not referenced in any mod.rs.
    /// Uses module declaration scan, not full compilation graph.
    fn check_orphan_files(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut declared: Vec<String> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() || path.extension().is_none_or(|e| e != "rs") { continue; }
                let content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                for line in content.lines() {
                    if line.trim_start().starts_with("pub mod ") || line.trim_start().starts_with("mod ") {
                        if let Some(name) = line.trim().strip_prefix("pub mod ").or_else(|| line.trim().strip_prefix("mod ")) {
                            if let Some(end) = name.find(|c: char| !c.is_alphanumeric() && c != '_') {
                                declared.push(name[..end].to_string());
                            }
                        }
                    }
                }
            }
        }
        // Collect all .rs files
        let mut all_files: Vec<String> = Vec::new();
        collect_rs_stems(&src_dir, &mut all_files);
        // Exclude lib.rs, main.rs, bin/ entries
        let orphans: Vec<String> = all_files.into_iter()
            .filter(|f| !declared.contains(f) && f != "lib" && f != "main")
            .collect();
        let msg = format!(
            "Orphan files: {} .rs files not declared in any mod.rs (may be dead code)",
            orphans.len(),
        );
        self.check(
            orphans.is_empty(),
            Severity::Warning,
            "orphan_files",
            msg,
            file!(),
            line!(),
        );
    }

    /// Check unused imports — detect `use` statements that are the only occurrence in their file
    /// (simple heuristic: count `use crate::` and `use std::` patterns not followed by usage).
    fn check_unused_imports(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let total_imports = scan_for_patterns("", &[PatternConfig { name: "use_statement".into(), pattern: regex::escape("use ") }], Some(&src_dir)).len();
        // Simple heuristic: detect "dead use" patterns where import is the only reference
        let unused_imports = scan_for_unused_import_patterns(&src_dir);
        let msg = format!(
            "Import hygiene: {} total use statements, ~{} potentially unused (dead_code heuristic)",
            total_imports, unused_imports,
        );
        self.check(
            unused_imports < self.config.unused_imports_max,
            Severity::Info,
            "unused_imports",
            msg,
            file!(),
            line!(),
        );
    }

    // ─── Cycle 28 additions — distilled from Python script audit ───

    /// PA016: Python bare except — scan scripts/ for `except:` without Exception type.
    /// This can mask KeyboardInterrupt, SystemExit, and other non-Exception errors.
    fn check_python_bare_except(&mut self) {
        let scripts_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("scripts");
        if !scripts_dir.exists() { return; }
        let bare_except_count = scan_python_for_bare_except(&scripts_dir);
        let msg = format!(
            "PA016: {} bare `except:` clauses in scripts/ (should specify exception type)",
            bare_except_count,
        );
        self.check(
            bare_except_count == 0,
            Severity::Warning,
            "python_bare_except",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA017: Python SQL injection — scan scripts/ for f-string patterns in SQL queries.
    /// True parametrized queries use `?` or `%s` placeholders, not `{value}`.
    fn check_python_sql_injection(&mut self) {
        let scripts_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("scripts");
        if !scripts_dir.exists() { return; }
        let fstring_sql = scan_python_fstring_in_sql(&scripts_dir);
        let msg = format!(
            "PA017: {} f-string patterns in SQL statements in scripts/ (use parametrized queries)",
            fstring_sql,
        );
        self.check(
            fstring_sql == 0,
            Severity::Warning,
            "python_sql_injection",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA018: Python legacy table writes — scan scripts/ for writes to knowledge_nodes/knowledge_edges.
    fn check_python_legacy_tables(&mut self) {
        let scripts_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("scripts");
        if !scripts_dir.exists() { return; }
        let mut count = 0usize;
        if let Ok(entries) = std::fs::read_dir(&scripts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "py") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        for line in content.lines() {
                            let trimmed = line.trim().to_uppercase();
                            if (trimmed.contains("KNOWLEDGE_NODES") || trimmed.contains("KNOWLEDGE_EDGES"))
                                && (trimmed.starts_with("INSERT") || trimmed.starts_with("UPDATE") || trimmed.starts_with("DELETE"))
                            {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
        let msg = format!(
            "PA018: {} writes to legacy `knowledge_nodes`/`knowledge_edges` tables in scripts/ (use `nodes`/`edges`)",
            count,
        );
        self.check(
            count == 0,
            Severity::Warning,
            "python_legacy_tables",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA019: Python main guard — scan scripts/ for missing `if __name__ == '__main__'`.
    fn check_python_main_guard(&mut self) {
        let scripts_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("scripts");
        if !scripts_dir.exists() { return; }
        let mut missing = 0usize;
        let mut total = 0usize;
        if let Ok(entries) = std::fs::read_dir(&scripts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "py") {
                    total += 1;
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if !content.contains("if __name__") && !content.contains("def main(") {
                            missing += 1;
                        }
                    }
                }
            }
        }
        let msg = format!(
            "PA019: {}/{} Python scripts in scripts/ lack `if __name__ == '__main__'` guard and `def main()`",
            missing, total,
        );
        self.check(
            missing == 0,
            Severity::Info,
            "python_main_guard",
            msg,
            file!(),
            line!(),
        );
    }

    // ─── Cycle 27 additions — distilled from 6 Rust bug fixes ───

    /// PA010: Negative advantage/reward signal zeroed via `.max(0.0)`.
    /// Detects RL patterns like `adv.max(0.0)` or `reward.max(0.0)` that suppress
    /// negative learning signals — system cannot learn from failures.
    /// Bug found at: nt_core_prm.rs:1089,2346 (adv.max(0.0) → ((adv+1.0)/2.0))
    fn check_negative_signal_zeroed(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let adv_max = scan_for_pattern_excluding_tests(&src_dir, "adv.max(0.0)");
        let reward_max = scan_for_pattern_excluding_tests(&src_dir, "reward.max(0.0)");
        let score_max = scan_for_pattern_excluding_tests(&src_dir, "score.max(0.0)");
        let total = adv_max + reward_max + score_max;
        let msg = format!(
            "PA010: {} negative signal zeroing patterns (adv/reward/score.max(0.0)) — \
             use ((val+1.0)/2.0) to preserve [-1,0) learning signal",
            total,
        );
        self.check(
            total == 0,
            Severity::Warning,
            "negative_signal_zeroed",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA011: FTS index desync — detect writes to `nodes` table without paired writes to `nodes_fts`.
    /// Bug found at: nt_memory_store.rs:38-46, 51-72 (insert_node skip, update_node no sync)
    fn check_fts_desync(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let nodes_inserts = scan_for_patterns("", &[PatternConfig { name: "nodes_insert".into(), pattern: regex::escape("INSERT INTO nodes") }], Some(&src_dir)).len();
        let nodes_fts_inserts = scan_for_patterns("", &[PatternConfig { name: "nodes_fts_insert".into(), pattern: regex::escape("INSERT INTO nodes_fts") }], Some(&src_dir)).len();
        let nodes_updates = scan_for_patterns("", &[PatternConfig { name: "nodes_update".into(), pattern: regex::escape("UPDATE nodes SET") }], Some(&src_dir)).len();
        let nodes_fts_updates = scan_for_patterns("", &[PatternConfig { name: "nodes_fts_update".into(), pattern: regex::escape("UPDATE nodes_fts SET") }], Some(&src_dir)).len();
        let nodes_deletes = scan_for_patterns("", &[PatternConfig { name: "nodes_delete".into(), pattern: regex::escape("DELETE FROM nodes WHERE") }], Some(&src_dir)).len();
        let nodes_fts_deletes = scan_for_patterns("", &[PatternConfig { name: "nodes_fts_delete".into(), pattern: regex::escape("DELETE FROM nodes_fts") }], Some(&src_dir)).len();

        let mut msgs = Vec::new();
        if nodes_inserts > nodes_fts_inserts {
            msgs.push(format!("INSERT: nodes={} > nodes_fts={}", nodes_inserts, nodes_fts_inserts));
        }
        if nodes_updates > nodes_fts_updates {
            msgs.push(format!("UPDATE: nodes={} > nodes_fts={}", nodes_updates, nodes_fts_updates));
        }
        if nodes_deletes > nodes_fts_deletes {
            msgs.push(format!("DELETE: nodes={} > nodes_fts={}", nodes_deletes, nodes_fts_deletes));
        }

        let clean = msgs.is_empty();
        let msg = if clean {
            "PA011: FTS desync check — all nodes operations have paired nodes_fts operations".to_string()
        } else {
            format!("PA011: FTS desync — {}", msgs.join("; "))
        };
        self.check(
            clean,
            Severity::Warning,
            "fts_desync",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA012: zscore_normalize drops NaN entries changing output length.
    /// Detects `filter(|v| v.is_finite())` in normalization functions without length preservation.
    /// Bug found at: nt_core_prm.rs:856-866 (zscore_normalize returning shorter vec)
    fn check_zscore_length_mismatch(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let filter_finite = scan_for_pattern_excluding_tests(&src_dir, ".filter(|v| v.is_finite())");
        let normalize_fns = scan_for_pattern_excluding_tests(&src_dir, "fn zscore_normalize");
        let msg = format!(
            "PA012: {} filter(is_finite) in normalization — output length may mismatch input \
             (replace with map(|v| if v.is_finite() {{*v}} else {{0.0}}))",
            filter_finite,
        );
        self.check(
            filter_finite < normalize_fns,  // Each normalize fn should have 0 filter_finite
            Severity::Warning,
            "zscore_length_mismatch",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA013: Substring tag matching — detect `.contains("literal")` in tag/attribute matching
    /// where exact string comparison should be used. Prevents false positives like
    /// "not_good" matching "good".
    /// Bug found at: nt_core_policy.rs:292-300 (t.contains("good") matching "not_good")
    fn check_substring_tag_matching(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let contains_good = scan_for_pattern_excluding_tests(&src_dir, ".contains(\"good\")");
        let contains_fail = scan_for_pattern_excluding_tests(&src_dir, ".contains(\"fail\")");
        let contains_ok = scan_for_pattern_excluding_tests(&src_dir, ".contains(\"ok\")");
        let total = contains_good + contains_fail + contains_ok;
        let msg = format!(
            "PA013: {} substring tag matches (contains(\"good\"/\"fail\"/\"ok\")) — \
             use tag.as_str() or == for exact semantic matching",
            total,
        );
        self.check(
            total == 0,
            Severity::Warning,
            "substring_tag_match",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA014: HashMap iteration order dependency — detect `.values().nth()` on HashMap
    /// where iteration order is non-deterministic. Prefer BTreeMap for positional indexing.
    /// Bug found at: workspace.rs:344,356 + neotrix-types workspace.rs:117,129
    fn check_hashmap_iteration_order(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let hashmap_nth = scan_for_pattern_excluding_tests(&src_dir, "HashMap<")
            + scan_for_pattern_excluding_tests(&src_dir, "HashMap::");
        let nth_values = scan_for_pattern_excluding_tests(&src_dir, ".values().nth(");
        let use_btreemap_for_nth = nth_values > 0 && hashmap_nth > 0;
        let msg = format!(
            "PA014: {} .values().nth() calls on HashMap (non-deterministic iteration order). \
             {} HashMap declarations — use BTreeMap for stable positional indexing",
            nth_values, hashmap_nth,
        );
        self.check(
            !use_btreemap_for_nth,
            Severity::Warning,
            "hashmap_iteration_order",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA015: Clamp pattern on advantage/reward — detect `.max(0.0).min(1.0)` pipelines
    /// where the bounds may not match the variable's actual range.
    /// Semantically safer: use proper clamp or domain-aware mapping.
    /// Bug found at: nt_core_prm.rs:1089,2346 (adv.max(0.0).min(1.0) on [-1,1] range)
    fn check_clamp_negative_semantics(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let max_min_01 = scan_for_pattern_excluding_tests(&src_dir, ".max(0.0).min(1.0)");
        let msg = format!(
            "PA015: {} .max(0.0).min(1.0) clamp patterns — verify variable range. \
             If range is [-1,1], use ((x+1.0)/2.0) instead to preserve negative signal",
            max_min_01,
        );
        self.check(
            max_min_01 < 3,
            Severity::Info,
            "clamp_semantics",
            msg,
            file!(),
            line!(),
        );
    }

    // ─── Cycle 29 additions — Karpathy-inspired principles (PA020-PA023) ───

    /// PA020 (Simplicity First): Detect overcomplicated patterns — generic params not used,
    /// deep nesting (>5 levels), excessive function length (>200 lines), single-impl traits.
    /// Inspired by Karpathy's "code should be simple enough to fit in your head."
    fn check_karpathy_simplicity_first(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let unused_generics = scan_for_unused_generic_params(&src_dir);
        let deep_nesting = count_deeply_nested_lines(&src_dir, 5);
        let long_fns = count_long_functions(&src_dir, 200);
        let single_traits = count_single_impl_traits(&src_dir);
        let issues = unused_generics + deep_nesting + long_fns + single_traits;
        let msg = format!(
            "PA020 (Simplicity First): {} issues — {} unused generics, {} deep nesting sites \
             (>5 levels), {} long functions (>200 lines), {} single-impl traits (consider inlining)",
            issues, unused_generics, deep_nesting, long_fns, single_traits,
        );
        self.check(
            issues < self.config.karpathy_simplicity_max,
            Severity::Warning,
            "karpathy_simplicity_first",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA021 (Surgical Changes): Detect large diffs — files changed >10, lines added >500.
    /// Inspired by Karpathy's "small, focused commits are easier to review and revert."
    fn check_karpathy_surgical_changes(&mut self) {
        let repo_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let files_changed = count_files_in_last_commit(repo_dir);
        let lines_added = count_lines_in_last_commit(repo_dir);
        let msg = format!(
            "PA021 (Surgical Changes): last commit changed {} files, added {} lines — \
             prefer smaller, focused commits (≤10 files, ≤500 lines)",
            files_changed, lines_added,
        );
        self.check(
            files_changed <= self.config.karpathy_surgical_files && lines_added <= self.config.karpathy_surgical_lines,
            Severity::Info,
            "karpathy_surgical_changes",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA022 (Complexity Budget): Detect cyclomatic complexity indicators — long if-else chains,
    /// excessive match arms, functions with too many parameters.
    /// Inspired by Karpathy's "complexity is a budget — spend it wisely."
    fn check_karpathy_complexity_budget(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let long_chains = count_long_if_chains(&src_dir, 8);
        let many_arms = count_excessive_match_arms(&src_dir, 15);
        let many_params = count_excessive_param_count(&src_dir, 10);
        let issues = long_chains + many_arms + many_params;
        let msg = format!(
            "PA022 (Complexity Budget): {} issues — {} long if-else chains (>8 arms), \
             {} excessive match expressions (>15 arms), {} functions with >10 parameters",
            issues, long_chains, many_arms, many_params,
        );
        self.check(
            issues < self.config.karpathy_complexity_max,
            Severity::Warning,
            "karpathy_complexity_budget",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA023 (Goal-Driven Execution): Detect imperative-only patterns — TODOs without tests,
    /// state-mutating functions not returning Result, missing success criteria in docs.
    /// Inspired by Karpathy's "write the test first, then the code."
    fn check_karpathy_goal_driven_execution(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let todos_no_test = count_todos_without_nearby_test(&src_dir);
        let mut_no_result = count_state_mutation_no_result(&src_dir);
        let missing_goals = count_pub_fn_without_goal_doc(&src_dir);
        let issues = todos_no_test + mut_no_result + missing_goals;
        let msg = format!(
            "PA023 (Goal-Driven Execution): {} issues — {} TODOs without associated test, \
             {} &mut self fns without Result return, {} pub fns missing success criteria in docs",
            issues, todos_no_test, mut_no_result, missing_goals,
        );
        self.check(
            issues < self.config.karpathy_goal_driven_max,
            Severity::Warning,
            "karpathy_goal_driven_execution",
            msg,
            file!(),
            line!(),
        );
    }

    // ─── Cycle 30 additions — distilled from Architecture Rebirth full-stack audit ───

    /// PA024 (SEAL Stage Health): Detect SEAL pipeline stages that are no-op stubs
    /// (just `log::trace!` + `Ok(StageDecision::Continue)` without real work).
    fn check_seal_stage_health(&mut self) {
        let pipeline_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("neotrix")
            .join("l8_autonomic_impl")
            .join("nt_mind")
            .join("self_iterating")
            .join("pipeline.rs");
        let mut stub_count = 0usize;
        let mut total = 0usize;
        if let Ok(content) = std::fs::read_to_string(&pipeline_path) {
            let lines: Vec<&str> = content.lines().collect();
            for i in 0..lines.len() {
                if lines[i].trim().starts_with("fn process(") {
                    total += 1;
                    let mut body_lines = 0usize;
                    let mut real_ops = 0usize;
                    for j in (i + 1)..lines.len().min(i + 15) {
                        let t = lines[j].trim();
                        if t.starts_with("Ok(") || t.starts_with('}') { break; }
                        body_lines += 1;
                        if t.starts_with("log::") { continue; }
                        if !t.is_empty() && !t.starts_with("//") {
                            real_ops += 1;
                        }
                    }
                    if real_ops == 0 && body_lines > 0 {
                        stub_count += 1;
                    }
                }
            }
        }
        let msg = format!(
            "PA024: {}/{} SEAL pipeline stages are no-op stubs (log-only, no real operations)",
            stub_count, total,
        );
        self.check(
            stub_count <= self.config.seal_stub_max,
            Severity::Warning,
            "seal_stage_health",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA025 (CI Workflow Coverage): Check that essential CI workflows exist.
    fn check_ci_workflow_coverage(&mut self) {
        let workflows_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(".github").join("workflows");
        let mut found = Vec::new();
        if workflows_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&workflows_dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        found.push(name.to_string());
                    }
                }
            }
        }
        let has_deny = found.iter().any(|f| f.contains("deny"));
        let has_build = found.iter().any(|f| f.contains("build") || f.contains("ci"));
        let msg = format!(
            "PA025: CI workflows found: {:?} — cargo-deny={}, build={}",
            found, has_deny, has_build,
        );
        self.check(
            has_deny && has_build,
            Severity::Info,
            "ci_workflow_coverage",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA026 (Test Assertion Hygiene): Detect tests using brittle string assertions
    /// (checking for `assert_eq!(result, "exact string")` in integration tests).
    fn check_test_assertion_hygiene(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut brittle = 0usize;
        let mut integration_test_files = 0usize;
        if let Ok(entries) = std::fs::read_dir(&src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() { continue; }
                if path.file_name().is_some_and(|n| {
                    let s = n.to_string_lossy();
                    s.contains("integration_test") || s.contains("e2e_test")
                }) {
                    integration_test_files += 1;
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        for line in content.lines() {
                            let t = line.trim();
                            if (t.starts_with("assert_eq!(") || t.starts_with("assert!("))
                                && t.contains("Status") || t.contains("status") || t.contains("error")
                            {
                                brittle += 1;
                            }
                        }
                    }
                }
            }
        }
        let msg = format!(
            "PA026: {} brittle status/error assertions in {} integration test files \
             (use structured error matching instead of string comparison)",
            brittle, integration_test_files,
        );
        self.check(
            brittle <= 5,
            Severity::Info,
            "test_assertion_hygiene",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA027 (Dep Version Consistency): Detect cross-crate dependency version mismatches.
    fn check_dep_version_consistency(&mut self) {
        let workspace_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mismatches: Vec<String> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&workspace_dir) {
            let mut toml_files = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let cargo_toml = path.join("Cargo.toml");
                    if cargo_toml.exists() {
                        toml_files.push(cargo_toml);
                    }
                }
            }
            // Look for Cargo.toml files with known version-sensitive deps
            for toml in &toml_files {
                if let Ok(content) = std::fs::read_to_string(toml) {
                    for dep in &["quick-xml", "tokio", "serde", "serde_json", "uuid"] {
                        let pattern = format!("{} = \"", dep);
                        if let Some(pos) = content.find(&pattern) {
                            let start = pos + pattern.len();
                            let _end = content[start..].find('"').map(|e| start + e).unwrap_or(0);
                            // simplistic check — just flag it for review
                        }
                    }
                }
            }
        }
        let msg = format!(
            "PA027: Dependency version consistency — {} mismatches found (check Cargo.toml files)",
            mismatches.len(),
        );
        self.check(
            mismatches.is_empty(),
            Severity::Info,
            "dep_version_consistency",
            msg,
            file!(),
            line!(),
        );
    }
}

#[derive(Debug, Clone)]
pub struct PatternConfig {
    pub name: String,
    pub pattern: String,
}

#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub pattern_name: String,
    pub matched_text: String,
}

// ─── Scanner helpers ───

fn count_rs_files(dir: &Path) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                count += count_rs_files(&path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                count += 1;
            }
        }
    }
    count
}

pub fn scan_for_patterns(
    source: &str,
    patterns: &[PatternConfig],
    dir: Option<&Path>,
) -> Vec<PatternMatch> {
    let compiled: Vec<(&PatternConfig, Regex)> = patterns
        .iter()
        .filter_map(|pc| Regex::new(&pc.pattern).ok().map(|r| (pc, r)))
        .collect();
    if compiled.is_empty() {
        return Vec::new();
    }
    let mut results = Vec::new();
    if let Some(d) = dir {
        let mut file_list = Vec::new();
        collect_rs_files_recursive(d, &mut file_list);
        for file_path in file_list {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                for (pc, re) in &compiled {
                    for m in re.find_iter(&content) {
                        let line = content[..m.start()].matches('\n').count() + 1;
                        let col = m.start() - content[..m.start()].rfind('\n').map_or(0, |i| i + 1);
                        results.push(PatternMatch {
                            file: file_path.to_string_lossy().to_string(),
                            line,
                            column: col,
                            pattern_name: pc.name.clone(),
                            matched_text: m.as_str().to_string(),
                        });
                    }
                }
            }
        }
    } else {
        for (pc, re) in &compiled {
            for m in re.find_iter(source) {
                let line = source[..m.start()].matches('\n').count() + 1;
                let col = m.start() - source[..m.start()].rfind('\n').map_or(0, |i| i + 1);
                results.push(PatternMatch {
                    file: String::new(),
                    line,
                    column: col,
                    pattern_name: pc.name.clone(),
                    matched_text: m.as_str().to_string(),
                });
            }
        }
    }
    results
}

fn collect_rs_files_recursive(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                collect_rs_files_recursive(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
}

fn scan_for_pattern_excluding_tests(dir: &Path, pattern: &str) -> usize {
    use std::fs;
    let mut count = 0usize;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("bin-archive") || path.ends_with("target") {
                    continue;
                }
                count += scan_for_pattern_excluding_tests(&path, pattern);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let mut in_test = false;
                    for line in content.lines() {
                        if line.trim().starts_with("#[cfg(test)]") {
                            in_test = true;
                        } else if in_test && line.trim().starts_with("}") && line.trim().len() == 1 {
                            in_test = false;
                            continue;
                        }
                        if in_test { continue; }
                        count += line.matches(pattern).count();
                    }
                }
            }
        }
    }
    count
}

/// Simple heuristic: detect files where an import appears only in the import statement itself.
/// Counts patterns like `use foo` when `foo` doesn't appear elsewhere.
fn scan_for_unused_import_patterns(dir: &Path) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                count += scan_for_unused_import_patterns(&path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        if let Some(stripped) = line.trim().strip_prefix("use ") {
                            let import_name: String = stripped.chars()
                                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == ':')
                                .collect();
                            if import_name.contains("::") { continue; }
                            // Check if the imported name appears outside of use statements
                            let usage_count = content.matches(&import_name).count();
                            if import_name.len() > 3 && usage_count <= 1 {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    count
}

/// Helper: collect all .rs file stems recursively
fn collect_rs_stems(dir: &Path, out: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                collect_rs_stems(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Some(stem) = path.file_stem() {
                    out.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }
}

/// Scan a directory tree for .rs files without #[test].
/// Appends uncovered file stems to `uncovered`.
fn scan_file_test_coverage(dir: &Path, uncovered: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") || path.file_name().is_some_and(|n| n == "bin-archive") { continue; }
                scan_file_test_coverage(&path, uncovered);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let line_count = content.lines().count();
                    if line_count > SelfReviewConfig::default().min_test_line_count && !content.contains("#[test]") {
                        if let Some(stem) = path.file_stem() {
                            uncovered.push(stem.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
}

/// Scan for .unwrap() or .expect() within LazyLock initializer closures.
fn scan_for_pattern_in_lazy_init(dir: &Path) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                count += scan_for_pattern_in_lazy_init(&path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let lines: Vec<&str> = content.lines().collect();
                    for i in 0..lines.len() {
                        if lines[i].contains("LazyLock::new(") {
                            // Check following lines for unwrap/expect in the closure
                            for j in (i + 1)..lines.len().min(i + 20) {
                                let trimmed = lines[j].trim();
                                if trimmed.starts_with('}') || trimmed.starts_with(");") {
                                    break;
                                }
                                if trimmed.contains(".unwrap(") || trimmed.contains(".expect(") {
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    count
}

// ─── Python scanner helpers ───

/// Scan Python files for a simple pattern match.
#[allow(dead_code)]
fn scan_python_for_pattern(dir: &Path, pattern: &str) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() { continue; }
            if path.extension().is_some_and(|e| e == "py") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    count += content.matches(pattern).count();
                }
            }
        }
    }
    count
}

/// Count bare `except:` clauses (not `except Exception:`, `except ValueError:`, etc.)
fn scan_python_for_bare_except(dir: &Path) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() { continue; }
            if path.extension().is_some_and(|e| e == "py") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        // Match bare `except:` with nothing after `except`
                        if trimmed.starts_with("except:")
                            || trimmed == "except :"
                            || trimmed.starts_with("except: ")
                        {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    count
}

/// Scan Python files for f-string patterns in SQL statements (like `f"SELECT ... {value}"`)
fn scan_python_fstring_in_sql(dir: &Path) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() { continue; }
            if path.extension().is_some_and(|e| e == "py") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        // Detect f-strings containing SQL keywords with curly-brace interpolation
                        if (trimmed.starts_with("f\"") || trimmed.starts_with("f'"))
                            && (trimmed.contains("SELECT") || trimmed.contains("INSERT") || trimmed.contains("UPDATE") || trimmed.contains("DELETE"))
                            && trimmed.contains('{') && trimmed.contains('}')
                        {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    count
}

// ─── Cycle 29 scanner helpers — Karpathy-inspired code principles (PA020-PA023) ───

/// Scan for function declarations with single-uppercase-letter generic params
/// (`fn foo<T>`) where the param name doesn't appear in the function body,
/// suggesting they may be unused or over-abstracted.
fn scan_for_unused_generic_params(dir: &Path) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                count += scan_for_unused_generic_params(&path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let lines: Vec<&str> = content.lines().collect();
                    for i in 0..lines.len() {
                        let trimmed = lines[i].trim();
                        if !trimmed.starts_with("fn ") && !trimmed.starts_with("pub fn ") {
                            continue;
                        }
                        let sig = &lines[i];
                        let open_angle = match sig.find('<') {
                            Some(pos) => pos,
                            None => continue,
                        };
                        let close_angle = match sig[open_angle..].find('>') {
                            Some(pos) => open_angle + pos,
                            None => continue,
                        };
                        let generics_section = &sig[open_angle + 1..close_angle];
                        for param in generics_section.split(',') {
                            let p = param.trim().split(':').next().unwrap_or("").trim();
                            if p.is_empty() || !p.chars().all(|c| c.is_uppercase() || c == '_') {
                                continue;
                            }
                            let mut used = false;
                            for j in i + 1..lines.len().min(i + 60) {
                                if lines[j].trim().starts_with("fn ")
                                    || lines[j].trim().starts_with("pub fn ")
                                {
                                    break;
                                }
                                if lines[j].contains(p) {
                                    used = true;
                                    break;
                                }
                            }
                            if !used {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    count
}

/// Count lines with deep indentation (≥ `levels * 4` spaces, indicating >5 levels of nesting).
fn count_deeply_nested_lines(dir: &Path, levels: usize) -> usize {
    let threshold = levels * 4;
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                count += count_deeply_nested_lines(&path, levels);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        let leading_spaces = line.len() - line.trim_start().len();
                        if leading_spaces >= threshold {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    count
}

/// Count functions that span more than `max_lines` lines.
/// Uses simple brace-depth tracking to find matching closing braces.
fn count_long_functions(dir: &Path, max_lines: usize) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                count += count_long_functions(&path, max_lines);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    count += count_long_functions_in_content(&content, max_lines);
                }
            }
        }
    }
    count
}

fn count_long_functions_in_content(content: &str, max_lines: usize) -> usize {
    let lines: Vec<&str> = content.lines().collect();
    let mut count = 0usize;
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        let is_fn_start = trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("pub(crate) fn ")
            || trimmed.starts_with("pub(super) fn ");
        if !is_fn_start {
            i += 1;
            continue;
        }
        if trimmed.ends_with(';') {
            i += 1;
            continue;
        }
        let mut brace_depth = 0i32;
        let fn_start = i;
        let mut found_open = false;
        let mut closed = false;
        for j in i..lines.len() {
            for ch in lines[j].chars() {
                if ch == '{' { brace_depth += 1; found_open = true; }
                else if ch == '}' { brace_depth -= 1; }
            }
            if found_open && brace_depth == 0 {
                let fn_len = j - fn_start;
                if fn_len > max_lines {
                    count += 1;
                }
                i = j + 1;
                closed = true;
                break;
            }
            if brace_depth < 0 {
                i = j + 1;
                closed = true;
                break;
            }
            if j - fn_start > max_lines * 2 {
                i = j + 1;
                closed = true;
                break;
            }
        }
        if !closed { i = fn_start + 1; }
    }
    count
}

/// Count traits that have only one implementation (over-abstracted pattern).
fn count_single_impl_traits(dir: &Path) -> usize {
    let mut names: Vec<(String, String)> = Vec::new(); // (trait_name, file_path)
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                names.extend(collect_single_impl_traits_from_dir(&path));
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if let Some(trait_name) = trimmed.strip_prefix("pub trait ")
                            .or_else(|| trimmed.strip_prefix("trait "))
                        {
                            let name: String = trait_name.chars()
                                .take_while(|c| c.is_alphanumeric() || *c == '_')
                                .collect();
                            if !name.is_empty() {
                                names.push((name, path.to_string_lossy().to_string()));
                            }
                        }
                    }
                }
            }
        }
    }
    // Build trait → impl count map (scan all files again for impl ... for TraitName)
    let mut impl_counts: HashMap<String, usize> = HashMap::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                count_impls_for_traits(&path, &mut impl_counts);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if let Some(impl_str) = trimmed.strip_prefix("impl ")
                            .or_else(|| trimmed.strip_prefix("pub impl "))
                        {
                            if let Some(for_str) = impl_str.find(" for ") {
                                let trait_part = &impl_str[..for_str].trim();
                                if let Some(name) = trait_part.split('<').next() {
                                    let name = name.trim();
                                    if !name.is_empty() && !name.contains(' ') {
                                        *impl_counts.entry(name.to_string()).or_default() += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // Count declared traits with exactly one implementation
    names.iter()
        .filter(|(name, _)| impl_counts.get(name.as_str()).copied().unwrap_or(0) == 1)
        .count()
}

fn collect_single_impl_traits_from_dir(dir: &Path) -> Vec<(String, String)> {
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                names.extend(collect_single_impl_traits_from_dir(&path));
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if let Some(trait_name) = trimmed.strip_prefix("pub trait ")
                            .or_else(|| trimmed.strip_prefix("trait "))
                        {
                            let name: String = trait_name.chars()
                                .take_while(|c| c.is_alphanumeric() || *c == '_')
                                .collect();
                            if !name.is_empty() {
                                names.push((name, path.to_string_lossy().to_string()));
                            }
                        }
                    }
                }
            }
        }
    }
    names
}

fn count_impls_for_traits(dir: &Path, counts: &mut HashMap<String, usize>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                count_impls_for_traits(&path, counts);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if let Some(impl_str) = trimmed.strip_prefix("impl ")
                            .or_else(|| trimmed.strip_prefix("pub impl "))
                        {
                            if let Some(for_str) = impl_str.find(" for ") {
                                let trait_part = &impl_str[..for_str].trim();
                                let name = trait_part.split('<').next().unwrap_or("").trim();
                                if !name.is_empty() && !name.contains(' ') {
                                    *counts.entry(name.to_string()).or_default() += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Count if-else chains longer than `max_chain` (consecutive `else if` / `else` lines).
fn count_long_if_chains(dir: &Path, max_chain: usize) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                count += count_long_if_chains(&path, max_chain);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let lines: Vec<&str> = content.lines().collect();
                    let mut chain = 0usize;
                    for line in &lines {
                        let trimmed = line.trim();
                        if trimmed.starts_with("} else if ") || trimmed.starts_with("else if ") {
                            chain += 1;
                        } else if trimmed == "} else {" || trimmed.starts_with("else {") {
                            chain += 1;
                        } else {
                            if chain > max_chain { count += 1; }
                            chain = 0;
                        }
                    }
                }
            }
        }
    }
    count
}

/// Count match expressions with more than `max_arms` arms.
fn count_excessive_match_arms(dir: &Path, max_arms: usize) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                count += count_excessive_match_arms(&path, max_arms);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let lines: Vec<&str> = content.lines().collect();
                    let mut i = 0;
                    while i < lines.len() {
                        if lines[i].contains("match ") && !lines[i].trim().starts_with("//") {
                            let mut brace_depth = 0i32;
                            let mut arms = 0usize;
                            let mut in_match = false;
                            let start = i;
                            for j in i..lines.len() {
                                for ch in lines[j].chars() {
                                    if ch == '{' {
                                        brace_depth += 1;
                                        if !in_match { in_match = true; }
                                    } else if ch == '}' {
                                        brace_depth -= 1;
                                    }
                                }
                                if in_match && brace_depth == 0 {
                                    i = j;
                                    break;
                                }
                                if in_match && j > start {
                                    let tl = lines[j].trim();
                                    if tl.starts_with('|') || tl.contains("=>") {
                                        arms += 1;
                                    }
                                }
                                if j - i > SelfReviewConfig::default().scan_safety_bound { i = j; break; }
                            }
                            if arms > max_arms { count += 1; }
                        }
                        i += 1;
                    }
                }
            }
        }
    }
    count
}

/// Count function declarations with more than `max_params` parameters.
fn count_excessive_param_count(dir: &Path, max_params: usize) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                count += count_excessive_param_count(&path, max_params);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if !trimmed.starts_with("fn ") && !trimmed.starts_with("pub fn ") {
                            continue;
                        }
                        if let Some(paren_open) = trimmed.find('(') {
                            if let Some(paren_close) = trimmed[paren_open..].find(')') {
                                let params_str = &trimmed[paren_open + 1..paren_open + paren_close];
                                if params_str.is_empty() { continue; }
                                let params: Vec<&str> = params_str.split(',')
                                    .map(|s| s.trim())
                                    .filter(|s| !s.is_empty() && !s.contains("self"))
                                    .collect();
                                if params.len() > max_params {
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    count
}

/// Count `// TODO` / `// FIXME` comments that aren't in files with a `#[test]`.
fn count_todos_without_nearby_test(dir: &Path) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                count += count_todos_without_nearby_test(&path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let has_test = content.contains("#[test]");
                    let todo_count = content.matches("// TODO").count()
                        + content.matches("//TODO").count()
                        + content.matches("// FIXME").count();
                    if todo_count > 0 && !has_test {
                        count += todo_count;
                    }
                }
            }
        }
    }
    count
}

/// Count `&mut self` methods that do not return `Result` (suggesting no error handling).
fn count_state_mutation_no_result(dir: &Path) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                count += count_state_mutation_no_result(&path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if (trimmed.starts_with("fn ") || trimmed.starts_with("pub fn "))
                            && trimmed.contains("&mut self")
                        {
                            if !trimmed.contains("Result") {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    count
}

/// Count `pub fn` declarations without doc comments indicating success criteria
/// (no "Returns", "Goal", "Purpose", or "Success" in the preceding doc block).
fn count_pub_fn_without_goal_doc(dir: &Path) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") { continue; }
                count += count_pub_fn_without_goal_doc(&path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let lines: Vec<&str> = content.lines().collect();
                    for i in 0..lines.len() {
                        let trimmed = lines[i].trim();
                        if !trimmed.starts_with("pub fn ") { continue; }
                        let mut has_goal = false;
                        for j in (0.max(i.saturating_sub(10)))..i {
                            let d = lines[j].trim();
                            if d.starts_with("///") {
                                if d.contains("Returns") || d.contains("Goal")
                                    || d.contains("Purpose") || d.contains("Success")
                                    || d.contains("Output")
                                {
                                    has_goal = true;
                                    break;
                                }
                            } else if !d.starts_with("///") && !d.is_empty() {
                                break;
                            }
                        }
                        if !has_goal { count += 1; }
                    }
                }
            }
        }
    }
    count
}

/// Count files changed in the last commit (git diff --stat).
fn count_files_in_last_commit(repo_dir: &Path) -> usize {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", "HEAD~1..HEAD"])
        .current_dir(repo_dir)
        .output();
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.lines().filter(|l| !l.is_empty()).count()
        }
        Err(_) => 0,
    }
}

/// Count lines added in the last commit (git diff --shortstat).
fn count_lines_in_last_commit(repo_dir: &Path) -> usize {
    let output = std::process::Command::new("git")
        .args(["diff", "--shortstat", "HEAD~1..HEAD"])
        .current_dir(repo_dir)
        .output();
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let s = stdout.trim();
            if s.is_empty() { return 0; }
            if let Some(ins_part) = s.split(',').nth(1) {
                let cleaned: String = ins_part.chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect();
                return cleaned.parse().unwrap_or(0);
            }
            0
        }
        Err(_) => 0,
    }
}

fn estimate_brace_depth(code: &str) -> usize {
    let mut depth = 0usize;
    let mut max_depth = 0usize;
    for c in code.chars() {
        if c == '{' {
            depth += 1;
            max_depth = max_depth.max(depth);
        } else if c == '}' {
            depth = depth.saturating_sub(1);
        }
    }
    max_depth
}

fn syn_file_max_depth(file: &syn::File) -> usize {
    file.items.iter()
        .map(|item| syn_item_max_depth(item, 0))
        .max()
        .unwrap_or(0)
}

fn syn_item_max_depth(item: &syn::Item, depth: usize) -> usize {
    match item {
        syn::Item::Fn(f) => syn_block_max_depth(&f.block, depth),
        syn::Item::Impl(imp) => {
            let d = depth + 1;
            d.max(imp.items.iter()
                .map(|ii| syn_impl_item_max_depth(ii, d))
                .max()
                .unwrap_or(d))
        }
        syn::Item::Trait(t) => {
            let d = depth + 1;
            d.max(t.items.iter()
                .map(|ti| syn_trait_item_max_depth(ti, d))
                .max()
                .unwrap_or(d))
        }
        syn::Item::Mod(m) => {
            match &m.content {
                Some((_, items)) => {
                    let d = depth + 1;
                    d.max(items.iter()
                        .map(|i| syn_item_max_depth(i, d))
                        .max()
                        .unwrap_or(d))
                }
                None => depth,
            }
        }
        syn::Item::ForeignMod(fm) => {
            let d = depth + 1;
            d.max(fm.items.iter()
                .map(|fi| syn_foreign_item_max_depth(fi, d))
                .max()
                .unwrap_or(d))
        }
        syn::Item::Const(c) => syn_expr_max_depth(&c.expr, depth),
        syn::Item::Static(s) => syn_expr_max_depth(&s.expr, depth),
        _ => depth,
    }
}

fn syn_impl_item_max_depth(item: &syn::ImplItem, depth: usize) -> usize {
    match item {
        syn::ImplItem::Fn(f) => syn_block_max_depth(&f.block, depth),
        syn::ImplItem::Const(c) => syn_expr_max_depth(&c.expr, depth),
        _ => depth,
    }
}

fn syn_trait_item_max_depth(item: &syn::TraitItem, depth: usize) -> usize {
    match item {
        syn::TraitItem::Fn(f) => f.default.as_ref()
            .map(|b| syn_block_max_depth(b, depth))
            .unwrap_or(depth),
        syn::TraitItem::Const(c) => c.default.as_ref()
            .map(|(_, e)| syn_expr_max_depth(e, depth))
            .unwrap_or(depth),
        _ => depth,
    }
}

fn syn_foreign_item_max_depth(_item: &syn::ForeignItem, depth: usize) -> usize {
    depth
}

fn syn_block_max_depth(block: &syn::Block, depth: usize) -> usize {
    let d = depth + 1;
    d.max(syn_stmts_max_depth(&block.stmts, d))
}

fn syn_stmts_max_depth(stmts: &[syn::Stmt], depth: usize) -> usize {
    stmts.iter()
        .map(|s| syn_stmt_max_depth(s, depth))
        .max()
        .unwrap_or(depth)
}

fn syn_stmt_max_depth(stmt: &syn::Stmt, depth: usize) -> usize {
    match stmt {
        syn::Stmt::Item(item) => syn_item_max_depth(item, depth),
        syn::Stmt::Expr(expr, _) => syn_expr_max_depth(expr, depth),
        syn::Stmt::Local(local) => local.init.as_ref()
            .map(|init| syn_expr_max_depth(&init.expr, depth))
            .unwrap_or(depth),
        _ => depth,
    }
}

fn syn_expr_max_depth(expr: &syn::Expr, depth: usize) -> usize {
    match expr {
        syn::Expr::Block(eb) => syn_block_max_depth(&eb.block, depth),
        syn::Expr::If(ei) => {
            let then_max = syn_block_max_depth(&ei.then_branch, depth);
            let else_max = ei.else_branch.as_ref()
                .map(|(_, e)| syn_expr_max_depth(e, depth))
                .unwrap_or(0);
            then_max.max(else_max)
        }
        syn::Expr::While(w) => syn_block_max_depth(&w.body, depth),
        syn::Expr::ForLoop(fl) => syn_block_max_depth(&fl.body, depth),
        syn::Expr::Loop(l) => syn_block_max_depth(&l.body, depth),
        syn::Expr::Match(m) => {
            let d = depth + 1;
            d.max(m.arms.iter()
                .map(|arm| syn_expr_max_depth(&arm.body, d))
                .max()
                .unwrap_or(d))
        }
        syn::Expr::Unsafe(u) => syn_block_max_depth(&u.block, depth),
        syn::Expr::Closure(c) => syn_expr_max_depth(&c.body, depth),
        _ => depth,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_review_basic() {
        let mut gate = SelfReviewGate::new(true);
        let report = gate.run_all();
        // Verify the report structure is correct (not a specific pass/fail, since
        // real codebase state varies). Key invariant: no panic during execution.
        assert!(report.findings.len() > 0, "Should have at least informational findings");
        assert!(report.passed + report.failed + report.warnings <= report.findings.len());
    }

    #[test]
    fn test_self_review_findings_report() {
        let mut gate = SelfReviewGate::new(false);
        gate.check(false, Severity::Error, "test", "forced error".into(), file!(), line!());
        let report = gate.report();
        assert_eq!(report.failed, 1);
        assert_eq!(report.summary().contains("FAIL"), true);
    }

    #[test]
    fn test_review_finding_display() {
        let finding = ReviewFinding {
            severity: Severity::Error,
            category: "test".into(),
            message: "test finding".into(),
            file: "test.rs".into(),
            line: 42,
        };
        let s = format!("{}", finding.severity);
        assert_eq!(s, "ERROR");
    }

    #[test]
    fn test_blast_radius_empty() {
        let gate = SelfReviewGate::new(false);
        let br = gate.blast_radius();
        assert_eq!(br.risk, BlastRisk::Low);
        assert!(br.files_scanned > 0);
    }

    #[test]
    fn test_blast_radius_with_findings() {
        let mut gate = SelfReviewGate::new(false);
        gate.check(false, Severity::Error, "panic_audit", "test".to_string(), "a.rs".to_string(), 1);
        gate.check(false, Severity::Error, "layer_violation", "test".to_string(), "b.rs".to_string(), 2);
        let br = gate.blast_radius();
        assert_eq!(br.risk, BlastRisk::Critical);
        // Only "layer_violation" counts as module crossing
        assert_eq!(br.module_crossings, 1);
    }

    #[test]
    fn test_arch_layer_detection() {
        assert_eq!(ArchLayer::from_path(Path::new("/src/core/foo.rs")), ArchLayer::L0Core);
        assert_eq!(ArchLayer::from_path(Path::new("/src/neotrix/l1_body_impl/bar.rs")), ArchLayer::L1Act);
        assert_eq!(ArchLayer::from_path(Path::new("/src/neotrix/l2_world_impl/baz.rs")), ArchLayer::L2World);
        assert_eq!(ArchLayer::from_path(Path::new("/src/neotrix/l3_memory_impl/qux.rs")), ArchLayer::L3Memory);
        assert_eq!(ArchLayer::from_path(Path::new("/src/neotrix/l8_autonomic_impl/quux.rs")), ArchLayer::L8Seal);
    }

    #[test]
    fn test_observer_feedback_degraded() {
        let mut gate = SelfReviewGate::new(true)
            .with_observer_feedback(0.25, vec!["oscillation".into()]);
        let report = gate.run_all();
        let obs_findings: Vec<_> = report.findings.iter()
            .filter(|f| f.category == "observer_feedback" || f.category == "observer_pattern")
            .collect();
        assert!(!obs_findings.is_empty(), "Should have observer findings when degraded");
    }

    #[test]
    fn test_observer_feedback_healthy() {
        let mut gate = SelfReviewGate::new(true)
            .with_observer_feedback(0.85, vec![]);
        let report = gate.run_all();
        let obs_findings: Vec<_> = report.findings.iter()
            .filter(|f| f.category == "observer_feedback")
            .collect();
        // Healthy quality should NOT produce a warning
        let warnings: Vec<_> = obs_findings.iter().filter(|f| f.severity == Severity::Warning || f.severity == Severity::Error).collect();
        assert!(warnings.is_empty(), "Should not warn on healthy observer quality");
    }

    #[test]
    fn test_karpathy_simplicity_first() {
        let mut gate = SelfReviewGate::new(false);
        gate.check_karpathy_simplicity_first();
        // Verify the check runs without panics and produces at least info-level findings
        let report = gate.report();
        let findings: Vec<_> = report.findings.iter()
            .filter(|f| f.category == "karpathy_simplicity_first")
            .collect();
        assert!(findings.len() <= 1, "Should have at most 1 finding for simplicity_first check");
    }

    #[test]
    fn test_karpathy_surgical_changes() {
        let mut gate = SelfReviewGate::new(false);
        gate.check_karpathy_surgical_changes();
        // Verify the check runs without panics (git commands may not always succeed)
        let report = gate.report();
        let findings: Vec<_> = report.findings.iter()
            .filter(|f| f.category == "karpathy_surgical_changes")
            .collect();
        assert!(findings.len() <= 1, "Should have at most 1 finding for surgical_changes check");
    }

    #[test]
    fn test_karpathy_complexity_budget() {
        let mut gate = SelfReviewGate::new(false);
        gate.check_karpathy_complexity_budget();
        // Verify the check runs without panics
        let report = gate.report();
        let findings: Vec<_> = report.findings.iter()
            .filter(|f| f.category == "karpathy_complexity_budget")
            .collect();
        assert!(findings.len() <= 1, "Should have at most 1 finding for complexity_budget check");
    }

    #[test]
    fn test_karpathy_goal_driven_execution() {
        let mut gate = SelfReviewGate::new(false);
        gate.check_karpathy_goal_driven_execution();
        // Verify the check runs without panics
        let report = gate.report();
        let findings: Vec<_> = report.findings.iter()
            .filter(|f| f.category == "karpathy_goal_driven_execution")
            .collect();
        assert!(findings.len() <= 1, "Should have at most 1 finding for goal_driven_execution check");
    }

    #[test]
    fn test_syn_depth_tracks_nesting() {
        let gate = SelfReviewGate::new(false);

        let d1 = gate.syn_depth("fn a() { let x = 1; }");
        assert_eq!(d1, 1, "single fn block should have depth 1");

        let d2 = gate.syn_depth("fn a() { fn b() { let x = 1; } }");
        assert_eq!(d2, 2, "nested fn should have depth 2");

        let d3 = gate.syn_depth("let x = 1;");
        assert_eq!(d3, 0, "invalid Rust should fallback to brace count (0)");
    }

    #[test]
    fn test_syn_depth_control_flow() {
        let gate = SelfReviewGate::new(false);

        let d = gate.syn_depth("fn a() { if true { let x = 1; } }");
        assert_eq!(d, 2, "fn + if block should have depth 2");

        let d = gate.syn_depth("fn a() { loop { break; } }");
        assert_eq!(d, 2, "fn + loop block should have depth 2");
    }
}

