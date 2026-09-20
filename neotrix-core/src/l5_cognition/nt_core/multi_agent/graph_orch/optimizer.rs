//! DAG Optimizer — analyzes and transforms DAGs for better execution.
//!
//! Actions: merge sequential single-task nodes, identify critical path,
//! suggest parallelization opportunities.
//! Follows R-P126 (graph-based scheduling optimization).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::dag::{Dag, DagEdge, DagNode, EdgeType, NodeType};

/// Result of a critical path analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalPath {
    /// Ordered node ids on the critical path
    pub path: Vec<String>,
    /// Total depth (number of sequential hops)
    pub total_depth: u32,
}

/// A suggestion for parallelizing independent nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelizationSuggestion {
    /// Nodes that can run in parallel
    pub parallel_group: Vec<String>,
    /// Reason for the suggestion
    pub reason: String,
}

/// Optimizer that analyzes and transforms DAGs for improved scheduling.
pub struct DagOptimizer;

impl DagOptimizer {
    /// Optimize a DAG in place: merge sequential single-task nodes,
    /// then return suggestions for parallelization.
    ///
    /// Returns the critical path and parallelization suggestions.
    pub fn optimize(
        dag: &mut Dag,
    ) -> Result<(CriticalPath, Vec<ParallelizationSuggestion>), String> {
        Self::merge_sequential_single_task_nodes(dag)?;
        let critical = Self::identify_critical_path(dag);
        let suggestions = Self::suggest_parallelization(dag);
        Ok((critical, suggestions))
    }

    /// Merge chains of Sequential single-Task nodes into single nodes.
    ///
    /// If node A -> Sequential -> node B, and both are Task type with
    /// no branching, they can be merged. Merged node id = "merged_{first}_{last}".
    pub fn merge_sequential_single_task_nodes(dag: &mut Dag) -> Result<(), String> {
        loop {
            let merge_candidates = Self::find_merge_candidates(dag);
            if merge_candidates.is_empty() {
                break;
            }

            for (from_id, to_id) in merge_candidates {
                Self::merge_pair(dag, &from_id, &to_id)?;
            }
        }
        Ok(())
    }

    /// Find pairs of nodes that can be merged:
    /// - Both are Task nodes
    /// - Connected by a single Sequential edge
    /// - `from` has exactly one outgoing edge
    /// - `to` has exactly one incoming edge
    fn find_merge_candidates(dag: &Dag) -> Vec<(String, String)> {
        let mut candidates = Vec::new();

        for edge in &dag.edges {
            if edge.edge_type != EdgeType::Sequential {
                continue;
            }

            let from_node = match dag.get_node(&edge.from) {
                Some(n) => n,
                None => continue,
            };
            let to_node = match dag.get_node(&edge.to) {
                Some(n) => n,
                None => continue,
            };

            if from_node.node_type != NodeType::Task || to_node.node_type != NodeType::Task {
                continue;
            }

            // from must have exactly one outgoing edge
            if dag.outgoing_edges(&edge.from).len() != 1 {
                continue;
            }
            // to must have exactly one incoming edge
            if dag.incoming_edges(&edge.to).len() != 1 {
                continue;
            }

            candidates.push((edge.from.clone(), edge.to.clone()));
        }

        candidates
    }

    /// Merge two nodes: combine metadata, rewire edges, remove the second node.
    fn merge_pair(dag: &mut Dag, from_id: &str, to_id: &str) -> Result<(), String> {
        let new_id = format!("merged_{}_{}", from_id, to_id);

        // Build merged metadata
        let mut metadata = HashMap::new();
        if let Some(from_node) = dag.get_node(from_id) {
            metadata.extend(from_node.metadata.clone());
            metadata.insert("merged_from".to_string(), from_id.to_string());
        }
        if let Some(to_node) = dag.get_node(to_id) {
            metadata.extend(to_node.metadata.clone());
            metadata.insert("merged_to".to_string(), to_id.to_string());
        }

        let merged_node = DagNode::new(&new_id, NodeType::Task).with_metadata("merged", "true");
        // Copy merged metadata
        let mut final_node = merged_node;
        for (k, v) in metadata {
            final_node.metadata.insert(k, v);
        }

        dag.add_node(final_node)?;

        // Rewire edges from `from_id` to `new_id` (incoming)
        let incoming_from: Vec<DagEdge> = dag
            .incoming_edges(from_id)
            .into_iter()
            .map(|e| DagEdge::new(&e.from, &new_id, e.edge_type))
            .collect();
        // Rewire edges from `to_id` from `new_id` (outgoing)
        let outgoing_to: Vec<DagEdge> = dag
            .outgoing_edges(to_id)
            .into_iter()
            .map(|e| DagEdge::new(&new_id, &e.to, e.edge_type))
            .collect();

        // Remove old nodes and their edges
        dag.nodes.remove(from_id);
        dag.nodes.remove(to_id);
        dag.edges
            .retain(|e| e.from != from_id && e.to != from_id && e.from != to_id && e.to != to_id);

        // Add re-wired edges
        for e in incoming_from {
            let _ = dag.add_edge(e);
        }
        for e in outgoing_to {
            let _ = dag.add_edge(e);
        }

        Ok(())
    }

