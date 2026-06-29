use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Layered Agent Memory — VSA-based cross-session knowledge store.
/// Inspired by Claude Code's 4-layer memory architecture:
///   Layer 1: Explicit (AGENTS.md) — loaded each session
///   Layer 2: Auto-discovered (MEMORY.md) — patterns/lessons
///   Layer 3: Semantic retrieval (MemoryTool)
///   Layer 4: Full transcript (SessionTranscript)
///
/// This file implements Layers 1-3 as a unified VSA-backed store.

pub const MEMORY_VSA_DIM: usize = 4096;

/// Source of a memory entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemorySource {
    /// Explicitly written by agent or user
    Explicit,
    /// Automatically detected through pattern recognition
    AutoDiscovered,
    /// Extracted from a handler dispatch trace
    HandlerTrace,
    /// Inferred from E8 reasoning or GWT attention
    Inferred,
}

/// Confidence level assigned to a memory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    Low,
    Medium,
    High,
    Verified,
}

/// A single memory entry with VSA embedding support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: u64,
    pub content: String,
    pub tags: Vec<String>,
    pub source: MemorySource,
    pub confidence: ConfidenceLevel,
    pub access_count: u64,
    pub created_at: u64,
    pub last_accessed: u64,
    pub score: f64,
    /// VSA vector stored as 512-byte array (4096 bits = 512 u8s)
    pub vsa_embedding: Option<Vec<u8>>,
}

impl MemoryEntry {
    pub fn new(id: u64, content: String, tags: Vec<String>, source: MemorySource) -> Self {
        let now = unix_now();
        Self {
            id,
            content,
            tags,
            source,
            confidence: ConfidenceLevel::Medium,
            access_count: 0,
            created_at: now,
            last_accessed: now,
            score: 0.0,
            vsa_embedding: None,
        }
    }

    pub fn access(&mut self) {
        self.access_count += 1;
        self.last_accessed = unix_now();
    }
}

/// Summary or pattern extracted from multiple entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPattern {
    pub id: u64,
    pub summary: String,
    pub evidence_ids: Vec<u64>,
    pub confidence: ConfidenceLevel,
    pub created_at: u64,
    pub last_updated: u64,
    pub hit_count: u64,
}

impl MemoryPattern {
    pub fn new(id: u64, summary: String, evidence_ids: Vec<u64>) -> Self {
        let now = unix_now();
        Self {
            id,
            summary,
            evidence_ids,
            confidence: ConfidenceLevel::Low,
            created_at: now,
            last_updated: now,
            hit_count: 0,
        }
    }
}

/// A distilled lesson (the most mature form of memory)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLesson {
    pub id: u64,
    pub principle: String,
    pub context: String,
    pub pattern_ids: Vec<u64>,
    pub confidence: ConfidenceLevel,
    pub created_at: u64,
    pub applied_count: u64,
}

