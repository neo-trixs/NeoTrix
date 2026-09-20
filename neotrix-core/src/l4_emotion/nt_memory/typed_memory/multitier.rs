#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::entry::TypedMemoryEntry;
use super::estate::MemoryEstate;
use lru::LruCache;
use std::num::NonZeroUsize;

/// Hot store (VRAM) — fastest access.
#[derive(Debug)]
pub struct HotStore {
    cache: LruCache<String, TypedMemoryEntry>,
}
/// Warm store (RAM) — medium access.
#[derive(Debug)]
pub struct WarmStore {
    entries: HashMap<String, TypedMemoryEntry>,
}
/// Cold store (NVMe) — persisted.
#[derive(Debug)]
pub struct ColdStore {
    conn: Option<rusqlite::Connection>,
}

/// Learning placement scores for JitForWeights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementScore {
    pub entry_id: String,
    pub estate: MemoryEstate,
    pub hot_score: f64,
    pub warm_score: f64,
    pub cold_score: f64,
    pub last_access: i64,
    pub access_count: u64,
}

/// JitForWeights learning placement algorithm.
#[derive(Debug)]
pub struct JitForWeights {
    hot_weight: f64,
    warm_weight: f64,
    cold_weight: f64,
}

/// Tier assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    Hot,
    Warm,
    Cold,
}

impl HotStore {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(
                NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(1).unwrap()),
            ),
        }
    }
    pub fn insert(&mut self, entry: TypedMemoryEntry) -> Option<TypedMemoryEntry> {
        self.cache.put(entry.id.clone(), entry)
    }
    pub fn get(&mut self, id: &str) -> Option<&TypedMemoryEntry> {
        self.cache.get(id)
    }
    pub fn get_mut(&mut self, id: &str) -> Option<&mut TypedMemoryEntry> {
        self.cache.get_mut(id)
    }
    pub fn remove(&mut self, id: &str) -> Option<TypedMemoryEntry> {
        self.cache.pop(id)
    }
    pub fn len(&self) -> usize {
        self.cache.len()
    }
}

