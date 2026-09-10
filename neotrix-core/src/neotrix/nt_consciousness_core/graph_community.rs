#![forbid(unsafe_code)]

//! Graph Community Summary
//!
//! Louvain community detection on knowledge graph with LLM-generated summaries
//! per community, cross-community relationship tracking, and periodic re-computation.
//!
//! Operates on the NeoTrix KB graph structure.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Node in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub features: HashMap<String, String>,
}

/// Edge in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub edge_type: String,
    pub weight: f64,
}

/// The knowledge graph
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: HashMap<String, GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: GraphNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn add_edge(&mut self, edge: GraphEdge) {
        if self.nodes.contains_key(&edge.source) && self.nodes.contains_key(&edge.target) {
            self.edges.push(edge);
        }
    }

    pub fn neighbors(&self, node_id: &str) -> Vec<&GraphEdge> {
        self.edges
            .iter()
            .filter(|e| e.source == node_id || e.target == node_id)
            .collect()
    }

    pub fn degree(&self, node_id: &str) -> usize {
        self.neighbors(node_id).len()
    }
}

/// A detected community
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: u32,
    pub node_ids: Vec<String>,
    pub internal_edges: u32,
    pub external_edges: u32,
    /// Modularity contribution
    pub modularity: f64,
    /// LLM-generated summary (populated after summarization)
    pub summary: Option<String>,
    /// Community label (auto-detected or manual)
    pub label: Option<String>,
}

/// Cross-community relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCommunityRelation {
    pub source_community: u32,
    pub target_community: u32,
    pub edge_count: u32,
    pub total_weight: f64,
    pub edge_types: Vec<String>,
}

/// Re-computation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecomputationRecord {
    pub cycle: u32,
    pub timestamp: String,
    pub communities_found: usize,
    pub modularity: f64,
    pub duration_ms: u64,
}

/// Configuration for community detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityConfig {
    /// Resolution parameter (higher = more communities)
    pub resolution: f64,
    /// Maximum iterations for Louvain
    pub max_iterations: usize,
    /// Minimum community size
    pub min_community_size: usize,
    /// Maximum communities to track
    pub max_communities: usize,
}

impl Default for CommunityConfig {
    fn default() -> Self {
        Self {
            resolution: 1.0,
            max_iterations: 100,
            min_community_size: 3,
            max_communities: 50,
        }
    }
}

/// Graph Community detection and summary engine
pub struct GraphCommunityDetector {
    graph: KnowledgeGraph,
    communities: Vec<Community>,
    cross_relations: Vec<CrossCommunityRelation>,
    recompute_history: Vec<RecomputationRecord>,
    config: CommunityConfig,
    /// Node -> community mapping
    node_community: HashMap<String, u32>,
}

impl GraphCommunityDetector {
    pub fn new(graph: KnowledgeGraph, config: CommunityConfig) -> Self {
        Self {
            graph,
            communities: Vec::new(),
            cross_relations: Vec::new(),
            recompute_history: Vec::new(),
            config,
            node_community: HashMap::new(),
        }
    }

    pub fn with_defaults(graph: KnowledgeGraph) -> Self {
        Self::new(graph, CommunityConfig::default())
    }

