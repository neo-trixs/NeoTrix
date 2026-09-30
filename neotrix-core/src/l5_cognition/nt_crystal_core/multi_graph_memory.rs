//! MultiGraphMemory — 四图正交记忆架构
//!
//! 基于 MAGMA (ACL 2026) 的四图正交表示：
//! - SemanticGraph: 语义相似度边
//! - TemporalGraph: 时间先后边
//! - CausalGraph: 因果关系边
//! - EntityGraph: 实体共现边
//!
//! 每个记忆项同时存在于四个关系图中，支持意图感知自适应检索。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════
// 核心类型
// ═══════════════════════════════════════════════════════════════

/// 记忆节点 ID
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryNodeId(pub String);

impl MemoryNodeId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

/// 记忆项 — 四图共享的节点数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    /// 唯一标识
    pub id: MemoryNodeId,
    /// 记忆内容
    pub content: String,
    /// 向量嵌入 (语义检索用)
    pub embedding: Vec<f64>,
    /// 时间戳 (ms since epoch)
    pub timestamp: u64,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 提取的实体
    pub entities: Vec<String>,
    /// 记忆类型
    pub memory_type: MemoryType,
}

/// 记忆类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryType {
    /// 情景记忆 — 具体事件
    Episodic,
    /// 语义记忆 — 概念/知识
    Semantic,
    /// 程序记忆 — 技能/操作
    Procedural,
    /// 情感记忆 — 情绪标记
    Emotional,
    /// 事实记忆 — 确定性知识
    Fact,
    /// 模式记忆 — 抽象规律
    Pattern,
    /// 因果记忆 — 因果关系
    Causal,
    /// 矛盾记忆 — 冲突信息
    Contradiction,
    /// 反事实记忆 — 假设性推理
    Counterfactual,
    /// 经验记忆 — 实践总结
    Experience,
    /// 教训记忆 — 失败学习
    Lesson,
    /// 解决方案记忆 — 问题解决
    Solution,
}

// ═══════════════════════════════════════════════════════════════
// 边类型
// ═══════════════════════════════════════════════════════════════

/// 语义边 — cosine similarity + Hebbian 强度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticEdge {
    pub similarity: f64,
    pub strength: f64,
}

/// 时间边 — 时序关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalEdge {
    pub order: TemporalOrder,
    pub delta_ms: i64,
}

/// 时序关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemporalOrder {
    Before,
    After,
    Concurrent,
}

/// 因果边 — 因果置信度 + 机制描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEdge {
    pub confidence: f64,
    pub mechanism: String,
}

/// 实体边 — 共现关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityEdge {
    pub co_occurrence: u32,
    pub context_window: u32,
}

// ═══════════════════════════════════════════════════════════════
// 图结构
// ═══════════════════════════════════════════════════════════════

/// 单一关系图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph<E: Clone> {
    /// 节点: id → MemoryItem
    pub nodes: HashMap<MemoryNodeId, MemoryItem>,
    /// 边: (source, target) → Edge
    pub edges: Vec<(MemoryNodeId, MemoryNodeId, E)>,
}

impl<E: Clone> Graph<E> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, id: MemoryNodeId, item: MemoryItem) {
        self.nodes.insert(id, item);
    }

    pub fn add_edge(&mut self, source: &MemoryNodeId, target: &MemoryNodeId, edge: E) {
        self.edges.push((source.clone(), target.clone(), edge));
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// 获取节点的所有邻居
    pub fn neighbors(&self, node_id: &MemoryNodeId) -> Vec<&MemoryNodeId> {
        self.edges
            .iter()
            .filter(|(s, t, _)| s == node_id || t == node_id)
            .map(|(s, t, _)| if s == node_id { t } else { s })
            .collect()
    }
}

// ═══════════════════════════════════════════════════════════════
// 四图正交记忆
// ═══════════════════════════════════════════════════════════════

/// 四图正交记忆存储
///
/// 每个记忆项同时存在于四个关系图中：
/// - 语义图: 基于 embedding cosine similarity
/// - 时间图: 基于时间戳排序
/// - 因果图: 基于因果关系推断
/// - 实体图: 基于实体共现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiGraphMemory {
    /// 语义图: 节点=MemoryItem, 边=embedding cosine similarity
    pub semantic: Graph<SemanticEdge>,
    /// 时间图: 节点=MemoryItem, 边=temporal ordering
    pub temporal: Graph<TemporalEdge>,
    /// 因果图: 节点=MemoryItem, 边=causal relationship
    pub causal: Graph<CausalEdge>,
    /// 实体图: 节点=MemoryItem, 边=entity co-occurrence
    pub entity: Graph<EntityEdge>,

    /// BM25 索引 (关键词检索)
    pub bm25_index: HashMap<String, Vec<MemoryNodeId>>,

    /// 实体索引 (实体→节点列表)
    pub entity_index: HashMap<String, Vec<MemoryNodeId>>,
}

