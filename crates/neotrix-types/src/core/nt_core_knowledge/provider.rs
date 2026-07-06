/// KnowledgeProvider trait — inspired by GenericAgent's SOP knowledge system.
/// External knowledge sources that can be queried at runtime.
/// Each provider implements `query()` which returns relevant knowledge entries.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub source: KnowledgeSourceLabel,
    pub relevance: f32,    // 0.0 - 1.0
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeSourceLabel {
    Builtin,
    File { path: String },
    User { name: String },
}

/// Provider trait: any external source that can supply knowledge entries
pub trait KnowledgeProvider: Send + Sync {
    fn name(&self) -> &str;
    /// Query the provider for relevant knowledge
    fn query(&self, query: &str, max_results: usize) -> Vec<KnowledgeEntry>;
    /// Get all available entries (for indexing)
    fn all_entries(&self) -> Vec<KnowledgeEntry>;
}

/// A provider that reads SOP (Standard Operating Procedure) files from a directory.
/// Inspired by GenericAgent's `memory/` directory patterns.
pub struct SopDirectoryProvider {
    name: String,
    dir: PathBuf,
    entries: Vec<KnowledgeEntry>,
}

impl SopDirectoryProvider {
    pub fn new(name: impl Into<String>, dir: PathBuf) -> Self {
        let mut provider = Self {
            name: name.into(),
            dir,
            entries: Vec::new(),
        };
        provider.reload();
        provider
    }

    /// Reload entries from disk
    pub fn reload(&mut self) {
        self.entries.clear();
        if !self.dir.exists() {
            return;
        }
        if let Ok(entries) = std::fs::read_dir(&self.dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "md" || e == "txt").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let title = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        self.entries.push(KnowledgeEntry {
                            id: format!("sop-{}", title),
                            title,
                            content,
                            source: KnowledgeSourceLabel::File {
                                path: path.to_string_lossy().to_string(),
                            },
                            relevance: 0.5,
                            tags: vec!["sop".into(), "knowledge".into()],
                        });
                    }
                }
            }
        }
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

impl KnowledgeProvider for SopDirectoryProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn query(&self, query: &str, max_results: usize) -> Vec<KnowledgeEntry> {
        let query_lower = query.to_lowercase();
        let mut scored: Vec<(f32, &KnowledgeEntry)> = self.entries
            .iter()
            .map(|e| {
                let score = if e.title.to_lowercase().contains(&query_lower) {
                    0.9
                } else if e.content.to_lowercase().contains(&query_lower) {
                    0.6
                } else {
                    0.1
                };
                (score, e)
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter()
            .take(max_results)
            .map(|(score, e)| {
                let mut entry = e.clone();
                entry.relevance = score;
                entry
            })
            .collect()
    }

    fn all_entries(&self) -> Vec<KnowledgeEntry> {
        self.entries.clone()
    }
}

/// A composite provider that queries multiple sub-providers
pub struct CompositeProvider {
    name: String,
    providers: Vec<Box<dyn KnowledgeProvider>>,
}

impl CompositeProvider {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            providers: Vec::new(),
        }
    }

    pub fn add(&mut self, provider: Box<dyn KnowledgeProvider>) {
        self.providers.push(provider);
    }
}

impl KnowledgeProvider for CompositeProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn query(&self, query: &str, max_results: usize) -> Vec<KnowledgeEntry> {
        let per_provider = (max_results / self.providers.len().max(1)).max(1);
        let mut results = Vec::new();
        for provider in &self.providers {
            results.extend(provider.query(query, per_provider));
        }
        results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(max_results);
        results
    }

    fn all_entries(&self) -> Vec<KnowledgeEntry> {
        let mut all = Vec::new();
        for provider in &self.providers {
            all.extend(provider.all_entries());
        }
        all
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sop_provider_empty_dir() {
        let tmp = std::env::temp_dir().join("neotrix-test-sop-empty");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).expect("create temp dir");
        let provider = SopDirectoryProvider::new("test", tmp.clone());
        assert_eq!(provider.entry_count(), 0);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_sop_provider_loads_md() {
        let tmp = std::env::temp_dir().join("neotrix-test-sop-md");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).expect("create temp dir");
        std::fs::write(tmp.join("test_sop.md"), "# Test SOP\nDo X then Y").expect("write test file");
        let provider = SopDirectoryProvider::new("test", tmp.clone());
        assert_eq!(provider.entry_count(), 1);
        let results = provider.query("test", 5);
        assert!(!results.is_empty());
        assert!(results[0].content.contains("Test SOP"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_composite_provider() {
        let mut composite = CompositeProvider::new("composite");
        let tmp1 = std::env::temp_dir().join("neotrix-test-cp1");
        let tmp2 = std::env::temp_dir().join("neotrix-test-cp2");
        let _ = std::fs::remove_dir_all(&tmp1);
        let _ = std::fs::remove_dir_all(&tmp2);
        std::fs::create_dir_all(&tmp1).expect("create temp dir1");
        std::fs::create_dir_all(&tmp2).expect("create temp dir2");
        std::fs::write(tmp1.join("a.md"), "content A").expect("write a.md");
        std::fs::write(tmp2.join("b.md"), "content B").expect("write b.md");
        composite.add(Box::new(SopDirectoryProvider::new("p1", tmp1.clone())));
        composite.add(Box::new(SopDirectoryProvider::new("p2", tmp2.clone())));
        assert_eq!(composite.all_entries().len(), 2);
        let _ = std::fs::remove_dir_all(&tmp1);
        let _ = std::fs::remove_dir_all(&tmp2);
    }

    #[test]
    fn test_knowledge_entry_serde() {
        let entry = KnowledgeEntry {
            id: "test-1".into(),
            title: "Test".into(),
            content: "content".into(),
            source: KnowledgeSourceLabel::Builtin,
            relevance: 0.8,
            tags: vec!["rust".into()],
        };
        let json = serde_json::to_string(&entry).expect("serialize should succeed");
        let deserialized: KnowledgeEntry = serde_json::from_str(&json).expect("deserialize should succeed");
        assert_eq!(deserialized.id, "test-1");
    }
}
