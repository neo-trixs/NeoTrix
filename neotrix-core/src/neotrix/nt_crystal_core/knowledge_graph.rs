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
        Self { nodes: HashMap::new(), edges: Vec::new(), entity_index: HashMap::new() }
    }

    pub fn add_node(&mut self, label: &str, embedding: Vec<f64>, node_type: &str) -> NodeId {
        let id = NodeId(uuid::Uuid::new_v4().to_string());
        let entities = extract_entities(label);
        let node = GraphNode {
            id: id.clone(), label: label.to_string(), embedding, node_type: node_type.to_string(),
            created_at: now_ms(), metadata: HashMap::new(),
        };
        self.nodes.insert(id.clone(), node);
        for e in &entities {
            self.entity_index.entry(e.clone()).or_default().push(id.clone());
        }
        id
    }

    pub fn add_edge(&mut self, source: &NodeId, target: &NodeId, edge_type: EdgeType, weight: f64) -> EdgeId {
        let id = EdgeId(uuid::Uuid::new_v4().to_string());
        self.edges.push(GraphEdge {
            id: id.clone(), source: source.clone(), target: target.clone(),
            edge_type, weight, valid_from: now_ms(), valid_to: None, metadata: HashMap::new(),
        });
        id
    }

    pub fn query_by_type(&self, edge_type: &EdgeType) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| std::mem::discriminant(&e.edge_type) == std::mem::discriminant(edge_type)).collect()
    }

    pub fn temporal_query(&self, time: u64) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.valid_from <= time && e.valid_to.map_or(true, |t| t > time)).collect()
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

    pub fn sync_to_hypercube(&self) -> Vec<(NodeId, Vec<f64>)> {
        self.nodes.iter().map(|(id, n)| (id.clone(), n.embedding.clone())).collect()
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        (self.nodes.len(), self.edges.len(), self.entity_index.len())
    }
}

fn extract_entities(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|w| w.chars().next().map_or(false, |c| c.is_uppercase()) && w.len() > 2)
        .map(|w| w.to_string()).collect()
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
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
}