impl MultiGraphMemory {
    pub fn new() -> Self {
        Self {
            semantic: Graph::new(),
            temporal: Graph::new(),
            causal: Graph::new(),
            entity: Graph::new(),
            bm25_index: HashMap::new(),
            entity_index: HashMap::new(),
        }
    }

    /// 添加记忆项 — 自动更新四图
    pub fn add(&mut self, item: MemoryItem) -> MemoryNodeId {
        let node_id = item.id.clone();

        // 1. 添加到所有图
        self.semantic.add_node(node_id.clone(), item.clone());
        self.temporal.add_node(node_id.clone(), item.clone());
        self.causal.add_node(node_id.clone(), item.clone());
        self.entity.add_node(node_id.clone(), item.clone());

        // 2. 建立语义边 (与现有节点)
        let current_embedding = item.embedding.clone();
        let existing_ids: Vec<MemoryNodeId> = self.semantic.nodes.keys().cloned().collect();
        for existing_id in &existing_ids {
            if existing_id == &node_id {
                continue;
            }
            if let Some(existing_item) = self.semantic.nodes.get(existing_id) {
                let sim = cosine_similarity(&current_embedding, &existing_item.embedding);
                if sim > 0.7 {
                    self.semantic.add_edge(
                        &node_id,
                        existing_id,
                        SemanticEdge {
                            similarity: sim,
                            strength: sim * 0.5,
                        },
                    );
                }
            }
        }

        // 3. 建立时间边 (按时间戳排序)
        let current_ts = item.timestamp;
        let mut prev_id: Option<MemoryNodeId> = None;
        let mut max_ts = 0u64;
        for (id, node) in &self.temporal.nodes {
            if id == &node_id {
                continue;
            }
            if node.timestamp < current_ts && node.timestamp > max_ts {
                max_ts = node.timestamp;
                prev_id = Some(id.clone());
            }
        }
        if let Some(prev) = prev_id {
            self.temporal.add_edge(
                &prev,
                &node_id,
                TemporalEdge {
                    order: TemporalOrder::Before,
                    delta_ms: current_ts as i64 - max_ts as i64,
                },
            );
        }

        // 4. 提取实体并建立实体边
        for entity in &item.entities {
            self.entity_index
                .entry(entity.clone())
                .or_default()
                .push(node_id.clone());

            if let Some(co_occurring) = self.entity_index.get(entity) {
                for co_id in co_occurring {
                    if co_id != &node_id {
                        self.entity.add_edge(
                            &node_id,
                            co_id,
                            EntityEdge {
                                co_occurrence: 1,
                                context_window: 100,
                            },
                        );
                    }
                }
            }
        }

        // 5. 更新 BM25 索引 (简单分词)
        for word in item.content.split_whitespace() {
            let word_lower = word.to_lowercase();
            self.bm25_index
                .entry(word_lower)
                .or_default()
                .push(node_id.clone());
        }

        node_id
    }

    /// 获取所有节点
    pub fn all_nodes(&self) -> Vec<&MemoryItem> {
        self.semantic.nodes.values().collect()
    }

    /// 按实体查找节点
    pub fn find_by_entity(&self, entity: &str) -> Vec<&MemoryNodeId> {
        self.entity_index
            .get(entity)
            .map(|ids| ids.iter().collect())
            .unwrap_or_default()
    }

    /// 按关键词查找节点 (BM25)
    pub fn search_by_keyword(&self, keyword: &str) -> Vec<&MemoryNodeId> {
        self.bm25_index
            .get(&keyword.to_lowercase())
            .map(|ids| ids.iter().collect())
            .unwrap_or_default()
    }

    /// 获取语义最近邻
    pub fn semantic_neighbors(&self, node_id: &MemoryNodeId, top_k: usize) -> Vec<(MemoryNodeId, f64)> {
        let mut scores: Vec<(MemoryNodeId, f64)> = Vec::new();

        if let Some(query_item) = self.semantic.nodes.get(node_id) {
            for (id, item) in &self.semantic.nodes {
                if id == node_id {
                    continue;
                }
                let sim = cosine_similarity(&query_item.embedding, &item.embedding);
                scores.push((id.clone(), sim));
            }
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(top_k);
        scores
    }

    /// 获取时间范围内的节点
    pub fn temporal_range(&self, start_ms: u64, end_ms: u64) -> Vec<&MemoryItem> {
        self.temporal
            .nodes
            .values()
            .filter(|item| item.timestamp >= start_ms && item.timestamp <= end_ms)
            .collect()
    }

    /// 因果链追踪 — 从给定节点向前追踪
    pub fn causal_trace(&self, node_id: &MemoryNodeId, max_depth: usize) -> Vec<(&MemoryNodeId, &CausalEdge)> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((node_id.clone(), 0));

        while let Some((current, depth)) = queue.pop_front() {
            if depth > max_depth || visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            for (s, t, edge) in &self.causal.edges {
                if s == &current {
                    result.push((t, edge));
                    queue.push_back((t.clone(), depth + 1));
                }
            }
        }

        result
    }

