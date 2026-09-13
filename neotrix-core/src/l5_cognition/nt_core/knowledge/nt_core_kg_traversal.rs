//! Knowledge Graph Traversal — 知识图谱遍历
//!
//! 吸收 KB 经验:
//! - BFS/DFS 遍历
//! - 最短路径 (Dijkstra)
//! - 社区检测
//! - 中心性分析
//! - 子图提取

use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};

/// 知识图谱
pub struct KnowledgeGraph {
    nodes: HashMap<String, _KGNode>,
    edges: HashMap<String, Vec<_KGEdge>>,
    reverse_edges: HashMap<String, Vec<_KGEdge>>,
}

/// 图节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _KGNode {
    pub id: String,
    pub node_type: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub embedding: Option<Vec<f32>>,
}

/// 图边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _KGEdge {
    pub source: String,
    pub target: String,
    pub edge_type: String,
    pub weight: f64,
    pub properties: HashMap<String, serde_json::Value>,
}

/// 遍历结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalResult {
    pub nodes: Vec<_KGNode>,
    pub edges: Vec<_KGEdge>,
    pub paths: Vec<Vec<String>>,
    pub distances: HashMap<String, f64>,
}

/// 社区
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: String,
    pub nodes: Vec<String>,
    pub density: f64,
    pub modularity: f64,
}

/// 中心性指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _CentralityMetrics {
    pub node_id: String,
    pub _degree_centrality: f64,
    pub betweenness_centrality: f64,
    pub closeness_centrality: f64,
    pub eigenvector_centrality: f64,
}

