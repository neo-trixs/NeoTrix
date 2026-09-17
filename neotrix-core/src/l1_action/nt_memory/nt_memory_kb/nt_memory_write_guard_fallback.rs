//! P1: Deterministic Fallback + Verbatim-Dup 写前检查
//!
//! 吸收 cumora COORDINATION.md §5c (deterministic fallback) 和 §5b (verbatim-dup)。
//!
//! **Deterministic Fallback**: 当 AI classifier/LLM 不可用时 (503/timeout)，
//! 不静默放弃, 而是 carving out 最窄的确定性 case:
//! - 恰好 1 个待处理写操作
//! - 内容 < 阈值 (小写操作, 风险低)
//! - 无冲突信号 (无并发写、无 hold)
//! → 直接放行。其他情况 fail-closed。
//!
//! **Verbatim-Dup**: KB 写入前, 检查是否已有相同内容的节点/edge。
//! 纯函数 + 确定性 (normalize → hash → lookup), 零 LLM 成本。
//! 防止多 agent/多轮次写入产生重复知识节点。

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Deterministic fallback 的内容大小阈值 — 超过此值不走 fallback (风险太高)。
pub const FALLBACK_MAX_CONTENT_CHARS: usize = 2000;

/// Deterministic fallback 的并发写操作上限 — 超过 1 个不走 fallback (无法确定安全)。
pub const FALLBACK_MAX_PENDING: usize = 1;

/// 写操作的确定性分类结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeterministicVerdict {
    /// AI classifier 可用时的正常路径 — 由 classifier 决定。
    ClassifierAvailable,
    /// AI classifier 不可用, 但满足最窄确定性 case → 放行。
    FallbackAllow,
    /// AI classifier 不可用, 且不满足确定性 case → fail-closed 拒绝。
    FallbackReject(String),
}

/// Deterministic fallback 判断 — classifier 不可用时的最窄安全放行。
///
/// 条件 (全部满足才放行):
/// 1. pending_count ≤ FALLBACK_MAX_PENDING (只有 1 个写操作)
/// 2. content_chars ≤ FALLBACK_MAX_CONTENT_CHARS (小写操作)
/// 3. 无 hold token 冲突 (没有已 hold 的同 scope 写操作)
///
/// 对齐 cumora §5c: "exactly one stall, someone else spoke last,
/// ≤30 minutes silent, no other cards/events" 的最窄确定性 case。
pub fn deterministic_fallback(
    pending_count: usize,
    content_chars: usize,
    has_hold_conflict: bool,
    classifier_error: &str,
) -> DeterministicVerdict {
    if pending_count == 0 {
        // 无待处理写操作 — 无需 fallback。
        return DeterministicVerdict::ClassifierAvailable;
    }

    if pending_count > FALLBACK_MAX_PENDING {
        return DeterministicVerdict::FallbackReject(format!(
            "classifier 不可用 ({}), 但 pending_count={} > {}, 无法确定安全",
            classifier_error, pending_count, FALLBACK_MAX_PENDING
        ));
    }

    if content_chars > FALLBACK_MAX_CONTENT_CHARS {
        return DeterministicVerdict::FallbackReject(format!(
            "classifier 不可用 ({}), 但 content_chars={} > {}, 风险太高",
            classifier_error, content_chars, FALLBACK_MAX_CONTENT_CHARS
        ));
    }

    if has_hold_conflict {
        return DeterministicVerdict::FallbackReject(format!(
            "classifier 不可用 ({}), 但存在 hold 冲突, 无法确定安全",
            classifier_error
        ));
    }

    DeterministicVerdict::FallbackAllow
}

/// 内容指纹 — 用于 verbatim-dup 检测。
/// 标准化: trim + lowercase + collapse whitespace + hash。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentFingerprint {
    pub hash: u64,
    pub normalized_len: usize,
}

/// 标准化内容用于指纹计算:
/// 1. trim 首尾空白
/// 2. lowercase
/// 3. collapse 连续空白为单个空格
pub fn normalize_content(s: &str) -> String {
    let trimmed = s.trim().to_lowercase();
    let mut result = String::with_capacity(trimmed.len());
    let mut prev_space = false;
    for c in trimmed.chars() {
        if c.is_whitespace() {
            if !prev_space {
                result.push(' ');
                prev_space = true;
            }
        } else {
            result.push(c);
            prev_space = false;
        }
    }
    result
}

/// 计算内容指纹 (verbatim-dup 专用, 与 nt_normalizer::content_fingerprint 独立)。
pub fn verbatim_content_fingerprint(s: &str) -> ContentFingerprint {
    let normalized = normalize_content(s);
    let mut hasher = DefaultHasher::new();
    normalized.hash(&mut hasher);
    ContentFingerprint {
        hash: hasher.finish(),
        normalized_len: normalized.len(),
    }
}

