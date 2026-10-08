#![forbid(unsafe_code)]

//! R-P117 ADD-only writes — every memory write is an append, never UPDATE/DELETE.
//!
//! `AddOnlyMemoryEntry` is the fundamental unit: facts carry temporal windows (R-P120)
//! and supersession pointers, so invalidation closes the window without mutating
//! the original record.

use serde::{Deserialize, Serialize};
use std::fmt;

use std::time::{SystemTime, UNIX_EPOCH};

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Unique identifier for a memory entry.
pub type EntryId = String;

/// A single immutable fact with temporal validity window.
///
/// Fields:
/// - `id`: unique, monotonically increasing within a store
/// - `content`: the fact body (opaque string)
/// - `created_at`: wall-clock write timestamp (Unix seconds)
/// - `valid_from`: earliest timestamp at which this fact holds
/// - `valid_to`: latest timestamp (exclusive); `None` means "currently valid"
/// - `superseded_by`: if invalidated, the entry that replaced this one
/// - `source`: provenance tag (session id, tool name, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// ⚠️ 2026-09-30 修正：此处原为 `pub struct AddOnlyAddOnlyMemoryEntry` ——
/// **「AddOnly」前缀被重复拼接了两次**，是一次未完成的批量改名事故
/// （`AddOnlyMemoryEntry` → `AddOnlyMemoryEntry` 改名时对已改名类型又改了一次）。
///
/// 后果：同目录另外 3 个文件都 `use super::memory_entry::AddOnlyMemoryEntry`
/// （期望 `AddOnlyMemoryEntry`），而全仓**正确拼法 `AddOnlyMemoryEntry` 出现 0 处**
/// ⇒ 该模块因此产生 8 个编译错误，被 `nt_memory/mod.rs` 整簇注释掉，
/// 从此再不参与编译（4 文件 / 789 行）。
///
/// 判定：其余 3 个文件一律用 `AddOnlyMemoryEntry`，且 `AddOnlyMemoryEntry` 全仓 0 处
/// ⇒ 真实意图就是 `AddOnlyMemoryEntry`，改**定义处这一处**即完全自洽。
pub struct AddOnlyMemoryEntry {
    pub id: EntryId,
    pub content: String,
    pub created_at: i64,
    pub valid_from: i64,
    pub valid_to: Option<i64>,
    pub superseded_by: Option<EntryId>,
    pub source: String,
}

impl AddOnlyMemoryEntry {
    /// Create a new entry valid from now, with no expiry.
    pub fn new(content: impl Into<String>, source: impl Into<String>) -> Self {
        let ts = now_ts();
        Self {
            id: String::new(), // caller assigns
            content: content.into(),
            created_at: ts,
            valid_from: ts,
            valid_to: None,
            superseded_by: None,
            source: source.into(),
        }
    }

    /// Builder: override valid_from.
    pub fn valid_from(mut self, ts: i64) -> Self {
        self.valid_from = ts;
        self
    }

    /// Builder: set a closed validity window [from, to).
    pub fn valid_window(mut self, from: i64, to: i64) -> Self {
        self.valid_from = from;
        self.valid_to = Some(to);
        self
    }

    /// Builder: override created_at (for replay / import).
    pub fn created_at(mut self, ts: i64) -> Self {
        self.created_at = ts;
        self
    }

