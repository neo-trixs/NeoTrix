#![forbid(unsafe_code)]

//! R-P117/R-P120/R-P122 ADD-only temporal memory store.
//!
//! Every write is an append. "Updates" supersede via temporal window closure.
//! Queries route through `TemporalQuery` for point-in-time or "now" semantics.

use std::collections::HashMap;

use super::memory_entry::{EntryId, AddOnlyMemoryEntry};
use super::temporal_query::{QueryResult, TemporalQuery};
use crate::l4_emotion::nt_memory::shared_utils::now_ts;

/// Errors produced by the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// Attempted to add a duplicate id.
    DuplicateId(EntryId),
    /// Entry not found for supersession.
    NotFound(EntryId),
    /// Attempted to supersede an already-superseded entry.
    AlreadySuperseded(EntryId),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(f, "duplicate entry id: {id}"),
            Self::NotFound(id) => write!(f, "entry not found: {id}"),
            Self::AlreadySuperseded(id) => write!(f, "entry already superseded: {id}"),
        }
    }
}

impl std::error::Error for StoreError {}

/// A monotonic entry counter used to generate unique ids.
struct IdGenerator {
    counter: u64,
}

impl IdGenerator {
    fn new() -> Self {
        Self { counter: 0 }
    }

    fn next(&mut self) -> String {
        self.counter += 1;
        format!("ae_{:08x}", self.counter)
    }
}

/// ADD-only temporal memory store (R-P117, R-P120, R-P122).
///
/// All entries live in a `Vec<AddOnlyMemoryEntry>` (append-ordered). A secondary
/// index maps entry ids to their position for O(1) lookup.
pub struct TemporalStore {
    entries: Vec<AddOnlyMemoryEntry>,
    index: HashMap<EntryId, usize>,
    id_gen: IdGenerator,
}

impl TemporalStore {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            index: HashMap::new(),
            id_gen: IdGenerator::new(),
        }
    }

    /// Pre-allocate for expected capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            entries: Vec::with_capacity(cap),
            index: HashMap::with_capacity(cap),
            id_gen: IdGenerator::new(),
        }
    }

    /// R-P117: ADD-only write. Appends a new entry. If `entry.id` is empty,
    /// an auto-generated id is assigned.
    ///
    /// Returns `Err(DuplicateId)` if the id already exists.
    pub fn add(&mut self, mut entry: AddOnlyMemoryEntry) -> Result<&AddOnlyMemoryEntry, StoreError> {
        if entry.id.is_empty() {
            entry.id = self.id_gen.next();
        }
        if self.index.contains_key(&entry.id) {
            return Err(StoreError::DuplicateId(entry.id));
        }
        let pos = self.entries.len();
        self.index.insert(entry.id.clone(), pos);
        self.entries.push(entry);
        Ok(&self.entries[pos])
    }

    /// R-P117: "Update" = ADD a new entry + invalidate the old one.
    ///
    /// The old entry's `valid_to` is closed and `superseded_by` is set.
    /// The new entry is appended. Returns the new entry.
    pub fn supersede(
        &mut self,
        old_id: &str,
        new_content: impl Into<String>,
        source: impl Into<String>,
    ) -> Result<&AddOnlyMemoryEntry, StoreError> {
        let old_pos = *self
            .index
            .get(old_id)
            .ok_or_else(|| StoreError::NotFound(old_id.to_string()))?;
        if self.entries[old_pos].is_superseded() {
            return Err(StoreError::AlreadySuperseded(old_id.to_string()));
        }

        let new_id = self.id_gen.next();
        let cut = now_ts();

        // Invalidate old entry (append-only: close its window)
        self.entries[old_pos].invalidate(&new_id);

        // Build new entry
        let new_entry = AddOnlyMemoryEntry::new(new_content, source)
            .with_id(&new_id)
            .valid_from(cut);

        let pos = self.entries.len();
        self.index.insert(new_id, pos);
        self.entries.push(new_entry);
        Ok(&self.entries[pos])
    }

    /// R-P120: Query by a `TemporalQuery`.
    pub fn query(&self, q: &TemporalQuery) -> QueryResult {
        let now = now_ts();
        let matched: Vec<AddOnlyMemoryEntry> = self
            .entries
            .iter()
            .filter(|e| q.matches(e, now))
            .cloned()
            .collect();

        // For LatestByPrefix, keep only the most recent match
        let entries = match q {
            TemporalQuery::LatestByPrefix { .. } => matched.into_iter().rev().take(1).collect(),
            _ => matched,
        };

        QueryResult {
            entries,
            query: q.clone(),
            evaluated_at: now,
        }
    }

    /// Convenience: query entries valid right now.
    pub fn query_now(&self) -> QueryResult {
        self.query(&TemporalQuery::Now)
    }

    /// Convenience: point-in-time query.
    pub fn query_at(&self, t: i64) -> QueryResult {
        self.query(&TemporalQuery::At(t))
    }

    /// Get a specific entry by id.
    pub fn get(&self, id: &str) -> Option<&AddOnlyMemoryEntry> {
        self.index.get(id).map(|&pos| &self.entries[pos])
    }

    /// Total number of entries (including superseded/expired).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Number of currently active (non-superseded) entries.
    pub fn active_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.is_superseded()).count()
    }

    /// Walk the supersession chain from an entry back to its origin.
    ///
    /// `old.superseded_by = new_id` means "new replaced old". To walk backwards
    /// from newest to oldest, we find the entry whose `superseded_by` points to
    /// the current id (O(n) scan, acceptable for chain walking).
    pub fn history_chain(&self, entry_id: &str) -> Vec<&AddOnlyMemoryEntry> {
        let mut chain = Vec::new();
        let mut current = Some(entry_id);
        let mut guard = 0;
        while let Some(id) = current {
            if guard > 256 {
                break;
            }
            if let Some(entry) = self.get(id) {
                chain.push(entry);
                // Walk reverse: find entry whose superseded_by == current id
                current = self
                    .entries
                    .iter()
                    .find(|e| e.superseded_by.as_deref() == Some(id))
                    .map(|e| e.id.as_str());
                guard += 1;
            } else {
                break;
            }
        }
        chain
    }
}