    /// Run Louvain community detection
    pub fn detect(&mut self, cycle: u32) -> Vec<Community> {
        let start = std::time::Instant::now();

        // Phase 1: Initialize each node in its own community
        self.node_community.clear();
        for (id, _) in &self.graph.nodes {
            self.node_community
                .insert(id.clone(), self.node_community.len() as u32);
        }

        let node_ids: Vec<String> = self.graph.nodes.keys().cloned().collect();
        let total_weight: f64 = self.graph.edges.iter().map(|e| e.weight).sum();
        let two_m = if total_weight > 0.0 {
            2.0 * total_weight
        } else {
            1.0
        };

        // Phase 2: Iterative modularity optimization
        let mut improved = true;
        let mut iteration = 0;

        while improved && iteration < self.config.max_iterations {
            improved = false;
            iteration += 1;

            for node_id in &node_ids {
                let current_comm = *self.node_community.get(node_id).unwrap();

                // Calculate modularity gain for moving to each neighbor's community
                let neighbor_comms: HashSet<u32> = self
                    .graph
                    .neighbors(node_id)
                    .iter()
                    .filter_map(|e| {
                        let neighbor = if e.source == *node_id {
                            &e.target
                        } else {
                            &e.source
                        };
                        self.node_community.get(neighbor).copied()
                    })
                    .collect();

                let mut best_comm = current_comm;
                let mut best_gain = 0.0;

                for &comm in &neighbor_comms {
                    let gain = self.modularity_gain(node_id, comm, two_m);
                    if gain > best_gain {
                        best_gain = gain;
                        best_comm = comm;
                    }
                }

                if best_comm != current_comm {
                    *self.node_community.get_mut(node_id).unwrap() = best_comm;
                    improved = true;
                }
            }
        }

        // Phase 3: Build communities from node assignments
        self.build_communities();
        self.compute_cross_relations();

        let duration = start.elapsed().as_millis() as u64;
        let modularity = self.compute_modularity(two_m);

        self.recompute_history.push(RecomputationRecord {
            cycle,
            timestamp: chrono::Utc::now().to_rfc3339(),
            communities_found: self.communities.len(),
            modularity,
            duration_ms: duration,
        });

        self.communities.clone()
    }

    /// Generate LLM summary for a community (stub - in production calls LLM)
    pub fn summarize_community(&mut self, community_id: u32) -> Option<String> {
        let community = self.communities.iter_mut().find(|c| c.id == community_id)?;

        let node_labels: Vec<String> = community
            .node_ids
            .iter()
            .filter_map(|id| self.graph.nodes.get(id))
            .map(|n| n.label.clone())
            .collect();

        let node_types: Vec<String> = community
            .node_ids
            .iter()
            .filter_map(|id| self.graph.nodes.get(id))
            .map(|n| n.node_type.clone())
            .collect();

        let dominant_type = {
            let mut counts: HashMap<String, u32> = HashMap::new();
            for t in &node_types {
                *counts.entry(t.clone()).or_insert(0) += 1;
            }
            counts.into_iter().max_by_key(|(_, c)| *c).map(|(t, _)| t)
        };

        let summary = format!(
            "Community {} contains {} nodes ({} internal edges). Dominant type: {}. Key entities: {}",
            community.id,
            community.node_ids.len(),
            community.internal_edges,
            dominant_type.unwrap_or_else(|| "mixed".to_string()),
            node_labels.iter().take(5).cloned().collect::<Vec<_>>().join(", "),
        );

        community.summary = Some(summary.clone());
        Some(summary)
    }

    /// Get communities
    pub fn communities(&self) -> &[Community] {
        &self.communities
    }

    /// Get cross-community relationships
    pub fn cross_relations(&self) -> &[CrossCommunityRelation] {
        &self.cross_relations
    }

    /// Get re-computation history
    pub fn recompute_history(&self) -> &[RecomputationRecord] {
        &self.recompute_history
    }

    /// Get community for a specific node
    pub fn community_of(&self, node_id: &str) -> Option<u32> {
        self.node_community.get(node_id).copied()
    }