/// Verbatim-Dup 检测结果。
#[derive(Debug, Clone, PartialEq)]
pub enum VerbatimDupResult {
    /// 无重复 — 可以写入。
    Unique,
    /// 发现完全相同的内容 — 拒绝写入。
    /// 内含已有节点的 id 和创建时间。
    Duplicate {
        existing_id: String,
        existing_title: String,
        created_at_ms: u64,
    },
    /// 内容相似但非完全相同 (normalized 后 hash 匹配但原始内容不同)。
    /// 标记为 requires_review 而非直接拒绝。
    NearDuplicate {
        existing_id: String,
        similarity: f64,
    },
}

/// Verbatim-Dup 写前检查 — 在 KB 写入前检测重复内容。
///
/// 对齐 cumora §5b: "pre-INSERT check: re-query the latest non-self peer
/// message body and compare it to the draft (trimmed). If verbatim-identical
/// → ROLLBACK + HELD."
///
/// NeoTrix 版本:
/// - 已有节点 title + content 的指纹 → 新写入 title + content 的指纹
/// - hash 相同 → Duplicate (拒绝)
/// - hash 不同但 title 相同 → NearDuplicate (标记审查)
/// - 都不同 → Unique (放行)
pub fn check_verbatim_dup(
    new_title: &str,
    new_content: &str,
    existing_entries: &[(String, String, String, u64)], // (id, title, content, created_at_ms)
) -> VerbatimDupResult {
    let new_fp = verbatim_content_fingerprint(&format!("{} {}", new_title, new_content));

    for (id, title, content, created_at) in existing_entries {
        let existing_fp = verbatim_content_fingerprint(&format!("{} {}", title, content));
        if existing_fp.hash == new_fp.hash {
            return VerbatimDupResult::Duplicate {
                existing_id: id.clone(),
                existing_title: title.clone(),
                created_at_ms: *created_at,
            };
        }
        // title 相同但 content 不同 → near-duplicate
        if normalize_content(new_title) == normalize_content(title) {
            return VerbatimDupResult::NearDuplicate {
                existing_id: id.clone(),
                similarity: 0.8, // title 匹配, content 不匹配
            };
        }
    }

    VerbatimDupResult::Unique
}

/// Hold 冲突检测 — 检查是否有未消费的 hold token 与当前写操作冲突。
pub fn has_hold_conflict(scope: &str) -> bool {
    neotrix_types::write_guard_types::GLOBAL_HOLD_TOKENS.has_valid(scope)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_fallback_single_small_no_conflict() {
        let v = deterministic_fallback(1, 100, false, "503 no accounts");
        assert_eq!(v, DeterministicVerdict::FallbackAllow);
    }

    #[test]
    fn test_deterministic_fallback_multiple_pending() {
        let v = deterministic_fallback(3, 100, false, "503");
        assert!(matches!(v, DeterministicVerdict::FallbackReject(_)));
    }

    #[test]
    fn test_deterministic_fallback_large_content() {
        let v = deterministic_fallback(1, 5000, false, "503");
        assert!(matches!(v, DeterministicVerdict::FallbackReject(_)));
    }

    #[test]
    fn test_deterministic_fallback_hold_conflict() {
        let v = deterministic_fallback(1, 100, true, "503");
        assert!(matches!(v, DeterministicVerdict::FallbackReject(_)));
    }

    #[test]
    fn test_deterministic_fallback_zero_pending() {
        let v = deterministic_fallback(0, 100, false, "503");
        assert_eq!(v, DeterministicVerdict::ClassifierAvailable);
    }

    #[test]
    fn test_normalize_content() {
        assert_eq!(normalize_content("  Hello  World  "), "hello world");
        assert_eq!(normalize_content("a\n\n\nb"), "a b");
        assert_eq!(normalize_content(""), "");
    }

    #[test]
    fn test_content_fingerprint_deterministic() {
        let fp1 = verbatim_content_fingerprint("Hello World");
        let fp2 = verbatim_content_fingerprint("hello world");
        assert_eq!(fp1.hash, fp2.hash); // case-insensitive
        let fp3 = verbatim_content_fingerprint("Hello  World"); // extra space
        assert_eq!(fp1.hash, fp3.hash); // whitespace collapsed
    }

    #[test]
    fn test_verbatim_dup_unique() {
        let existing = vec![
            ("id1".into(), "Rust".into(), "systems language".into(), 1000),
        ];
        let r = check_verbatim_dup("Python", "scripting language", &existing);
        assert_eq!(r, VerbatimDupResult::Unique);
    }

    #[test]
    fn test_verbatim_dup_exact_match() {
        let existing = vec![
            ("id1".into(), "Rust".into(), "systems language".into(), 1000),
        ];
        let r = check_verbatim_dup("rust", "systems language", &existing);
        assert!(matches!(r, VerbatimDupResult::Duplicate { ref existing_id, .. } if existing_id == "id1"));
    }

    #[test]
    fn test_verbatim_dup_same_title_different_content() {
        let existing = vec![
            ("id1".into(), "Rust".into(), "v1 content".into(), 1000),
        ];
        let r = check_verbatim_dup("rust", "v2 content", &existing);
        assert!(matches!(r, VerbatimDupResult::NearDuplicate { .. }));
    }

    #[test]
    fn test_verbatim_dup_empty_existing() {
        let r = check_verbatim_dup("title", "content", &[]);
        assert_eq!(r, VerbatimDupResult::Unique);
    }
}
