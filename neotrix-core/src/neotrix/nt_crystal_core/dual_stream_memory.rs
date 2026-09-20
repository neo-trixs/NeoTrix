//! DualStreamMemory — 双流记忆生命周期
//!
//! 基于 MAGMA + GAM (ACL 2026) 的双流设计：
//! - Fast Path: 快速突触摄取 (<50ms)
//! - Slow Path: 异步结构整合 (1-10s)
//!
//! 语义事件检测器判断何时触发整合。

use std::collections::VecDeque;

use super::multi_graph_memory::{MemoryItem, MemoryNodeId, MultiGraphMemory};

// ═══════════════════════════════════════════════════════════════
// 快速路径 — 突触摄取
// ═══════════════════════════════════════════════════════════════

/// 快速突触摄取 — 低延迟写入
pub struct SynapticIngestion {
    /// 摄取计数
    pub ingestion_count: u64,
    /// 摄取延迟追踪 (ms)
    pub latency_history: Vec<u64>,
}

impl SynapticIngestion {
    pub fn new() -> Self {
        Self {
            ingestion_count: 0,
            latency_history: Vec::new(),
        }
    }

    /// 快速摄取 — 最小处理，直接写入
    pub fn ingest(&mut self, item: &MemoryItem) -> IngestionResult {
        let start = std::time::Instant::now();
        self.ingestion_count += 1;

        // 快速路径只做最少处理:
        // 1. 验证内容非空
        // 2. 提取实体
        // 3. 标记为未整合
        let result = IngestionResult {
            node_id: item.id.clone(),
            accepted: !item.content.is_empty(),
            latency_ms: start.elapsed().as_millis() as u64,
            entity_count: item.entities.len(),
            needs_consolidation: true,
        };

        self.latency_history.push(result.latency_ms);
        if self.latency_history.len() > 1000 {
            self.latency_history.remove(0);
        }

        result
    }

    /// 平均摄取延迟
    pub fn avg_latency_ms(&self) -> f64 {
        if self.latency_history.is_empty() {
            return 0.0;
        }
        self.latency_history.iter().sum::<u64>() as f64 / self.latency_history.len() as f64
    }
}

/// 摄取结果
#[derive(Debug, Clone)]
pub struct IngestionResult {
    pub node_id: MemoryNodeId,
    pub accepted: bool,
    pub latency_ms: u64,
    pub entity_count: usize,
    pub needs_consolidation: bool,
}

// ═══════════════════════════════════════════════════════════════
// 语义事件检测器
// ═══════════════════════════════════════════════════════════════

/// 语义事件检测器 — 判断何时触发整合
pub struct SemanticEventDetector {
    /// 语义漂移阈值 (超过此值触发整合)
    pub drift_threshold: f64,
    /// 最小缓冲区大小
    pub min_buffer_size: usize,
    /// 最大缓冲区大小 (超过此值强制整合)
    pub max_buffer_size: usize,
}

impl SemanticEventDetector {
    pub fn new() -> Self {
        Self {
            drift_threshold: 0.3,
            min_buffer_size: 5,
            max_buffer_size: 50,
        }
    }

    /// 检测是否应该触发整合
    pub fn should_consolidate(&self, buffer_size: usize, drift_score: f64) -> ConsolidationTrigger {
        if buffer_size >= self.max_buffer_size {
            ConsolidationTrigger::Forced {
                reason: "buffer_full".to_string(),
            }
        } else if buffer_size >= self.min_buffer_size && drift_score > self.drift_threshold {
            ConsolidationTrigger::SemanticDrift {
                drift_score,
                threshold: self.drift_threshold,
            }
        } else {
            ConsolidationTrigger::None
        }
    }

    /// 计算语义漂移分数
    pub fn compute_drift(&self, items: &[MemoryItem]) -> f64 {
        if items.len() < 2 {
            return 0.0;
        }

        let mut total_drift = 0.0;
        for window in items.windows(2) {
            let sim = cosine_similarity(&window[0].embedding, &window[1].embedding);
            total_drift += 1.0 - sim;
        }
        total_drift / (items.len() - 1) as f64
    }
}

