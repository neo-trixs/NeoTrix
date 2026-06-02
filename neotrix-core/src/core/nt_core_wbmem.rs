use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::Instant;
use chrono::{DateTime, Utc};

use crate::core::nt_core_bank::ReasoningBank;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u32,
    pub tags: Vec<String>,
    pub confidence: f64,
    pub edited: bool,
    pub original_content: Option<String>,
    pub pinned: bool,
}

impl MemoryEntry {
    pub fn new(id: &str, content: &str, source: &str) -> Self {
        let now = Utc::now();
        Self {
            id: id.to_string(),
            content: content.to_string(),
            source: source.to_string(),
            created_at: now,
            last_accessed: now,
            access_count: 0,
            tags: Vec::new(),
            confidence: 1.0,
            edited: false,
            original_content: None,
            pinned: false,
        }
    }

    pub fn touch(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCheckpoint {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub description: String,
    pub entries: Vec<MemoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_entries: usize,
    pub avg_confidence: f64,
    pub top_sources: Vec<(String, u32)>,
    pub top_tags: Vec<(String, u32)>,
    pub oldest_entry: DateTime<Utc>,
    pub newest_entry: DateTime<Utc>,
    pub checkpoint_count: usize,
    pub dream_enabled: bool,
    pub edited_entries: usize,
    pub pinned_entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamReport {
    pub entries_before: usize,
    pub entries_after: usize,
    pub merged: usize,
    pub pruned: usize,
    pub duration_ms: u64,
}

pub struct WhiteBoxMemory {
    pub entries: Vec<MemoryEntry>,
    pub checkpoints: Vec<MemoryCheckpoint>,
    pub auto_consolidate: bool,
    pub last_dream_time: Option<DateTime<Utc>>,
}

impl Default for WhiteBoxMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl WhiteBoxMemory {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            checkpoints: Vec::new(),
            auto_consolidate: false,
            last_dream_time: None,
        }
    }

    pub fn view(&self, id: &str) -> Result<&MemoryEntry, String> {
        self.entries.iter().find(|e| e.id == id).ok_or_else(|| format!("Memory entry not found: {}", id))
    }

    pub fn list(&self, filter: Option<&str>) -> Vec<&MemoryEntry> {
        match filter {
            Some(f) if f == "pinned" => self.entries.iter().filter(|e| e.pinned).collect(),
            Some(f) if f == "edited" => self.entries.iter().filter(|e| e.edited).collect(),
            Some(source) => self.entries.iter().filter(|e| e.source == source).collect(),
            None => self.entries.iter().collect(),
        }
    }

    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        let q = query.to_lowercase();
        self.entries.iter().filter(|e| {
            e.content.to_lowercase().contains(&q)
                || e.tags.iter().any(|t| t.to_lowercase().contains(&q))
                || e.source.to_lowercase().contains(&q)
        }).collect()
    }

    pub fn stats(&self) -> MemoryStats {
        let total = self.entries.len();
        let avg_conf = if total > 0 {
            self.entries.iter().map(|e| e.confidence).sum::<f64>() / total as f64
        } else {
            0.0
        };

        let mut source_counts: HashMap<String, u32> = HashMap::new();
        let mut tag_counts: HashMap<String, u32> = HashMap::new();
        for e in &self.entries {
            *source_counts.entry(e.source.clone()).or_default() += 1;
            for t in &e.tags {
                *tag_counts.entry(t.clone()).or_default() += 1;
            }
        }

        let mut top_sources: Vec<(String, u32)> = source_counts.into_iter().collect();
        top_sources.sort_by(|a, b| b.1.cmp(&a.1));
        top_sources.truncate(10);

        let mut top_tags: Vec<(String, u32)> = tag_counts.into_iter().collect();
        top_tags.sort_by(|a, b| b.1.cmp(&a.1));
        top_tags.truncate(10);

        let oldest = self.entries.iter().map(|e| e.created_at).min().unwrap_or_else(Utc::now);
        let newest = self.entries.iter().map(|e| e.created_at).max().unwrap_or_else(Utc::now);

        MemoryStats {
            total_entries: total,
            avg_confidence: avg_conf,
            top_sources,
            top_tags,
            oldest_entry: oldest,
            newest_entry: newest,
            checkpoint_count: self.checkpoints.len(),
            dream_enabled: self.auto_consolidate,
            edited_entries: self.entries.iter().filter(|e| e.edited).count(),
            pinned_entries: self.entries.iter().filter(|e| e.pinned).count(),
        }
    }

