//! Self-review data types — findings, reports, config, gate shape.
//! Pure move from `mod.rs` (God-file split) — zero behavior change.

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use neotrix_types::shared::Severity;
use serde::{Deserialize, Serialize};

// ─── moved: mod.rs:27-58 ReviewFinding/SelfReviewReport ───
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

// ─── moved: mod.rs:60-93 BlastRadiusReport/BlastRisk ───
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

// ─── moved: mod.rs:95-154 ArchLayer ───
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
        if p.contains("l0_core") || p.contains("/core/") {
            Self::L0Core
        } else if p.contains("l1_body") || p.contains("l1_act") {
            Self::L1Act
        } else if p.contains("l2_world") {
            Self::L2World
        } else if p.contains("l3_memory") {
            Self::L3Memory
        } else if p.contains("l4_cognition") {
            Self::L4Cognition
        } else if p.contains("l5_prm") || p.contains("nt_core_prm") {
            Self::L5Prm
        } else if p.contains("l6_self") || p.contains("l6_autonomic") {
            Self::L6Self
        } else if p.contains("l7_capability") {
            Self::L7Capability
        } else if p.contains("l8_autonomic") || p.contains("l8_seal") {
            Self::L8Seal
        } else if p.contains("l9_transcendent") {
            Self::L9Transcendent
        } else {
            Self::Unknown
        }
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

// ─── moved: mod.rs:156-209 SelfReviewConfig ───
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

// ─── moved: mod.rs:211-232 SelfReviewGate struct+Default ───
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

// ─── moved: mod.rs:1633-1646 PatternConfig/PatternMatch ───
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
