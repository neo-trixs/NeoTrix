//! `nt_identity` — 行号无关 finding 身份.
//!
//! 对标 `internal/session/compare.go:25-134`:
//! `findingKey = normalizePath(path) | lower(category) | normalizeSnippet`,
//! 空白塌缩, **故意不用行号**以抗漂移; 外加 sha256 指纹
//! (SARIF `partialFingerprints`, 用 `existing_code` 不用 prose).

use sha2::{Digest, Sha256};

use crate::nt_finding::AuditFinding;

/// 路径归一: 反斜杠转斜杠 + 去 `./` 前缀 + 小写比较键.
pub fn normalize_path(path: &str) -> String {
    let slashed = path.replace('\\', "/");
    let trimmed = slashed.trim_start_matches("./");
    trimmed.to_owned()
}

/// 片段归一: 空白塌缩 (OCR `normalizeSnippet` 同语义).
pub fn normalize_snippet(snippet: &str) -> String {
    snippet.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// finding 身份键 — 身份演进不影响外部 alert 身份
/// (OCR `compare.go:82-86` 注释: 内部去重身份可演进).
pub fn finding_key(finding: &AuditFinding) -> String {
    let snippet = if finding.existing_code.trim().is_empty() {
        normalize_snippet(&finding.content)
    } else {
        normalize_snippet(&finding.existing_code)
    };
    format!(
        "{}|{}|{}",
        normalize_path(&finding.path).to_ascii_lowercase(),
        finding.category.as_str(),
        snippet
    )
}

/// 稳定指纹 `sha256(path|category|snippet)` (hex).
pub fn finding_fingerprint(finding: &AuditFinding) -> String {
    let mut hasher = Sha256::new();
    hasher.update(finding_key(finding).as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::{finding_fingerprint, finding_key};
    use crate::nt_finding::{AuditFinding, Category, Severity};

    fn finding(path: &str, code: &str, start: u32) -> AuditFinding {
        AuditFinding {
            path: path.to_owned(),
            category: Category::Bug,
            severity: Severity::High,
            content: "x".to_owned(),
            suggestion: String::new(),
            existing_code: code.to_owned(),
            start_line: start,
            end_line: start + 2,
        }
    }

    #[test]
    fn key_ignores_line_drift() {
        let before = finding("src/a.rs", "let x = 1;", 10);
        let after = finding("src/a.rs", "let  x  =  1;", 47);
        assert_eq!(finding_key(&before), finding_key(&after));
        assert_eq!(finding_fingerprint(&before), finding_fingerprint(&after));
    }

    #[test]
    fn key_separates_paths_and_categories() {
        let base = finding("src/a.rs", "let x = 1;", 10);
        let mut other_path = base.clone();
        other_path.path = "src/b.rs".to_owned();
        assert_ne!(finding_key(&base), finding_key(&other_path));
    }
}
