use serde::{Deserialize, Serialize};

use super::critic::CriticNode;
use crate::neotrix::nt_act_code::code_review_pipeline::{
    CommentResolver, IssueCategory, IssueSeverity, ReviewComment, ReviewFileDiff,
};

// ─── RiskLevel ───

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

// ─── FileMetadata ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: String,
    pub language: String,
    pub scope: String,
    pub is_test: bool,
    pub old_path: Option<String>,
    pub added_lines: u32,
    pub deleted_lines: u32,
}

// ─── PlanOutput ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanOutput {
    pub change_summary: String,
    pub suspected_issues: Vec<String>,
    pub scope: String,
    pub risk_level: RiskLevel,
}

// ─── FilteredOutput ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteredOutput {
    pub kept: Vec<ReviewComment>,
    pub eliminated: Vec<(ReviewComment, String)>,
    pub needs_human_review: Vec<ReviewComment>,
}

impl FilteredOutput {
    pub fn all_kept_ids(&self) -> Vec<String> {
        self.kept.iter().map(|c| c.id.clone()).collect()
    }

    pub fn total_eliminated(&self) -> usize {
        self.eliminated.len()
    }

    pub fn has_actionable(&self) -> bool {
        !self.kept.is_empty()
    }
}

// ─── Tool system for MainStage ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewTool {
    ReadFile(String, u32, u32),
    SearchCode(String),
    CheckDependency(String),
    LookupRule(String),
    GetFunctionBody(String),
    GetTypeDefinition(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreeStageToolResult {
    FileContent(String),
    SearchResults(Vec<String>),
    DependencyInfo(String),
    RuleDescription(String),
    FunctionBody(String),
    TypeDefinition(String),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub tool: ReviewTool,
    pub result_summary: String,
}

// ─── PlanStage trait ───

pub trait PlanStage: Send + Sync + std::fmt::Debug {
    /// Analyze the diff without external tools.
    /// Returns a structured plan output.
    fn analyze(&self, diff: &str, metadata: &FileMetadata) -> PlanOutput;
}

// ─── MainStage trait ───

pub trait MainStage: Send + Sync + std::fmt::Debug {
    /// Synthesize review comments using plan output and available tools.
    /// Returns comments with their tool call audit trail.
    fn review(
        &self,
        plan: &PlanOutput,
        diff: &str,
        metadata: &FileMetadata,
    ) -> (Vec<ReviewComment>, Vec<ToolCallRecord>);
}

// ─── FilterStage trait ───

pub trait FilterStage: Send + Sync + std::fmt::Debug {
    /// Adversarially filter false positives from review comments.
    fn filter(
        &self,
        plan: &PlanOutput,
        comments: &[ReviewComment],
        diff: &str,
        metadata: &FileMetadata,
    ) -> FilteredOutput;
}

// ─── ThreeStagePipeline orchestrator ───

#[derive(Debug)]
pub struct ThreeStagePipeline {
    pub plan_stage: Box<dyn PlanStage>,
    pub main_stage: Box<dyn MainStage>,
    pub filter_stage: Box<dyn FilterStage>,
    pub comment_resolver: CommentResolver,
}

impl ThreeStagePipeline {
    pub fn new(
        plan_stage: Box<dyn PlanStage>,
        main_stage: Box<dyn MainStage>,
        filter_stage: Box<dyn FilterStage>,
    ) -> Self {
        Self {
            plan_stage,
            main_stage,
            filter_stage,
            comment_resolver: CommentResolver::new(),
        }
    }

    pub fn with_comment_resolver(mut self, resolver: CommentResolver) -> Self {
        self.comment_resolver = resolver;
        self
    }

    /// Run the full three-stage pipeline for a single file diff.
    pub fn review_file(
        &self,
        diff: &str,
        metadata: &FileMetadata,
        diffs: &[ReviewFileDiff],
    ) -> FilteredOutput {
        let plan = self.plan_stage.analyze(diff, metadata);
        let (mut comments, _tool_trail) = self.main_stage.review(&plan, diff, metadata);

        self.comment_resolver.resolve_comments(&mut comments, diffs);

        self.filter_stage.filter(&plan, &comments, diff, metadata)
    }
}

// ─── SimplePlanStage (deterministic implementation) ───

#[derive(Debug, Clone)]
pub struct SimplePlanStage;

impl SimplePlanStage {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimplePlanStage {
    fn default() -> Self {
        Self::new()
    }
}

impl PlanStage for SimplePlanStage {
    fn analyze(&self, diff: &str, metadata: &FileMetadata) -> PlanOutput {
        let added = metadata.added_lines;
        let deleted = metadata.deleted_lines;
        let is_new = metadata.old_path.is_none() && added > 0 && deleted == 0;

        let change_summary = if is_new {
            format!("New file '{}' with {} lines added", metadata.path, added)
        } else if deleted > added * 2 {
            format!(
                "Significant deletion in '{}': {} lines removed, {} added",
                metadata.path, deleted, added
            )
        } else if added > deleted * 2 {
            format!(
                "Major addition in '{}': {} lines added, {} deleted",
                metadata.path, added, deleted
            )
        } else {
            format!(
                "Modification in '{}': +{} / -{} lines",
                metadata.path, added, deleted
            )
        };

        let mut suspected_issues: Vec<String> = Vec::new();
        let diff_lower = diff.to_lowercase();
        let additions_text: String = diff
            .lines()
            .filter(|l| l.starts_with('+'))
            .map(|l| &l[1..])
            .collect::<Vec<&str>>()
            .join("\n");

        if additions_text.contains("unsafe") {
            suspected_issues.push("Unsafe code usage — verify safety invariants".into());
        }
        if additions_text.contains(".unwrap()") || additions_text.contains(".expect(") {
            suspected_issues.push("Potential panic from unwrap/expect — prefer ? operator".into());
        }
        if additions_text.contains("password")
            || additions_text.contains("secret")
            || additions_text.contains("api_key")
        {
            suspected_issues.push("Sensitive data exposure risk".into());
        }
        if diff_lower.contains("todo") || diff_lower.contains("fixme") {
            suspected_issues.push("Incomplete code (TODO/FIXME) — address before merge".into());
        }
        if diff_lower.contains("dbg!") {
            suspected_issues.push("Debug print statement (dbg!) should be removed".into());
        }
        if additions_text.contains("panic!") {
            suspected_issues.push("Panic in production code — return Result instead".into());
        }

        let scope = if additions_text.contains("unsafe")
            || additions_text.contains("password")
            || additions_text.contains("secret")
        {
            "security".into()
        } else if additions_text.contains("loop")
            || additions_text.contains("O(n")
            || additions_text.contains("clone")
        {
            "performance".into()
        } else if additions_text.contains("unwrap")
            || additions_text.contains("expect")
            || additions_text.contains("panic")
        {
            "correctness".into()
        } else if metadata.is_test {
            "testing".into()
        } else {
            "general".into()
        };

        let risk_level = if suspected_issues
            .iter()
            .any(|i| i.contains("Panic") || i.contains("Sensitive") || i.contains("Unsafe"))
        {
            RiskLevel::High
        } else if !suspected_issues.is_empty() {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        PlanOutput {
            change_summary,
            suspected_issues,
            scope,
            risk_level,
        }
    }
}

// ─── SimpleMainStage (deterministic implementation) ───

#[derive(Debug, Clone)]
pub struct SimpleMainStage;

impl SimpleMainStage {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleMainStage {
    fn default() -> Self {
        Self::new()
    }
}

fn classify_severity_by_text(text: &str) -> IssueSeverity {
    if text.contains("unsafe")
        || text.contains("panic!")
        || text.contains("password")
        || text.contains("secret")
        || text.contains("sslmode=disable")
    {
        IssueSeverity::Critical
    } else if text.contains("unwrap")
        || text.contains("expect")
        || text.contains("unreachable")
        || text.contains("injection")
        || text.contains("vulnerability")
    {
        IssueSeverity::High
    } else if text.contains("todo!")
        || text.contains("FIXME")
        || text.contains("HACK")
        || text.contains("unimplemented")
    {
        IssueSeverity::Medium
    } else if text.contains("TODO") || text.contains("dbg!") || text.contains("println") {
        IssueSeverity::Low
    } else {
        IssueSeverity::Medium
    }
}

fn classify_category(keyword: &str) -> IssueCategory {
    let kw_lower = keyword.to_lowercase();
    if kw_lower.contains("unsafe")
        || kw_lower.contains("password")
        || kw_lower.contains("secret")
        || kw_lower.contains("injection")
    {
        IssueCategory::UnsafeCode
    } else if kw_lower.contains("unwrap")
        || kw_lower.contains("expect")
        || kw_lower.contains("error")
        || kw_lower.contains("panic")
    {
        IssueCategory::ErrorHandling
    } else if kw_lower.contains("loop")
        || kw_lower.contains("clone")
        || kw_lower.contains("alloc")
        || kw_lower.contains("n+1")
    {
        IssueCategory::Performance
    } else if kw_lower.contains("test") {
        IssueCategory::Testing
    } else if kw_lower.contains("doc") || kw_lower.contains("comment") {
        IssueCategory::Documentation
    } else if kw_lower.contains("todo") || kw_lower.contains("fixme") || kw_lower.contains("hack") {
        IssueCategory::Maintainability
    } else {
        IssueCategory::Correctness
    }
}

impl MainStage for SimpleMainStage {
    fn review(
        &self,
        plan: &PlanOutput,
        diff: &str,
        metadata: &FileMetadata,
    ) -> (Vec<ReviewComment>, Vec<ToolCallRecord>) {
        let mut comments = Vec::new();
        let mut records = Vec::new();
        let mut comment_id = 0u32;

        let additions: Vec<String> = diff
            .lines()
            .filter(|l| l.starts_with('+'))
            .map(|l| l[1..].to_string())
            .collect();

        if plan.suspected_issues.is_empty() && additions.is_empty() {
            return (comments, records);
        }

        let patterns: Vec<(&str, &str, Option<&str>)> = vec![
            (
                ".unwrap()",
                "Unsafe unwrap call — may panic at runtime",
                Some("Prefer the ? operator or proper match/if-let handling"),
            ),
            (
                ".expect(",
                "Unsafe expect call — panics with custom message",
                Some("Prefer the ? operator or proper error propagation"),
            ),
            (
                "panic!",
                "Panic in production code — crashes the process",
                Some("Return Result<T, E> instead of panicking"),
            ),
            (
                "unsafe ",
                "Unsafe block — bypasses Rust's safety guarantees",
                Some("Verify safety invariants and add // SAFETY: comment"),
            ),
            (
                "todo!",
                "Unimplemented code path — will panic at runtime",
                Some("Implement before merging"),
            ),
            (
                "unreachable!",
                "Unreachable assertion — may hide logic bugs",
                Some("Verify this path is truly unreachable at compile time"),
            ),
            (
                "dbg!",
                "Debug print statement left in production code",
                Some("Remove dbg! before merging"),
            ),
            (
                "sslmode=disable",
                "SSL/TLS disabled for database connection",
                Some("Enable SSL for secure database communication"),
            ),
            (
                "password=",
                "Hardcoded password in source code",
                Some("Use environment variables or a secrets manager"),
            ),
            (
                "api_key",
                "Potential API key hardcoded in source",
                Some("Use environment variables or a secrets manager"),
            ),
            (
                "secret",
                "Potential secret hardcoded in source",
                Some("Use environment variables or a secrets manager"),
            ),
        ];

        let addition_text: String = additions.join("\n");

        'patterns: for (pattern, message, suggestion) in &patterns {
            for (line_idx, line) in addition_text.lines().enumerate() {
                if line.contains(pattern) {
                    let line_trimmed = line.trim();
                    let full_message = if line_trimmed.len() > 120 {
                        format!("{}: {}", message, &line_trimmed[..120])
                    } else {
                        format!("{}: {}", message, line_trimmed)
                    };

                    let severity = classify_severity_by_text(pattern);
                    let category = classify_category(pattern);
                    let new_line_number = (line_idx + 1) as u32;

                    comments.push(ReviewComment {
                        id: format!("c-{}", comment_id),
                        file: metadata.path.clone(),
                        severity,
                        category,
                        message: full_message,
                        existing_code: line_trimmed.to_string(),
                        start_line: Some(new_line_number),
                        end_line: Some(new_line_number),
                        suggestion: suggestion.map(|s| s.to_string()),
                        anchor_lines: vec![line_trimmed.to_string()],
                        match_confidence: 0.85,
                        needs_relocation: false,
                    });
                    comment_id += 1;

                    records.push(ToolCallRecord {
                        tool: ReviewTool::SearchCode(pattern.to_string()),
                        result_summary: format!("Found '{}' at line {}", pattern, new_line_number),
                    });

                    continue 'patterns;
                }
            }
        }

        let mut diagnostics: Vec<(IssueSeverity, IssueCategory, String, String, Option<String>)> =
            Vec::new();

        if plan.scope == "security" && metadata.language == "rust" {
            if !addition_text.contains("// SAFETY:") && addition_text.contains("unsafe") {
                diagnostics.push((
                    IssueSeverity::High,
                    IssueCategory::UnsafeCode,
                    "Unsafe block without safety comment — add // SAFETY: explaining invariants"
                        .into(),
                    addition_text
                        .lines()
                        .find(|l| l.contains("unsafe"))
                        .unwrap_or("unsafe { }")
                        .trim()
                        .to_string(),
                    Some("Add // SAFETY: <reason> comment above each unsafe block".into()),
                ));
            }
        }

        if metadata.language == "rust" {
            let open_brace = addition_text.matches('{').count();
            let close_brace = addition_text.matches('}').count();
            if open_brace > close_brace + 2 {
                diagnostics.push((
                    IssueSeverity::Medium,
                    IssueCategory::Correctness,
                    "Possible unbalanced braces — more '{' than '}'".into(),
                    format!(
                        "{:+} braces imbalance",
                        open_brace as i32 - close_brace as i32
                    ),
                    Some("Check for missing closing braces".into()),
                ));
            }
        }

        for (severity, category, message, code, suggestion) in &diagnostics {
            comments.push(ReviewComment {
                id: format!("c-{}", comment_id),
                file: metadata.path.clone(),
                severity: severity.clone(),
                category: category.clone(),
                message: message.clone(),
                existing_code: code.clone(),
                start_line: None,
                end_line: None,
                suggestion: suggestion.clone(),
                anchor_lines: Vec::new(),
                match_confidence: 0.7,
                needs_relocation: false,
            });
            comment_id += 1;

            records.push(ToolCallRecord {
                tool: ReviewTool::LookupRule(metadata.language.clone()),
                result_summary: format!("Applied structural heuristic: {}", message),
            });
        }

        (comments, records)
    }
}

// ─── SimpleFilterStage (deterministic implementation) ───

#[derive(Debug, Clone)]
pub struct SimpleFilterStage;

impl SimpleFilterStage {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleFilterStage {
    fn default() -> Self {
        Self::new()
    }
}

fn simulate_self_consistency_check(comment: &ReviewComment, diff: &str) -> Option<String> {
    let evidence = comment.existing_code.trim();
    if evidence.is_empty() {
        return Some("No evidence code provided — cannot prove from diff alone".into());
    }

    let diff_normalized = diff.to_lowercase();
    let evidence_normalized = evidence.to_lowercase();

    if diff_normalized.contains(&evidence_normalized) {
        return None;
    }

    let evidence_alpha: String = evidence_normalized
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    if evidence_alpha.len() < 5 {
        return None;
    }

    if diff_normalized.contains(&evidence_alpha) {
        return None;
    }

    Some(format!(
        "Cannot prove '{}' from diff alone",
        comment.message.chars().take(60).collect::<String>()
    ))
}

fn simulate_cross_perspective(comment: &ReviewComment) -> Vec<&'static str> {
    let mut challengers = Vec::new();

    let msg = comment.message.to_lowercase();

    let author_agrees = !msg.contains("unreachable")
        && !msg.contains("impossible")
        && !comment.existing_code.trim().is_empty();
    let reviewer_agrees = comment.match_confidence >= 0.6;
    let maintainer_agrees = comment.match_confidence >= 0.7;

    if !author_agrees {
        challengers.push("Author perspective: code is intentional");
    }
    if !reviewer_agrees {
        challengers.push("Reviewer perspective: confidence too low to flag");
    }
    if !maintainer_agrees {
        challengers.push("Maintainer perspective: not critical enough for blocking merge");
    }

    challengers
}

impl FilterStage for SimpleFilterStage {
    fn filter(
        &self,
        plan: &PlanOutput,
        comments: &[ReviewComment],
        diff: &str,
        metadata: &FileMetadata,
    ) -> FilteredOutput {
        let mut kept = Vec::new();
        let mut eliminated = Vec::new();
        let mut needs_human_review = Vec::new();

        let _ = (plan, metadata);

        for comment in comments {
            let mut elimination_reasons = Vec::new();

            let self_consistency_reason = simulate_self_consistency_check(comment, diff);
            if let Some(reason) = self_consistency_reason {
                elimination_reasons.push(format!("(self-consistency) {}", reason));
            }

            let challengers = simulate_cross_perspective(comment);
            if challengers.len() >= 2 {
                elimination_reasons.push(format!(
                    "(cross-perspective) {} perspectives disputed this: {}",
                    challengers.len(),
                    challengers.join("; ")
                ));
            }

            if comment.match_confidence < 0.5 && comment.start_line.is_none() {
                elimination_reasons.push(format!(
                    "(evidence-gate) match_confidence {:.2} < 0.5 with no resolved line position",
                    comment.match_confidence
                ));
            }

            if elimination_reasons.is_empty() {
                kept.push(comment.clone());
            } else if elimination_reasons.len() <= 1
                && comment.severity != IssueSeverity::Low
                && comment.severity != IssueSeverity::Info
            {
                needs_human_review.push(comment.clone());
            } else {
                eliminated.push((comment.clone(), elimination_reasons.join("; ")));
            }
        }

        FilteredOutput {
            kept,
            eliminated,
            needs_human_review,
        }
    }
}

// ─── CriticNode → ThreeStagePipeline adapter ───

/// Adapter that wraps ThreeStagePipeline as a CriticNode-compatible review method.
pub struct CriticPipelineAdapter {
    pub pipeline: ThreeStagePipeline,
}

impl CriticPipelineAdapter {
    pub fn new(pipeline: ThreeStagePipeline) -> Self {
        Self { pipeline }
    }

    pub fn new_default() -> Self {
        Self {
            pipeline: ThreeStagePipeline::new(
                Box::new(SimplePlanStage::new()),
                Box::new(SimpleMainStage::new()),
                Box::new(SimpleFilterStage::new()),
            ),
        }
    }

    /// Review a file diff through the three-stage pipeline, returning filtered comments.
    pub fn review(
        &self,
        diff: &str,
        metadata: &FileMetadata,
        diffs: &[ReviewFileDiff],
    ) -> FilteredOutput {
        self.pipeline.review_file(diff, metadata, diffs)
    }
}

/// Extension trait: add three-stage review to CriticNode without breaking existing API.
pub trait CriticNodeExt {
    fn review_with_pipeline(
        &self,
        pipeline: &ThreeStagePipeline,
        diff: &str,
        metadata: &FileMetadata,
        diffs: &[ReviewFileDiff],
    ) -> FilteredOutput;
}

impl CriticNodeExt for CriticNode {
    fn review_with_pipeline(
        &self,
        pipeline: &ThreeStagePipeline,
        diff: &str,
        metadata: &FileMetadata,
        diffs: &[ReviewFileDiff],
    ) -> FilteredOutput {
        pipeline.review_file(diff, metadata, diffs)
    }
}

// ─── Data-layer conversion helpers ───

pub fn diff_to_file_metadata(file: &ReviewFileDiff) -> FileMetadata {
    let (added, deleted) =
        file.hunks
            .iter()
            .flat_map(|h| &h.lines)
            .fold((0u32, 0u32), |(add, del), line| match line.line_type {
                crate::neotrix::nt_act_code::code_review_pipeline::DiffLineType::Addition => {
                    (add + 1, del)
                }
                crate::neotrix::nt_act_code::code_review_pipeline::DiffLineType::Deletion => {
                    (add, del + 1)
                }
                crate::neotrix::nt_act_code::code_review_pipeline::DiffLineType::Context => {
                    (add, del)
                }
            });

    let ext = std::path::Path::new(&file.file)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_string();

    let language = match ext.as_str() {
        "rs" => "rust",
        "js" | "ts" | "jsx" | "tsx" => "typescript",
        "py" => "python",
        "go" => "go",
        "java" => "java",
        "rb" => "ruby",
        _ => "unknown",
    }
    .to_string();

    let is_test = file.file.contains("_test")
        || file.file.contains("/tests/")
        || file.file.contains("/test/");

    let scope = if file.file.starts_with("src/") || file.file.starts_with("neotrix-core/") {
        "code"
    } else if file.file.starts_with("docs/") {
        "documentation"
    } else if file.file.starts_with("tests/") || file.file.starts_with("test/") {
        "test"
    } else {
        "general"
    }
    .to_string();

    FileMetadata {
        path: file.file.clone(),
        language,
        scope,
        is_test,
        old_path: file.old_path.clone(),
        added_lines: added,
        deleted_lines: deleted,
    }
}

pub fn extract_diff_text(file: &ReviewFileDiff) -> String {
    let mut lines = Vec::new();
    for hunk in &file.hunks {
        let header = format!(
            "@@ -{},{} +{},{} @@",
            hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count
        );
        lines.push(header);
        for dl in &hunk.lines {
            let prefix = match dl.line_type {
                crate::neotrix::nt_act_code::code_review_pipeline::DiffLineType::Addition => "+",
                crate::neotrix::nt_act_code::code_review_pipeline::DiffLineType::Deletion => "-",
                crate::neotrix::nt_act_code::code_review_pipeline::DiffLineType::Context => " ",
            };
            lines.push(format!("{}{}", prefix, dl.content));
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_act_code::code_review_pipeline::{
        DiffHunk, DiffLine, DiffLineType, DiffStatus,
    };

    fn sample_file_diff() -> ReviewFileDiff {
        ReviewFileDiff {
            file: "src/main.rs".into(),
            status: DiffStatus::Modified,
            old_path: None,
            hunks: vec![DiffHunk {
                old_start: 1,
                old_count: 5,
                new_start: 1,
                new_count: 7,
                lines: vec![
                    DiffLine {
                        line_type: DiffLineType::Addition,
                        old_line: None,
                        new_line: Some(1),
                        content: "fn main() {".into(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Addition,
                        old_line: None,
                        new_line: Some(2),
                        content: "    let x = get_value().unwrap();".into(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Addition,
                        old_line: None,
                        new_line: Some(3),
                        content: "    let y = compute(42);".into(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Addition,
                        old_line: None,
                        new_line: Some(4),
                        content: "    println!(\"{:?}\", y);".into(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Addition,
                        old_line: None,
                        new_line: Some(5),
                        content: "    unsafe {".into(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Addition,
                        old_line: None,
                        new_line: Some(6),
                        content: "        let ptr = std::ptr::null();".into(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Addition,
                        old_line: None,
                        new_line: Some(7),
                        content: "    }".into(),
                    },
                ],
            }],
        }
    }

    #[test]
    fn test_plan_stage_analysis() {
        let file = sample_file_diff();
        let diff_text = extract_diff_text(&file);
        let metadata = diff_to_file_metadata(&file);
        let stage = SimplePlanStage::new();
        let plan = stage.analyze(&diff_text, &metadata);

        assert!(plan.suspected_issues.iter().any(|i| i.contains("unwrap")));
        assert!(plan.suspected_issues.iter().any(|i| i.contains("Unsafe")));
        assert_eq!(plan.risk_level, RiskLevel::High);
        assert_eq!(plan.scope, "security");
        assert!(plan.change_summary.contains("src/main.rs"));
    }

    #[test]
    fn test_plan_stage_clean_diff() {
        let diff = "@@ -1,3 +1,3 @@\n fn existing() {}\n+let x = 1;\n fn other() {}\n";
        let metadata = FileMetadata {
            path: "lib.rs".into(),
            language: "rust".into(),
            scope: String::new(),
            is_test: false,
            old_path: None,
            added_lines: 1,
            deleted_lines: 0,
        };
        let stage = SimplePlanStage::new();
        let plan = stage.analyze(diff, &metadata);

        assert!(plan.suspected_issues.is_empty());
        assert_eq!(plan.risk_level, RiskLevel::Low);
        assert_eq!(plan.scope, "general");
    }

    #[test]
    fn test_main_stage_generates_comments() {
        let file = sample_file_diff();
        let diff_text = extract_diff_text(&file);
        let metadata = diff_to_file_metadata(&file);
        let plan = SimplePlanStage::new().analyze(&diff_text, &metadata);
        let stage = SimpleMainStage::new();

        let (comments, records) = stage.review(&plan, &diff_text, &metadata);

        assert!(
            !comments.is_empty(),
            "should generate comments for security issues"
        );
        assert!(comments.iter().any(|c| c.message.contains(".unwrap()")));
        assert!(comments
            .iter()
            .any(|c| c.message.contains("unsafe") || c.message.contains("Unsafe")));
        assert!(!records.is_empty(), "should record tool calls");
    }

    #[test]
    fn test_main_stage_empty_diff() {
        let diff = "";
        let metadata = FileMetadata {
            path: "empty.rs".into(),
            language: "rust".into(),
            scope: String::new(),
            is_test: false,
            old_path: None,
            added_lines: 0,
            deleted_lines: 0,
        };
        let plan = PlanOutput {
            change_summary: "No changes".into(),
            suspected_issues: vec![],
            scope: "general".into(),
            risk_level: RiskLevel::Low,
        };
        let stage = SimpleMainStage::new();
        let (comments, _) = stage.review(&plan, diff, &metadata);
        assert!(comments.is_empty());
    }

    #[test]
    fn test_filter_stage_keeps_valid() {
        let comment = ReviewComment {
            id: "c-0".into(),
            file: "src/main.rs".into(),
            severity: IssueSeverity::High,
            category: IssueCategory::Security,
            message: "Unsafe unwrap call".into(),
            existing_code: ".unwrap()".into(),
            start_line: Some(2),
            end_line: Some(2),
            suggestion: Some("Use ? operator".into()),
            anchor_lines: vec!["    let x = get_value().unwrap();".into()],
            match_confidence: 0.85,
            needs_relocation: false,
        };
        let diff = "fn main() {\n    let x = get_value().unwrap();\n    unsafe {}\n}";
        let plan = PlanOutput {
            change_summary: "test".into(),
            suspected_issues: vec!["unwrap".into()],
            scope: "correctness".into(),
            risk_level: RiskLevel::Medium,
        };
        let metadata = FileMetadata {
            path: "src/main.rs".into(),
            language: "rust".into(),
            scope: String::new(),
            is_test: false,
            old_path: None,
            added_lines: 3,
            deleted_lines: 0,
        };
        let filter = SimpleFilterStage::new();
        let result = filter.filter(&plan, &[comment], diff, &metadata);

        assert_eq!(result.kept.len(), 1, "valid comment should be kept");
        assert!(result.eliminated.is_empty());
    }

    #[test]
    fn test_filter_stage_eliminates_low_confidence() {
        let comment = ReviewComment {
            id: "c-0".into(),
            file: "src/main.rs".into(),
            severity: IssueSeverity::Low,
            category: IssueCategory::Style,
            message: "Consider renaming".into(),
            existing_code: "".into(),
            start_line: None,
            end_line: None,
            suggestion: None,
            anchor_lines: vec![],
            match_confidence: 0.2,
            needs_relocation: true,
        };
        let diff = "fn main() {}";
        let plan = PlanOutput {
            change_summary: "test".into(),
            suspected_issues: vec![],
            scope: "style".into(),
            risk_level: RiskLevel::Low,
        };
        let metadata = FileMetadata {
            path: "src/main.rs".into(),
            language: "rust".into(),
            scope: String::new(),
            is_test: false,
            old_path: None,
            added_lines: 1,
            deleted_lines: 0,
        };
        let filter = SimpleFilterStage::new();
        let result = filter.filter(&plan, &[comment], diff, &metadata);

        assert!(
            !result.eliminated.is_empty() || result.kept.is_empty(),
            "low-confidence comment should be eliminated"
        );
    }

    #[test]
    fn test_filter_stage_severity_kept_for_human_review() {
        let comment = ReviewComment {
            id: "c-0".into(),
            file: "critical.rs".into(),
            severity: IssueSeverity::Critical,
            category: IssueCategory::Security,
            message: "Potential vulnerability".into(),
            existing_code: "unsafe { ... }".into(),
            start_line: None,
            end_line: None,
            suggestion: None,
            anchor_lines: vec![],
            match_confidence: 0.4,
            needs_relocation: true,
        };
        let diff = "fn main() { unsafe { let p = std::ptr::null(); } }";
        let plan = PlanOutput {
            change_summary: "test".into(),
            suspected_issues: vec!["unsafe".into()],
            scope: "security".into(),
            risk_level: RiskLevel::High,
        };
        let metadata = FileMetadata {
            path: "critical.rs".into(),
            language: "rust".into(),
            scope: String::new(),
            is_test: false,
            old_path: None,
            added_lines: 1,
            deleted_lines: 0,
        };
        let filter = SimpleFilterStage::new();
        let result = filter.filter(&plan, &[comment], diff, &metadata);

        assert!(
            !result.needs_human_review.is_empty(),
            "critical but uncertain should go to human review"
        );
    }

    #[test]
    fn test_three_stage_pipeline_integration() {
        let file = sample_file_diff();
        let diff_text = extract_diff_text(&file);
        let metadata = diff_to_file_metadata(&file);

        let pipeline = ThreeStagePipeline::new(
            Box::new(SimplePlanStage::new()),
            Box::new(SimpleMainStage::new()),
            Box::new(SimpleFilterStage::new()),
        );

        let result = pipeline.review_file(&diff_text, &metadata, &[file]);

        assert!(
            result.has_actionable(),
            "pipeline should produce actionable comments"
        );
        assert!(result.kept.iter().any(|c| c.message.contains(".unwrap()")));
    }

    #[test]
    fn test_critic_pipeline_adapter() {
        let file = sample_file_diff();
        let diff_text = extract_diff_text(&file);
        let metadata = diff_to_file_metadata(&file);

        let adapter = CriticPipelineAdapter::new_default();
        let result = adapter.review(&diff_text, &metadata, &[file]);

        assert!(
            result.has_actionable(),
            "adapter should produce actionable comments"
        );
    }

    #[test]
    fn test_critic_node_extension_trait() {
        let file = sample_file_diff();
        let diff_text = extract_diff_text(&file);
        let metadata = diff_to_file_metadata(&file);

        let critic = CriticNode::new();
        let pipeline = ThreeStagePipeline::new(
            Box::new(SimplePlanStage::new()),
            Box::new(SimpleMainStage::new()),
            Box::new(SimpleFilterStage::new()),
        );

        let result = critic.review_with_pipeline(&pipeline, &diff_text, &metadata, &[file]);
        assert!(result.has_actionable());
    }

    #[test]
    fn test_diff_to_file_metadata() {
        let file = sample_file_diff();
        let metadata = diff_to_file_metadata(&file);
        assert_eq!(metadata.path, "src/main.rs");
        assert_eq!(metadata.language, "rust");
        assert!(!metadata.is_test);
        assert_eq!(metadata.added_lines, 7);
        assert_eq!(metadata.deleted_lines, 0);
    }

    #[test]
    fn test_extract_diff_text_roundtrip() {
        let file = sample_file_diff();
        let text = extract_diff_text(&file);
        assert!(text.contains("@@"));
        assert!(text.contains(".unwrap()"));
        assert!(text.contains("unsafe"));
        assert!(text.starts_with("@@"));
    }

    #[test]
    fn test_filtered_output_helpers() {
        let mut comments = Vec::new();
        for i in 0..3u32 {
            comments.push(ReviewComment {
                id: format!("c-{}", i),
                file: "test.rs".into(),
                severity: IssueSeverity::Medium,
                category: IssueCategory::Correctness,
                message: format!("issue {}", i),
                existing_code: format!("code {}", i),
                start_line: Some(i),
                end_line: Some(i),
                suggestion: None,
                anchor_lines: vec![],
                match_confidence: 0.9,
                needs_relocation: false,
            });
        }
        let output = FilteredOutput {
            kept: comments,
            eliminated: vec![],
            needs_human_review: vec![],
        };
        assert_eq!(output.all_kept_ids().len(), 3);
        assert!(output.has_actionable());
        assert_eq!(output.total_eliminated(), 0);
    }

    #[test]
    fn test_tool_call_record() {
        let record = ToolCallRecord {
            tool: ReviewTool::SearchCode("unsafe".into()),
            result_summary: "Found 3 matches".into(),
        };
        assert!(record.result_summary.contains("3 matches"));
    }

    #[test]
    fn test_risk_level_ordering() {
        assert_ne!(RiskLevel::Low as i32, RiskLevel::High as i32);
        assert_ne!(RiskLevel::Medium as i32, RiskLevel::Low as i32);
    }

    #[test]
    fn test_plan_output_serialization() {
        let plan = PlanOutput {
            change_summary: "test".into(),
            suspected_issues: vec!["issue".into()],
            scope: "security".into(),
            risk_level: RiskLevel::High,
        };
        let json = serde_json::to_string(&plan).unwrap();
        let deserialized: PlanOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.change_summary, "test");
        assert_eq!(deserialized.scope, "security");
    }

    #[test]
    fn test_review_tool_deserialization() {
        let tool = ReviewTool::SearchCode("unwrap".into());
        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: ReviewTool = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, ReviewTool::SearchCode(ref s) if s == "unwrap"));
    }
}