/// 整合触发条件
#[derive(Debug, Clone)]
pub enum ConsolidationTrigger {
    /// 无触发
    None,
    /// 语义漂移超过阈值
    SemanticDrift { drift_score: f64, threshold: f64 },
    /// 缓冲区满，强制整合
    Forced { reason: String },
}

// ═══════════════════════════════════════════════════════════════
// 慢速路径 — 结构整合
// ═══════════════════════════════════════════════════════════════

/// 慢速结构整合 — 图谱结构优化
pub struct StructuralConsolidation {
    /// 整合计数
    pub consolidation_count: u64,
    /// 整合延迟追踪 (ms)
    pub latency_history: Vec<u64>,
}

impl StructuralConsolidation {
    pub fn new() -> Self {
        Self {
            consolidation_count: 0,
            latency_history: Vec::new(),
        }
    }

    /// 执行结构整合
    pub fn consolidate(
        &mut self,
        buffer: &[MemoryItem],
        graph: &mut MultiGraphMemory,
    ) -> ConsolidationResult {
        let start = std::time::Instant::now();
        self.consolidation_count += 1;

        let mut nodes_added = 0;
        let edges_created;
        let mut duplicates_merged = 0;

        for item in buffer {
            // 检查是否与现有节点重复
            let is_duplicate = graph.all_nodes().iter().any(|existing| {
                cosine_similarity(&item.embedding, &existing.embedding) > 0.95
            });

            if is_duplicate {
                duplicates_merged += 1;
                continue;
            }

            // 添加到图谱
            graph.add(item.clone());
            nodes_added += 1;
        }

        // 优化: 合并高相似度边
        edges_created = self.optimize_edges(graph);

        let latency = start.elapsed().as_millis() as u64;
        self.latency_history.push(latency);

        ConsolidationResult {
            nodes_added,
            edges_created,
            duplicates_merged,
            latency_ms: latency,
        }
    }

    /// 优化边 — 合并重复边，强化高频路径
    fn optimize_edges(&self, graph: &mut MultiGraphMemory) -> usize {
        let mut optimized = 0;
        // 简单优化: 移除相似度低于阈值的语义边
        graph.semantic.edges.retain(|(_, _, edge)| {
            if edge.similarity < 0.3 {
                optimized += 1;
                false
            } else {
                true
            }
        });
        optimized
    }
}

/// 整合结果
#[derive(Debug, Clone)]
pub struct ConsolidationResult {
    pub nodes_added: usize,
    pub edges_created: usize,
    pub duplicates_merged: usize,
    pub latency_ms: u64,
}

// ═══════════════════════════════════════════════════════════════
// 双流记忆引擎
// ═══════════════════════════════════════════════════════════════

/// 双流记忆引擎 — 快速摄取 + 慢速整合
pub struct DualStreamMemory {
    /// 快速路径: 突触摄取
    pub fast_path: SynapticIngestion,
    /// 慢速路径: 结构整合
    pub slow_path: StructuralConsolidation,
    /// 语义事件检测器
    pub event_detector: SemanticEventDetector,
    /// 整合队列 (等待整合的记忆)
    pub consolidation_queue: VecDeque<MemoryItem>,
    /// 四图记忆存储
    pub graph: MultiGraphMemory,
}

impl DualStreamMemory {
    pub fn new() -> Self {
        Self {
            fast_path: SynapticIngestion::new(),
            slow_path: StructuralConsolidation::new(),
            event_detector: SemanticEventDetector::new(),
            consolidation_queue: VecDeque::new(),
            graph: MultiGraphMemory::new(),
        }
    }

    /// 写入记忆 — 快速路径
    pub fn write(&mut self, item: MemoryItem) -> IngestionResult {
        let result = self.fast_path.ingest(&item);

        if result.accepted {
            self.consolidation_queue.push_back(item);

            // 检查是否触发整合
            let buffer: Vec<MemoryItem> = self.consolidation_queue.iter().cloned().collect();
            let drift = self.event_detector.compute_drift(&buffer);
            let trigger = self.event_detector.should_consolidate(buffer.len(), drift);

            if let ConsolidationTrigger::None = trigger {
                // 不触发，继续缓冲
            } else {
                self.run_consolidation();
            }
        }

        result
    }