    /// Builder: set id explicitly (for deterministic ids).
    /// **显式时间**构造（2026-09-30 新增）。
    ///
    /// ⛔ 为什么需要它：`new()` 用 `now_ts()`（**真实时钟**）当 `valid_from`，
    /// 于是「刚创建」这条事实在**过去的时间点**上无效。本模块两个自带测试
    /// 正是因此失败：
    /// · `entry_with_no_valid_to_is_always_valid` 断言 `is_valid_at(0)`，
    ///   但 `valid_from ≈ 1.7e9` ⇒ 必假；
    /// · `query_at_point_in_time` 同理。
    ///
    /// ⚠️ 这不只是测试问题：**时态数据的构造必须可控**，
    /// 否则任何「在某历史时刻查询」的调用都无法复现。
    /// ⇒ 提供显式时间入口，生产侧也该用它做回放/导入/迁移。
    pub fn at(content: impl Into<String>, source: impl Into<String>, ts: i64) -> Self {
        Self {
            id: String::new(), // caller assigns
            content: content.into(),
            created_at: ts,
            valid_from: ts,
            valid_to: None,
            superseded_by: None,
            source: source.into(),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Is this fact valid at timestamp `t`?
    #[inline]
    pub fn is_valid_at(&self, t: i64) -> bool {
        self.valid_from <= t && self.valid_to.is_none_or(|to| t < to)
    }

    /// Is this fact currently valid (no valid_to, or valid_to > now)?
    #[inline]
    pub fn is_valid_now(&self) -> bool {
        self.valid_to.is_none_or(|to| to > now_ts())
    }

    /// Has this entry been superseded?
    #[inline]
    pub fn is_superseded(&self) -> bool {
        self.superseded_by.is_some()
    }

    /// Close the validity window (set valid_to = now). Does NOT mutate content.
    pub fn invalidate(&mut self, replaced_by: &str) {
        self.valid_to = Some(now_ts());
        self.superseded_by = Some(replaced_by.to_string());
    }
}

impl fmt::Display for AddOnlyMemoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match (&self.valid_to, &self.superseded_by) {
            (None, None) => "active",
            (Some(_), Some(_)) => "superseded",
            (Some(_), None) => "expired",
            (None, Some(_)) => "superseded(no-expiry)",
        };
        write!(
            f,
            "[{}] {} (valid {}..{}, {})",
            &self.id[..self.id.len().min(12)],
            &self.content[..self.content.len().min(40)],
            self.valid_from,
            self.valid_to.map_or("now".to_string(), |t| t.to_string()),
            status,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_is_valid_at_respects_window() {
        let e = AddOnlyMemoryEntry::new("fact", "test")
            .with_id("e1")
            .valid_window(100, 200);
        assert!(!e.is_valid_at(50));
        assert!(e.is_valid_at(100));
        assert!(e.is_valid_at(199));
        assert!(!e.is_valid_at(200));
    }

    #[test]
    fn entry_with_no_valid_to_is_always_valid() {
        // 用**显式时间**构造（`new()` 走真实时钟，见 `at()` 的文档）：
        // ts=0 ⇒ 整条时间轴上「始终有效」才是可断言的。
        let e = AddOnlyMemoryEntry::at("fact", "test", 0).with_id("e2");
        assert!(e.is_valid_at(0));
        assert!(e.is_valid_at(i64::MAX));
    }

    #[test]
    fn invalidate_closes_window() {
        let mut e = AddOnlyMemoryEntry::new("old", "test").with_id("e3");
        e.invalidate("e4");
        assert!(e.is_superseded());
        assert!(e.valid_to.is_some());
        assert_eq!(e.superseded_by.as_deref(), Some("e4"));
    }

    #[test]
    fn display_truncates_long_content() {
        let long = "x".repeat(100);
        let e = AddOnlyMemoryEntry::new(&long, "test").with_id("a_very_long_id_here");
        let s = e.to_string();
        assert!(s.len() < 120);
    }

    #[test]
    fn entry_default_state_not_superseded() {
        let e = AddOnlyMemoryEntry::new("fact", "test").with_id("e1");
        assert!(!e.is_superseded());
        assert!(e.superseded_by.is_none());
        assert!(e.valid_to.is_none());
    }

    #[test]
    fn entry_valid_from_builder() {
        let e = AddOnlyMemoryEntry::new("fact", "test").valid_from(500);
        assert_eq!(e.valid_from, 500);
    }

    #[test]
    fn entry_created_at_builder() {
        let e = AddOnlyMemoryEntry::new("fact", "test").created_at(999);
        assert_eq!(e.created_at, 999);
    }

    #[test]
    fn entry_valid_window_sets_both() {
        let e = AddOnlyMemoryEntry::new("fact", "test").valid_window(10, 20);
        assert_eq!(e.valid_from, 10);
        assert_eq!(e.valid_to, Some(20));
    }

    #[test]
    fn entry_valid_now_with_no_expiry() {
        let e = AddOnlyMemoryEntry::new("fact", "test");
        assert!(e.is_valid_now());
    }

    #[test]
    fn entry_with_id_preserves_id() {
        let e = AddOnlyMemoryEntry::new("fact", "test").with_id("my_id");
        assert_eq!(e.id, "my_id");
    }
}
