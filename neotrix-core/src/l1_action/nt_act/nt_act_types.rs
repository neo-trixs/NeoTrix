//! NeoTrix NT-ACT types
//!
//! Canonical shared types for L1 Action layer.
//! L5 (nt_mind) and other consumers import from here instead of duplicating.

use serde::{Deserialize, Serialize};

/// Canonical project health snapshot — single source of truth for project metrics.
///
/// Used by: evolution loop (L5), bench (L1), goal generator (L1), self-diagnose (L5),
/// evolution daemon (L5). All layers share this type via L1 (no L5→L1 dependency).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ProjectSnapshot {
    pub total_files: usize,
    pub total_lines: usize,
    pub large_files: Vec<String>,
    pub modules_without_tests: Vec<String>,
    pub file_unsafe_hotspots: Vec<String>,
    pub unsafe_count: usize,
    pub unwrap_count: usize,
    pub todo_count: usize,
    pub compile_errors: usize,
    pub compile_warnings: usize,
    pub test_count: usize,
    pub test_failures: usize,
}