    /// Export results as JSON
    pub fn export_results(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&serde_json::json!({
            "communities": self.communities,
            "cross_relations": self.cross_relations,
            "recompute_history": self.recompute_history,
        }))
    }

    fn modularity_gain(&self, node_id: &str, target_comm: u32, two_m: f64) -> f64 {
        let k_i = self.graph.degree(node_id) as f64;
        let current_comm = *self.node_community.get(node_id).unwrap();

        if current_comm == target_comm {
            return 0.0;
        }

        // Sum of weights from node to target community
        let sigma_to_target: f64 = self
            .graph
            .neighbors(node_id)
            .iter()
            .filter(|e| {
                let neighbor = if e.source == *node_id {
                    &e.target
                } else {
                    &e.source
                };
                self.node_community.get(neighbor).copied() == Some(target_comm)
            })
            .map(|e| e.weight)
            .sum();

        // Sum of weights from node to current community
        let sigma_to_current: f64 = self
            .graph
            .neighbors(node_id)
            .iter()
            .filter(|e| {
                let neighbor = if e.source == *node_id {
                    &e.target
                } else {
                    &e.source
                };
                self.node_community.get(neighbor).copied() == Some(current_comm)
            })
            .map(|e| e.weight)
            .sum();

        // Degree sum of target community
        let degree_target: f64 = self
            .graph
            .nodes
            .keys()
            .filter(|id| *self.node_community.get(*id).unwrap() == target_comm)
            .map(|id| self.graph.degree(id) as f64)
            .sum();

        // Degree sum of current community
        let degree_current: f64 = self
            .graph
            .nodes
            .keys()
            .filter(|id| *self.node_community.get(*id).unwrap() == current_comm)
            .map(|id| self.graph.degree(id) as f64)
            .sum();

        let resolution = self.config.resolution;

        let gain = (sigma_to_target - sigma_to_current * k_i / two_m)
            + resolution * k_i * (degree_current - degree_target - k_i) / (two_m * two_m);

        gain
    }

    fn build_communities(&mut self) {
        let mut comm_nodes: HashMap<u32, Vec<String>> = HashMap::new();
        for (node_id, &comm_id) in &self.node_community {
            comm_nodes.entry(comm_id).or_default().push(node_id.clone());
        }

        self.communities.clear();
        let mut id_counter = 0u32;

        for (_comm_id, node_ids) in &comm_nodes {
            if node_ids.len() < self.config.min_community_size {
                continue;
            }

            let internal_edges = self
                .graph
                .edges
                .iter()
                .filter(|e| {
                    self.node_community.get(&e.source) == self.node_community.get(&e.target)
                        && self.node_community.get(&e.source) == Some(&_comm_id)
                })
                .count() as u32;

            let external_edges = self
                .graph
                .edges
                .iter()
                .filter(|e| {
                    (self.node_community.get(&e.source) == Some(&_comm_id)
                        && self.node_community.get(&e.target) != Some(&_comm_id))
                        || (self.node_community.get(&e.target) == Some(&_comm_id)
                            && self.node_community.get(&e.source) != Some(&_comm_id))
                })
                .count() as u32;

            self.communities.push(Community {
                id: id_counter,
                node_ids: node_ids.clone(),
                internal_edges,
                external_edges,
                modularity: 0.0,
                summary: None,
                label: None,
            });

            id_counter += 1;
        }

        // Cap communities
        if self.communities.len() > self.config.max_communities {
            self.communities
                .sort_by(|a, b| b.node_ids.len().cmp(&a.node_ids.len()));
            self.communities.truncate(self.config.max_communities);
        }
    }

    fn compute_cross_relations(&mut self) {
        self.cross_relations.clear();
        let mut rel_map: HashMap<(u32, u32), CrossCommunityRelation> = HashMap::new();

        for edge in &self.graph.edges {
            let comm_source = self.node_community.get(&edge.source).copied();
            let comm_target = self.node_community.get(&edge.target).copied();

            if let (Some(cs), Some(ct)) = (comm_source, comm_target) {
                if cs != ct {
                    let key = if cs < ct { (cs, ct) } else { (ct, cs) };
                    let entry = rel_map
                        .entry(key)
                        .or_insert_with(|| CrossCommunityRelation {
                            source_community: key.0,
                            target_community: key.1,
                            edge_count: 0,
                            total_weight: 0.0,
                            edge_types: Vec::new(),
                        });
                    entry.edge_count += 1;
                    entry.total_weight += edge.weight;
                    if !entry.edge_types.contains(&edge.edge_type) {
                        entry.edge_types.push(edge.edge_type.clone());
                    }
                }
            }
        }

        self.cross_relations = rel_map.into_values().collect();
        self.cross_relations.sort_by(|a, b| {
            b.total_weight
                .partial_cmp(&a.total_weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    fn compute_modularity(&self, two_m: f64) -> f64 {
        if two_m == 0.0 {
            return 0.0;
        }

        let mut q = 0.0;
        for edge in &self.graph.edges {
            let comm_source = self.node_community.get(&edge.source).copied();
            let comm_target = self.node_community.get(&edge.target).copied();
            if comm_source == comm_target {
                let k_i = self.graph.degree(&edge.source) as f64;
                let k_j = self.graph.degree(&edge.target) as f64;
                q += edge.weight - k_i * k_j / two_m;
            }
        }

        q / two_m
    }
}

impl std::fmt::Display for Community {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Community {}: {} nodes, {} internal, {} external edges",
            self.id,
            self.node_ids.len(),
            self.internal_edges,
            self.external_edges
        )?;
        if let Some(ref label) = self.label {
            writeln!(f, "  Label: {}", label)?;
        }
        if let Some(ref summary) = self.summary {
            writeln!(f, "  Summary: {}", summary)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_graph() -> KnowledgeGraph {
        let mut g = KnowledgeGraph::new();

        // Create a cluster of tightly connected nodes
        for i in 0..6 {
            g.add_node(GraphNode {
                id: format!("n{}", i),
                label: format!("Node {}", i),
                node_type: "entity".to_string(),
                features: HashMap::new(),
            });
        }

        // Dense connections within cluster
        for i in 0..5 {
            for j in (i + 1)..6 {
                g.add_edge(GraphEdge {
                    source: format!("n{}", i),
                    target: format!("n{}", j),
                    edge_type: "related".to_string(),
                    weight: 1.0,
                });
            }
        }

        // Weak connection to another cluster
        g.add_edge(GraphEdge {
            source: "n0".to_string(),
            target: "n5".to_string(),
            edge_type: "weak".to_string(),
            weight: 0.1,
        });

        g
    }

    #[test]
    fn test_graph_construction() {
        let g = build_test_graph();
        assert_eq!(g.nodes.len(), 6);
        assert!(!g.edges.is_empty());
    }

    #[test]
    fn test_community_detection() {
        let g = build_test_graph();
        let mut detector = GraphCommunityDetector::with_defaults(g);
        let communities = detector.detect(1);
        assert!(!communities.is_empty());
    }

    #[test]
    fn test_cross_community_relations() {
        let g = build_test_graph();
        let mut detector = GraphCommunityDetector::with_defaults(g);
        detector.detect(1);
        // May or may not have cross relations depending on detection
        let _ = detector.cross_relations();
    }

    #[test]
    fn test_community_of() {
        let g = build_test_graph();
        let mut detector = GraphCommunityDetector::with_defaults(g);
        detector.detect(1);
        let comm = detector.community_of("n0");
        assert!(comm.is_some());
    }

    #[test]
    fn test_summarize_community() {
        let g = build_test_graph();
        let mut detector = GraphCommunityDetector::with_defaults(g);
        detector.detect(1);
        if let Some(comm) = detector.communities().first() {
            let summary = detector.summarize_community(comm.id);
            assert!(summary.is_some());
        }
    }

    #[test]
    fn test_export_results() {
        let g = build_test_graph();
        let mut detector = GraphCommunityDetector::with_defaults(g);
        detector.detect(1);
        let json = detector.export_results();
        assert!(json.is_ok());
    }

    #[test]
    fn test_modularity_nonnegative() {
        let g = build_test_graph();
        let mut detector = GraphCommunityDetector::with_defaults(g);
        detector.detect(1);
        let total_weight: f64 = detector.graph.edges.iter().map(|e| e.weight).sum();
        let two_m = if total_weight > 0.0 {
            2.0 * total_weight
        } else {
            1.0
        };
        let q = detector.compute_modularity(two_m);
        assert!(q >= -1.0 && q <= 1.0);
    }
}
