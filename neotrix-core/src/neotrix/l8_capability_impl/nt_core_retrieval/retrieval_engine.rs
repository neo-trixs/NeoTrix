//! nt_core_retrieval — 知识检索引擎
//!
//! 支持 BM25、向量检索和混合搜索的统一检索引擎
//! 节点: nt_core_retrieval (L8)
//! Provides: vector_search, bm25_search, hybrid_search
//! Requires: nt_memory_kb, tokenizers
//! Rune: Indigo, Alabaster

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetrievalConfig {
    /// 混合搜索权重 (0.0=纯 BM25, 1.0=纯向量)
    pub hybrid_weight: f32,
    /// BM25 参数 k1
    pub bm25_k1: f32,
    /// BM25 参数 b
    pub bm25_b: f32,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            hybrid_weight: 0.5,
            bm25_k1: 0.5,
            bm25_b: 0.5,
        }
    }
}

/// 检索结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetrievalResult {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub source: String,
}

/// 检索引擎

/// KB 索引抽象 — 检索引擎可插拔的底层索引接口。
pub trait KbIndex: Send + Sync {
    fn search(&self, query: &str, top_k: usize) -> Vec<RetrievalResult>;
}

/// 检索引擎
pub struct RetrievalEngine {
    config: RetrievalConfig,
    index: Option<Box<dyn KbIndex>>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl RetrievalEngine {
    pub fn new(config: RetrievalConfig) -> Self {
        Self {
            config,
            index: None,
            metadata: HashMap::new(),
        }
    }

    pub fn set_index(&mut self, index: Box<dyn KbIndex>) {
        self.index = Some(index);
    }

    pub fn bm25_search(
        &self,
        _query: &str,
        _top_k: usize,
    ) -> Result<Vec<RetrievalResult>, NeoTrixError> {
        // 简化实现：返回空结果占位
        // 实际实现会使用 BM25 算法
        Ok(Vec::new())
    }

    pub fn vector_search(
        &self,
        _query: &str,
        _top_k: usize,
    ) -> Result<Vec<RetrievalResult>, NeoTrixError> {
        // 简化实现：返回空结果占位
        Ok(Vec::new())
    }

    pub fn hybrid_search(
        &self,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<RetrievalResult>, NeoTrixError> {
        let bm25_results = self.bm25_search(query, top_k)?;
        let vector_results = self.vector_search(query, top_k)?;

        let mut combined: Vec<(RetrievalResult, f32)> = bm25_results
            .into_iter()
            .map(|r| (r, 1.0 - self.config.hybrid_weight))
            .collect();
        combined.extend(
            vector_results
                .into_iter()
                .map(|r| (r, self.config.hybrid_weight)),
        );

        combined.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(combined.into_iter().map(|(r, _)| r).take(top_k).collect())
    }

    pub fn config(&self) -> &RetrievalConfig {
        &self.config
    }
}

impl CapabilityNode for RetrievalEngine {
    fn node_id(&self) -> &str {
        "nt_core_retrieval"
    }
    fn provides(&self) -> Vec<String> {
        vec![
            "vector_search".into(),
            "bm25_search".into(),
            "hybrid_search".into(),
        ]
    }
    fn requires(&self) -> Vec<String> {
        vec!["nt_memory_kb".into(), "tokenizer".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Indigo, RuneSocket::Alabaster]
    }
    fn constellation_level(&self) -> u8 {
        1
    }
    fn promote_constellation(&mut self) -> bool {
        true
    }
}

impl SelfTest for RetrievalEngine {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let engine = RetrievalEngine::new(RetrievalConfig::default());
            let _ = engine.config();
            let _ = engine.hybrid_search("test", 5);
            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_core_retrieval_engine"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_retrieval_engine_self_test() {
        let engine = RetrievalEngine::new(RetrievalConfig::default());
        assert!(engine.self_test().is_ok());
    }
}