impl MemoryLesson {
    pub fn new(id: u64, principle: String, context: String, pattern_ids: Vec<u64>) -> Self {
        Self {
            id,
            principle,
            context,
            pattern_ids,
            confidence: ConfidenceLevel::Low,
            created_at: unix_now(),
            applied_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemory {
    /// Session ID for scoping
    session_id: String,
    /// Layer 1: Explicit entries (AGENTS.md equivalent)
    pub explicit: Vec<MemoryEntry>,
    /// Layer 2: Auto-discovered entries
    pub auto_discovered: Vec<MemoryEntry>,
    /// Layer 2: Patterns distilled from entries
    pub patterns: Vec<MemoryPattern>,
    /// Layer 3: Lessons (semantic knowledge)
    pub lessons: Vec<MemoryLesson>,
    /// Tag index for fast lookup
    tag_index: HashMap<String, Vec<u64>>,
    /// Next ID counter
    next_id: u64,
    /// Max total entries before auto-prune
    max_entries: usize,
    /// File path for persistence
    path: Option<PathBuf>,
}

impl AgentMemory {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            explicit: Vec::new(),
            auto_discovered: Vec::new(),
            patterns: Vec::new(),
            lessons: Vec::new(),
            tag_index: HashMap::new(),
            next_id: 1,
            max_entries: 1_000,
            path: None,
        }
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if path.exists() {
            if let Some(loaded) = Self::load(&path) {
                self.explicit = loaded.explicit;
                self.auto_discovered = loaded.auto_discovered;
                self.patterns = loaded.patterns;
                self.lessons = loaded.lessons;
                self.tag_index = loaded.tag_index;
                self.next_id = loaded.next_id;
                self.max_entries = loaded.max_entries;
            }
        }
        self.path = Some(path);
        self
    }

    // ─── Entry management ──────────────────────────────────────────────

    pub fn add_explicit(&mut self, content: String, tags: Vec<String>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let entry = MemoryEntry::new(id, content, tags.clone(), MemorySource::Explicit);
        self.index_tags(id, &tags);
        self.explicit.push(entry);
        self.maybe_prune();
        self.save();
        id
    }

    pub fn add_discovered(&mut self, content: String, tags: Vec<String>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let entry = MemoryEntry::new(id, content, tags.clone(), MemorySource::AutoDiscovered);
        self.index_tags(id, &tags);
        self.auto_discovered.push(entry);
        self.maybe_prune();
        id
    }

    pub fn add_pattern(&mut self, summary: String, evidence_ids: Vec<u64>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let pattern = MemoryPattern::new(id, summary, evidence_ids);
        self.patterns.push(pattern);
        id
    }

    pub fn add_lesson(&mut self, principle: String, context: String, pattern_ids: Vec<u64>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let lesson = MemoryLesson::new(id, principle, context, pattern_ids);
        self.lessons.push(lesson);
        self.save();
        id
    }

    // ─── Query ──────────────────────────────────────────────────────────

    pub fn query_by_tag(&self, tag: &str) -> Vec<&MemoryEntry> {
        let mut results = Vec::new();
        if let Some(ids) = self.tag_index.get(tag) {
            for id in ids {
                if let Some(e) = self.find_entry(*id) {
                    results.push(e);
                }
            }
        }
        results
    }

    pub fn query_by_source(&self, source: MemorySource) -> Vec<&MemoryEntry> {
        let mut results: Vec<&MemoryEntry> = self
            .explicit
            .iter()
            .chain(self.auto_discovered.iter())
            .filter(|e| e.source == source)
            .collect();
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    pub fn high_confidence(&self) -> Vec<&MemoryEntry> {
        let mut results: Vec<&MemoryEntry> = self
            .explicit
            .iter()
            .chain(self.auto_discovered.iter())
            .filter(|e| {
                matches!(
                    e.confidence,
                    ConfidenceLevel::High | ConfidenceLevel::Verified
                )
            })
            .collect();
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        let q = query.to_lowercase();
        let mut results: Vec<&MemoryEntry> = self
            .explicit
            .iter()
            .chain(self.auto_discovered.iter())
            .filter(|e| {
                e.content.to_lowercase().contains(&q)
                    || e.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .collect();
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(20);
        results
    }

    pub fn all_entries(&self) -> Vec<&MemoryEntry> {
        let mut entries: Vec<&MemoryEntry> = self
            .explicit
            .iter()
            .chain(self.auto_discovered.iter())
            .collect();
        entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        entries
    }

    /// Render as markdown (MEMORY.md format)
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!("# Agent Memory — {}\n\n", self.session_id));

        if !self.lessons.is_empty() {
            md.push_str("## Lessons\n\n");
            for lesson in &self.lessons {
                let conf = format!("{:?}", lesson.confidence);
                md.push_str(&format!(
                    "- [{}] **{}** — {} (applied {}x)\n",
                    conf, lesson.principle, lesson.context, lesson.applied_count
                ));
            }
            md.push('\n');
        }

        md.push_str("## Key Observations\n\n");
        for entry in self.high_confidence().iter().take(20) {
            let tags = entry.tags.join(", ");
            md.push_str(&format!(
                "- {} [{}] - {}\n",
                entry.content, tags, entry.created_at
            ));
        }
        md.push('\n');

        if !self.patterns.is_empty() {
            md.push_str("## Patterns\n\n");
            for pattern in self.patterns.iter().rev().take(10) {
                md.push_str(&format!(
                    "- {} ({} evidence, hit {}x)\n",
                    pattern.summary,
                    pattern.evidence_ids.len(),
                    pattern.hit_count
                ));
            }
        }

        let total = self.explicit.len() + self.auto_discovered.len();
        md.push_str(&format!(
            "\n---\n*{} entries, {} patterns, {} lessons*\n",
            total,
            self.patterns.len(),
            self.lessons.len()
        ));
        md
    }

    // ─── Internal ──────────────────────────────────────────────────────

    fn find_entry(&self, id: u64) -> Option<&MemoryEntry> {
        self.explicit
            .iter()
            .chain(self.auto_discovered.iter())
            .find(|e| e.id == id)
    }

    fn index_tags(&mut self, id: u64, tags: &[String]) {
        for tag in tags {
            self.tag_index.entry(tag.clone()).or_default().push(id);
        }
    }

    fn maybe_prune(&mut self) {
        let total = self.explicit.len() + self.auto_discovered.len();
        if total <= self.max_entries {
            return;
        }
        // Remove lowest-scored auto-discovered entries
        self.auto_discovered.sort_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let excess = total - self.max_entries;
        let removed: Vec<u64> = self.auto_discovered.drain(..excess).map(|e| e.id).collect();
        for id in removed {
            self.tag_index.retain(|_, ids| {
                ids.retain(|i| *i != id);
                !ids.is_empty()
            });
        }
    }

    fn save(&self) {
        let Some(ref path) = self.path else { return };
        if let Ok(json) = serde_json::to_string(self) {
            let _ = fs::write(path, &json);
        }
    }

    fn load(path: &PathBuf) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn summary(&self) -> String {
        let total = self.explicit.len() + self.auto_discovered.len();
        let high_conf = self.high_confidence().len();
        format!(
            "AgentMemory: {} entries ({} high-confidence), {} patterns, {} lessons, {} tags",
            total,
            high_conf,
            self.patterns.len(),
            self.lessons.len(),
            self.tag_index.len()
        )
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

impl Default for AgentMemory {
    fn default() -> Self {
        Self::new("default".into())
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_query() {
        let mut mem = AgentMemory::new("test".into());
        let id = mem.add_explicit(
            "the sky is blue".into(),
            vec!["weather".into(), "fact".into()],
        );
        assert_eq!(id, 1);

        let results = mem.query_by_tag("weather");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "the sky is blue");
    }

    #[test]
    fn test_search() {
        let mut mem = AgentMemory::new("test".into());
        mem.add_explicit("rust is a systems language".into(), vec!["rust".into()]);
        mem.add_explicit("python is dynamically typed".into(), vec!["python".into()]);
        mem.add_explicit("neotrix uses VSA vectors".into(), vec!["vsa".into()]);

        let results = mem.search("rust");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_markdown_rendering() {
        let mut mem = AgentMemory::new("test".into());
        mem.add_explicit("observation a".into(), vec!["tag1".into()]);
        mem.add_explicit("observation b".into(), vec!["tag2".into()]);
        mem.add_lesson("be concise".into(), "in agent responses".into(), vec![]);

        let md = mem.to_markdown();
        assert!(md.contains("observation a"));
        assert!(md.contains("be concise"));
    }

    #[test]
    fn test_pattern_lifecycle() {
        let mut mem = AgentMemory::new("test".into());
        let id1 = mem.add_explicit("pattern part 1".into(), vec![]);
        let id2 = mem.add_explicit("pattern part 2".into(), vec![]);
        let pid = mem.add_pattern("recurring pattern".into(), vec![id1, id2]);
        assert_eq!(pid, 3);
        assert_eq!(mem.patterns.len(), 1);
    }

    #[test]
    fn test_prune_excess() {
        let mut mem = AgentMemory::new("prune-test".into());
        mem.max_entries = 5;
        for i in 0..10 {
            mem.add_discovered(format!("entry {}", i), vec![]);
        }
        assert!(mem.auto_discovered.len() <= 5);
    }
}