    /// Identify the critical path (longest path through the DAG).
    ///
    /// Uses dynamic programming on the topological order.
    pub fn identify_critical_path(dag: &Dag) -> CriticalPath {
        let order = match dag.topological_sort() {
            Ok(o) => o,
            Err(_) => {
                return CriticalPath {
                    path: Vec::new(),
                    total_depth: 0,
                }
            }
        };

        // dist[node] = longest path from any root to this node
        let mut dist: HashMap<&str, u32> = HashMap::new();
        let mut predecessor: HashMap<&str, Option<&str>> = HashMap::new();

        for id in &order {
            let max_incoming = dag
                .incoming_edges(id)
                .iter()
                .filter(|e| matches!(e.edge_type, EdgeType::Sequential))
                .filter_map(|e| {
                    let d = dist.get(e.from.as_str())?;
                    Some((d + 1, e.from.as_str()))
                })
                .max_by_key(|(d, _)| *d);

            match max_incoming {
                Some((d, pred)) => {
                    dist.insert(id.as_str(), d);
                    predecessor.insert(id.as_str(), Some(pred));
                }
                None => {
                    dist.insert(id.as_str(), 0);
                    predecessor.insert(id.as_str(), None);
                }
            }
        }

        // Find the node with maximum distance
        let end_node = dist
            .iter()
            .max_by_key(|(_, &d)| d)
            .map(|(&id, _)| id)
            .unwrap_or("");

        // Trace back
        let mut path = Vec::new();
        let mut current = Some(end_node);
        while let Some(node) = current {
            path.push(node.to_string());
            current = predecessor.get(node).and_then(|p| *p);
        }
        path.reverse();

        let total_depth = dist.get(end_node).copied().unwrap_or(0);

        CriticalPath { path, total_depth }
    }

