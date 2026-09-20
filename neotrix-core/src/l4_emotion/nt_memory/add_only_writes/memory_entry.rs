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
pub struct AddOnlyAddOnlyMemoryEntry {
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
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Is this fact valid at timestamp `t`?
    #[inline]
    pub fn is_valid_at(&self, t: i64) -> bool {
        self.valid_from <= t && self.valid_to.map_or(true, |to| t < to)
    }

    /// Is this fact currently valid (no valid_to, or valid_to > now)?
    #[inline]
    pub fn is_valid_now(&self) -> bool {
        self.valid_to.map_or(true, |to| to > now_ts())
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
        let e = AddOnlyMemoryEntry::new("fact", "test").with_id("e2");
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
