use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

fn instant_now() -> Instant { Instant::now() }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryTier {
    Core,
    Working,
    Archival,
}

impl MemoryTier {
    pub fn weight(&self) -> f64 {
        match self {
            MemoryTier::Core => 1.5,
            MemoryTier::Working => 1.0,
            MemoryTier::Archival => 0.5,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            MemoryTier::Core => "core",
            MemoryTier::Working => "working",
            MemoryTier::Archival => "archival",
        }
    }
}

impl std::fmt::Display for MemoryTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemoryEntry {
    pub id: Uuid,
    pub tier: MemoryTier,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: HashMap<String, String>,
    #[serde(skip, default = "instant_now")]
    pub created_at: Instant,
    #[serde(skip, default = "instant_now")]
    pub accessed_at: Instant,
    pub access_count: u64,
    pub superseded: bool,
    pub superseded_by: Option<Uuid>,
    pub old_version_id: Option<Uuid>,
}

impl AgentMemoryEntry {
    pub fn new(tier: MemoryTier, content: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            tier,
            content: content.to_string(),
            embedding: None,
            metadata: HashMap::new(),
            created_at: Instant::now(),
            accessed_at: Instant::now(),
            access_count: 1,
            superseded: false,
            superseded_by: None,
            old_version_id: None,
        }
    }

    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding); self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string()); self
    }

    pub fn access(&mut self) {
        self.access_count += 1;
        self.accessed_at = Instant::now();
    }

    pub fn consolidation_score(&self, now: Instant) -> f64 {
        let elapsed_hours = now.duration_since(self.created_at).as_secs_f64() / 3600.0;
        let recency_bonus = 1.0 / (elapsed_hours + 1.0).ln_1p();
        (self.access_count as f64) / (elapsed_hours + 1.0) * 100.0 + recency_bonus
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub core_max: usize,
    pub working_max: usize,
    pub archival_max: usize,
    pub auto_archival_score_threshold: f64,
    pub consolidation_interval_secs: u64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self { core_max: 50, working_max: 500, archival_max: 5000, auto_archival_score_threshold: 0.5, consolidation_interval_secs: 300 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemory {
    pub config: MemoryConfig,
    pub core: Vec<AgentMemoryEntry>,
    pub working: Vec<AgentMemoryEntry>,
    pub archival: Vec<AgentMemoryEntry>,
    #[serde(skip, default = "instant_now")]
    pub last_consolidation: Instant,
}

impl AgentMemory {
    pub fn new(config: MemoryConfig) -> Self {
        Self { config, core: Vec::new(), working: Vec::new(), archival: Vec::new(), last_consolidation: Instant::now() }
    }

    pub fn insert(&mut self, content: &str) -> Uuid {
        let entry = AgentMemoryEntry::new(MemoryTier::Core, content);
        let id = entry.id;
        self.core.push(entry);
        self.apply_budget_pressure();
        id
    }

    pub fn insert_with_embedding(&mut self, content: &str, embedding: Vec<f32>) -> Uuid {
        let mut entry = AgentMemoryEntry::new(MemoryTier::Core, content);
        entry.embedding = Some(embedding);
        let id = entry.id;
        self.core.push(entry);
        self.apply_budget_pressure();
        id
    }

    pub fn self_edit(&mut self, entry_id: &Uuid, new_content: &str) -> Result<Uuid, String> {
        let found = self.find_entry(entry_id).ok_or("Entry not found")?;
        let tier = found.tier;

        let existing = match tier {
            MemoryTier::Core => self.core.iter_mut().find(|e| e.id == *entry_id),
            MemoryTier::Working => self.working.iter_mut().find(|e| e.id == *entry_id),
            MemoryTier::Archival => self.archival.iter_mut().find(|e| e.id == *entry_id),
        };

        if let Some(existing) = existing {
            existing.superseded = true;
            existing.superseded_by = None;
        }

        let mut new_entry = AgentMemoryEntry::new(tier, new_content);
        new_entry.old_version_id = Some(*entry_id);
        let new_id = new_entry.id;

        let target = match tier {
            MemoryTier::Core => &mut self.core,
            MemoryTier::Working => &mut self.working,
            MemoryTier::Archival => &mut self.archival,
        };
        target.push(new_entry);

        Ok(new_id)
    }

    pub fn find_entry(&self, entry_id: &Uuid) -> Option<&AgentMemoryEntry> {
        self.core.iter().chain(self.working.iter()).chain(self.archival.iter())
            .find(|e| e.id == *entry_id)
    }

    pub fn find_entry_mut(&mut self, entry_id: &Uuid) -> Option<&mut AgentMemoryEntry> {
        self.core.iter_mut().chain(self.working.iter_mut()).chain(self.archival.iter_mut())
            .find(|e| e.id == *entry_id)
    }

    pub fn search_tier(&self, query: &str, tier: MemoryTier) -> Vec<&AgentMemoryEntry> {
        let q = query.to_lowercase();
        let pool = match tier {
            MemoryTier::Core => &self.core,
            MemoryTier::Working => &self.working,
            MemoryTier::Archival => &self.archival,
        };
        pool.iter().filter(|e| !e.superseded && e.content.to_lowercase().contains(&q)).collect()
    }

    pub fn search_all(&self, query: &str) -> Vec<(&AgentMemoryEntry, f64)> {
        let q = query.to_lowercase();
        let mut results = Vec::new();
        for entry in self.core.iter().chain(self.working.iter()).chain(self.archival.iter()) {
            if !entry.superseded && entry.content.to_lowercase().contains(&q) {
                let weight = entry.tier.weight();
                results.push((entry, weight));
            }
        }
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    pub fn recent_core(&self, n: usize) -> Vec<&AgentMemoryEntry> {
        let mut sorted: Vec<_> = self.core.iter().filter(|e| !e.superseded).collect();
        sorted.sort_by(|a, b| b.created_at.partial_cmp(&a.created_at).unwrap_or(std::cmp::Ordering::Equal));
        sorted.truncate(n);
        sorted
    }

    pub fn consolidate(&mut self) -> usize {
        let now = Instant::now();
        let mut moved = 0;
        let threshold = self.config.auto_archival_score_threshold;

        self.core.retain(|entry| {
            let score = entry.consolidation_score(now);
            if score < threshold && entry.access_count > 1 {
                let mut e = entry.clone();
                e.tier = MemoryTier::Working;
                self.working.push(e);
                moved += 1;
                false
            } else { true }
        });

        self.working.retain(|entry| {
            let score = entry.consolidation_score(now);
            let age_secs = now.duration_since(entry.created_at).as_secs_f64();
            if score < threshold * 0.5 && age_secs > 86400.0 {
                let mut e = entry.clone();
                e.tier = MemoryTier::Archival;
                self.archival.push(e);
                moved += 1;
                false
            } else { true }
        });

        self.core.sort_by(|a, b| {
            b.consolidation_score(now).partial_cmp(&a.consolidation_score(now))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        while self.core.len() > self.config.core_max {
            if let Some(evicted) = self.core.pop() {
                let mut e = evicted.clone();
                e.tier = MemoryTier::Working;
                self.working.push(e);
                moved += 1;
            }
        }

        self.last_consolidation = now;
        moved
    }

    pub fn apply_budget_pressure(&mut self) {
        if self.core.len() <= self.config.core_max { return; }
        let now = Instant::now();
        self.core.sort_by(|a, b| {
            b.consolidation_score(now).partial_cmp(&a.consolidation_score(now))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        while self.core.len() > self.config.core_max {
            if let Some(evicted) = self.core.pop() {
                let mut e = evicted.clone();
                e.tier = MemoryTier::Working;
                self.working.push(e);
            }
        }
    }

    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            core_count: self.core.len(),
            working_count: self.working.len(),
            archival_count: self.archival.len(),
            total: self.core.len() + self.working.len() + self.archival.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub core_count: usize,
    pub working_count: usize,
    pub archival_count: usize,
    pub total: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_entry_found_in_core() {
        let mut mem = AgentMemory::new(MemoryConfig::default());
        let id = mem.insert("hello world");
        assert_eq!(mem.core.len(), 1);
        assert!(mem.find_entry(&id).is_some());
    }

    #[test]
    fn test_search_tier_returns_matching() {
        let mut mem = AgentMemory::new(MemoryConfig::default());
        mem.insert("apple banana");
        mem.insert("grape orange");
        let res = mem.search_tier("apple", MemoryTier::Core);
        assert_eq!(res.len(), 1);
        assert!(res[0].content.contains("apple"));
    }

    #[test]
    fn test_search_all_tier_weighted() {
        let mut mem = AgentMemory::new(MemoryConfig { core_max: 50, working_max: 500, archival_max: 5000, auto_archival_score_threshold: 0.5, consolidation_interval_secs: 300 });
        let core_id = mem.insert("shared content");
        let working_entry = AgentMemoryEntry::new(MemoryTier::Working, "shared content");
        let working_id = working_entry.id;
        mem.working.push(working_entry);

        let res = mem.search_all("shared");
        assert_eq!(res.len(), 2);
        let core_weight = res.iter().find(|(e, _)| e.id == core_id).map(|(_, w)| *w).unwrap();
        let working_weight = res.iter().find(|(e, _)| e.id == working_id).map(|(_, w)| *w).unwrap();
        assert!(core_weight > working_weight);
    }

    #[test]
    fn test_self_edit_supersedes_old() {
        let mut mem = AgentMemory::new(MemoryConfig::default());
        let id = mem.insert("version 1");
        let new_id = mem.self_edit(&id, "version 2").unwrap();
        let old = mem.find_entry(&id).unwrap();
        assert!(old.superseded);
        let new = mem.find_entry(&new_id).unwrap();
        assert!(!new.superseded);
        assert_eq!(new.old_version_id, Some(id));
    }

    #[test]
    fn test_budget_pressure_keeps_core_within_limit() {
        let config = MemoryConfig { core_max: 3, working_max: 500, archival_max: 5000, auto_archival_score_threshold: 0.5, consolidation_interval_secs: 300 };
        let mut mem = AgentMemory::new(config);
        mem.insert("a");
        mem.insert("b");
        mem.insert("c");
        mem.insert("d");
        mem.insert("e");
        assert_eq!(mem.core.len(), 3);
        assert_eq!(mem.working.len(), 2);
    }

    #[test]
    fn test_consolidation_moves_low_score_entries() {
        let config = MemoryConfig { core_max: 50, working_max: 500, archival_max: 5000, auto_archival_score_threshold: 999999.0, consolidation_interval_secs: 300 };
        let mut mem = AgentMemory::new(config);
        let id = mem.insert("important");
        mem.find_entry_mut(&id).unwrap().access();
        let moved = mem.consolidate();
        assert!(moved > 0);
    }

    #[test]
    fn test_stats_counts() {
        let mut mem = AgentMemory::new(MemoryConfig::default());
        mem.insert("a");
        mem.insert("b");
        let stats = mem.stats();
        assert_eq!(stats.core_count, 2);
        assert_eq!(stats.total, 2);
    }
}