    pub fn edit_content(&mut self, id: &str, new_content: &str) -> Result<(), String> {
        let entry = self.entries.iter_mut().find(|e| e.id == id).ok_or_else(|| format!("Entry not found: {}", id))?;
        if !entry.edited {
            entry.original_content = Some(entry.content.clone());
        }
        entry.content = new_content.to_string();
        entry.edited = true;
        entry.touch();
        Ok(())
    }

    pub fn edit_tags(&mut self, id: &str, tags: Vec<String>) -> Result<(), String> {
        let entry = self.entries.iter_mut().find(|e| e.id == id).ok_or_else(|| format!("Entry not found: {}", id))?;
        entry.tags = tags;
        entry.touch();
        Ok(())
    }

    pub fn delete(&mut self, id: &str) -> Result<(), String> {
        let idx = self.entries.iter().position(|e| e.id == id).ok_or_else(|| format!("Entry not found: {}", id))?;
        self.entries.remove(idx);
        Ok(())
    }

    pub fn pin(&mut self, id: &str) -> Result<(), String> {
        let entry = self.entries.iter_mut().find(|e| e.id == id).ok_or_else(|| format!("Entry not found: {}", id))?;
        entry.pinned = !entry.pinned;
        entry.touch();
        Ok(())
    }

    pub fn create_checkpoint(&mut self, description: &str) {
        let checkpoint = MemoryCheckpoint {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            description: description.to_string(),
            entries: self.entries.clone(),
        };
        self.checkpoints.push(checkpoint);
    }

    pub fn rollback(&mut self, checkpoint_id: &str) -> Result<(), String> {
        let checkpoint = self.checkpoints.iter()
            .find(|c| c.id == checkpoint_id)
            .ok_or_else(|| format!("Checkpoint not found: {}", checkpoint_id))?;
        self.entries = checkpoint.entries.clone();
        Ok(())
    }

    pub fn list_checkpoints(&self) -> &[MemoryCheckpoint] {
        &self.checkpoints
    }

