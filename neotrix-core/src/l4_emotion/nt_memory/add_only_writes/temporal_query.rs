#![forbid(unsafe_code)]

//! R-P120 Temporal fact windows — point-in-time and "now" queries over ADD-only entries.
//!
//! `TemporalQuery` specifies which snapshot of the store to inspect,
//! enabling time-travel without mutating any stored fact.

use serde::{Deserialize, Serialize};

use super::memory_entry::AddOnlyMemoryEntry;

/// A time-travel query descriptor.
///
/// Variants:
/// - `Now`: returns entries valid at the current wall-clock time.
/// - `At(i64)`: returns entries valid at a specific Unix timestamp.
/// - `Range { from, to }`: returns entries valid at any point within [from, to).
/// - `Latest { subject_prefix }`: returns the most-recently-created entry whose id
///   starts with `subject_prefix`, regardless of temporal validity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TemporalQuery {
    /// Currently valid entries.
    Now,
    /// Point-in-time: valid at timestamp `t`.
    At(i64),
    /// Range: entries valid at any point within [from, to).
    Range { from: i64, to: i64 },
    /// Latest entry by id prefix (fast lookup, ignores temporal window).
    LatestByPrefix { prefix: String },
    /// All entries ever (regardless of validity).
    All,
}

impl TemporalQuery {
    pub fn now() -> Self {
        Self::Now
    }

    pub fn at(t: i64) -> Self {
        Self::At(t)
    }

    pub fn range(from: i64, to: i64) -> Self {
        Self::Range { from, to }
    }

    pub fn latest_by_prefix(prefix: impl Into<String>) -> Self {
        Self::LatestByPrefix {
            prefix: prefix.into(),
        }
    }

    pub fn all() -> Self {
        Self::All
    }

    /// Test whether a `AddOnlyMemoryEntry` satisfies this query.
    ///
    /// ⚠️ **`now` 只对 [`Self::Now`] 分支生效**（2026-09-30 补注）。
    /// 其余分支各用**自己的**时间语义：`At(t)` 看 `t`、`Range` 看自身区间、
    /// `LatestByPrefix`/`All` 与时间无关。
    ///
    /// 这曾是一个**真实 API 陷阱**：模块自带测试
    /// `query_at_point_in_time` 断言 `!at(150).matches(&e, 250)`
    /// （以为 `now=250` 会参与判断），实际实现忽略 `now`
    /// ⇒ 该断言**期望写错**，已按实现语义修正。
    /// 传 `now` 给非 `Now` 分支是无意义的，但为保持调用点签名统一而保留。
    pub fn matches(&self, entry: &AddOnlyMemoryEntry, now: i64) -> bool {
        match self {
            Self::Now => entry.is_valid_at(now),
            Self::At(t) => entry.is_valid_at(*t),
            Self::Range { from, to } => {
                // Entry is relevant if its validity window overlaps [from, to).
                entry.valid_from < *to && entry.valid_to.map_or(true, |vt| vt > *from)
            }
            Self::LatestByPrefix { prefix } => entry.id.starts_with(prefix),
            Self::All => true,
        }
    }
}

/// Result of filtering entries through a `TemporalQuery`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Entries that matched, ordered by `created_at` ascending.
    pub entries: Vec<AddOnlyMemoryEntry>,
    /// The query that produced this result (for auditing).
    pub query: TemporalQuery,
    /// Wall-clock time at which the query was evaluated.
    pub evaluated_at: i64,
}

impl QueryResult {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Convenience: return only the content strings.
    pub fn contents(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.content.as_str()).collect()
    }

    /// Convenience: return only the entry ids.
    pub fn ids(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.id.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l4_emotion::nt_memory::add_only_writes::memory_entry::AddOnlyMemoryEntry;

    fn entry(id: &str, from: i64, to: Option<i64>) -> AddOnlyMemoryEntry {
        AddOnlyMemoryEntry::new(format!("content-{id}"), "test")
            .with_id(id)
            .valid_window(from, to.unwrap_or(i64::MAX))
    }

    #[test]
    fn query_now_filters_by_current_validity() {
        let e1 = entry("a", 100, Some(500)); // expired
        let e2 = entry("b", 100, None); // still valid
        let q = TemporalQuery::Now;
        let now = 300;
        assert!(q.matches(&e1, now));
        assert!(q.matches(&e2, now));

        let now2 = 600;
        assert!(!q.matches(&e1, now2));
        assert!(q.matches(&e2, now2));
    }

    #[test]
    fn query_at_point_in_time() {
        // entry 有效期 [100, 200)；查询锚点 150 ⇒ 命中。
        // ⚠️ `now` 参数只对 `Now` 分支生效，此处传什么都不影响 `At` 的判定。
        let e = entry("a", 100, Some(200));
        let q = TemporalQuery::at(150);
        assert!(q.matches(&e, 0));
        assert!(q.matches(&e, 250));
        // 真正的「不命中」要换查询锚点，而不是换 `now`：
        assert!(!TemporalQuery::at(250).matches(&e, 0));
        assert!(!TemporalQuery::at(50).matches(&e, 0));
    }

    #[test]
    fn query_range_overlaps() {
        let e = entry("a", 100, Some(200));
        // range fully inside validity
        assert!(TemporalQuery::range(120, 180).matches(&e, 0));
        // range starts before validity, ends inside
        assert!(TemporalQuery::range(50, 150).matches(&e, 0));
        // range entirely after validity
        assert!(!TemporalQuery::range(200, 300).matches(&e, 0));
        // range entirely before validity
        assert!(!TemporalQuery::range(0, 99).matches(&e, 0));
    }

    #[test]
    fn query_result_contents_and_ids() {
        let entries = vec![entry("a", 100, None), entry("b", 200, None)];
        let result = QueryResult {
            entries: entries.clone(),
            query: TemporalQuery::Now,
            evaluated_at: 150,
        };
        assert_eq!(result.ids(), vec!["a", "b"]);
        assert_eq!(result.contents().len(), 2);
        assert!(!result.is_empty());
    }

    #[test]
    fn query_all_matches_everything() {
        let e = entry("a", 100, Some(150));
        assert!(TemporalQuery::All.matches(&e, 0));
        assert!(TemporalQuery::All.matches(&e, 200));
    }

    #[test]
    fn query_latest_by_prefix() {
        let e1 = entry("ent/001", 100, None);
        let e2 = entry("ent/002", 200, None);
        let e3 = entry("other/001", 300, None);
        let q = TemporalQuery::latest_by_prefix("ent/");
        assert!(q.matches(&e1, 0));
        assert!(q.matches(&e2, 0));
        assert!(!q.matches(&e3, 0));
    }

    #[test]
    fn query_range_no_overlap() {
        let e = entry("a", 100, Some(200));
        assert!(!TemporalQuery::range(300, 400).matches(&e, 0));
    }

    #[test]
    fn query_at_before_valid_from() {
        let e = entry("a", 100, Some(200));
        assert!(!TemporalQuery::at(50).matches(&e, 0));
    }

    #[test]
    fn query_result_empty() {
        let result = QueryResult {
            entries: vec![],
            query: TemporalQuery::Now,
            evaluated_at: 0,
        };
        assert!(result.is_empty());
        assert_eq!(result.len(), 0);
        assert!(result.ids().is_empty());
    }

    #[test]
    fn query_now_methods() {
        let q = TemporalQuery::now();
        assert!(matches!(q, TemporalQuery::Now));
        let q = TemporalQuery::at(100);
        assert!(matches!(q, TemporalQuery::At(100)));
        let q = TemporalQuery::all();
        assert!(matches!(q, TemporalQuery::All));
    }
}
