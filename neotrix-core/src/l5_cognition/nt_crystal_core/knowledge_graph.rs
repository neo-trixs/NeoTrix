//! KnowledgeGraphManager — 知识图谱管理
//!
//! 基于 Graphiti + Semantica + Mem0 Entity Linking。
//! - 时序边 (valid_from/valid_to)
//! - 四种正交边类型
//! - 实体链接
//! - HyperCube 双向同步

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeId(pub String);

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct EdgeId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: NodeId,
    pub label: String,
    pub embedding: Vec<f64>,
    pub node_type: String,
    pub created_at: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    Semantic,
    Temporal,
    Causal,
    Entity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub edge_type: EdgeType,
    pub weight: f64,
    pub valid_from: u64,
    pub valid_to: Option<u64>,
    pub metadata: HashMap<String, String>,
}

pub struct KnowledgeGraphManager {
    pub nodes: HashMap<NodeId, GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub entity_index: HashMap<String, Vec<NodeId>>,
}

impl KnowledgeGraphManager {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            entity_index: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, label: &str, embedding: Vec<f64>, node_type: &str) -> NodeId {
        let id = NodeId(uuid::Uuid::new_v4().to_string());
        let entities = extract_entities(label);
        let node = GraphNode {
            id: id.clone(),
            label: label.to_string(),
            embedding,
            node_type: node_type.to_string(),
            created_at: now_ms(),
            metadata: HashMap::new(),
        };
        self.nodes.insert(id.clone(), node);
        for e in &entities {
            self.entity_index
                .entry(e.clone())
                .or_default()
                .push(id.clone());
        }
        id
    }

    pub fn add_edge(
        &mut self,
        source: &NodeId,
        target: &NodeId,
        edge_type: EdgeType,
        weight: f64,
    ) -> EdgeId {
        let id = EdgeId(uuid::Uuid::new_v4().to_string());
        self.edges.push(GraphEdge {
            id: id.clone(),
            source: source.clone(),
            target: target.clone(),
            edge_type,
            weight,
            valid_from: now_ms(),
            valid_to: None,
            metadata: HashMap::new(),
        });
        id
    }

    pub fn query_by_type(&self, edge_type: &EdgeType) -> Vec<&GraphEdge> {
        self.edges
            .iter()
            .filter(|e| std::mem::discriminant(&e.edge_type) == std::mem::discriminant(edge_type))
            .collect()
    }

    pub fn temporal_query(&self, time: u64) -> Vec<&GraphEdge> {
        self.edges
            .iter()
            .filter(|e| e.valid_from <= time && e.valid_to.map_or(true, |t| t > time))
            .collect()
    }

    pub fn entity_link(&self, text: &str) -> Vec<&NodeId> {
        let entities = extract_entities(text);
        let mut results = Vec::new();
        for e in &entities {
            if let Some(ids) = self.entity_index.get(e) {
                results.extend(ids.iter());
            }
        }
        results
    }

    /// 精确标签查找（实体索引拉丁偏置时的兜底；中文节点主入口）。
    pub fn find_by_label(&self, label: &str) -> Option<&GraphNode> {
        self.nodes.values().find(|n| n.label == label)
    }

    /// 出边遍历（可选边型过滤）：推理/种子消费者的主入口。
    pub fn neighbors(&self, id: &NodeId, edge_type: Option<&EdgeType>) -> Vec<&GraphNode> {
        let mut out = Vec::new();
        for e in &self.edges {
            if &e.source != id {
                continue;
            }
            if let Some(t) = edge_type {
                if std::mem::discriminant(&e.edge_type) != std::mem::discriminant(t) {
                    continue;
                }
            }
            if let Some(n) = self.nodes.get(&e.target) {
                out.push(n);
            }
        }
        out
    }

    pub fn sync_to_hypercube(&self) -> Vec<(NodeId, Vec<f64>)> {
        self.nodes
            .iter()
            .map(|(id, n)| (id.clone(), n.embedding.clone()))
            .collect()
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        (self.nodes.len(), self.edges.len(), self.entity_index.len())
    }
}

fn is_index_token(c: char) -> bool {
    // 拉丁首字母大写（旧行为）+ CJK 统一表意（旧 extract_entities 漏中文，皇極种子补）。
    c.is_uppercase()
        || ('\u{3400}' <= c && c <= '\u{4DBF}'
            || '\u{4E00}' <= c && c <= '\u{9FFF}'
            || '\u{F900}' <= c && c <= '\u{FAFF}')
}

fn extract_entities(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|w| w.chars().next().map_or(false, is_index_token) && w.len() > 2)
        .map(|w| w.to_string())
        .collect()
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node_and_edge() {
        let mut kg = KnowledgeGraphManager::new();
        let n1 = kg.add_node("Rust Language", vec![0.1, 0.2], "concept");
        let n2 = kg.add_node("Ownership", vec![0.3, 0.4], "concept");
        let eid = kg.add_edge(&n1, &n2, EdgeType::Semantic, 0.8);
        assert!(!eid.0.is_empty());
        let (nodes, edges, _) = kg.stats();
        assert_eq!(nodes, 2);
        assert_eq!(edges, 1);
    }

    #[test]
    fn test_temporal_query() {
        let mut kg = KnowledgeGraphManager::new();
        let n1 = kg.add_node("A", vec![0.1], "t");
        let n2 = kg.add_node("B", vec![0.2], "t");
        kg.add_edge(&n1, &n2, EdgeType::Temporal, 1.0);
        let now = now_ms();
        assert!(!kg.temporal_query(now).is_empty());
    }

    #[test]
    fn test_entity_link() {
        let mut kg = KnowledgeGraphManager::new();
        kg.add_node("Alice learned Rust", vec![0.1], "event");
        let linked = kg.entity_link("Tell me about Alice");
        assert!(!linked.is_empty());
    }

    #[test]
    fn test_find_by_label_exact() {
        let mut kg = KnowledgeGraphManager::new();
        let id = kg.add_node("邵雍", vec![0.1], "person");
        let found = kg.find_by_label("邵雍");
        assert!(found.is_some(), "exact hit");
        assert_eq!(found.map(|n| n.id.clone()), Some(id));
        assert!(kg.find_by_label("邵子").is_none());
    }

    #[test]
    fn test_cjk_entity_indexed() {
        // 旧 extract_entities 只收拉丁大写首字母，中文永不建索引（皇極种子缺口）。
        let mut kg = KnowledgeGraphManager::new();
        kg.add_node("邵雍", vec![0.1], "person");
        let linked = kg.entity_link("邵雍 皇極");
        assert!(!linked.is_empty(), "CJK 首字应建索引");
    }

    #[test]
    fn test_neighbors_filters_type() {
        let mut kg = KnowledgeGraphManager::new();
        let a = kg.add_node("A", vec![0.1], "t");
        let b = kg.add_node("B", vec![0.2], "t");
        let c = kg.add_node("C", vec![0.3], "t");
        kg.add_edge(&a, &b, EdgeType::Temporal, 1.0);
        kg.add_edge(&a, &c, EdgeType::Semantic, 1.0);
        assert_eq!(kg.neighbors(&a, None).len(), 2);
        assert_eq!(kg.neighbors(&a, Some(&EdgeType::Temporal)).len(), 1);
        assert!(kg.neighbors(&b, None).is_empty(), "只走出边");
    }
}
