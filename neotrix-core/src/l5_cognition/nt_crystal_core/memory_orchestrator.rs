//! MemoryOrchestrator — 统一记忆架构
//!
//! 基于 Mem0 v3 + LangMem + Magic-Context 的统一记忆入口。
//! - ADD-only 累积 (Mem0 v3)
//! - 多信号融合检索: 语义+BM25+实体 (Mem0 v3)
//! - 双路径管理: 热路径+后台 (LangMem)
//! - 确定性衰减 (Magic-Context)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════
// 核心类型
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryId(pub String);

impl Default for MemoryId {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryId {
    pub fn new() -> Self { Self(uuid::Uuid::new_v4().to_string()) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: MemoryId,
    pub content: String,
    pub embedding: Vec<f64>,
    pub timestamp: u64,
    pub entities: Vec<String>,
    pub namespace: String,
    pub access_count: u32,
    pub last_accessed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryNamespace {
    Experience,
    Knowledge,
    Skills,
    Custom(String),
}

impl MemoryNamespace {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Experience => "experience",
            Self::Knowledge => "knowledge",
            Self::Skills => "skills",
            Self::Custom(s) => s,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 检索结果
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct RetrieveResult {
    pub record: MemoryRecord,
    pub semantic_score: f64,
    pub bm25_score: f64,
    pub entity_score: f64,
    pub fused_score: f64,
}

// ═══════════════════════════════════════════════════════════════
// MemoryOrchestrator
// ═══════════════════════════════════════════════════════════════

/// 统一记忆编排器 — Crystal Core 记忆的单一入口
pub struct MemoryOrchestrator {
    /// 存储: namespace → records
    pub records: HashMap<String, Vec<MemoryRecord>>,
    /// 实体索引: entity → memory_ids
    pub entity_index: HashMap<String, Vec<MemoryId>>,
    /// BM25 索引: word → memory_ids
    pub bm25_index: HashMap<String, Vec<MemoryId>>,
    /// 衰减配置
    pub decay_config: DecayConfig,
    /// 统计
    pub stats: OrchestratorStats,
}

#[derive(Debug, Clone)]
pub struct DecayConfig {
    /// 衰减半衰期 (ms)
    pub half_life_ms: u64,
    /// 最小保留分数
    pub min_retention: f64,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self { half_life_ms: 86_400_000, min_retention: 0.1 }
    }
}

#[derive(Debug, Clone, Default)]
pub struct OrchestratorStats {
    pub total_writes: u64,
    pub total_reads: u64,
    pub total_consolidations: u64,
}

impl Default for MemoryOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryOrchestrator {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            entity_index: HashMap::new(),
            bm25_index: HashMap::new(),
            decay_config: DecayConfig::default(),
            stats: OrchestratorStats::default(),
        }
    }

    /// ADD-only 写入 — 只追加，不更新不删除
    pub fn store(&mut self, content: &str, embedding: Vec<f64>, namespace: MemoryNamespace) -> MemoryId {
        let id = MemoryId::new();
        let now = now_ms();
        let entities = extract_entities(content);

        let record = MemoryRecord {
            id: id.clone(),
            content: content.to_string(),
            embedding,
            timestamp: now,
            entities: entities.clone(),
            namespace: namespace.as_str().to_string(),
            access_count: 0,
            last_accessed: now,
        };

        let ns = namespace.as_str().to_string();
        self.records.entry(ns).or_default().push(record);

        // 更新实体索引
        for entity in &entities {
            self.entity_index.entry(entity.clone()).or_default().push(id.clone());
        }

        // 更新 BM25 索引
        for word in content.split_whitespace() {
            self.bm25_index.entry(word.to_lowercase()).or_default().push(id.clone());
        }

        self.stats.total_writes += 1;
        id
    }