    pub fn dream_cycle(&mut self) -> DreamReport {
        let before = self.entries.len();
        let start = Instant::now();

        let mut merged = 0usize;
        let mut i = 0;
        while i < self.entries.len() {
            let mut j = i + 1;
            while j < self.entries.len() {
                let a = self.entries[i].clone();
                let b = self.entries[j].clone();
                let same_source = a.source == b.source;
                let tag_overlap = a.tags.iter().filter(|t| b.tags.contains(t)).count();
                let should_merge = same_source && tag_overlap > 0 && !a.pinned && !b.pinned;
                let similar = if should_merge {
                    let words_a: Vec<&str> = a.content.split_whitespace().collect();
                    let words_b: Vec<&str> = b.content.split_whitespace().collect();
                    let overlap = words_a.iter().filter(|w| words_b.contains(w)).count();
                    let sim = if words_a.len().max(words_b.len()) > 0 {
                        overlap as f64 / words_a.len().max(words_b.len()) as f64
                    } else {
                        0.0
                    };
                    sim > 0.5
                } else {
                    false
                };
                if similar {
                    let mut combined_tags: Vec<String> = Vec::new();
                    for t in &a.tags {
                        if !combined_tags.contains(t) { combined_tags.push(t.clone()); }
                    }
                    for t in &b.tags {
                        if !combined_tags.contains(t) { combined_tags.push(t.clone()); }
                    }
                    let created = if a.created_at < b.created_at { a.created_at } else { b.created_at };
                    let last_acc = if a.last_accessed >= b.last_accessed { a.last_accessed } else { b.last_accessed };
                    self.entries[i].content = format!("{}\n\n{}", a.content, b.content);
                    self.entries[i].tags = combined_tags;
                    self.entries[i].confidence = (a.confidence + b.confidence) / 2.0;
                    self.entries[i].created_at = created;
                    self.entries[i].last_accessed = last_acc;
                    self.entries.remove(j);
                    merged += 1;
                } else {
                    j += 1;
                }
            }
            i += 1;
        }

        let prune_threshold: f64 = 0.15;
        let prune_age = chrono::Duration::days(30);
        let now = Utc::now();
        let pruned_before = self.entries.len();
        self.entries.retain(|e| {
            if e.pinned { return true; }
            if e.confidence < prune_threshold && now.signed_duration_since(e.last_accessed) > prune_age {
                return false;
            }
            true
        });
        let pruned = pruned_before - self.entries.len();

        self.last_dream_time = Some(Utc::now());

        DreamReport {
            entries_before: before,
            entries_after: self.entries.len(),
            merged,
            pruned,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    pub fn set_auto_consolidate(&mut self, enabled: bool) {
        self.auto_consolidate = enabled;
    }

    pub fn sync_from_brain(&mut self, bank: &ReasoningBank) {
        let now = Utc::now();
        for mem in bank.memories() {
            if !self.entries.iter().any(|e| e.id == mem.id) {
                self.entries.push(MemoryEntry {
                    id: mem.id.clone(),
                    content: mem.task_description.clone(),
                    source: format!("{:?}", mem.task_type),
                    created_at: DateTime::from_timestamp(mem.timestamp, 0).unwrap_or(now),
                    last_accessed: DateTime::from_timestamp(mem.lifecycle.last_accessed, 0).unwrap_or(now),
                    access_count: mem.lifecycle.access_count as u32,
                    tags: vec![format!("{:?}", mem.tier)],
                    confidence: mem.lifecycle.confidence,
                    edited: false,
                    original_content: None,
                    pinned: false,
                });
            }
        }
    }

    pub fn sync_to_brain(&self, bank: &mut ReasoningBank) {
        let mems: Vec<_> = bank.memories().iter().cloned().collect();
        for entry in &self.entries {
            let content = &entry.content;
            if let Some(mem) = mems.iter().find(|m| m.id == entry.id) {
                if entry.edited {
                    let edited_content = content.clone();
                    let mut new_mem = mem.clone();
                    new_mem.task_description = edited_content;
                    bank.store(new_mem);
                }
            }
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let dir = std::path::Path::new(&home).join(".neotrix");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("whitebox_memory.json");
        let data = serde_json::json!({
            "entries": self.entries,
            "checkpoints": self.checkpoints,
            "auto_consolidate": self.auto_consolidate,
            "last_dream_time": self.last_dream_time,
        });
        let json = serde_json::to_string_pretty(&data).map_err(|e| format!("Serialize: {}", e))?;
        std::fs::write(&path, &json).map_err(|e| format!("Write: {}", e))?;
        Ok(())
    }

    pub fn load() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = std::path::Path::new(&home).join(".neotrix").join("whitebox_memory.json");
        if !path.exists() {
            return Self::new();
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::new(),
        };
        let data: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return Self::new(),
        };
        let entries: Vec<MemoryEntry> = serde_json::from_value(data["entries"].clone()).unwrap_or_default();
        let checkpoints: Vec<MemoryCheckpoint> = serde_json::from_value(data["checkpoints"].clone()).unwrap_or_default();
        let auto_consolidate = data["auto_consolidate"].as_bool().unwrap_or(false);
        let last_dream_time: Option<DateTime<Utc>> = serde_json::from_value(data["last_dream_time"].clone()).unwrap_or(None);
        Self { entries, checkpoints, auto_consolidate, last_dream_time }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_whitebox_empty() {
        let wbm = WhiteBoxMemory::new();
        assert!(wbm.entries.is_empty());
        assert!(wbm.checkpoints.is_empty());
        assert!(!wbm.auto_consolidate);
    }

    #[test]
    fn test_memory_entry_new() {
        let entry = MemoryEntry::new("id-1", "test content", "conversation");
        assert_eq!(entry.id, "id-1");
        assert_eq!(entry.content, "test content");
        assert_eq!(entry.source, "conversation");
        assert!(!entry.edited);
        assert!(!entry.pinned);
    }

    #[test]
    fn test_view_found() {
        let mut wbm = WhiteBoxMemory::new();
        wbm.entries.push(MemoryEntry::new("id-1", "hello", "test"));
        let entry = wbm.view("id-1").unwrap();
        assert_eq!(entry.content, "hello");
    }

    #[test]
    fn test_view_not_found() {
        let wbm = WhiteBoxMemory::new();
        assert!(wbm.view("nonexistent").is_err());
    }

    #[test]
    fn test_list_no_filter() {
        let mut wbm = WhiteBoxMemory::new();
        wbm.entries.push(MemoryEntry::new("a", "a", "src1"));
        wbm.entries.push(MemoryEntry::new("b", "b", "src2"));
        assert_eq!(wbm.list(None).len(), 2);
    }

    #[test]
    fn test_list_source_filter() {
        let mut wbm = WhiteBoxMemory::new();
        wbm.entries.push(MemoryEntry::new("a", "a", "chat"));
        wbm.entries.push(MemoryEntry::new("b", "b", "task"));
        let listed = wbm.list(Some("chat"));
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, "a");
    }

    #[test]
    fn test_list_pinned_filter() {
        let mut wbm = WhiteBoxMemory::new();
        let mut e = MemoryEntry::new("a", "a", "test");
        e.pinned = true;
        wbm.entries.push(e);
        wbm.entries.push(MemoryEntry::new("b", "b", "test"));
        let listed = wbm.list(Some("pinned"));
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, "a");
    }

