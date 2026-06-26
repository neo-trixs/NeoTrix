use std::path::Path;

use super::comment_resolver::CommentResolver;
use super::rule_resolver::{LayeredRuleResolver, PathRule, ReviewCmdConfig};
use super::types::{
    DiffLineType, DiffStatus, IssueCategory, IssueSeverity, RelocationRequest, ReviewComment,
    ReviewFileDiff, ReviewResult, RELOCATION_THRESHOLD,
};

pub struct CodeReviewPipeline {
    pub comment_resolver: CommentResolver,
    pub rule_resolver: LayeredRuleResolver,
}

impl CodeReviewPipeline {
    pub fn new() -> Self {
        Self {
            comment_resolver: CommentResolver::new(),
            rule_resolver: LayeredRuleResolver::new(),
        }
    }

    pub fn with_config(mut self, config: ReviewCmdConfig) -> Self {
        self.rule_resolver = self.rule_resolver.with_config(config);
        self
    }

    pub fn with_rules(mut self, rules: Vec<PathRule>) -> Self {
        self.rule_resolver = self.rule_resolver.with_cli_rules(rules);
        self
    }

    pub fn with_project_config(mut self, base: &Path) -> Self {
        self.rule_resolver = self.rule_resolver.with_project_config(base);
        self
    }

    pub fn with_global_config(mut self) -> Self {
        self.rule_resolver = self.rule_resolver.with_global_config();
        self
    }

    pub fn run_deterministic_review(&self, diffs: &[ReviewFileDiff]) -> ReviewResult {
        let mut comments = Vec::new();
        let mut comment_id = 0u32;

        for file_diff in diffs {
            if file_diff.status == DiffStatus::Deleted {
                continue;
            }
            let rule = self.rule_resolver.resolve(&file_diff.file);
            if !rule.should_review {
                continue;
            }
            for hunk in &file_diff.hunks {
                let content: String = hunk
                    .lines
                    .iter()
                    .filter(|l| l.line_type == DiffLineType::Addition)
                    .map(|l| l.content.as_str())
                    .collect::<Vec<&str>>()
                    .join("\n");

                let deferred = self.deterministic_scan(&content, &rule.rule_text);
                for issue in deferred {
                    comments.push(ReviewComment {
                        id: format!("c-{}", comment_id),
                        file: file_diff.file.clone(),
                        severity: issue.0,
                        category: IssueCategory::Security,
                        message: issue.1,
                        existing_code: issue.2.clone(),
                        start_line: None,
                        end_line: None,
                        suggestion: issue.3,
                        anchor_lines: Vec::new(),
                        match_confidence: 0.0,
                        needs_relocation: false,
                    });
                    comment_id += 1;
                }
            }
        }

        self.comment_resolver.resolve_comments(&mut comments, diffs);

        let warning_count = comments
            .iter()
            .filter(|c| c.severity == IssueSeverity::Low || c.severity == IssueSeverity::Medium)
            .count();
        let error_count = comments
            .iter()
            .filter(|c| {
                c.severity == IssueSeverity::High
                    || c.severity == IssueSeverity::High
                    || c.severity == IssueSeverity::Critical
            })
            .count();

        ReviewResult {
            comment_count: comments.len(),
            file_count: diffs.len(),
            warning_count,
            error_count,
            comments,
        }
    }

    /// Resolve comments from LLM-generated review, computing line positions and confidence.
    /// Returns a list of relocation requests for comments where matching failed.
    pub fn resolve_llm_comments(
        &self,
        comments: &mut [ReviewComment],
        diffs: &[ReviewFileDiff],
    ) -> Vec<RelocationRequest> {
        let mut requests = Vec::new();
        for comment in comments.iter_mut() {
            if comment.start_line.is_some()
                && comment.end_line.is_some()
                && comment.match_confidence >= RELOCATION_THRESHOLD
            {
                continue;
            }
            if comment.needs_relocation {
                if let Some(diff) = diffs.iter().find(|d| d.file == comment.file) {
                    self.comment_resolver.relocate_comment(comment, diff);
                }
            } else if let Some(diff) = diffs.iter().find(|d| d.file == comment.file) {
                self.comment_resolver.resolve_single(comment, diff);
            }

            if comment.match_confidence < RELOCATION_THRESHOLD && comment.start_line.is_none() {
                requests.push(RelocationRequest {
                    comment_id: comment.id.clone(),
                    file: comment.file.clone(),
                    existing_code: comment.existing_code.clone(),
                    message: comment.message.clone(),
                    current_confidence: comment.match_confidence,
                });
            }
        }
        requests
    }

    fn deterministic_scan(
        &self,
        added_content: &str,
        rule_text: &str,
    ) -> Vec<(IssueSeverity, String, String, Option<String>)> {
        let mut findings = Vec::new();
        let _ = rule_text;

        let patterns: Vec<(&str, IssueSeverity, &str, &str)> = vec![
            (
                ".unwrap()",
                IssueSeverity::High,
                "Unsafe unwrap call",
                "Prefer proper error handling with ? operator or match",
            ),
            (
                ".expect(",
                IssueSeverity::High,
                "Unsafe expect call",
                "Prefer proper error handling with ? operator or match",
            ),
            (
                "panic!",
                IssueSeverity::Critical,
                "Panic in production code",
                "Return Result instead",
            ),
            (
                "unsafe ",
                IssueSeverity::High,
                "Unsafe block",
                "Verify safety invariants and add // SAFETY: comment",
            ),
            (
                "todo!",
                IssueSeverity::Medium,
                "Unimplemented code",
                "Implement before merging",
            ),
            (
                "unreachable!",
                IssueSeverity::High,
                "Unreachable assertion",
                "Verify this path is truly unreachable",
            ),
            (
                "dbg!",
                IssueSeverity::Low,
                "Debug print",
                "Remove dbg! before merging",
            ),
            (
                "TODO",
                IssueSeverity::Low,
                "TODO comment",
                "Address before merging",
            ),
            (
                "FIXME",
                IssueSeverity::Medium,
                "FIXME comment",
                "Fix before merging",
            ),
            (
                "HACK",
                IssueSeverity::Medium,
                "HACK comment",
                "Refactor to proper solution",
            ),
            (
                "sslmode=disable",
                IssueSeverity::Critical,
                "SSL disabled",
                "Enable SSL for database connections",
            ),
            (
                "password=",
                IssueSeverity::Critical,
                "Password in code",
                "Use environment variables or secrets manager",
            ),
            (
                "api_key",
                IssueSeverity::Critical,
                "Potential API key",
                "Use environment variables",
            ),
            (
                "secret",
                IssueSeverity::Critical,
                "Potential secret",
                "Use environment variables",
            ),
            (
                "std::process::Command::new(\"rm",
                IssueSeverity::Critical,
                "Dangerous command",
                "Avoid rm in production code",
            ),
        ];

        for (pattern, severity, title, suggestion) in &patterns {
            for (_line_idx, line) in added_content.lines().enumerate() {
                if line.contains(pattern) {
                    let line_trimmed = line.trim();
                    if line_trimmed.len() > 120 {
                        findings.push((
                            severity.clone(),
                            format!("{}: {}", title, line_trimmed[..120].to_string()),
                            line_trimmed.to_string(),
                            Some(suggestion.to_string()),
                        ));
                    } else {
                        findings.push((
                            severity.clone(),
                            format!("{}: {}", title, line_trimmed),
                            line_trimmed.to_string(),
                            Some(suggestion.to_string()),
                        ));
                    }
                }
            }
        }

        findings
    }
}

impl Default for CodeReviewPipeline {
    fn default() -> Self {
        Self::new()
    }
}
