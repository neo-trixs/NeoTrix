//! `nt_baseline` — 基线对比四桶.
//!
//! 对标 `internal/session/compare.go`: multiset 匹配, 四桶
//! `new / persisting / resolved / not_reviewed`.
//! 覆盖语义: 仅 `Completed + Reused` 算"看过",
//! `Failed / Waived` 不算 — 防"未重审即 resolved"蒙混
//! (剧场检测的弹药).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::nt_finding::AuditFinding;
use crate::nt_identity::finding_key;

/// 单文件评审状态 (OCR `Coverage` 语义子集).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewState {
    Completed,
    Reused,
    Failed,
    Waived,
}

impl ReviewState {
    /// 只有 Completed/Reused 算"看过".
    pub fn counts_as_reviewed(self) -> bool {
        matches!(self, Self::Completed | Self::Reused)
    }
}

/// 基线对比结果.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BaselineDiff {
    pub new: Vec<AuditFinding>,
    pub persisting: Vec<AuditFinding>,
    pub resolved: Vec<AuditFinding>,
    pub not_reviewed: Vec<AuditFinding>,
}

/// 对比基线 (`before`) 与本轮 (`after`).
/// `reviewed` 为本轮实际看过的文件路径集合 (归一化后比较).
pub fn compare_baseline(
    before: &[AuditFinding],
    after: &[AuditFinding],
    reviewed: &[String],
) -> BaselineDiff {
    let reviewed_norm: Vec<String> = reviewed
        .iter()
        .map(|path| path.replace('\\', "/").trim_start_matches("./").to_owned())
        .collect();
    let is_reviewed = |path: &str| {
        let norm = path.replace('\\', "/");
        let norm = norm.trim_start_matches("./");
        reviewed_norm.iter().any(|seen| seen == norm)
    };

    let mut after_pool: HashMap<String, Vec<AuditFinding>> = HashMap::new();
    for finding in after {
        after_pool
            .entry(finding_key(finding))
            .or_default()
            .push(finding.clone());
    }

    let mut diff = BaselineDiff::default();
    for finding in before {
        let key = finding_key(finding);
        if let Some(bucket) = after_pool.get_mut(&key) {
            if !bucket.is_empty() {
                let matched = bucket.remove(0);
                diff.persisting.push(matched);
                continue;
            }
        }
        // 基线有、本轮无: 看过才算 resolved, 否则 not_reviewed.
        if is_reviewed(&finding.path) {
            diff.resolved.push(finding.clone());
        } else {
            diff.not_reviewed.push(finding.clone());
        }
    }
    for (_, mut bucket) in after_pool {
        diff.new.append(&mut bucket);
    }
    diff
}

#[cfg(test)]
mod tests {
    use super::{ReviewState, compare_baseline};
    use crate::nt_finding::{AuditFinding, Category, Severity};

    fn finding(path: &str, code: &str) -> AuditFinding {
        AuditFinding {
            path: path.to_owned(),
            category: Category::Bug,
            severity: Severity::High,
            content: "c".to_owned(),
            suggestion: String::new(),
            existing_code: code.to_owned(),
            start_line: 1,
            end_line: 2,
        }
    }

    #[test]
    fn four_buckets() {
        let before = vec![
            finding("a.rs", "let x = 1;"), // persisting (行号漂移也认)
            finding("b.rs", "let y = 2;"), // resolved (看过)
            finding("c.rs", "let z = 3;"), // not_reviewed (没看过)
        ];
        let after = vec![
            finding("a.rs", "let x = 1;"),
            finding("d.rs", "let w = 4;"), // new
        ];
        let diff = compare_baseline(&before, &after, &["a.rs".to_owned(), "b.rs".to_owned(), "d.rs".to_owned()]);
        assert_eq!(diff.persisting.len(), 1);
        assert_eq!(diff.resolved.len(), 1);
        assert_eq!(diff.not_reviewed.len(), 1);
        assert_eq!(diff.new.len(), 1);
    }

    #[test]
    fn only_completed_counts_as_reviewed() {
        assert!(ReviewState::Completed.counts_as_reviewed());
        assert!(ReviewState::Reused.counts_as_reviewed());
        assert!(!ReviewState::Failed.counts_as_reviewed());
        assert!(!ReviewState::Waived.counts_as_reviewed());
    }
}
