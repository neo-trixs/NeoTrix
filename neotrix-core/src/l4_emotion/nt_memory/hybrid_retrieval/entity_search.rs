#![forbid(unsafe_code)]

//! Entity search — inverted index from entity names to document IDs.
//!
//! Supports exact entity lookup and query-based entity matching.

use std::collections::{HashMap, HashSet};

// 2026-09-29: `ScoredDoc` 已统一到 `super::ScoredDoc`（本模块 mod.rs）。
// 原先本文件有一份同名同字段的副本，导致 `fusion_engine.rs` 需要三条
// `use ... as *ScoredDoc` 别名才能把三种检索结果拼起来。
use super::ScoredDoc;

pub struct EntityIndex {
    entity_to_docs: HashMap<String, HashSet<String>>,
    doc_entities: HashMap<String, Vec<String>>,
}

impl EntityIndex {
    pub fn new() -> Self {
        Self {
            entity_to_docs: HashMap::new(),
            doc_entities: HashMap::new(),
        }
    }

    /// Index a document with its associated entities.
    pub fn index(&mut self, doc_id: &str, entities: &[String]) {
        self.doc_entities
            .insert(doc_id.to_string(), entities.to_vec());
        for entity in entities {
            self.entity_to_docs
                .entry(entity.to_lowercase())
                .or_insert_with(HashSet::new)
                .insert(doc_id.to_string());
        }
    }

    /// Search by entity name (case-insensitive exact match).
    pub fn search_by_entity(&self, entity_name: &str) -> Vec<ScoredDoc> {
        let key = entity_name.to_lowercase();
        self.entity_to_docs
            .get(&key)
            .map(|docs| {
                docs.iter()
                    .map(|id| ScoredDoc {
                        id: id.clone(),
                        score: 1.0,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Search by query text — counts entity overlap between query words and indexed entities.
    pub fn search(&self, query: &str, top_k: usize) -> Vec<ScoredDoc> {
        let query_lower = query.to_lowercase();
        let query_entities: Vec<&str> = query_lower.split_whitespace().collect();
        if query_entities.is_empty() {
            return vec![];
        }

        let mut scores: HashMap<String, f64> = HashMap::new();

        for query_ent in &query_entities {
            if let Some(docs) = self.entity_to_docs.get(*query_ent) {
                for doc_id in docs {
                    *scores.entry(doc_id.clone()).or_insert(0.0) += 1.0;
                }
            }
        }

        let max_score = scores.values().copied().fold(0.0f64, f64::max);

        let mut results: Vec<ScoredDoc> = scores
            .into_iter()
            .map(|(id, raw)| ScoredDoc {
                id,
                score: if max_score > 0.0 {
                    raw / max_score
                } else {
                    0.0
                },
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(top_k);
        results
    }

    pub fn len(&self) -> usize {
        self.doc_entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.doc_entities.is_empty()
    }
}

impl Default for EntityIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_by_entity() {
        let mut idx = EntityIndex::new();
        idx.index("doc1", &["rust".into(), "systems".into()]);
        idx.index("doc2", &["python".into(), "ml".into()]);
        idx.index("doc3", &["rust".into(), "web".into()]);

        let results = idx.search_by_entity("rust");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_by_entity_case_insensitive() {
        let mut idx = EntityIndex::new();
        idx.index("doc1", &["Rust".into()]);

        let results = idx.search_by_entity("rust");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_search() {
        let mut idx = EntityIndex::new();
        idx.index("doc1", &["rust".into(), "systems".into()]);
        idx.index("doc2", &["python".into(), "ml".into()]);

        let results = idx.search("rust systems", 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "doc1");
        assert!((results[0].score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_empty_query() {
        let mut idx = EntityIndex::new();
        idx.index("doc1", &["a".into()]);
        let results = idx.search("", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_no_match() {
        let mut idx = EntityIndex::new();
        idx.index("doc1", &["foo".into()]);
        let results = idx.search_by_entity("bar");
        assert!(results.is_empty());
    }
}