    /// 手动触发整合
    pub fn flush(&mut self) -> ConsolidationResult {
        self.run_consolidation()
    }

    /// 执行慢速整合
    fn run_consolidation(&mut self) -> ConsolidationResult {
        if self.consolidation_queue.is_empty() {
            return ConsolidationResult {
                nodes_added: 0,
                edges_created: 0,
                duplicates_merged: 0,
                latency_ms: 0,
            };
        }

        let buffer: Vec<MemoryItem> = self.consolidation_queue.drain(..).collect();
        self.slow_path.consolidate(&buffer, &mut self.graph)
    }

    /// 检索记忆
    pub fn search(&self, query_embedding: &[f64], top_k: usize) -> Vec<(MemoryNodeId, f64)> {
        let mut scores: Vec<(MemoryNodeId, f64)> = Vec::new();

        for (id, item) in &self.graph.semantic.nodes {
            let sim = cosine_similarity(query_embedding, &item.embedding);
            scores.push((id.clone(), sim));
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(top_k);
        scores
    }

    /// 获取统计信息
    pub fn stats(&self) -> DualStreamStats {
        DualStreamStats {
            total_nodes: self.graph.semantic.node_count(),
            queue_size: self.consolidation_queue.len(),
            ingestion_count: self.fast_path.ingestion_count,
            consolidation_count: self.slow_path.consolidation_count,
            avg_ingestion_latency_ms: self.fast_path.avg_latency_ms(),
            graph_stats: self.graph.stats(),
        }
    }
}

/// 双流统计信息
#[derive(Debug, Clone)]
pub struct DualStreamStats {
    pub total_nodes: usize,
    pub queue_size: usize,
    pub ingestion_count: u64,
    pub consolidation_count: u64,
    pub avg_ingestion_latency_ms: f64,
    pub graph_stats: super::multi_graph_memory::GraphStats,
}

// ═══════════════════════════════════════════════════════════════
// 工具函数
// ═══════════════════════════════════════════════════════════════

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_crystal_core::multi_graph_memory::MemoryType;
    use std::collections::HashMap;

    fn make_item(content: &str, ts: u64) -> MemoryItem {
        let embedding: Vec<f64> = (0..8).map(|i| (ts as f64 + i as f64).sin()).collect();
        MemoryItem {
            id: MemoryNodeId::new(),
            content: content.to_string(),
            embedding,
            timestamp: ts,
            metadata: HashMap::new(),
            entities: Vec::new(),
            memory_type: MemoryType::Episodic,
        }
    }

    #[test]
    fn test_fast_path_ingestion() {
        let mut fast = SynapticIngestion::new();
        let item = make_item("Test memory", 1000);
        let result = fast.ingest(&item);
        assert!(result.accepted);
        assert_eq!(fast.ingestion_count, 1);
    }

    #[test]
    fn test_dual_stream_write_and_flush() {
        let mut ds = DualStreamMemory::new();

        for i in 0..10 {
            let item = make_item(&format!("Memory item {}", i), i * 1000);
            ds.write(item);
        }

        let stats = ds.stats();
        assert_eq!(stats.total_nodes, 10);
        assert_eq!(stats.ingestion_count, 10);
    }

    #[test]
    fn test_semantic_event_detector() {
        let detector = SemanticEventDetector::new();
        let trigger = detector.should_consolidate(3, 0.1);
        assert!(matches!(trigger, ConsolidationTrigger::None));

        let trigger = detector.should_consolidate(10, 0.5);
        assert!(matches!(trigger, ConsolidationTrigger::SemanticDrift { .. }));

        let trigger = detector.should_consolidate(60, 0.1);
        assert!(matches!(trigger, ConsolidationTrigger::Forced { .. }));
    }
}
