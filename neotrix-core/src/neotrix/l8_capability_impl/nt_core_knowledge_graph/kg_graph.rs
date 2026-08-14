//! nt_core_knowledge_graph — 知识图图结构
//!
//! 基于图的知识表示和图遍历算法
//! 节点: nt_core_knowledge_graph (L8)
//! Provides: graph_traversal, node_query, edge_update
//! Requires: nt_memory_kb, serde
//! Rune: Crimson, Alabaster

#![forbid(unsafe_code)]

use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KGConfig {
    /// 是否启用有向边
    pub directed: bool,
    /// 最大度数限制
    pub max_degree: usize,
    /// 是否自动清理孤立节点
    pub auto_prune: bool,
}

impl Default for KGConfig {
    fn default() -> Self {
        Self {
            directed: true,
            max_degree: 1000,
            auto_prune: true,
        }
    }
}

/// 知识图节点
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KGNode {
    pub id: String,
    pub label: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// 知识图边
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KGEdge {
    pub from: String,
    pub to: String,
    pub label: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// 邻居查询结果 (focused edge view)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KGFocusEdge {
    pub to: String,
    pub label: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// 知识图
pub struct KnowledgeGraph {
    config: KGConfig,
    nodes: HashMap<String, KGNode>,
    edges: HashMap<String, KGEdge>,
    /// 预留: 图级统计元数据, 待检索/嵌入观测需要时填充
    _metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl KnowledgeGraph {
    pub fn new(config: KGConfig) -> Self {
        Self {
            config,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            _metadata: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: KGNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn add_edge(&mut self, edge: KGEdge) {
        self.edges.insert(edge.from.clone() + "->" + &edge.to, edge);
    }

    pub fn neighbors(&self, node_id: &str) -> Vec<KGFocusEdge> {
        let mut result = Vec::new();
        for edge in self.edges.values() {
            if edge.from == node_id {
                result.push(KGFocusEdge {
                    to: edge.to.clone(),
                    label: edge.label.clone(),
                    properties: edge.properties.clone(),
                });
            }
            if !self.config.directed && edge.to == node_id {
                result.push(KGFocusEdge {
                    to: edge.from.clone(),
                    label: edge.label.clone(),
                    properties: edge.properties.clone(),
                });
            }
        }
        result
    }

    pub fn find_path(&self, start: &str, end: &str) -> Option<Vec<String>> {
        // 简化的 BFS 路径查找
        let mut visited = HashSet::new();
        let mut queue: Vec<(String, Vec<String>)> = vec![(start.into(), vec![start.into()])];

        while !queue.is_empty() {
            let (current, path) = queue.remove(0);
            if current == end {
                return Some(path.clone());
            }
            if !visited.insert(current.clone()) {
                continue;
            }

            for neighbor in self.neighbors(&current) {
                if !visited.contains(&neighbor.to) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor.to.clone());
                    queue.push((neighbor.to, new_path));
                }
            }
        }
        None
    }

    pub fn config(&self) -> &KGConfig {
        &self.config
    }
}

impl CapabilityNode for KnowledgeGraph {
    fn node_id(&self) -> &str {
        "nt_core_knowledge_graph"
    }
    fn provides(&self) -> Vec<String> {
        vec![
            "graph_traversal".into(),
            "node_query".into(),
            "edge_update".into(),
        ]
    }
    fn requires(&self) -> Vec<String> {
        vec!["nt_memory_kb".into(), "serde".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Crimson, RuneSocket::Alabaster]
    }
    fn constellation_level(&self) -> u8 {
        1
    }
    fn promote_constellation(&mut self) -> bool {
        true
    }
}

impl SelfTest for KnowledgeGraph {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let mut kg = KnowledgeGraph::new(KGConfig::default());

            let node = KGNode {
                id: "n1".into(),
                label: "概念".into(),
                properties: HashMap::new(),
            };
            kg.add_node(node);

            let edge = KGEdge {
                from: "n1".into(),
                to: "n2".into(),
                label: "relates".into(),
                properties: HashMap::new(),
            };
            kg.add_edge(edge);

            let neighbors = kg.neighbors("n1");
            assert!(!neighbors.is_empty());

            let path = kg.find_path("n1", "n2");
            assert!(path.is_some());

            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_core_knowledge_graph"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_knowledge_graph_self_test() {
        let kg = KnowledgeGraph::new(KGConfig::default());
        assert!(kg.self_test().is_ok());
    }
}
