//! VSA HyperCube 联想扩召回 (C4 阶段 — 检索语义增强)
//!
//! 用 VSA (Vector Symbolic Architecture) 超向量做概念联想: 查询词 bundle 成
//! 查询超向量, 与词表各词超向量算相似度, 召回关联概念词注入检索。
//!
//! 与 KB embedding 的区别 (CONTEXT.md 消歧):
//! - KB embedding = 向量存储 (余弦, 语义召回) — 已有 semantic_search
//! - VSA embedding = 符号超向量 (关联回忆) — 本模块做概念扩召
//!
//! 词向量 = 字符 trigram 的确定性 ±1 超向量 bundle → 共享 trigram 的词
//! (如 retrieval/retrieve) 相似度高, 无需外部 embedding 端点, 纯本地可测。

use std::collections::HashMap;

use crate::core::nt_core_hcube::vsa::{VsaBackend, VSAEngine};

/// VSA 联想扩召器
pub struct VsaAssociativeExpander {
    engine: VSAEngine,
    dim: usize,
    /// 词 → 超向量 (确定性, 惰性生成)
    vectors: HashMap<String, Vec<f64>>,
}

impl Default for VsaAssociativeExpander {
    fn default() -> Self {
        Self::new(1024)
    }
}

impl VsaAssociativeExpander {
    pub fn new(dim: usize) -> Self {
        Self {
            engine: VSAEngine::new(dim),
            dim,
            vectors: HashMap::new(),
        }
    }

    /// 插入词条 (惰性生成超向量)
    pub fn insert_term(&mut self, term: &str) -> Vec<f64> {
        let v = self.term_vector(term);
        self.vectors.insert(term.to_string(), v.clone());
        v
    }

    /// 批量插入
    pub fn insert_terms<I: IntoIterator<Item = String>>(&mut self, terms: I) {
        for t in terms {
            self.insert_term(&t);
        }
    }

    /// 词表大小
    pub fn vocab_size(&self) -> usize {
        self.vectors.len()
    }

    /// 联想召回: 查询词 → 关联概念词 (按 VSA 相似度排序)
    pub fn expand(&self, query: &str, top_k: usize) -> Vec<(String, f64)> {
        if self.vectors.is_empty() {
            return Vec::new();
        }
        let q_vec = self.query_vector(query);
        if q_vec.iter().all(|x| *x == 0.0) {
            return Vec::new();
        }
        let mut scored: Vec<(String, f64)> = self
            .vectors
            .iter()
            .map(|(term, v)| (term.clone(), self.engine.similarity(&q_vec, v)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    /// 扩召回查询: 原查询 + 关联概念词 (供检索注入)
    pub fn expand_query(&self, query: &str, top_k: usize) -> String {
        let related: Vec<String> = self
            .expand(query, top_k)
            .into_iter()
            .map(|(t, _)| t)
            .collect();
        if related.is_empty() {
            query.to_string()
        } else {
            format!("{} {}", query, related.join(" "))
        }
    }

    /// 查询超向量 = 查询词超向量 bundle
    fn query_vector(&self, query: &str) -> Vec<f64> {
        let terms: Vec<&str> = query.split_whitespace().collect();
        if terms.is_empty() {
            return vec![0.0; self.dim];
        }
        let vecs: Vec<Vec<f64>> = terms.iter().map(|t| self.term_vector(t)).collect();
        let refs: Vec<&[f64]> = vecs.iter().map(|v| v.as_slice()).collect();
        self.engine.bundle(&refs)
    }

    /// 词超向量 = 字符 trigram 超向量 bundle (共享 trigram → 语义相近)
    fn term_vector(&self, term: &str) -> Vec<f64> {
        let chars: Vec<char> = term.to_lowercase().chars().collect();
        if chars.len() < 3 {
            return self.ngram_vector(term);
        }
        let trigrams: Vec<Vec<f64>> = chars
            .windows(3)
            .map(|w| self.ngram_vector(&w.iter().collect::<String>()))
            .collect();
        let refs: Vec<&[f64]> = trigrams.iter().map(|v| v.as_slice()).collect();
        self.engine.bundle(&refs)
    }

    /// n-gram 确定性 ±1 超向量 (FNV-1a 种子 + splitmix64 伪随机)
    fn ngram_vector(&self, ngram: &str) -> Vec<f64> {
        let seed = fnv1a(ngram);
        (0..self.dim)
            .map(|i| {
                let r = splitmix64(seed.wrapping_add(i as u64));
                if (r & 1) == 1 { 1.0 } else { -1.0 }
            })
            .collect()
    }
}

/// FNV-1a 64-bit 哈希
fn fnv1a(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// splitmix64 伪随机 (确定性)
fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expander() -> VsaAssociativeExpander {
        let mut e = VsaAssociativeExpander::new(1024);
        e.insert_terms(
            [
                "retrieval", "retrieve", "search", "query", "index",
                "generation", "generate", "llm", "prompt",
                "graph", "knowledge", "entity", "relation",
            ]
            .iter()
            .map(|s| s.to_string()),
        );
        e
    }

    #[test]
    fn test_deterministic_vector() {
        let e = VsaAssociativeExpander::new(256);
        let a = e.term_vector("retrieval");
        let b = e.term_vector("retrieval");
        assert_eq!(a, b, "同词应生成确定性相同向量");
    }

    #[test]
    fn test_similar_terms_high_similarity() {
        let e = VsaAssociativeExpander::new(1024);
        // 共享 trigram 的词应相似 (retrieval vs retrieve)
        let a = e.term_vector("retrieval");
        let b = e.term_vector("retrieve");
        let sim = e.engine.similarity(&a, &b);
        assert!(sim > 0.3, "共享 trigram 词应相似: sim={}", sim);
        // 无关词应低相似
        let c = e.term_vector("zzzzqqqq");
        let sim2 = e.engine.similarity(&a, &c);
        assert!(sim2 < 0.2, "无关词应低相似: sim={}", sim2);
    }

    #[test]
    fn test_expand_returns_related_terms() {
        let e = expander();
        let related = e.expand("retrieval", 3);
        assert!(!related.is_empty(), "应召回关联词");
        // 最相关的应是 retrieval 自身或 retrieve
        let top = related[0].0.as_str();
        assert!(
            top == "retrieval" || top == "retrieve" || top == "search",
            "top 关联词应语义相关, 实际: {}", top
        );
    }

    #[test]
    fn test_expand_query_appends_related() {
        let e = expander();
        let expanded = e.expand_query("retrieval", 2);
        assert!(expanded.contains("retrieval"), "扩召查询应保留原词");
        assert!(expanded.len() > "retrieval".len(), "应追加关联词: {}", expanded);
    }

    #[test]
    fn test_empty_vocab_returns_empty() {
        let e = VsaAssociativeExpander::new(256);
        assert!(e.expand("anything", 5).is_empty());
        assert_eq!(e.expand_query("anything", 5), "anything");
    }

    #[test]
    fn test_bundle_similar_to_components() {
        let e = VsaAssociativeExpander::new(1024);
        let a = e.term_vector("graph");
        let b = e.term_vector("knowledge");
        let q = e.query_vector("graph knowledge");
        assert!(e.engine.similarity(&q, &a) > 0.3, "查询向量应与成分词相似");
        assert!(e.engine.similarity(&q, &b) > 0.3);
    }

    #[test]
    fn test_vocab_size() {
        let mut e = VsaAssociativeExpander::new(256);
        assert_eq!(e.vocab_size(), 0);
        e.insert_term("hello");
        e.insert_term("world");
        assert_eq!(e.vocab_size(), 2);
    }
}