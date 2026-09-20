//! Elastic Memory Orchestrator — 吸收自 AutoAgent
//!
//! 三级记忆压缩: raw records → compressed trajectories → reusable episodic abstractions
//! 减少 60-80% token 开销，同时保留决策关键证据。

use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

/// 原始记录 — 完整保真度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawRecord {
    pub id: String,
    pub content: String,
    pub timestamp: u64,
    pub importance: f64, // 0.0 - 1.0
}

/// 压缩轨迹 — 冗余过滤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedTrajectory {
    pub id: String,
    pub summary: String,
    pub key_evidence: Vec<String>,
    pub timestamp: u64,
    pub compression_ratio: f64,
}

/// 可复用情景抽象 — 模式提取
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicAbstraction {
    pub id: String,
    pub pattern: String,
    pub reuse_count: u32,
    pub success_rate: f64,
    pub domain: String,
}

/// 弹性记忆编排器
pub struct ElasticMemoryOrchestrator {
    raw_buffer: VecDeque<RawRecord>,
    compressed_buffer: VecDeque<CompressedTrajectory>,
    episodic_store: Vec<EpisodicAbstraction>,
    max_raw_size: usize,
    max_compressed_size: usize,
}

impl ElasticMemoryOrchestrator {
    pub fn new() -> Self {
        Self {
            raw_buffer: VecDeque::new(),
            compressed_buffer: VecDeque::new(),
            episodic_store: Vec::new(),
            max_raw_size: 100,
            max_compressed_size: 50,
        }
    }

    /// 添加原始记录
    pub fn add_raw(&mut self, record: RawRecord) {
        self.raw_buffer.push_back(record);
        if self.raw_buffer.len() > self.max_raw_size {
            self.raw_buffer.pop_front();
        }
    }

    /// 压缩轨迹 — 过滤低重要性，保留关键证据
    pub fn compress(&mut self) {
        let mut to_compress: Vec<RawRecord> = Vec::new();
        while self.raw_buffer.len() > self.max_raw_size / 2 {
            if let Some(record) = self.raw_buffer.pop_front() {
                to_compress.push(record);
            }
        }

        if to_compress.is_empty() {
            return;
        }

        // 按重要性排序，保留高重要性的
        to_compress.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
        let keep_count = to_compress.len() / 3; // 保留 1/3
        let key_evidence: Vec<String> = to_compress.iter()
            .take(keep_count)
            .map(|r| r.content.clone())
            .collect();

        let summary = format!("Compressed {} records, kept {} key evidence",
            to_compress.len(), key_evidence.len());

        let trajectory = CompressedTrajectory {
            id: format!("traj_{}", self.compressed_buffer.len()),
            summary,
            key_evidence,
            timestamp: now(),
            compression_ratio: keep_count as f64 / to_compress.len() as f64,
        };

        self.compressed_buffer.push_back(trajectory);
        if self.compressed_buffer.len() > self.max_compressed_size {
            self.compressed_buffer.pop_front();
        }
    }

    /// 提取情景抽象 — 识别可复用模式
    pub fn extract_abstraction(&mut self, pattern: String, domain: String) {
        // 检查是否已有相似模式
        if let Some(existing) = self.episodic_store.iter_mut().find(|e| e.domain == domain && similar(&e.pattern, &pattern)) {
            existing.reuse_count += 1;
            existing.success_rate = (existing.success_rate * (existing.reuse_count - 1) as f64 + 1.0) / existing.reuse_count as f64;
        } else {
            self.episodic_store.push(EpisodicAbstraction {
                id: format!("epi_{}", self.episodic_store.len()),
                pattern,
                reuse_count: 1,
                success_rate: 1.0,
                domain,
            });
        }
    }

    /// 获取上下文 — 组合三层结果，固定大小输出
    pub fn get_context(&self) -> MemoryContext {
        MemoryContext {
            recent_raw: self.raw_buffer.iter().rev().take(5).cloned().collect(),
            recent_compressed: self.compressed_buffer.iter().rev().take(3).cloned().collect(),
            relevant_abstractions: self.episodic_store.iter()
                .filter(|e| e.reuse_count > 1)
                .take(5)
                .cloned()
                .collect(),
        }
    }

    /// 统计信息
    pub fn stats(&self) -> OrchestratorStats {
        OrchestratorStats {
            raw_count: self.raw_buffer.len(),
            compressed_count: self.compressed_buffer.len(),
            abstraction_count: self.episodic_store.len(),
            total_token_savings: self.compressed_buffer.iter()
                .map(|c| (1.0 - c.compression_ratio) * c.key_evidence.len() as f64 * 100.0)
                .sum(),
        }
    }
}

/// 固定大小上下文输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContext {
    pub recent_raw: Vec<RawRecord>,
    pub recent_compressed: Vec<CompressedTrajectory>,
    pub relevant_abstractions: Vec<EpisodicAbstraction>,
}

/// 编排器统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorStats {
    pub raw_count: usize,
    pub compressed_count: usize,
    pub abstraction_count: usize,
    pub total_token_savings: f64,
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn similar(a: &str, b: &str) -> bool {
    // 简单相似度: 包含相同关键词
    let a_words: Vec<&str> = a.split_whitespace().collect();
    let b_words: Vec<&str> = b.split_whitespace().collect();
    let common = a_words.iter().filter(|w| b_words.contains(w)).count();
    common as f64 / a_words.len().max(b_words.len()) as f64 > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elastic_memory_basic() {
        let mut orch = ElasticMemoryOrchestrator::new();

        // Add 20 raw records
        for i in 0..20 {
            orch.add_raw(RawRecord {
                id: format!("r{}", i),
                content: format!("Record content {}", i),
                timestamp: now(),
                importance: (i as f64) / 20.0,
            });
        }

        assert_eq!(orch.raw_buffer.len(), 20);

        // Compress
        orch.compress();

        // Should have fewer raw records after compression
        assert!(orch.raw_buffer.len() < 20);

        // Context should be bounded
        let ctx = orch.get_context();
        assert!(ctx.recent_raw.len() <= 5);
        assert!(ctx.recent_compressed.len() <= 3);
    }

    #[test]
    fn test_abstraction_extraction() {
        let mut orch = ElasticMemoryOrchestrator::new();

        orch.extract_abstraction("error handling pattern".to_string(), "coding".to_string());
        orch.extract_abstraction("error handling pattern".to_string(), "coding".to_string());

        assert_eq!(orch.episodic_store.len(), 1);
        assert_eq!(orch.episodic_store[0].reuse_count, 2);
    }
}
