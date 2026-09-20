use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use super::session_bridge::Bridge;

/// A node in the weave graph representing a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaveNode {
    pub session_id: String,
    pub topic_count: usize,
}

/// A weighted edge connecting two sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaveEdge {
    pub from: String,
    pub to: String,
    pub weight: f64,
}

/// Graph of session-to-session connections derived from bridge data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaveGraph {
    pub nodes: Vec<WeaveNode>,
    pub edges: Vec<WeaveEdge>,
}

/// Build a weave graph from a set of bridges.
///
/// Nodes are deduplicated by session_id; `topic_count` is set to the number
/// of distinct sessions the node is bridged to (its degree in the graph).
/// Edges are created 1:1 from bridges with strength as weight.
pub fn build_graph(bridges: &[Bridge]) -> WeaveGraph {
    let mut adjacency: HashMap<String, HashSet<String>> = HashMap::new();

    for b in bridges {
        adjacency
            .entry(b.from_session.clone())
            .or_default()
            .insert(b.to_session.clone());
        adjacency
            .entry(b.to_session.clone())
            .or_default()
            .insert(b.from_session.clone());
    }

    let nodes: Vec<WeaveNode> = adjacency
        .iter()
        .map(|(id, neighbors)| WeaveNode {
            session_id: id.clone(),
            topic_count: neighbors.len(),
        })
        .collect();

    let edges: Vec<WeaveEdge> = bridges
        .iter()
        .map(|b| WeaveEdge {
            from: b.from_session.clone(),
            to: b.to_session.clone(),
            weight: b.strength,
        })
        .collect();

    WeaveGraph { nodes, edges }
}

/// Find connected components in the weave graph using BFS.
///
/// Returns a list of clusters, where each cluster is a list of session IDs.
pub fn find_clusters(graph: &WeaveGraph) -> Vec<Vec<String>> {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for node in &graph.nodes {
        adj.entry(node.session_id.as_str()).or_default();
    }
    for edge in &graph.edges {
        adj.entry(edge.from.as_str())
            .or_default()
            .push(&edge.to);
        adj.entry(edge.to.as_str())
            .or_default()
            .push(&edge.from);
    }

    let mut visited: HashSet<&str> = HashSet::new();
    let mut clusters = Vec::new();

    for node in &graph.nodes {
        if visited.contains(node.session_id.as_str()) {
            continue;
        }

        let mut cluster = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(node.session_id.as_str());
        visited.insert(node.session_id.as_str());

        while let Some(current) = queue.pop_front() {
            cluster.push(current.to_string());
            if let Some(neighbors) = adj.get(current) {
                for &neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        clusters.push(cluster);
    }

    clusters.sort_by(|a, b| b.len().cmp(&a.len()));
    clusters
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l6_meta::nt_nexus::memory_weaving::session_bridge::Bridge;

    fn bridge(from: &str, to: &str, strength: f64) -> Bridge {
        Bridge {
            from_session: from.to_string(),
            to_session: to.to_string(),
            shared_patterns: vec!["p1".into()],
            strength,
        }
    }

    #[test]
    fn build_graph_empty() {
        let graph = build_graph(&[]);
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn build_graph_single_bridge() {
        let graph = build_graph(&[bridge("s1", "s2", 0.8)]);
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].weight, 0.8);
    }

    #[test]
    fn build_graph_node_degree() {
        let graph = build_graph(&[
            bridge("s1", "s2", 0.5),
            bridge("s1", "s3", 0.6),
        ]);
        let s1 = graph.nodes.iter().find(|n| n.session_id == "s1").unwrap();
        assert_eq!(s1.topic_count, 2);
    }

    #[test]
    fn find_clusters_single() {
        let graph = build_graph(&[bridge("s1", "s2", 0.5)]);
        let clusters = find_clusters(&graph);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].len(), 2);
    }

    #[test]
    fn find_clusters_two_components() {
        let graph = build_graph(&[
            bridge("s1", "s2", 0.5),
            bridge("s3", "s4", 0.7),
        ]);
        let clusters = find_clusters(&graph);
        assert_eq!(clusters.len(), 2);
    }

    #[test]
    fn find_clusters_transitive() {
        let graph = build_graph(&[
            bridge("s1", "s2", 0.5),
            bridge("s2", "s3", 0.6),
        ]);
        let clusters = find_clusters(&graph);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].len(), 3);
    }

    #[test]
    fn find_clusters_isolated_node() {
        let mut graph = build_graph(&[bridge("s1", "s2", 0.5)]);
        graph.nodes.push(WeaveNode {
            session_id: "s99".into(),
            topic_count: 0,
        });
        let clusters = find_clusters(&graph);
        assert_eq!(clusters.len(), 2);
    }

    #[test]
    fn find_clusters_sorted_by_size() {
        let graph = build_graph(&[
            bridge("s1", "s2", 0.5),
            bridge("s3", "s4", 0.6),
            bridge("s4", "s5", 0.7),
        ]);
        let clusters = find_clusters(&graph);
        assert!(clusters[0].len() >= clusters[1].len());
    }
}
