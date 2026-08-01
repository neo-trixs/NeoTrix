//! Persistent graph-structured memory layer for the Global Workspace.
//! Implements graph-architecture memory with typed nodes, relational edges,
//! semantic search, BFS traversal, LRU eviction, TTL expiration, and GWT specialist integration.
//! Inspired by "Graph Memory" from the State of the Graph 2026 map — graph structure
//! as the core architecture of agent memory for long-term persistent, queryable behavior.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GraphMemoryNodeType {
    Concept,
    Session,
    SpecialistActivation,
    Decision,
    Reward,
    Skill,
    Reflection,
    Goal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMemoryNode {
    pub id: String,
    pub node_type: GraphMemoryNodeType,
    pub content: String,
    pub embedding: Vec<f32>,
    pub timestamp: u64,
    pub salience: f64,
    pub ttl: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMemoryEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub weight: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSnapshot {
    pub nodes: Vec<GraphMemoryNode>,
    pub edges: Vec<GraphMemoryEdge>,
    pub center_id: String,
    pub radius: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMemoryStore {
    nodes: HashMap<String, GraphMemoryNode>,
    edges: Vec<GraphMemoryEdge>,
    max_nodes: usize,
    max_edges: usize,
    access_order: Vec<String>,
}

impl GraphMemoryStore {
    pub fn new(max_nodes: usize, max_edges: usize) -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            max_nodes,
            max_edges,
            access_order: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: GraphMemoryNode) -> Result<String, String> {
        let id = node.id.clone();
        if id.is_empty() {
            return Err("Node id cannot be empty".into());
        }
        if self.nodes.len() >= self.max_nodes {
            self.evict_lru_inner(1);
        }
        let is_new = !self.nodes.contains_key(&id);
        self.nodes.insert(id.clone(), node);
        if is_new {
            self.access_order.push(id.clone());
        } else {
            self.touch(&id);
        }
        Ok(id)
    }

    pub fn add_edge(
        &mut self,
        source: &str,
        target: &str,
        relation: &str,
        weight: f64,
    ) -> Result<(), String> {
        if !self.nodes.contains_key(source) {
            return Err(format!("Source node '{}' not found", source));
        }
        if !self.nodes.contains_key(target) {
            return Err(format!("Target node '{}' not found", target));
        }
        let weight = weight.max(0.0).min(1.0);
        self.edges.push(GraphMemoryEdge {
            source: source.to_string(),
            target: target.to_string(),
            relation: relation.to_string(),
            weight,
            timestamp: timestamp_millis(),
        });
        if self.edges.len() > self.max_edges {
            self.edges.remove(0);
        }
        Ok(())
    }

    pub fn get_node(&self, id: &str) -> Option<&GraphMemoryNode> {
        self.nodes.get(id)
    }

    pub fn get_edges(&self, node_id: &str) -> Vec<&GraphMemoryEdge> {
        self.edges
            .iter()
            .filter(|e| e.source == node_id || e.target == node_id)
            .collect()
    }

    pub fn semantic_search(&self, query_embedding: &[f32], top_k: usize) -> Vec<&GraphMemoryNode> {
        let mut scored: Vec<(f64, &GraphMemoryNode)> = self
            .nodes
            .values()
            .filter(|n| !n.embedding.is_empty())
            .map(|n| {
                let sim = cosine_similarity(query_embedding, &n.embedding);
                (sim, n)
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(top_k).map(|(_, n)| n).collect()
    }

    pub fn bfs(
        &self,
        start_id: &str,
        max_depth: usize,
        relation_filter: &[String],
    ) -> Vec<String> {
        if !self.nodes.contains_key(start_id) {
            return Vec::new();
        }
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        visited.insert(start_id.to_string());
        queue.push_back((start_id.to_string(), 0usize));
        result.push(start_id.to_string());

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            for edge in &self.edges {
                let neighbor = if edge.source == current {
                    Some(&edge.target)
                } else if edge.target == current {
                    Some(&edge.source)
                } else {
                    None
                };
                if let Some(nid) = neighbor {
                    if !relation_filter.is_empty()
                        && !relation_filter.contains(&edge.relation)
                    {
                        continue;
                    }
                    if visited.insert(nid.clone()) {
                        queue.push_back((nid.clone(), depth + 1));
                        result.push(nid.clone());
                    }
                }
            }
        }
        result
    }

    pub fn recent_nodes(&self, count: usize) -> Vec<&GraphMemoryNode> {
        let mut sorted: Vec<&GraphMemoryNode> = self.nodes.values().collect();
        sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        sorted.into_iter().take(count).collect()
    }

    pub fn nodes_by_type(&self, node_type: GraphMemoryNodeType) -> Vec<&GraphMemoryNode> {
        self.nodes
            .values()
            .filter(|n| n.node_type == node_type)
            .collect()
    }

    pub fn prune_expired(&mut self) {
        let now = timestamp_millis();
        let expired: HashSet<String> = self
            .nodes
            .iter()
            .filter(|(_, n)| n.ttl > 0 && now > n.timestamp.saturating_add(n.ttl))
            .map(|(id, _)| id.clone())
            .collect();
        for id in &expired {
            self.nodes.remove(id);
        }
        self.access_order.retain(|x| !expired.contains(x));
        self.edges
            .retain(|e| !expired.contains(&e.source) && !expired.contains(&e.target));
    }

    pub fn evict_lru(&mut self, count: usize) {
        self.evict_lru_inner(count);
    }

    fn evict_lru_inner(&mut self, count: usize) {
        let remove: Vec<String> = self
            .access_order
            .drain(..count.min(self.access_order.len()))
            .collect();
        for id in &remove {
            self.nodes.remove(id);
        }
        self.edges
            .retain(|e| !remove.contains(&e.source) && !remove.contains(&e.target));
    }

    pub fn subgraph(&self, center_id: &str, radius: usize) -> GraphSnapshot {
        let node_ids = self.bfs(center_id, radius, &[]);
        let nodes: Vec<GraphMemoryNode> = node_ids
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .cloned()
            .collect();
        let node_set: HashSet<&String> = node_ids.iter().collect();
        let edges: Vec<GraphMemoryEdge> = self
            .edges
            .iter()
            .filter(|e| node_set.contains(&e.source) && node_set.contains(&e.target))
            .cloned()
            .collect();
        GraphSnapshot {
            nodes,
            edges,
            center_id: center_id.to_string(),
            radius,
        }
    }

    pub fn merge_other(&mut self, other: &mut GraphMemoryStore) {
        for (id, node) in other.nodes.drain() {
            if self.nodes.len() >= self.max_nodes {
                self.evict_lru_inner(1);
            }
            if !self.nodes.contains_key(&id) {
                self.access_order.push(id.clone());
            }
            self.nodes.insert(id, node);
        }
        for edge in other.edges.drain(..) {
            if self.max_edges == 0 {
                break;
            }
            if self.edges.len() >= self.max_edges {
                self.edges.remove(0);
            }
            self.edges.push(edge);
        }
    }

    fn touch(&mut self, id: &str) {
        if let Some(pos) = self.access_order.iter().position(|x| x == id) {
            self.access_order.remove(pos);
            self.access_order.push(id.to_string());
        }
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)) as f64
}

fn timestamp_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub trait GraphMemoryAgent {
    fn remember(&mut self, node: GraphMemoryNode);
    fn recall(&self, query: &str) -> Vec<&GraphMemoryNode>;
    fn associate(&mut self, source: &str, target: &str, relation: &str, weight: f64);
    fn consolidate(&mut self);
    fn snapshot(&self) -> GraphSnapshot;
}

impl GraphMemoryAgent for GraphMemoryStore {
    fn remember(&mut self, node: GraphMemoryNode) {
        let _ = self.add_node(node);
    }

    fn recall(&self, query: &str) -> Vec<&GraphMemoryNode> {
        let query_lower = query.to_lowercase();
        let mut matches: Vec<&GraphMemoryNode> = self
            .nodes
            .values()
            .filter(|n| n.content.to_lowercase().contains(&query_lower))
            .collect();
        matches.sort_by(|a, b| {
            b.salience
                .partial_cmp(&a.salience)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matches
    }

    fn associate(&mut self, source: &str, target: &str, relation: &str, weight: f64) {
        let _ = self.add_edge(source, target, relation, weight);
    }

    fn consolidate(&mut self) {
        self.prune_expired();
    }

    fn snapshot(&self) -> GraphSnapshot {
        GraphSnapshot {
            nodes: self.nodes.values().cloned().collect(),
            edges: self.edges.clone(),
            center_id: String::new(),
            radius: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryGraphSpecialist {
    pub store: GraphMemoryStore,
    pub activation_frequency: u64,
    pub name: String,
    pub last_activation: u64,
}

impl MemoryGraphSpecialist {
    pub fn new(name: &str, max_nodes: usize, max_edges: usize) -> Self {
        Self {
            store: GraphMemoryStore::new(max_nodes, max_edges),
            activation_frequency: 0,
            name: name.to_string(),
            last_activation: 0,
        }
    }

    pub fn store_broadcast(
        &mut self,
        content: &str,
        node_type: GraphMemoryNodeType,
        embedding: Vec<f32>,
    ) -> String {
        let now = timestamp_millis();
        let node = GraphMemoryNode {
            id: format!("gmem-{}-{}", self.name, now),
            node_type,
            content: content.to_string(),
            embedding,
            timestamp: now,
            salience: 0.5,
            ttl: 0,
            metadata: HashMap::new(),
        };
        self.store.add_node(node).unwrap_or_default()
    }

    pub fn recall_similar(
        &mut self,
        query_embedding: &[f32],
        top_k: usize,
    ) -> Vec<&GraphMemoryNode> {
        self.activation_frequency += 1;
        self.last_activation = timestamp_millis();
        self.store.semantic_search(query_embedding, top_k)
    }

    pub fn salience(&self) -> f64 {
        let freq_factor = (self.activation_frequency as f64).min(100.0) / 100.0;
        let recency_factor = if self.last_activation > 0 {
            let elapsed = timestamp_millis().saturating_sub(self.last_activation);
            (1.0f64).min(60000.0f64 / (elapsed as f64 + 1000.0))
        } else {
            0.0
        };
        freq_factor * 0.4 + recency_factor * 0.6
    }

    pub fn consolidate(&mut self) {
        self.store.prune_expired();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_node(id: &str, node_type: GraphMemoryNodeType) -> GraphMemoryNode {
        GraphMemoryNode {
            id: id.to_string(),
            node_type,
            content: format!("content of {}", id),
            embedding: Vec::new(),
            timestamp: timestamp_millis(),
            salience: 0.5,
            ttl: 0,
            metadata: HashMap::new(),
        }
    }

    fn test_node_with_embedding(id: &str, emb: Vec<f32>) -> GraphMemoryNode {
        GraphMemoryNode {
            id: id.to_string(),
            node_type: GraphMemoryNodeType::Concept,
            content: format!("content of {}", id),
            embedding: emb,
            timestamp: timestamp_millis(),
            salience: 0.5,
            ttl: 0,
            metadata: HashMap::new(),
        }
    }

    fn test_node_with_ts(id: &str, ts: u64) -> GraphMemoryNode {
        GraphMemoryNode {
            id: id.to_string(),
            node_type: GraphMemoryNodeType::Concept,
            content: format!("content of {}", id),
            embedding: Vec::new(),
            timestamp: ts,
            salience: 0.5,
            ttl: 0,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_add_node_and_retrieve_by_id() {
        let mut store = GraphMemoryStore::new(100, 500);
        let node = test_node("n1", GraphMemoryNodeType::Concept);
        let id = store.add_node(node).unwrap();
        assert_eq!(id, "n1");
        let retrieved = store.get_node("n1").unwrap();
        assert_eq!(retrieved.content, "content of n1");
        assert_eq!(retrieved.node_type, GraphMemoryNodeType::Concept);
    }

    #[test]
    fn test_add_edge_validates_node_existence() {
        let mut store = GraphMemoryStore::new(100, 500);
        store.add_node(test_node("n1", GraphMemoryNodeType::Concept)).unwrap();
        let result = store.add_edge("n1", "nonexistent", "caused", 0.8);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_semantic_search_returns_top_k() {
        let mut store = GraphMemoryStore::new(100, 500);
        store.add_node(test_node_with_embedding("a", vec![1.0, 0.0, 0.0])).unwrap();
        store.add_node(test_node_with_embedding("b", vec![0.0, 1.0, 0.0])).unwrap();
        store.add_node(test_node_with_embedding("c", vec![0.9, 0.1, 0.0])).unwrap();
        let query = vec![1.0, 0.0, 0.0];
        let results = store.semantic_search(&query, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "a");
        assert_eq!(results[1].id, "c");
    }

    #[test]
    fn test_bfs_traversal_finds_connected_nodes() {
        let mut store = GraphMemoryStore::new(100, 500);
        store.add_node(test_node("a", GraphMemoryNodeType::Concept)).unwrap();
        store.add_node(test_node("b", GraphMemoryNodeType::Concept)).unwrap();
        store.add_node(test_node("c", GraphMemoryNodeType::Concept)).unwrap();
        store.add_node(test_node("d", GraphMemoryNodeType::Concept)).unwrap();
        store.add_edge("a", "b", "caused", 1.0).unwrap();
        store.add_edge("b", "c", "precedes", 1.0).unwrap();
        store.add_edge("c", "d", "supports", 1.0).unwrap();
        let result = store.bfs("a", 2, &[]);
        assert!(result.contains(&"a".to_string()));
        assert!(result.contains(&"b".to_string()));
        assert!(result.contains(&"c".to_string()));
        assert!(!result.contains(&"d".to_string()));
    }

    #[test]
    fn test_recent_nodes_returns_correctly_ordered() {
        let mut store = GraphMemoryStore::new(100, 500);
        store.add_node(test_node_with_ts("old", 100)).unwrap();
        store.add_node(test_node_with_ts("mid", 200)).unwrap();
        store.add_node(test_node_with_ts("new", 300)).unwrap();
        let recent = store.recent_nodes(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].id, "new");
        assert_eq!(recent[1].id, "mid");
    }

    #[test]
    fn test_nodes_by_type_filter_works() {
        let mut store = GraphMemoryStore::new(100, 500);
        store.add_node(test_node("s1", GraphMemoryNodeType::Skill)).unwrap();
        store.add_node(test_node("s2", GraphMemoryNodeType::Skill)).unwrap();
        store.add_node(test_node("g1", GraphMemoryNodeType::Goal)).unwrap();
        let skills = store.nodes_by_type(GraphMemoryNodeType::Skill);
        assert_eq!(skills.len(), 2);
        let goals = store.nodes_by_type(GraphMemoryNodeType::Goal);
        assert_eq!(goals.len(), 1);
    }

    #[test]
    fn test_prune_expired_removes_ttl_expired_nodes() {
        let mut store = GraphMemoryStore::new(100, 500);
        store.add_node(GraphMemoryNode {
            id: "expired".into(),
            node_type: GraphMemoryNodeType::Concept,
            content: "gone".into(),
            embedding: vec![],
            timestamp: 0,
            salience: 0.5,
            ttl: 1,
            metadata: HashMap::new(),
        }).unwrap();
        store.add_node(GraphMemoryNode {
            id: "permanent".into(),
            node_type: GraphMemoryNodeType::Concept,
            content: "stays".into(),
            embedding: vec![],
            timestamp: timestamp_millis(),
            salience: 0.5,
            ttl: 0,
            metadata: HashMap::new(),
        }).unwrap();
        store.prune_expired();
        assert!(store.get_node("expired").is_none());
        assert!(store.get_node("permanent").is_some());
    }

    #[test]
    fn test_evict_lru_removes_least_recently_accessed() {
        let mut store = GraphMemoryStore::new(100, 500);
        store.add_node(test_node("first", GraphMemoryNodeType::Concept)).unwrap();
        store.add_node(test_node("second", GraphMemoryNodeType::Concept)).unwrap();
        store.add_node(test_node("third", GraphMemoryNodeType::Concept)).unwrap();
        store.evict_lru(2);
        assert!(store.get_node("first").is_none());
        assert!(store.get_node("second").is_none());
        assert!(store.get_node("third").is_some());
    }

    #[test]
    fn test_subgraph_extracts_neighborhood() {
        let mut store = GraphMemoryStore::new(100, 500);
        store.add_node(test_node("c", GraphMemoryNodeType::Concept)).unwrap();
        store.add_node(test_node("n1", GraphMemoryNodeType::Concept)).unwrap();
        store.add_node(test_node("n2", GraphMemoryNodeType::Concept)).unwrap();
        store.add_node(test_node("far", GraphMemoryNodeType::Concept)).unwrap();
        store.add_edge("c", "n1", "caused", 1.0).unwrap();
        store.add_edge("c", "n2", "supports", 1.0).unwrap();
        store.add_edge("n1", "far", "precedes", 1.0).unwrap();
        let sg = store.subgraph("c", 1);
        assert_eq!(sg.radius, 1);
        assert_eq!(sg.center_id, "c");
        let sg_ids: HashSet<String> = sg.nodes.iter().map(|n| n.id.clone()).collect();
        assert!(sg_ids.contains("c"));
        assert!(sg_ids.contains("n1"));
        assert!(sg_ids.contains("n2"));
        assert!(!sg_ids.contains("far"));
    }

    #[test]
    fn test_merge_stores_with_dedup() {
        let mut store1 = GraphMemoryStore::new(100, 500);
        store1.add_node(test_node("shared", GraphMemoryNodeType::Concept)).unwrap();
        store1.add_node(test_node("only1", GraphMemoryNodeType::Concept)).unwrap();

        let mut store2 = GraphMemoryStore::new(100, 500);
        store2.add_node(test_node("shared", GraphMemoryNodeType::Concept)).unwrap();
        store2.add_node(test_node("only2", GraphMemoryNodeType::Concept)).unwrap();

        store1.merge_other(&mut store2);
        assert!(store1.get_node("shared").is_some());
        assert!(store1.get_node("only1").is_some());
        assert!(store1.get_node("only2").is_some());
    }

    #[test]
    fn test_memory_graph_specialist_recall() {
        let mut specialist = MemoryGraphSpecialist::new("test", 100, 500);
        specialist.store_broadcast("important memory", GraphMemoryNodeType::Session, vec![]);
        let results = specialist.store.recall("important");
        assert!(!results.is_empty());
        assert!(results.iter().any(|n| n.content.contains("important")));
    }

    #[test]
    fn test_consolidation_prunes_expired() {
        let mut store = GraphMemoryStore::new(100, 500);
        store.add_node(GraphMemoryNode {
            id: "exp".into(),
            node_type: GraphMemoryNodeType::Concept,
            content: "x".into(),
            embedding: vec![],
            timestamp: 0,
            salience: 0.5,
            ttl: 1,
            metadata: HashMap::new(),
        }).unwrap();
        store.add_node(test_node("keep", GraphMemoryNodeType::Concept)).unwrap();
        store.consolidate();
        assert!(store.get_node("exp").is_none());
        assert!(store.get_node("keep").is_some());
    }

    #[test]
    fn test_edge_with_self_is_valid() {
        let mut store = GraphMemoryStore::new(100, 500);
        store.add_node(test_node("self", GraphMemoryNodeType::Concept)).unwrap();
        let result = store.add_edge("self", "self", "refines", 1.0);
        assert!(result.is_ok());
        let edges = store.get_edges("self");
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].relation, "refines");
    }

    #[test]
    fn test_large_node_set_does_not_panic() {
        let mut store = GraphMemoryStore::new(10000, 50000);
        for i in 0..5000 {
            let node = test_node(&format!("n{}", i), GraphMemoryNodeType::Concept);
            store.add_node(node).unwrap();
        }
        assert_eq!(store.nodes.len(), 5000);
    }

    #[test]
    fn test_get_edges_for_non_existent_node_returns_empty() {
        let store = GraphMemoryStore::new(100, 500);
        let edges = store.get_edges("nonexistent");
        assert!(edges.is_empty());
    }

    #[test]
    fn test_add_node_empty_id_returns_error() {
        let mut store = GraphMemoryStore::new(100, 500);
        let node = GraphMemoryNode {
            id: "".into(),
            node_type: GraphMemoryNodeType::Concept,
            content: "no id".into(),
            embedding: vec![],
            timestamp: 0,
            salience: 0.5,
            ttl: 0,
            metadata: HashMap::new(),
        };
        let result = store.add_node(node);
        assert!(result.is_err());
    }

    #[test]
    fn test_bfs_with_relation_filter() {
        let mut store = GraphMemoryStore::new(100, 500);
        store.add_node(test_node("a", GraphMemoryNodeType::Concept)).unwrap();
        store.add_node(test_node("b", GraphMemoryNodeType::Concept)).unwrap();
        store.add_node(test_node("c", GraphMemoryNodeType::Concept)).unwrap();
        store.add_edge("a", "b", "caused", 1.0).unwrap();
        store.add_edge("a", "c", "supports", 1.0).unwrap();
        let filter = vec!["caused".to_string()];
        let result = store.bfs("a", 2, &filter);
        assert!(result.contains(&"b".to_string()));
        assert!(!result.contains(&"c".to_string()));
    }

    #[test]
    fn test_graph_snapshot_from_subgraph() {
        let mut store = GraphMemoryStore::new(100, 500);
        store.add_node(test_node("center", GraphMemoryNodeType::Concept)).unwrap();
        store.add_node(test_node("leaf", GraphMemoryNodeType::Concept)).unwrap();
        store.add_edge("center", "leaf", "caused", 0.9).unwrap();
        let snap = store.subgraph("center", 5);
        assert_eq!(snap.nodes.len(), 2);
        assert_eq!(snap.edges.len(), 1);
        assert_eq!(snap.center_id, "center");
    }

    #[test]
    fn test_specialist_salience_increases_with_frequency() {
        let mut spec = MemoryGraphSpecialist::new("freq", 100, 500);
        let s0 = spec.salience();
        spec.activation_frequency = 50;
        let s1 = spec.salience();
        assert!(s1 >= s0);
    }

    #[test]
    fn test_evict_lru_cascades_edge_removal() {
        let mut store = GraphMemoryStore::new(5, 50);
        store.add_node(test_node("a", GraphMemoryNodeType::Concept)).unwrap();
        store.add_node(test_node("b", GraphMemoryNodeType::Concept)).unwrap();
        store.add_node(test_node("c", GraphMemoryNodeType::Concept)).unwrap();
        store.add_edge("a", "b", "caused", 1.0).unwrap();
        store.add_edge("a", "c", "supports", 1.0).unwrap();
        store.evict_lru(1);
        assert!(store.get_node("a").is_none());
        let remaining_edges: Vec<&GraphMemoryEdge> = store.edges.iter().collect();
        assert!(remaining_edges.is_empty());
    }

    #[test]
    fn test_add_node_updates_existing() {
        let mut store = GraphMemoryStore::new(100, 500);
        let n1 = test_node("same", GraphMemoryNodeType::Concept);
        store.add_node(n1).unwrap();
        let n2 = GraphMemoryNode {
            id: "same".into(),
            node_type: GraphMemoryNodeType::Goal,
            content: "updated".into(),
            embedding: vec![],
            timestamp: 999,
            salience: 0.9,
            ttl: 0,
            metadata: HashMap::new(),
        };
        store.add_node(n2).unwrap();
        let node = store.get_node("same").unwrap();
        assert_eq!(node.node_type, GraphMemoryNodeType::Goal);
        assert_eq!(node.content, "updated");
        assert_eq!(node.salience, 0.9);
    }

    #[test]
    fn test_bfs_non_existent_start_returns_empty() {
        let store = GraphMemoryStore::new(100, 500);
        let result = store.bfs("nowhere", 5, &[]);
        assert!(result.is_empty());
    }
}