    /// Suggest parallelization opportunities.
    ///
    /// Finds nodes at the same depth with no data dependency between them.
    pub fn suggest_parallelization(dag: &Dag) -> Vec<ParallelizationSuggestion> {
        let order = match dag.topological_sort() {
            Ok(o) => o,
            Err(_) => return Vec::new(),
        };

        // Compute depth
        let mut depth: HashMap<&str, u32> = HashMap::new();
        for id in &order {
            let max_pred = dag
                .incoming_edges(id)
                .iter()
                .filter(|e| matches!(e.edge_type, EdgeType::Sequential))
                .filter_map(|e| depth.get(e.from.as_str()))
                .copied()
                .max();
            depth.insert(id.as_str(), max_pred.map_or(0, |d| d + 1));
        }

        // Group by depth
        let mut by_depth: HashMap<u32, Vec<&str>> = HashMap::new();
        for id in &order {
            let d = depth[id.as_str()];
            by_depth.entry(d).or_default().push(id);
        }

        let mut suggestions = Vec::new();
        for (d, nodes) in &by_depth {
            if nodes.len() > 1 {
                suggestions.push(ParallelizationSuggestion {
                    parallel_group: nodes.iter().map(|s| s.to_string()).collect(),
                    reason: format!(
                        "These {} nodes are at depth {} with no dependencies between them",
                        nodes.len(),
                        d
                    ),
                });
            }
        }

        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::super::dag::{DagEdge, DagNode, EdgeType, NodeType};
    use super::*;

    fn linear_task_dag() -> Dag {
        let mut dag = Dag::new("linear");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("c", NodeType::Task)).unwrap();
        dag.add_edge(DagEdge::new("a", "b", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("b", "c", EdgeType::Sequential))
            .unwrap();
        dag
    }

    #[test]
    fn merge_sequential_tasks() {
        let mut dag = linear_task_dag();
        DagOptimizer::merge_sequential_single_task_nodes(&mut dag).unwrap();
        // All three should merge into one
        assert_eq!(dag.node_count(), 1);
        let merged_id = dag.nodes.keys().next().unwrap();
        assert!(merged_id.starts_with("merged_"));
    }

    #[test]
    fn merge_preserves_non_task_nodes() {
        let mut dag = Dag::new("mixed");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("decision", NodeType::Decision))
            .unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        dag.add_edge(DagEdge::new("a", "decision", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("decision", "b", EdgeType::Sequential))
            .unwrap();

        DagOptimizer::merge_sequential_single_task_nodes(&mut dag).unwrap();
        // Decision node prevents merging
        assert_eq!(dag.node_count(), 3);
    }

    #[test]
    fn critical_path_linear() {
        let dag = linear_task_dag();
        let cp = DagOptimizer::identify_critical_path(&dag);
        assert_eq!(cp.path, vec!["a", "b", "c"]);
        assert_eq!(cp.total_depth, 2);
    }

    #[test]
    fn critical_path_diamond() {
        let mut dag = Dag::new("diamond");
        dag.add_node(DagNode::new("a", NodeType::Split)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("c", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("d", NodeType::Join)).unwrap();
        dag.add_edge(DagEdge::new("a", "b", EdgeType::Parallel))
            .unwrap();
        dag.add_edge(DagEdge::new("a", "c", EdgeType::Parallel))
            .unwrap();
        dag.add_edge(DagEdge::new("b", "d", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("c", "d", EdgeType::Sequential))
            .unwrap();

        let cp = DagOptimizer::identify_critical_path(&dag);
        assert!(cp.path.contains(&"a".to_string()));
        assert!(cp.path.contains(&"d".to_string()));
    }

    #[test]
    fn suggest_parallelization_diamond() {
        let mut dag = Dag::new("diamond");
        dag.add_node(DagNode::new("a", NodeType::Split)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("c", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("d", NodeType::Join)).unwrap();
        dag.add_edge(DagEdge::new("a", "b", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("a", "c", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("b", "d", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("c", "d", EdgeType::Sequential))
            .unwrap();

        let suggestions = DagOptimizer::suggest_parallelization(&dag);
        assert!(!suggestions.is_empty());
        // b and c should be in a parallel group
        let group: Vec<&str> = suggestions[0]
            .parallel_group
            .iter()
            .map(|s| s.as_str())
            .collect();
        assert!(group.contains(&"b"));
        assert!(group.contains(&"c"));
    }

    #[test]
    fn optimize_full_pipeline() {
        let mut dag = linear_task_dag();
        let (cp, suggestions) = DagOptimizer::optimize(&mut dag).unwrap();
        // After merge, critical path is the single merged node
        assert_eq!(cp.path.len(), 1);
        assert!(suggestions.is_empty()); // single node, no parallelization
    }

    #[test]
    fn critical_path_empty_dag() {
        let dag = Dag::new("empty");
        let cp = DagOptimizer::identify_critical_path(&dag);
        assert!(cp.path.is_empty());
        assert_eq!(cp.total_depth, 0);
    }

    #[test]
    fn parallelization_single_node_no_suggestion() {
        let mut dag = Dag::new("single");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        let suggestions = DagOptimizer::suggest_parallelization(&dag);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn merge_preserves_outgoing_edges() {
        let mut dag = Dag::new("merge-out");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("c", NodeType::Task)).unwrap();
        dag.add_edge(DagEdge::new("a", "b", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("b", "c", EdgeType::Sequential))
            .unwrap();

        DagOptimizer::merge_sequential_single_task_nodes(&mut dag).unwrap();
        // Merged node should have edge to c
        let merged_id = dag.nodes.keys().next().unwrap();
        assert_eq!(dag.outgoing_edges(merged_id).len(), 1);
        assert_eq!(dag.outgoing_edges(merged_id)[0].to, "c");
    }

    #[test]
    fn merge_no_mergeable_pairs() {
        let mut dag = Dag::new("no-merge");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Decision))
            .unwrap();
        dag.add_edge(DagEdge::new("a", "b", EdgeType::Sequential))
            .unwrap();
        DagOptimizer::merge_sequential_single_task_nodes(&mut dag).unwrap();
        assert_eq!(dag.node_count(), 2);
    }

    #[test]
    fn critical_path_single_node() {
        let mut dag = Dag::new("single");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        let cp = DagOptimizer::identify_critical_path(&dag);
        assert_eq!(cp.path, vec!["a"]);
        assert_eq!(cp.total_depth, 0);
    }

    #[test]
    fn optimize_diamond_preserves_structure() {
        let mut dag = Dag::new("diamond");
        dag.add_node(DagNode::new("a", NodeType::Split)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("c", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("d", NodeType::Join)).unwrap();
        dag.add_edge(DagEdge::new("a", "b", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("a", "c", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("b", "d", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("c", "d", EdgeType::Sequential))
            .unwrap();

        let (cp, suggestions) = DagOptimizer::optimize(&mut dag).unwrap();
        assert!(cp.path.len() >= 2);
        // b and c are at same depth
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn parallelization_no_suggestions_for_linear() {
        let mut dag = linear_task_dag();
        DagOptimizer::merge_sequential_single_task_nodes(&mut dag).unwrap();
        let suggestions = DagOptimizer::suggest_parallelization(&dag);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn critical_path_serialization() {
        let cp = CriticalPath {
            path: vec!["a".into(), "b".into()],
            total_depth: 1,
        };
        let json = serde_json::to_string(&cp).unwrap();
        let back: CriticalPath = serde_json::from_str(&json).unwrap();
        assert_eq!(back.path, vec!["a", "b"]);
        assert_eq!(back.total_depth, 1);
    }

    #[test]
    fn parallelization_suggestion_serialization() {
        let s = ParallelizationSuggestion {
            parallel_group: vec!["a".into(), "b".into()],
            reason: "same depth".into(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: ParallelizationSuggestion = serde_json::from_str(&json).unwrap();
        assert_eq!(back.parallel_group.len(), 2);
    }
}