impl WarmStore {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
    pub fn insert(&mut self, entry: TypedMemoryEntry) -> Option<TypedMemoryEntry> {
        self.entries.insert(entry.id.clone(), entry)
    }
    pub fn get(&self, id: &str) -> Option<&TypedMemoryEntry> {
        self.entries.get(id)
    }
    pub fn get_mut(&mut self, id: &str) -> Option<&mut TypedMemoryEntry> {
        self.entries.get_mut(id)
    }
    pub fn remove(&mut self, id: &str) -> Option<TypedMemoryEntry> {
        self.entries.remove(id)
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl ColdStore {
    pub fn new(path: Option<String>) -> Result<Self, String> {
        let conn = if let Some(ref p) = path {
            Some(rusqlite::Connection::open(p).map_err(|e| e.to_string())?)
        } else {
            None
        };
        Ok(Self { conn })
    }
    pub fn insert(&mut self, entry: &TypedMemoryEntry) -> Result<(), String> {
        if let Some(conn) = &self.conn {
            conn.execute("INSERT OR REPLACE INTO typed_memory_cold_archive (id, estate, content, confidence, timestamp, ttl, access_count, compressed_summary, archived_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)", rusqlite::params![entry.id, entry.estate.as_str(), entry.content, entry.confidence, entry.timestamp, entry.ttl, entry.access_count, None::<String>, SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64]).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn get(&self, id: &str) -> Option<TypedMemoryEntry> {
        let conn = self.conn.as_ref()?;
        let mut stmt = conn.prepare("SELECT id, estate, content, confidence, timestamp, ttl, access_count FROM typed_memory_cold_archive WHERE id=?1").ok()?;
        let mut rows = stmt.query(rusqlite::params![id]).ok()?;
        rows.next().ok()?.map(|row| {
            let estate_str: String = row.get(1).unwrap_or_default();
            TypedMemoryEntry {
                id: row.get(0).unwrap_or_default(),
                estate: MemoryEstate::from_str(&estate_str),
                content: row.get(2).unwrap_or_default(),
                confidence: row.get(3).unwrap_or(0.5),
                timestamp: row.get(4).unwrap_or(0),
                ttl: row.get::<_, Option<i64>>(5).unwrap_or(None),
                conflicts: Vec::new(),
                access_count: row.get(6).unwrap_or(0),
            }
        })
    }
    pub fn len(&self) -> usize {
        self.conn
            .as_ref()
            .map(|c| {
                c.query_row("SELECT COUNT(*) FROM typed_memory_cold_archive", [], |r| {
                    r.get(0)
                })
                .unwrap_or(0)
            })
            .unwrap_or(0)
    }
}

impl JitForWeights {
    pub fn new() -> Self {
        Self {
            hot_weight: 0.7,
            warm_weight: 0.2,
            cold_weight: 0.1,
        }
    }
    pub fn with_weights(mut self, hot: f64, warm: f64, cold: f64) -> Self {
        self.hot_weight = hot;
        self.warm_weight = warm;
        self.cold_weight = cold;
        self
    }

    pub fn compute_scores(&self, entries: &[TypedMemoryEntry]) -> Vec<PlacementScore> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        entries
            .iter()
            .map(|entry| {
                let age = (now - entry.timestamp) as f64;
                let recency = 1.0 / (1.0 + age / 3600.0);
                let access_factor = (entry.access_count as f64).min(100.0) / 100.0;
                let hot_score = recency * access_factor * self.hot_weight + entry.confidence * 0.3;
                let warm_score =
                    (1.0 - recency) * access_factor * self.warm_weight + entry.confidence * 0.2;
                let cold_score =
                    (1.0 - recency.min(0.5)) * self.cold_weight + entry.confidence * 0.1;
                PlacementScore {
                    entry_id: entry.id.clone(),
                    estate: entry.estate,
                    hot_score: hot_score.clamp(0.0, 1.0),
                    warm_score: warm_score.clamp(0.0, 1.0),
                    cold_score: cold_score.clamp(0.0, 1.0),
                    last_access: entry.timestamp,
                    access_count: entry.access_count,
                }
            })
            .collect()
    }

    pub fn assign_tier(&self, score: &PlacementScore) -> Tier {
        if score.hot_score > 0.6 && score.access_count > 5 {
            Tier::Hot
        } else if score.warm_score > score.cold_score {
            Tier::Warm
        } else {
            Tier::Cold
        }
    }
}

/// MemoryMultitier: HotStore (VRAM) → WarmStore (RAM) → ColdStore (NVMe).
#[derive(Debug)]
pub struct MemoryMultitier {
    hot: HotStore,
    warm: WarmStore,
    cold: ColdStore,
    jit: JitForWeights,
    capacity_hot: usize,
    capacity_warm: usize,
}

impl MemoryMultitier {
    pub fn new(capacity_hot: usize, capacity_warm: usize) -> Result<Self, String> {
        Ok(Self {
            hot: HotStore::new(capacity_hot),
            warm: WarmStore::new(),
            cold: ColdStore::new(None)?,
            jit: JitForWeights::new(),
            capacity_hot,
            capacity_warm,
        })
    }
    pub fn with_cold_path(
        capacity_hot: usize,
        capacity_warm: usize,
        cold_path: String,
    ) -> Result<Self, String> {
        Ok(Self {
            hot: HotStore::new(capacity_hot),
            warm: WarmStore::new(),
            cold: ColdStore::new(Some(cold_path))?,
            jit: JitForWeights::new(),
            capacity_hot,
            capacity_warm,
        })
    }

    pub fn insert(&mut self, entry: TypedMemoryEntry) -> Tier {
        let scores = self.jit.compute_scores(&[entry.clone()]);
        if let Some(score) = scores.first() {
            let tier = self.jit.assign_tier(score);
            match tier {
                Tier::Hot => {
                    self.hot.insert(entry);
                }
                Tier::Warm => {
                    self.warm.insert(entry);
                }
                Tier::Cold => {
                    let _ = self.cold.insert(&entry);
                }
            }
            tier
        } else {
            Tier::Warm
        }
    }

    pub fn get(&mut self, id: &str) -> Option<TypedMemoryEntry> {
        let found_in_hot = self.hot.get(id).cloned();
        if found_in_hot.is_some() {
            return found_in_hot;
        }
        let found_in_warm = self.warm.get(id).cloned();
        if found_in_warm.is_some() {
            return found_in_warm;
        }
        let entry = self.cold.get(id)?.clone();
        self.hot.insert(entry.clone());
        Some(entry)
    }
    pub fn get_mut(&mut self, id: &str) -> Option<&mut TypedMemoryEntry> {
        if let Some(entry) = self.hot.get_mut(id) {
            return Some(entry);
        }
        if let Some(entry) = self.warm.get_mut(id) {
            return Some(entry);
        }
        None
    }
    pub fn sync_to_cold(&mut self) -> Result<(), String> {
        for (_, entry) in self.warm.entries.iter() {
            self.cold.insert(entry)?;
        }
        Ok(())
    }
    pub fn tier_stats(&self) -> (usize, usize, usize) {
        (self.hot.len(), self.warm.len(), self.cold.len())
    }
}

impl Default for MemoryMultitier {
    fn default() -> Self {
        Self::new(50, 500).unwrap_or(Self::new(1, 1).expect("minimal"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    fn make_test_entry(
        id: &str,
        estate: MemoryEstate,
        confidence: f64,
        access_count: u64,
    ) -> TypedMemoryEntry {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        TypedMemoryEntry::new(id.into(), estate, format!("content for {}", id), confidence)
    }
    #[test]
    fn test_hot_store() {
        let mut s = HotStore::new(2);
        let e = make_test_entry("h1", MemoryEstate::Working, 0.9, 10);
        s.insert(e.clone());
        assert_eq!(s.len(), 1);
        assert!(s.get("h1").is_some());
    }
    #[test]
    fn test_warm_store() {
        let mut s = WarmStore::new();
        let e = make_test_entry("w1", MemoryEstate::Semantic, 0.7, 3);
        s.insert(e.clone());
        assert!(s.get("w1").is_some());
    }
    #[test]
    fn test_jit_scores() {
        let jit = JitForWeights::new();
        let entries = vec![
            make_test_entry("e1", MemoryEstate::Working, 0.9, 10),
            make_test_entry("e2", MemoryEstate::Semantic, 0.5, 1),
        ];
        let scores = jit.compute_scores(&entries);
        assert_eq!(scores.len(), 2);
        assert!(scores[0].hot_score >= 0.0);
    }
    #[test]
    fn test_jit_assign_tier() {
        let jit = JitForWeights::new();
        let score = PlacementScore {
            entry_id: "t1".into(),
            estate: MemoryEstate::Working,
            hot_score: 0.8,
            warm_score: 0.3,
            cold_score: 0.1,
            last_access: 0,
            access_count: 10,
        };
        assert_eq!(jit.assign_tier(&score), Tier::Hot);
    }
    #[test]
    fn test_multitier_insert_get() {
        let mut m = MemoryMultitier::new(10, 100).unwrap();
        let e = make_test_entry("mt1", MemoryEstate::Procedural, 0.85, 8);
        m.insert(e);
        assert!(m.get("mt1").is_some());
    }
    #[test]
    fn test_multitier_stats() {
        let mut m = MemoryMultitier::new(10, 100).unwrap();
        m.insert(make_test_entry("mts1", MemoryEstate::Working, 0.9, 10));
        let (hot, warm, cold) = m.tier_stats();
        assert!(hot + warm + cold >= 1);
    }
    #[test]
    fn test_jit_custom_weights() {
        let jit = JitForWeights::new().with_weights(0.9, 0.08, 0.02);
        assert_eq!(jit.hot_weight, 0.9);
    }
}
