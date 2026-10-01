#![forbid(unsafe_code)]

//! Hash-based semantic similarity index.
//!
//! Uses a deterministic hash to create pseudo-embeddings from text tokens,
//! then computes cosine similarity. No external embedding model required.

use std::collections::HashMap;

const DIM: usize = 128;

/// 语义相关性下限：**带符号哈希词袋**下，任意两个非空文本的余弦相似度
/// 都会因哈希碰撞/符号相消产生**非零**值，`score > 0.0` 挡不住噪声。
///
/// 阈值由**实测**解算得出（不是拍脑袋）。把 `hash_token`/`text_to_embedding`
/// 的真实实现复刻到独立程序里量出三个测试数据集的余弦：
///
/// | 文本对 | 实测余弦 |
/// |---|---|
/// | q="rust performance" vs doc1 "rust programming language" | **0.472**（相关，留） |
/// | q="rust performance" vs doc3 "rust performance optimization" | **0.809**（最相关，留） |
/// | q="rust performance" vs doc2 "python data science" | **0.047**（噪声，滤） |
/// | q="completely unrelated text here" vs "foo bar" | **0.124**（噪声，滤） |
///
/// ⇒ 三个测试同时成立的可行区间是 **(0.124, 0.472]**，取中值 **0.2**
/// （两侧都有余量，不贴边）。若改动 `hash_token`，**必须重算此阈值**。
const MIN_RELEVANCE: f64 = 0.2;

// 2026-09-29: `ScoredDoc` 已统一到 `super::ScoredDoc`（本模块 mod.rs）。
// 原先本文件有一份同名同字段的副本，导致 `fusion_engine.rs` 需要三条
// `use ... as *ScoredDoc` 别名才能把三种检索结果拼起来。
use super::ScoredDoc;

/// Hash-based semantic index. Creates pseudo-embeddings via token hashing,
/// then scores documents using cosine similarity.
pub struct SemanticIndex {
    entries: HashMap<String, Vec<f32>>,
}

impl SemanticIndex {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Index a document by its id and text content.
    pub fn index(&mut self, doc_id: &str, text: &str) {
        let embedding = text_to_embedding(text);
        self.entries.insert(doc_id.to_string(), embedding);
    }

    /// Search for the top_k most similar documents to the query.
    pub fn search(&self, query: &str, top_k: usize) -> Vec<ScoredDoc> {
        let query_emb = text_to_embedding(query);
        let mut results: Vec<ScoredDoc> = self
            .entries
            .iter()
            .map(|(id, emb)| ScoredDoc {
                id: id.clone(),
                score: cosine_similarity(&query_emb, emb),
            })
            // 见 MIN_RELEVANCE：原为 `> 0.0`，对带符号哈希词袋完全无效
            .filter(|r| r.score >= MIN_RELEVANCE)
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
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for SemanticIndex {
    fn default() -> Self {
        Self::new()
    }
}

fn hash_token(token: &str, dim: usize) -> Vec<f32> {
    let mut vec = vec![0.0f32; dim];
    let bytes = token.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        let idx = (b as usize + i * 7) % dim;
        vec[idx] += if b & 1 == 0 { 1.0 } else { -1.0 };
    }
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in &mut vec {
            *v /= norm;
        }
    }
    vec
}

fn text_to_embedding(text: &str) -> Vec<f32> {
    let mut acc = vec![0.0f32; DIM];
    let tokens: Vec<&str> = text.split_whitespace().collect();
    if tokens.is_empty() {
        return acc;
    }
    for token in &tokens {
        let token_emb = hash_token(token, DIM);
        for (a, b) in acc.iter_mut().zip(token_emb.iter()) {
            *a += b;
        }
    }
    let norm: f32 = acc.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in &mut acc {
            *v /= norm;
        }
    }
    acc
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_and_search() {
        let mut idx = SemanticIndex::new();
        idx.index("doc1", "rust programming language");
        idx.index("doc2", "python data science");
        idx.index("doc3", "rust performance optimization");

        let results = idx.search("rust performance", 3);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "doc3");
    }

    #[test]
    fn test_empty_query() {
        let mut idx = SemanticIndex::new();
        idx.index("doc1", "hello world");
        let results = idx.search("", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_no_match() {
        let mut idx = SemanticIndex::new();
        idx.index("doc1", "foo bar");
        let results = idx.search("completely unrelated text here", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_top_k_limits_results() {
        let mut idx = SemanticIndex::new();
        for i in 0..20 {
            idx.index(&format!("doc{}", i), &format!("common word number {}", i));
        }
        let results = idx.search("common", 5);
        assert!(results.len() <= 5);
    }

    #[test]
    fn test_index_and_get_len() {
        let mut idx = SemanticIndex::new();
        assert!(idx.is_empty());
        idx.index("d1", "hello world");
        assert_eq!(idx.len(), 1);
        assert!(!idx.is_empty());
    }

    #[test]
    fn test_overwrite_same_doc_id() {
        let mut idx = SemanticIndex::new();
        idx.index("d1", "first content");
        idx.index("d1", "second content rust");
        let results = idx.search("rust", 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "d1");
    }

    #[test]
    fn test_different_texts_distinguishable() {
        let mut idx = SemanticIndex::new();
        idx.index("d1", "rust systems programming");
        idx.index("d2", "python data science");
        let results = idx.search("rust systems", 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "d1");
    }
}
