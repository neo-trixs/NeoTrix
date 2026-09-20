#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

use super::estate::MemoryEstate;

/// A single typed memory entry with full metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedMemoryEntry {
    pub id: String,
    pub estate: MemoryEstate,
    pub content: String,
    pub confidence: f64,
    pub timestamp: i64,
    pub ttl: Option<i64>,
    pub conflicts: Vec<ConflictRecord>,
    pub access_count: u64,
}

/// A record of a conflict between this entry and another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRecord {
    pub conflicting_id: String,
    pub strategy: ConflictStrategy,
    pub resolved_at: i64,
    pub outcome: ConflictOutcome,
}

/// The outcome of a conflict resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictOutcome {
    Resolved,
    Superseded,
    Merged,
    Pending,
}

/// Available conflict resolution strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictStrategy {
    LatestWins,
    HighestConfidence,
    MajorityVote,
    Manual,
}

impl ConflictStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LatestWins => "latest_wins",
            Self::HighestConfidence => "highest_confidence",
            Self::MajorityVote => "majority_vote",
            Self::Manual => "manual",
        }
    }
}

impl TypedMemoryEntry {
    pub fn new(id: String, estate: MemoryEstate, content: String, confidence: f64) -> Self {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        Self {
            id, estate,
            content,
            confidence: confidence.clamp(0.0, 1.0),
            timestamp,
            ttl: None,
            conflicts: Vec::new(),
            access_count: 0,
        }
    }

    pub fn with_ttl(mut self, ttl_seconds: i64) -> Self { self.ttl = Some(ttl_seconds); self }
    pub fn with_conflicts(mut self, conflicts: Vec<ConflictRecord>) -> Self { self.conflicts = conflicts; self }

    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
            self.timestamp + ttl < now
        } else { false }
    }

    pub fn record_access(&mut self) { self.access_count += 1; }
    pub fn add_conflict(&mut self, record: ConflictRecord) { self.conflicts.push(record); }
    pub fn decay_confidence(&mut self, factor: f64) { self.confidence = (self.confidence * factor).clamp(0.0, 1.0); }
}

impl Default for TypedMemoryEntry {
    fn default() -> Self {
        Self { id: String::new(), estate: MemoryEstate::default(), content: String::new(), confidence: 0.5, timestamp: 0, ttl: None, conflicts: Vec::new(), access_count: 0 }
    }
}

/// In-memory store for typed memory entries.
#[derive(Debug)]
pub struct TypedMemoryStore {
    entries: HashMap<String, TypedMemoryEntry>,
    hot_cache: lru::LruCache<String, ()>,
    capacity: usize,
}

impl TypedMemoryStore {
    pub fn new(capacity: usize) -> Self {
        Self { entries: HashMap::with_capacity(capacity), hot_cache: lru::LruCache::new(std::num::NonZeroUsize::new(capacity.min(1000)).unwrap_or(std::num::NonZeroUsize::new(1).unwrap())), capacity }
    }
    pub fn insert(&mut self, entry: TypedMemoryEntry) -> Option<TypedMemoryEntry> {
        if self.entries.len() >= self.capacity && !self.entries.contains_key(&entry.id) {
            if let Some((oldest_id, _)) = self.entries.iter().min_by_key(|(_, e)| e.timestamp) {
                let oldest_id = oldest_id.clone();
                self.entries.remove(&oldest_id);
            }
        }
        let old = self.entries.insert(entry.id.clone(), entry);
        old
    }
    pub fn get(&mut self, id: &str) -> Option<&TypedMemoryEntry> {
        let entry = self.entries.get(id)?;
        if !entry.is_expired() { let _ = self.hot_cache.put(id.to_string(), ()); Some(entry) } else { None }
    }
    pub fn get_mut(&mut self, id: &str) -> Option<&mut TypedMemoryEntry> { self.entries.get_mut(id) }
    pub fn remove(&mut self, id: &str) -> bool { self.entries.remove(id).is_some() }
    pub fn entries_by_estate(&self, estate: MemoryEstate) -> Vec<&TypedMemoryEntry> { self.entries.values().filter(|e| e.estate == estate).collect() }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn purge_expired(&mut self) -> usize {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        let expired: Vec<String> = self.entries.iter().filter(|(_, e)| e.ttl.map_or(false, |ttl| e.timestamp + ttl < now)).map(|(id, _)| id.clone()).collect();
        for id in &expired { self.entries.remove(id); }
        expired.len()
    }
}

impl Default for TypedMemoryStore { fn default() -> Self { Self::new(1000) } }

#[cfg(test)]
mod tests {
    use super::*;
    fn make_entry(estate: MemoryEstate) -> TypedMemoryEntry { TypedMemoryEntry::new(format!("test-{}", estate.as_str()), estate, format!("content for {}", estate.as_str()), 0.9) }
    #[test] fn test_entry_creation() { let e = make_entry(MemoryEstate::Semantic); assert_eq!(e.estate, MemoryEstate::Semantic); assert_eq!(e.confidence, 0.9); }
    #[test] fn test_entry_confidence_clamp() { let e = TypedMemoryEntry::new("t".into(), MemoryEstate::Working, "c".into(), 1.5); assert_eq!(e.confidence, 1.0); let e2 = TypedMemoryEntry::new("t2".into(), MemoryEstate::Working, "c2".into(), -0.5); assert_eq!(e2.confidence, 0.0); }
    #[test] fn test_store_insert_and_get() { let mut s = TypedMemoryStore::new(10); let e = make_entry(MemoryEstate::Semantic); s.insert(e.clone()); assert_eq!(s.len(), 1); assert!(s.get(&e.id).is_some()); }
    #[test] fn test_store_purge_expired() { let mut s = TypedMemoryStore::new(10); let mut e = make_entry(MemoryEstate::Reflexive); e.ttl = Some(-1); s.insert(e); let mut e2 = make_entry(MemoryEstate::Episodic); s.insert(e2); assert_eq!(s.purge_expired(), 1); }
    #[test] fn test_decay_confidence() { let mut e = TypedMemoryEntry::new("d1".into(), MemoryEstate::Semantic, "c".into(), 0.8); e.decay_confidence(0.5); assert_eq!(e.confidence, 0.4); }
}