    #[test]
    fn test_search_by_content() {
        let mut wbm = WhiteBoxMemory::new();
        wbm.entries.push(MemoryEntry::new("a", "hello world", "test"));
        wbm.entries.push(MemoryEntry::new("b", "foo bar", "test"));
        let results = wbm.search("hello");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "a");
    }

    #[test]
    fn test_search_by_tag() {
        let mut wbm = WhiteBoxMemory::new();
        let mut e = MemoryEntry::new("a", "content", "test");
        e.tags = vec!["important".to_string()];
        wbm.entries.push(e);
        wbm.entries.push(MemoryEntry::new("b", "other", "test"));
        let results = wbm.search("important");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_edit_content_stores_original() {
        let mut wbm = WhiteBoxMemory::new();
        wbm.entries.push(MemoryEntry::new("id-1", "original", "test"));
        wbm.edit_content("id-1", "edited").unwrap();
        let entry = wbm.view("id-1").unwrap();
        assert_eq!(entry.content, "edited");
        assert!(entry.edited);
        assert_eq!(entry.original_content.as_deref(), Some("original"));
    }

    #[test]
    fn test_edit_content_idempotent_original() {
        let mut wbm = WhiteBoxMemory::new();
        wbm.entries.push(MemoryEntry::new("id-1", "original", "test"));
        wbm.edit_content("id-1", "v1").unwrap();
        wbm.edit_content("id-1", "v2").unwrap();
        let entry = wbm.view("id-1").unwrap();
        assert_eq!(entry.original_content.as_deref(), Some("original"));
    }

    #[test]
    fn test_edit_content_not_found() {
        let mut wbm = WhiteBoxMemory::new();
        assert!(wbm.edit_content("nonexistent", "x").is_err());
    }

    #[test]
    fn test_edit_tags() {
        let mut wbm = WhiteBoxMemory::new();
        wbm.entries.push(MemoryEntry::new("id-1", "c", "test"));
        wbm.edit_tags("id-1", vec!["a".into(), "b".into()]).unwrap();
        let entry = wbm.view("id-1").unwrap();
        assert_eq!(entry.tags, vec!["a", "b"]);
    }

    #[test]
    fn test_delete() {
        let mut wbm = WhiteBoxMemory::new();
        wbm.entries.push(MemoryEntry::new("id-1", "c", "test"));
        wbm.entries.push(MemoryEntry::new("id-2", "d", "test"));
        assert!(wbm.delete("id-1").is_ok());
        assert_eq!(wbm.entries.len(), 1);
        assert_eq!(wbm.entries[0].id, "id-2");
    }

    #[test]
    fn test_delete_not_found() {
        let mut wbm = WhiteBoxMemory::new();
        assert!(wbm.delete("x").is_err());
    }

    #[test]
    fn test_pin_toggle() {
        let mut wbm = WhiteBoxMemory::new();
        wbm.entries.push(MemoryEntry::new("id-1", "c", "test"));
        wbm.pin("id-1").unwrap();
        assert!(wbm.view("id-1").unwrap().pinned);
        wbm.pin("id-1").unwrap();
        assert!(!wbm.view("id-1").unwrap().pinned);
    }

    #[test]
    fn test_checkpoint_and_rollback() {
        let mut wbm = WhiteBoxMemory::new();
        wbm.entries.push(MemoryEntry::new("id-1", "original", "test"));
        wbm.create_checkpoint("before edit");
        wbm.edit_content("id-1", "edited").unwrap();
        assert_eq!(wbm.entries.len(), 1);
        assert_eq!(wbm.view("id-1").unwrap().content, "edited");

        let cp_id = wbm.checkpoints[0].id.clone();
        wbm.rollback(&cp_id).unwrap();
        assert_eq!(wbm.view("id-1").unwrap().content, "original");
    }

    #[test]
    fn test_rollback_invalid() {
        let mut wbm = WhiteBoxMemory::new();
        assert!(wbm.rollback("bad-id").is_err());
    }

    #[test]
    fn test_list_checkpoints() {
        let mut wbm = WhiteBoxMemory::new();
        wbm.create_checkpoint("cp1");
        wbm.create_checkpoint("cp2");
        assert_eq!(wbm.list_checkpoints().len(), 2);
    }

    #[test]
    fn test_dream_cycle_empty() {
        let mut wbm = WhiteBoxMemory::new();
        let report = wbm.dream_cycle();
        assert_eq!(report.entries_before, 0);
        assert_eq!(report.merged, 0);
        assert_eq!(report.pruned, 0);
    }

    #[test]
    fn test_dream_cycle_merges_similar() {
        let mut wbm = WhiteBoxMemory::new();
        let mut e1 = MemoryEntry::new("a", "hello world foo", "chat");
        e1.tags = vec!["greeting".into()];
        let mut e2 = MemoryEntry::new("b", "hello world bar", "chat");
        e2.tags = vec!["greeting".into()];
        wbm.entries.push(e1);
        wbm.entries.push(e2);
        let report = wbm.dream_cycle();
        assert_eq!(report.merged, 1);
        assert_eq!(wbm.entries.len(), 1);
    }

    #[test]
    fn test_dream_cycle_prunes_low_confidence() {
        let mut wbm = WhiteBoxMemory::new();
        let mut old_entry = MemoryEntry::new("old", "stale data", "noise");
        old_entry.confidence = 0.1;
        old_entry.last_accessed = Utc::now() - chrono::Duration::days(60);
        wbm.entries.push(old_entry);
        wbm.entries.push(MemoryEntry::new("fresh", "good data", "source"));
        let report = wbm.dream_cycle();
        assert_eq!(report.pruned, 1);
        assert_eq!(wbm.entries.len(), 1);
    }

    #[test]
    fn test_dream_cycle_keeps_pinned() {
        let mut wbm = WhiteBoxMemory::new();
        let mut pinned = MemoryEntry::new("pinned", "important", "chat");
        pinned.pinned = true;
        pinned.confidence = 0.1;
        pinned.last_accessed = Utc::now() - chrono::Duration::days(60);
        wbm.entries.push(pinned);
        let report = wbm.dream_cycle();
        assert_eq!(report.pruned, 0);
        assert_eq!(wbm.entries.len(), 1);
    }

    #[test]
    fn test_set_auto_consolidate() {
        let mut wbm = WhiteBoxMemory::new();
        assert!(!wbm.auto_consolidate);
        wbm.set_auto_consolidate(true);
        assert!(wbm.auto_consolidate);
    }

    #[test]
    fn test_stats() {
        let mut wbm = WhiteBoxMemory::new();
        let mut e1 = MemoryEntry::new("a", "content a", "chat");
        e1.tags = vec!["tag1".into()];
        e1.confidence = 0.9;
        let mut e2 = MemoryEntry::new("b", "content b", "task");
        e2.tags = vec!["tag2".into()];
        e2.confidence = 0.7;
        let mut e3 = MemoryEntry::new("c", "content c", "chat");
        e3.edited = true;
        e3.pinned = true;
        e3.confidence = 1.0;
        wbm.entries.push(e1);
        wbm.entries.push(e2);
        wbm.entries.push(e3);

        let stats = wbm.stats();
        assert_eq!(stats.total_entries, 3);
        assert!((stats.avg_confidence - 0.866666).abs() < 0.001);
        assert_eq!(stats.edited_entries, 1);
        assert_eq!(stats.pinned_entries, 1);
        assert!(stats.top_sources.iter().any(|(s, _)| s == "chat"));
        assert!(stats.top_tags.iter().any(|(t, _)| t == "tag1"));
    }

    #[test]
    fn test_stats_empty() {
        let wbm = WhiteBoxMemory::new();
        let stats = wbm.stats();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.avg_confidence, 0.0);
        assert_eq!(stats.edited_entries, 0);
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut wbm = WhiteBoxMemory::new();
        wbm.entries.push(MemoryEntry::new("a", "Hello World", "test"));
        let results = wbm.search("hello");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_touch_increments_access_count() {
        let mut wbm = WhiteBoxMemory::new();
        wbm.entries.push(MemoryEntry::new("a", "content", "test"));
        let before = wbm.view("a").unwrap().access_count;
        wbm.edit_content("a", "new").unwrap();
        let after = wbm.view("a").unwrap().access_count;
        assert!(after > before);
    }

    #[test]
    fn test_save_load_roundtrip() {
        let mut wbm = WhiteBoxMemory::new();
        wbm.entries.push(MemoryEntry::new("a", "content", "test"));
        wbm.auto_consolidate = true;
        wbm.save().unwrap();

        let loaded = WhiteBoxMemory::load();
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].content, "content");
        assert!(loaded.auto_consolidate);
        assert!(loaded.last_dream_time.is_none());

        let _ = std::fs::remove_file(
            std::path::Path::new(&std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
                .join(".neotrix").join("whitebox_memory.json")
        );
    }

    #[test]
    fn test_load_nonexistent() {
        let loaded = WhiteBoxMemory::load();
        assert!(loaded.entries.is_empty());
    }

    #[test]
    fn test_sync_from_brain_adds_entries() {
        let mut bank = ReasoningBank::new(100);
        let mem = crate::core::nt_core_bank::ReasoningMemory::new("test memory", crate::core::knowledge::TaskType::General, &[], 0.8);
        bank.store(mem);

        let mut wbm = WhiteBoxMemory::new();
        wbm.sync_from_brain(&bank);
        assert_eq!(wbm.entries.len(), 1);
        assert_eq!(wbm.entries[0].content, "test memory");
    }

    #[test]
    fn test_sync_from_brain_idempotent() {
        let mut bank = ReasoningBank::new(100);
        let mem = crate::core::nt_core_bank::ReasoningMemory::new("test", crate::core::knowledge::TaskType::General, &[], 0.8);
        bank.store(mem);
        let mut wbm = WhiteBoxMemory::new();
        wbm.sync_from_brain(&bank);
        wbm.sync_from_brain(&bank);
        assert_eq!(wbm.entries.len(), 1);
    }

    #[test]
    fn test_list_edited_filter() {
        let mut wbm = WhiteBoxMemory::new();
        let mut e1 = MemoryEntry::new("a", "original", "test");
        e1.edited = true;
        wbm.entries.push(e1);
        wbm.entries.push(MemoryEntry::new("b", "never edited", "test"));
        let listed = wbm.list(Some("edited"));
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, "a");
    }

    #[test]
    fn test_checkpoint_preserves_state() {
        let mut wbm = WhiteBoxMemory::new();
        wbm.entries.push(MemoryEntry::new("a", "v1", "test"));
        wbm.create_checkpoint("at v1");
        wbm.edit_content("a", "v2").unwrap();
        wbm.entries.push(MemoryEntry::new("b", "new", "test"));
        assert_eq!(wbm.entries.len(), 2);

        let cp_id = wbm.checkpoints[0].id.clone();
        wbm.rollback(&cp_id).unwrap();
        assert_eq!(wbm.entries.len(), 1);
        assert_eq!(wbm.entries[0].content, "v1");
    }
}
