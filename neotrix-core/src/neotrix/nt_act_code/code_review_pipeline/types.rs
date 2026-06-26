use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum DiffStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiffLineType {
    Context,
    Addition,
    Deletion,
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_count: u32,
    pub new_start: u32,
    pub new_count: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
pub struct ReviewFileDiff {
    pub file: String,
    pub status: DiffStatus,
    pub old_path: Option<String>,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IssueCategory {
    Security,
    Performance,
    Correctness,
    Style,
    ErrorHandling,
    UnsafeCode,
    Testing,
    Documentation,
    Maintainability,
    Compliance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    pub id: String,
    pub file: String,
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub message: String,
    pub existing_code: String,
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
    pub suggestion: Option<String>,
    /// Anchor lines survive code modification — used for drift detection on re-review.
    pub anchor_lines: Vec<String>,
    /// Confidence of the evidence match (0.0–1.0).
    pub match_confidence: f32,
    /// Set true when re-reviewing and anchors don't match the current diff.
    pub needs_relocation: bool,
}

/// Minimum confidence threshold below which a comment triggers re-location.
pub const RELOCATION_THRESHOLD: f32 = 0.70;

/// Request to re-locate a comment via LLM when evidence-based matching fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelocationRequest {
    pub comment_id: String,
    pub file: String,
    pub existing_code: String,
    pub message: String,
    pub current_confidence: f32,
}

#[derive(Debug, Clone)]
pub struct ReviewResult {
    pub comments: Vec<ReviewComment>,
    pub file_count: usize,
    pub comment_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}