    /// 多信号融合检索
    pub fn retrieve(&mut self, query: &str, query_embedding: &[f64], top_k: usize) -> Vec<RetrieveResult> {
        let all: Vec<MemoryRecord> = self.records.values().flat_map(|v| v.iter().cloned()).collect();
        let mut results: Vec<RetrieveResult> = Vec::new();

        for record in &all {
            let semantic = cosine_sim(&record.embedding, query_embedding);
            let bm25 = compute_bm25_score(query, &record.content);
            let entity = compute_entity_score(query, &record.entities);
            let fused = 0.5 * semantic + 0.3 * bm25 + 0.2 * entity;

            results.push(RetrieveResult {
                record: record.clone(),
                semantic_score: semantic,
                bm25_score: bm25,
                entity_score: entity,
                fused_score: fused,
            });
        }

        results.sort_by(|a, b| b.fused_score.partial_cmp(&a.fused_score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);

        // 更新访问计数（R-P0-3：写回真实记录，否则用进废退失效）
        let now = now_ms();
        for r in &results {
            for records in self.records.values_mut() {
                if let Some(rec) = records.iter_mut().find(|a| a.id == r.record.id) {
                    rec.access_count += 1;
                    rec.last_accessed = now;
                    break;
                }
            }
        }

        self.stats.total_reads += 1;
        results
    }

    /// 后台整合: 衰减 + 去重
    pub fn consolidate(&mut self) -> ConsolidationReport {
        let mut removed = 0;
        let mut merged = 0;

        for records in self.records.values_mut() {
            // 去重: 移除 embedding 相似度 > 0.95 的重复
            let mut keep = Vec::new();
            for record in records.drain(..) {
                let is_dup = keep.iter().any(|r: &MemoryRecord| {
                    cosine_sim(&r.embedding, &record.embedding) > 0.95
                });
                if is_dup {
                    merged += 1;
                } else {
                    keep.push(record);
                }
            }
            let before = keep.len();
            // 衰减
            //
            // ⚠️ 此行曾编译不过（E0502）：`records` 是 `self.records.values_mut()`
            //    借出的 `&mut`，闭包里再调 `self.decay_score(..)` 又要 `&self`
            //    ⇒ 同一 `self` 同时可变与不可变借用。
            //
            // ✅ `decay_score` 实际只读 `self.decay_config`（我核对过它的函数体：
            //    唯一用到 self 的地方是 `self.decay_config.half_life_ms`），
            //    而 `decay_config` 不参与 `self.records` 的借用
            //    ⇒ 在循环外先克隆一份，彻底切断闭包对 `self` 的依赖。
            //    这比「改成 &self 形参」更小改动，且不改动 decay_score 的签名
            //    （它可能还有别的调用方）。
            let decay_cfg = self.decay_config.clone();
            keep.retain(|r| Self::decay_score_with(r, &decay_cfg) >= decay_cfg.min_retention);
            removed += before - keep.len();
            *records = keep;
        }

        self.stats.total_consolidations += 1;
        ConsolidationReport { removed, merged }
    }

    /// 计算衰减分数
    fn decay_score(&self, record: &MemoryRecord) -> f64 {
        Self::decay_score_with(record, &self.decay_config)
    }

    /// 衰减分数的**纯函数**形式 —— 只依赖 `DecayConfig`，不依赖 `self`。
    ///
    /// 存在的理由：`consolidate()` 在 `self.records.values_mut()` 的借用期内
    /// 需要调用它，而那个借用已是 `&mut self` ⇒ 再调 `&self` 方法必然 E0502。
    /// 抽成关联函数后，闭包只需捕获 `DecayConfig`（`Copy` 语义的小结构）。
    fn decay_score_with(record: &MemoryRecord, cfg: &DecayConfig) -> f64 {
        let age_ms = now_ms().saturating_sub(record.timestamp);
        let half_lives = age_ms as f64 / cfg.half_life_ms as f64;
        let base_decay = 0.5_f64.powf(half_lives);
        let access_boost = (record.access_count as f64).log2().max(0.0) * 0.1;
        (base_decay + access_boost).min(1.0)
    }
}

#[derive(Debug, Clone)]
pub struct ConsolidationReport {
    pub removed: usize,
    pub merged: usize,
}

// ═══════════════════════════════════════════════════════════════
// 工具函数
// ═══════════════════════════════════════════════════════════════

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

fn cosine_sim(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() { return 0.0; }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let nb: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na * nb) }
}

fn extract_entities(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|w| w.chars().next().is_some_and(|c| c.is_uppercase()) && w.len() > 2)
        .map(|w| w.to_string())
        .collect()
}

fn compute_bm25_score(query: &str, content: &str) -> f64 {
    let query_words: Vec<&str> = query.split_whitespace().collect();
    let content_words: Vec<&str> = content.split_whitespace().collect();
    if content_words.is_empty() { return 0.0; }
    let mut score = 0.0;
    for qw in &query_words {
        let count = content_words.iter().filter(|cw| cw.to_lowercase() == qw.to_lowercase()).count();
        if count > 0 {
            score += 1.0 + (count as f64).ln();
        }
    }
    score / query_words.len().max(1) as f64
}

fn compute_entity_score(query: &str, entities: &[String]) -> f64 {
    if entities.is_empty() { return 0.0; }
    let matches = entities.iter().filter(|e| query.to_lowercase().contains(&e.to_lowercase())).count();
    matches as f64 / entities.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve() {
        let mut orch = MemoryOrchestrator::new();
        let id = orch.store("Alice learned Rust", vec![0.1, 0.2, 0.3], MemoryNamespace::Experience);
        assert!(!id.0.is_empty());
        let results = orch.retrieve("Alice", &[0.1, 0.2, 0.3], 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_consolidation() {
        let mut orch = MemoryOrchestrator::new();
        orch.store("test1", vec![0.1, 0.2], MemoryNamespace::Knowledge);
        orch.store("test1", vec![0.1, 0.20001], MemoryNamespace::Knowledge); // near-duplicate
        let report = orch.consolidate();
        assert!(report.merged > 0);
    }

    #[test]
    fn test_decay() {
        let mut orch = MemoryOrchestrator::new();
        orch.store("old memory", vec![0.5, 0.5], MemoryNamespace::Experience);
        // Simulate old record
        if let Some(rec) = orch.records.values_mut().next().and_then(|v| v.first_mut()) {
            rec.timestamp = now_ms() - 86_400_000 * 30; // 30 days old
        }
        let score = orch.decay_score(orch.records.values().next().unwrap().first().unwrap());
        assert!(score < 0.5);
    }
}
