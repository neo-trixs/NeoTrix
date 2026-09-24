//! `nt_finding` — finding 分类法 (OCR 8 类 × 4 级, 硬编码三处同源的 Rust 版).
//!
//! 对标: `internal/tool/code_comment.go:16-48`,
//! `internal/model/review.go:7-21`, SARIF 映射 `sarif.go:155-185`.

use serde::{Deserialize, Serialize};

/// finding 分类 — 未知归 `Other` (OCR 同源语义).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Bug,
    Security,
    Performance,
    Maintainability,
    Test,
    Style,
    Documentation,
    Other,
}

impl Category {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bug => "bug",
            Self::Security => "security",
            Self::Performance => "performance",
            Self::Maintainability => "maintainability",
            Self::Test => "test",
            Self::Style => "style",
            Self::Documentation => "documentation",
            Self::Other => "other",
        }
    }

    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "bug" => Self::Bug,
            "security" => Self::Security,
            "performance" => Self::Performance,
            "maintainability" => Self::Maintainability,
            "test" => Self::Test,
            "style" => Self::Style,
            "documentation" => Self::Documentation,
            _ => Self::Other,
        }
    }
}

/// 严重度 — 未知归 `Low` (OCR 同源语义).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "critical" => Self::Critical,
            "high" => Self::High,
            "medium" => Self::Medium,
            _ => Self::Low,
        }
    }

    /// SARIF level 映射 (`sarif.go:155-185`):
    /// critical/high → error, medium → warning, low → note.
    pub fn sarif_level(self) -> &'static str {
        match self {
            Self::Critical | Self::High => "error",
            Self::Medium => "warning",
            Self::Low => "note",
        }
    }
}

/// 审计 finding 原子 (`LlmComment` 子集, 确定性字段).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFinding {
    pub path: String,
    pub category: Category,
    pub severity: Severity,
    pub content: String,
    #[serde(default)]
    pub suggestion: String,
    /// 命中代码原文 — 身份与指纹用它, 不用 LLM prose (跨轮稳定).
    #[serde(default)]
    pub existing_code: String,
    #[serde(default)]
    pub start_line: u32,
    #[serde(default)]
    pub end_line: u32,
}

#[cfg(test)]
mod tests {
    use super::{Category, Severity};

    #[test]
    fn unknown_maps_to_other_low() {
        assert_eq!(Category::parse("qubit"), Category::Other);
        assert_eq!(Severity::parse("cosmic"), Severity::Low);
    }

    #[test]
    fn sarif_levels_match_ocr() {
        assert_eq!(Severity::Critical.sarif_level(), "error");
        assert_eq!(Severity::High.sarif_level(), "error");
        assert_eq!(Severity::Medium.sarif_level(), "warning");
        assert_eq!(Severity::Low.sarif_level(), "note");
    }
}