impl KnowledgeGraph {
    /// 创建新的知识图谱
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            reverse_edges: HashMap::new(),
        }
    }

    /// 添加节点
    pub fn add_node(&mut self, node: _KGNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// 添加边
    pub fn add_edge(&mut self, edge: _KGEdge) {
        self.edges.entry(edge.source.clone()).or_insert_with(Vec::new).push(edge.clone());
        self.reverse_edges.entry(edge.target.clone()).or_insert_with(Vec::new).push(edge);
    }

    /// BFS 遍历
    pub fn bfs(&self, start: &str, max_depth: Option<usize>) -> TraversalResult {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut distances = HashMap::new();

        queue.push_back((start.to_string(), 0));
        visited.insert(start.to_string());
        distances.insert(start.to_string(), 0.0);

        while let Some((current, depth)) = queue.pop_front() {
            if let Some(max) = max_depth {
                if depth >= max {
                    continue;
                }
            }

            if let Some(node) = self.nodes.get(&current) {
                nodes.push(node.clone());
            }

            if let Some(adjacent_edges) = self.edges.get(&current) {
                for edge in adjacent_edges {
                    if !visited.contains(&edge.target) {
                        visited.insert(edge.target.clone());
                        queue.push_back((edge.target.clone(), depth + 1));
                        distances.insert(edge.target.clone(), depth as f64 + 1.0);
                        edges.push(edge.clone());
                    }
                }
            }
        }

        TraversalResult {
            nodes,
            edges,
            paths: Vec::new(),
            distances,
        }
    }

    /// DFS 遍历
    pub fn dfs(&self, start: &str, max_depth: Option<usize>) -> TraversalResult {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut distances = HashMap::new();

        stack.push((start.to_string(), 0));
        distances.insert(start.to_string(), 0.0);

        while let Some((current, depth)) = stack.pop() {
            if let Some(max) = max_depth {
                if depth >= max {
                    continue;
                }
            }

            if !visited.contains(&current) {
                visited.insert(current.clone());

                if let Some(node) = self.nodes.get(&current) {
                    nodes.push(node.clone());
                }

                if let Some(adjacent_edges) = self.edges.get(&current) {
                    for edge in adjacent_edges {
                        if !visited.contains(&edge.target) {
                            stack.push((edge.target.clone(), depth + 1));
                            distances.insert(edge.target.clone(), depth as f64 + 1.0);
                            edges.push(edge.clone());
                        }
                    }
                }
            }
        }

        TraversalResult {
            nodes,
            edges,
            paths: Vec::new(),
            distances,
        }
    }

    /// Dijkstra 最短路径
    pub(crate) fn _dijkstra(&self, start: &str, end: &str) -> Option<(Vec<String>, f64)> {
        let mut distances: HashMap<String, f64> = HashMap::new();
        let mut previous: HashMap<String, String> = HashMap::new();
        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();

        distances.insert(start.to_string(), 0.0);
        queue.push_back((0.0, start.to_string()));

        while let Some((dist, current)) = queue.pop_front() {
            if current == end {
                // 重建路径
                let mut path = vec![end.to_string()];
                let mut current = end.to_string();
                while let Some(prev) = previous.get(&current) {
                    path.push(prev.clone());
                    current = prev.clone();
                }
                path.reverse();
                return Some((path, dist));
            }

            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            if let Some(adjacent_edges) = self.edges.get(&current) {
                for edge in adjacent_edges {
                    let new_dist = dist + edge.weight;
                    if new_dist < *distances.get(&edge.target).unwrap_or(&f64::INFINITY) {
                        distances.insert(edge.target.clone(), new_dist);
                        previous.insert(edge.target.clone(), current.clone());
                        queue.push_back((new_dist, edge.target.clone()));
                    }
                }
            }
        }

        None
    }

    /// 计算度中心性
    pub(crate) fn _degree_centrality(&self) -> HashMap<String, f64> {
        let n = self.nodes.len() as f64;
        let mut centrality = HashMap::new();

        for node_id in self.nodes.keys() {
            let out_degree = self.edges.get(node_id).map(|e| e.len()).unwrap_or(0) as f64;
            let in_degree = self.reverse_edges.get(node_id).map(|e| e.len()).unwrap_or(0) as f64;
            centrality.insert(node_id.clone(), (out_degree + in_degree) / (n - 1.0));
        }

        centrality
    }

    /// 社区检测 (Louvain 简化版)
    pub fn detect_communities(&self) -> Vec<Community> {
        // 简化版: 基于连通分量
        let mut visited = HashSet::new();
        let mut communities = Vec::new();

        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                let mut community_nodes = Vec::new();
                let mut queue = VecDeque::new();
                queue.push_back(node_id.clone());
                visited.insert(node_id.clone());

                while let Some(current) = queue.pop_front() {
                    community_nodes.push(current.clone());

                    if let Some(edges) = self.edges.get(&current) {
                        for edge in edges {
                            if !visited.contains(&edge.target) {
                                visited.insert(edge.target.clone());
                                queue.push_back(edge.target.clone());
                            }
                        }
                    }
                }

                let density = self.calculate_density(&community_nodes);
                communities.push(Community {
                    id: uuid::Uuid::new_v4().to_string(),
                    nodes: community_nodes,
                    density,
                    modularity: 0.0,
                });
            }
        }

        communities
    }

    /// 计算子图密度
    fn calculate_density(&self, nodes: &[String]) -> f64 {
        let n = nodes.len() as f64;
        if n <= 1.0 {
            return 0.0;
        }

        let mut edge_count = 0.0;
        for node in nodes {
            if let Some(edges) = self.edges.get(node) {
                for edge in edges {
                    if nodes.contains(&edge.target) {
                        edge_count += 1.0;
                    }
                }
            }
        }

        edge_count / (n * (n - 1.0))
    }

    /// 获取节点
    pub fn get_node(&self, id: &str) -> Option<&_KGNode> {
        self.nodes.get(id)
    }

    /// 获取边
    pub fn get_edges(&self, node_id: &str) -> Option<&Vec<_KGEdge>> {
        self.edges.get(node_id)
    }

    /// 获取所有节点
    pub fn get_all_nodes(&self) -> &HashMap<String, _KGNode> {
        &self.nodes
    }

    /// 获取所有边
    pub fn get_all_edges(&self) -> &HashMap<String, Vec<_KGEdge>> {
        &self.edges
    }
}