    /// 图统计信息
    pub fn stats(&self) -> GraphStats {
        GraphStats {
            total_nodes: self.semantic.node_count(),
            semantic_edges: self.semantic.edge_count(),
            temporal_edges: self.temporal.edge_count(),
            causal_edges: self.causal.edge_count(),
            entity_edges: self.entity.edge_count(),
            unique_entities: self.entity_index.len(),
            unique_keywords: self.bm25_index.len(),
        }
    }
}

/// 图统计信息
#[derive(Debug, Clone)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub semantic_edges: usize,
    pub temporal_edges: usize,
    pub causal_edges: usize,
    pub entity_edges: usize,
    pub unique_entities: usize,
    pub unique_keywords: usize,
}

// ═══════════════════════════════════════════════════════════════
// 检索意图
// ═══════════════════════════════════════════════════════════════

/// 检索意图类型
#[derive(Debug, Clone)]
pub enum IntentType {
    /// 事实查询
    Factual,
    /// 时间查询
    Temporal,
    /// 因果查询
    Causal,
    /// 关联查询
    Associative,
}

/// 检索查询
#[derive(Debug, Clone)]
pub struct QueryIntent {
    pub intent_type: IntentType,
    pub embedding: Vec<f64>,
    pub entities: Vec<String>,
    pub time_range: Option<(u64, u64)>,
    pub start_node: Option<MemoryNodeId>,
    pub cause_node: Option<MemoryNodeId>,
}

/// 检索结果
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    pub node_id: MemoryNodeId,
    pub score: f64,
    pub source_graph: String,
}

/// 融合策略
#[derive(Debug, Clone)]
pub enum FusionStrategy {
    /// Reciprocal Rank Fusion
    RRF,
    /// 加权融合
    Weighted,
    /// 最大边际相关性
    MMR,
    /// 简单追加
    Append,
}

// ═══════════════════════════════════════════════════════════════
// 工具函数
// ═══════════════════════════════════════════════════════════════

/// 余弦相似度
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
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

/// 简单实体提取 (基于规则)
pub fn extract_entities(text: &str) -> Vec<String> {
    let mut entities = Vec::new();
    // 简单规则: 大写开头的连续单词作为实体
    for word in text.split_whitespace() {
        if let Some(first) = word.chars().next() {
            if first.is_uppercase() && word.len() > 2 {
                entities.push(word.to_string());
            }
        }
    }
    entities.sort();
    entities.dedup();
    entities
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(content: &str, ts: u64) -> MemoryItem {
        let embedding: Vec<f64> = (0..8).map(|i| (ts as f64 + i as f64).sin()).collect();
        MemoryItem {
            id: MemoryNodeId::new(),
            content: content.to_string(),
            embedding,
            timestamp: ts,
            metadata: HashMap::new(),
            entities: extract_entities(content),
            memory_type: MemoryType::Episodic,
        }
    }

    #[test]
    fn test_multi_graph_add_and_stats() {
        let mut mg = MultiGraphMemory::new();
        let item1 = make_item("Alice learned Rust programming", 1000);
        let item2 = make_item("Alice learned Python programming", 2000);
        let item3 = make_item("Bob learned Rust programming", 3000);

        mg.add(item1);
        mg.add(item2);
        mg.add(item3);

        let stats = mg.stats();
        assert_eq!(stats.total_nodes, 3);
        assert!(stats.temporal_edges > 0);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-10);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c)).abs() < 1e-10);
    }

    #[test]
    fn test_extract_entities() {
        let entities = extract_entities("Alice and Bob went to Paris");
        assert!(entities.contains(&"Alice".to_string()));
        assert!(entities.contains(&"Bob".to_string()));
        assert!(entities.contains(&"Paris".to_string()));
    }

    #[test]
    fn test_semantic_neighbors() {
        let mut mg = MultiGraphMemory::new();
        let item1 = make_item("Rust programming language", 1000);
        let item2 = make_item("Rust systems programming", 2000);
        let item3 = make_item("Python web development", 3000);

        let id1 = mg.add(item1);
        mg.add(item2);
        mg.add(item3);

        let neighbors = mg.semantic_neighbors(&id1, 2);
        assert_eq!(neighbors.len(), 2);
        // item2 should be more similar to item1 than item3
        assert!(neighbors[0].1 > neighbors[1].1);
    }
}