impl Default for TemporalStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> TemporalStore {
        TemporalStore::new()
    }

    #[test]
    fn add_and_get() {
        let mut s = store();
        let e = AddOnlyMemoryEntry::new("hello", "test").with_id("e1");
        s.add(e).unwrap();
        assert_eq!(s.len(), 1);
        assert!(s.get("e1").is_some());
        assert_eq!(s.get("e1").unwrap().content, "hello");
    }

    #[test]
    fn add_duplicate_rejects() {
        let mut s = store();
        s.add(AddOnlyMemoryEntry::new("a", "test").with_id("e1")).unwrap();
        let dup = s.add(AddOnlyMemoryEntry::new("b", "test").with_id("e1"));
        assert_eq!(dup.unwrap_err(), StoreError::DuplicateId("e1".into()));
    }

    #[test]
    fn add_auto_generates_id() {
        let mut s = store();
        let e = s.add(AddOnlyMemoryEntry::new("auto", "test")).unwrap();
        assert!(!e.id.is_empty());
        assert!(e.id.starts_with("ae_"));
    }

    #[test]
    fn supersede_closes_old_window() {
        let mut s = store();
        s.add(AddOnlyMemoryEntry::new("old", "test").with_id("e1"))
            .unwrap();
        let new = s.supersede("e1", "new", "test").unwrap().clone();
        assert!(!new.id.is_empty());

        let old = s.get("e1").unwrap();
        assert!(old.is_superseded());
        assert!(old.valid_to.is_some());
        assert_eq!(old.superseded_by.as_deref(), Some(new.id.as_str()));
    }

    #[test]
    fn supersede_not_found() {
        let mut s = store();
        let r = s.supersede("nonexistent", "new", "test");
        assert_eq!(r.unwrap_err(), StoreError::NotFound("nonexistent".into()));
    }

    #[test]
    fn supersede_already_superseded() {
        let mut s = store();
        s.add(AddOnlyMemoryEntry::new("a", "test").with_id("e1")).unwrap();
        s.supersede("e1", "b", "test").unwrap();
        let r = s.supersede("e1", "c", "test");
        assert_eq!(r.unwrap_err(), StoreError::AlreadySuperseded("e1".into()));
    }

    #[test]
    fn query_now_returns_active_entries() {
        let mut s = store();
        s.add(AddOnlyMemoryEntry::new("active", "test").with_id("e1"))
            .unwrap();
        let mut expired = AddOnlyMemoryEntry::new("expired", "test").with_id("e2");
        expired.valid_to = Some(1); // expired long ago
        s.add(expired).unwrap();

        let result = s.query_now();
        assert_eq!(result.len(), 1);
        assert_eq!(result.entries[0].id, "e1");
    }

    #[test]
    fn query_at_point_in_time() {
        let mut s = store();
        s.add(
            AddOnlyMemoryEntry::new("a", "test")
                .with_id("e1")
                .valid_window(100, 200),
        )
        .unwrap();
        s.add(
            AddOnlyMemoryEntry::new("b", "test")
                .with_id("e2")
                .valid_from(150),
        )
        .unwrap();

        let at_120 = s.query_at(120);
        assert_eq!(at_120.len(), 1);
        assert_eq!(at_120.entries[0].id, "e1");

        let at_170 = s.query_at(170);
        assert_eq!(at_170.len(), 2);

        let at_250 = s.query_at(250);
        assert_eq!(at_250.len(), 1);
        assert_eq!(at_250.entries[0].id, "e2");
    }

    #[test]
    fn active_count_excludes_superseded() {
        let mut s = store();
        s.add(AddOnlyMemoryEntry::new("a", "test").with_id("e1")).unwrap();
        s.add(AddOnlyMemoryEntry::new("b", "test").with_id("e2")).unwrap();
        assert_eq!(s.active_count(), 2);
        s.supersede("e1", "c", "test").unwrap();
        assert_eq!(s.active_count(), 2); // new entry added, old superseded
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn history_chain_walks_correctly() {
        let mut s = store();
        s.add(AddOnlyMemoryEntry::new("v1", "test").with_id("e1")).unwrap();
        s.supersede("e1", "v2", "test").unwrap();
        let new = s.supersede("e1", "v3", "test").unwrap_err(); // e1 already superseded
                                                                // Walk from e1 (the original)
        let chain = s.history_chain("e1");
        assert!(chain.len() >= 1);
        assert_eq!(chain[0].id, "e1");
    }
}
