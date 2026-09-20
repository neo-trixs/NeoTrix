//! # 分层内存管线 (LightMem pattern)
//!
//! 四阶段管线: compress → segment → extract → index
//! 借鉴 LightMem (ICLR 2026) 的分层内存管理。

use serde::{Deserialize, Serialize};

/// 压缩结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedChunk {
    pub text: String,
    pub compression_ratio: f64,
    pub preserved_tokens: usize,
}

/// 主题分段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicSegment {
    pub topic_id: u32,
    pub content: String,
    pub importance: f64,
    pub entities: Vec<String>,
}

/// 记忆条目 -- re-export canonical definition from L0
pub use crate::l0_substrate::nt_core_substrate_types::MemoryEntry;

/// 索引结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexResult {
    pub vector_ids: Vec<u64>,
    pub bm25_terms: Vec<String>,
    pub entity_links: Vec<(String, String)>,
}

/// 分层内存管线 trait
pub trait TieredPipeline: Send + Sync {
    /// 阶段 1: 预压缩
    fn compress(&self, raw: &str) -> CompressedChunk;
    
    /// 阶段 2: 主题分段
    fn segment(&self, chunk: &CompressedChunk) -> Vec<TopicSegment>;
    
    /// 阶段 3: 重要性提取
    fn extract(&self, segments: &[TopicSegment], threshold: f64) -> Vec<MemoryEntry>;
    
    /// 阶段 4: 多模态索引
    fn index(&self, entries: &[MemoryEntry]) -> IndexResult;
    
    /// 管线名称
    fn name(&self) -> &str;
}

/// 简单压缩实现 (截断 + 关键句保留)
pub struct SimpleCompressor {
    pub max_ratio: f64,
}

impl Default for SimpleCompressor {
    fn default() -> Self {
        Self { max_ratio: 0.5 }
    }
}

impl TieredPipeline for SimpleCompressor {
    fn compress(&self, raw: &str) -> CompressedChunk {
        let target_len = (raw.len() as f64 * self.max_ratio) as usize;
        if raw.len() <= target_len {
            return CompressedChunk {
                text: raw.to_string(),
                compression_ratio: 1.0,
                preserved_tokens: raw.split_whitespace().count(),
            };
        }
        // 简单策略: 保留前半部分
        let truncated = raw.chars().take(target_len).collect::<String>();
        CompressedChunk {
            text: truncated,
            compression_ratio: self.max_ratio,
            preserved_tokens: (raw.split_whitespace().count() as f64 * self.max_ratio) as usize,
        }
    }
    
    fn segment(&self, chunk: &CompressedChunk) -> Vec<TopicSegment> {
        // 简单策略: 按段落分割
        chunk.text.split("\n\n")
            .enumerate()
            .map(|(i, p)| TopicSegment {
                topic_id: i as u32,
                content: p.to_string(),
                importance: 0.5,
                entities: vec![],
            })
            .collect()
    }
    
    fn extract(&self, segments: &[TopicSegment], threshold: f64) -> Vec<MemoryEntry> {
        segments.iter()
            .filter(|s| s.importance >= threshold)
            .enumerate()
            .map(|(i, s)| MemoryEntry {
                id: format!("mem_{}", i),
                content: s.content.clone(),
                importance: s.importance,
                embedding: vec![],
                metadata: std::collections::HashMap::new(),
            })
            .collect()
    }
    
    fn index(&self, entries: &[MemoryEntry]) -> IndexResult {
        IndexResult {
            vector_ids: (0..entries.len() as u64).collect(),
            bm25_terms: entries.iter().flat_map(|e| e.content.split_whitespace().map(String::from)).collect(),
            entity_links: vec![],
        }
    }
    
    fn name(&self) -> &str {
        "simple_compressor"
    }
}
