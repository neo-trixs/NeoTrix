#![forbid(unsafe_code)]

//! BM25 keyword search index.
//!
//! Standard BM25 scoring with k1=1.5, b=0.75.
//! Uses term frequency and document frequency for probabilistic ranking.

use std::collections::HashMap;

const K1: f64 = 1.5;
const B: f64 = 0.75;

// 2026-09-29: `ScoredDoc` 已统一到 `super::ScoredDoc`（本模块 mod.rs）。
// 原先本文件有一份同名同字段的副本，导致 `fusion_engine.rs` 需要三条
// `use ... as *ScoredDoc` 别名才能把三种检索结果拼起来。
use super::ScoredDoc;

struct DocInfo {
    term_freqs: HashMap<String, u32>,
    doc_len: u32,
}

pub struct BM25Index {
    docs: HashMap<String, DocInfo>,
    doc_freqs: HashMap<String, u32>,
    total_docs: u32,
    avg_doc_len: f64,
}

impl BM25Index {
    pub fn new() -> Self {
        Self {
            docs: HashMap::new(),
            doc_freqs: HashMap::new(),
            total_docs: 0,
            avg_doc_len: 0.0,
        }
    }

    /// Index a document by id and text content.
    pub fn index(&mut self, doc_id: &str, text: &str) {
        let tokens = tokenize(text);
        let doc_len = tokens.len() as u32;
        let mut term_freqs: HashMap<String, u32> = HashMap::new();
        for token in &tokens {
            *term_freqs.entry(token.clone()).or_insert(0) += 1;
        }

        for term in term_freqs.keys() {
            *self.doc_freqs.entry(term.clone()).or_insert(0) += 1;
        }

        self.total_docs += 1;
        let prev = self.avg_doc_len * (self.total_docs - 1) as f64;
        self.avg_doc_len = (prev + doc_len as f64) / self.total_docs as f64;

        self.docs.insert(
            doc_id.to_string(),
            DocInfo {
                term_freqs,
                doc_len,
            },
        );
    }

    /// Search for top_k documents matching the query.
    pub fn search(&self, query: &str, top_k: usize) -> Vec<ScoredDoc> {
        let query_tokens = tokenize(query);
        if query_tokens.is_empty() {
            return vec![];
        }

        let mut scores: HashMap<String, f64> = HashMap::new();

        for token in &query_tokens {
            let df = self.doc_freqs.get(token).copied().unwrap_or(0) as f64;
            if df == 0.0 {
                continue;
            }
            let idf = ((self.total_docs as f64 - df + 0.5) / (df + 0.5) + 1.0).ln();

            for (doc_id, info) in &self.docs {
                let tf = info.term_freqs.get(token).copied().unwrap_or(0) as f64;
                let norm = 1.0 - B + B * (info.doc_len as f64 / self.avg_doc_len);
                let term_score = idf * (tf * (K1 + 1.0)) / (tf + K1 * norm);
                *scores.entry(doc_id.clone()).or_insert(0.0) += term_score;
            }
        }

        let mut results: Vec<ScoredDoc> = scores
            .into_iter()
            .map(|(id, score)| ScoredDoc { id, score })
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
        self.docs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }
}

impl Default for BM25Index {
    fn default() -> Self {
        Self::new()
    }
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_and_search() {
        let mut idx = BM25Index::new();
        idx.index("doc1", "rust programming language");
        idx.index("doc2", "python programming language");
        idx.index("doc3", "rust performance optimization");

        let results = idx.search("rust", 3);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "doc1");
        assert!(results[0].score > 0.0);
    }

    #[test]
    fn test_ranking_prefers_exact_match() {
        let mut idx = BM25Index::new();
        idx.index("a", "hello world");
        idx.index("b", "hello beautiful world");
        idx.index("c", "completely different");

        let results = idx.search("hello", 3);
        assert!(results.len() >= 2);
        assert_ne!(results[0].id, "c");
    }

    #[test]
    fn test_no_match_returns_empty() {
        let mut idx = BM25Index::new();
        idx.index("doc1", "foo bar baz");
        let results = idx.search("xyz", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_empty_query() {
        let mut idx = BM25Index::new();
        idx.index("doc1", "test");
        let results = idx.search("", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_top_k() {
        let mut idx = BM25Index::new();
        for i in 0..50 {
            idx.index(&format!("d{}", i), &format!("search keyword doc {}", i));
        }
        let results = idx.search("search keyword", 5);
        assert!(results.len() <= 5);
    }
}
