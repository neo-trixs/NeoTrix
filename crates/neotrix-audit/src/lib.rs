//! neotrix-audit — 确定性审计核心.
//!
//! 吸收来源: `alibaba/open-code-review` (Apache-2.0, OCR `ocr` CLI).
//! 只移植语言无关的确定性机制 (LLM 语义判断仍归 rev-officer / skills):
//! - `nt_finding`: 8 分类 × 4 级 + SARIF level 映射
//!   (`internal/tool/code_comment.go`, `internal/model/review.go`)
//! - `nt_identity`: 行号无关 finding 身份 + sha256 指纹
//!   (`internal/session/compare.go`, `sarif.go` fingerprints)
//! - `nt_baseline`: 基线对比四桶 (new/persisting/resolved/not_reviewed)
//!   (`internal/session/compare.go`)
//! - `nt_rules`: 四层规则编排 (Custom > Project > Global > System)
//!   (`internal/config/rules/system_rules.go`)
//! - `nt_select`: 确定性选文件 (secret > user > allowlist > size)
//!   (`internal/agent/selection.go`)
//! - `prompts/review_filter.md`: 误报抑制 prompt 资产 (M3), 随 crate 版本化.

#![forbid(unsafe_code)]

pub mod nt_baseline;
pub mod nt_finding;
pub mod nt_identity;
pub mod nt_rules;
pub mod nt_select;

pub use nt_baseline::{BaselineDiff, compare_baseline};
pub use nt_finding::{AuditFinding, Category, Severity};
pub use nt_identity::{finding_fingerprint, finding_key};
pub use nt_rules::{Layer, ResolvedRule, RuleResolver};
pub use nt_select::{ExclusionReason, SelectConfig, SelectOutcome, select_files};

/// 误报抑制 prompt (OCR `review_filter_task` 方法论的语言无关版).
/// LLM 审计后处理用: 默认 approve, 仅 Ground A/B 可删, 五类 Protected 永不删.
pub fn review_filter_prompt() -> &'static str {
    include_str!("../prompts/review_filter.md")
}
