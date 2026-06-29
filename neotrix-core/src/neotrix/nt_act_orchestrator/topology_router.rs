use crate::neotrix::nt_act_orchestrator::process::ProcessType;
use crate::neotrix::nt_act_orchestrator::state_graph::StateGraph;
use std::collections::HashMap;

/// Features extracted from a DAG that inform topology selection.
#[derive(Debug, Clone)]
pub struct DagFeatures {
    /// Maximum number of nodes in any parallel level.
    pub parallel_width: usize,
    /// Length of the longest dependency chain.
    pub critical_path_depth: usize,
    /// Total number of nodes in the graph.
    pub node_count: usize,
    /// Ratio of edges to nodes.
    pub edge_density: f64,
    /// Domain hint derived from ArtifactType distribution, if clear.
    pub domain_hint: Option<String>,
}

/// Analyzes DAG structure to extract routing features.
pub struct DagFeatureAnalyzer;

impl DagFeatureAnalyzer {
    /// Analyze the given StateGraph and extract DagFeatures.
    pub fn analyze(&self, graph: &StateGraph) -> DagFeatures {
        let node_count = graph.nodes.len();

        let (parallel_width, critical_path_depth) = self.compute_parallel_metrics(graph);

        let edge_count = graph.edges.len();
        let edge_density = if node_count > 0 {
            edge_count as f64 / node_count as f64
        } else {
            0.0
        };

        let domain_hint = self.infer_domain_hint(graph);

        DagFeatures {
            parallel_width,
            critical_path_depth,
            node_count,
            edge_density,
            domain_hint,
        }
    }

    /// Compute parallel width (max nodes in any level) and critical path depth
    /// using a topological level assignment.
    fn compute_parallel_metrics(&self, graph: &StateGraph) -> (usize, usize) {
        if graph.nodes.is_empty() {
            return (0, 0);
        }

        let sorted = match graph.topological_sort() {
            Ok(order) => order,
            Err(_) => return (0, graph.nodes.len()),
        };

        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for id in graph.nodes.keys() {
            in_degree.entry(id.as_str()).or_insert(0);
        }
        for edge in &graph.edges {
            *in_degree.entry(&edge.to).or_insert(0) += 1;
        }

        let mut level: HashMap<&str, usize> = HashMap::new();
        for id in &sorted {
            let parent_levels: Vec<usize> = graph
                .edges
                .iter()
                .filter(|e| e.to == *id)
                .filter_map(|e| level.get(e.from.as_str()))
                .copied()
                .collect();
            let l = parent_levels.into_iter().max().unwrap_or(0) + 1;
            level.insert(id, l);
        }

        let mut level_counts: HashMap<usize, usize> = HashMap::new();
        for &l in level.values() {
            *level_counts.entry(l).or_insert(0) += 1;
        }
        let parallel_width = level_counts.values().copied().max().unwrap_or(0);
        let critical_path_depth = level.values().copied().max().unwrap_or(0);

        (parallel_width, critical_path_depth)
    }

    /// Infer domain hint from the dominant ArtifactType in the graph.
    fn infer_domain_hint(&self, graph: &StateGraph) -> Option<String> {
        let mut type_counts: HashMap<&str, usize> = HashMap::new();
        for node in graph.nodes.values() {
            let label = node.artifact_type.label();
            *type_counts.entry(label).or_insert(0) += 1;
        }
        let total = graph.nodes.len();
        if total == 0 {
            return None;
        }
        let dominant = type_counts.into_iter().max_by_key(|&(_, count)| count)?;
        let ratio = dominant.1 as f64 / total as f64;
        if ratio >= 0.4 {
            Some(dominant.0.to_string())
        } else {
            None
        }
    }
}

/// Selects the optimal ProcessType based on DAG features.
#[derive(Debug, Clone)]
pub struct TopologyRouter;

impl TopologyRouter {
    /// Select a ProcessType based on the given DAG features.
    ///
    /// Rules:
    /// - High parallelism (width > 3) + many nodes (count > 8) → Parallel
    /// - Deep critical path (depth > 4) + linear structure → Sequential
    /// - Mixed with high domain diversity → Hierarchical
    /// - Very large (count > 50) with any parallelism → Hybrid
    /// - Default → CustomDag
    pub fn select_process_type(features: &DagFeatures) -> ProcessType {
        if features.node_count > 50 && features.parallel_width > 1 {
            return ProcessType::Hybrid;
        }
        if features.parallel_width > 3 && features.node_count > 8 {
            return ProcessType::Parallel;
        }
        if features.critical_path_depth > 4 && features.parallel_width <= 2 {
            return ProcessType::Sequential;
        }
        if features.parallel_width > 1 && features.domain_hint.is_none() {
            return ProcessType::Hierarchical;
        }
        ProcessType::CustomDag
    }

    /// Convenience method: analyze the graph and select topology in one call.
    pub fn analyze_and_select(graph: &StateGraph, domain: Option<&str>) -> ProcessType {
        let analyzer = DagFeatureAnalyzer;
        let mut features = analyzer.analyze(graph);
        if features.domain_hint.is_none() {
            if let Some(d) = domain {
                features.domain_hint = Some(d.to_string());
            }
        }
        Self::select_process_type(&features)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_act_orchestrator::state_graph::{
        ArtifactNode, ArtifactType, StateGraph,
    };

    fn add_chain(graph: &mut StateGraph, count: usize) {
        let mut prev: Option<String> = None;
        for i in 0..count {
            let id = format!("n{}", i);
            graph.add_node(ArtifactNode::new(
                &id,
                ArtifactType::Task,
                &format!("Task {}", i),
            ));
            if let Some(p) = prev {
                graph.add_edge(&p, &id);
            }
            prev = Some(id);
        }
    }

    fn add_fan_out(graph: &mut StateGraph, root: &str, leaves: usize) {
        for i in 0..leaves {
            let id = format!("{}.leaf{}", root, i);
            graph.add_node(ArtifactNode::new(
                &id,
                ArtifactType::Task,
                &format!("Leaf {}", i),
            ));
            graph.add_edge(root, &id);
        }
    }

    fn add_fan_in(graph: &mut StateGraph, merge: &str, sources: usize) {
        for i in 0..sources {
            let id = format!("{}.src{}", merge, i);
            graph.add_node(ArtifactNode::new(
                &id,
                ArtifactType::Task,
                &format!("Src {}", i),
            ));
            graph.add_edge(&id, merge);
        }
    }

    #[test]
    fn test_linear_chain_features() {
        let mut graph = StateGraph::new();
        add_chain(&mut graph, 5);
        let analyzer = DagFeatureAnalyzer;
        let features = analyzer.analyze(&graph);
        assert_eq!(features.node_count, 5);
        assert_eq!(features.parallel_width, 1);
        assert_eq!(features.critical_path_depth, 5);
        assert!(features.edge_density >= 0.8);
    }

    #[test]
    fn test_diamond_features() {
        let mut graph = StateGraph::new();
        graph.add_node(ArtifactNode::new("root", ArtifactType::Proposal, "root"));
        graph.add_node(ArtifactNode::new("left", ArtifactType::Task, "left"));
        graph.add_node(ArtifactNode::new("right", ArtifactType::Task, "right"));
        graph.add_node(ArtifactNode::new("merge", ArtifactType::Review, "merge"));
        graph.add_edge("root", "left");
        graph.add_edge("root", "right");
        graph.add_edge("left", "merge");
        graph.add_edge("right", "merge");
        let analyzer = DagFeatureAnalyzer;
        let features = analyzer.analyze(&graph);
        assert_eq!(features.node_count, 4);
        assert_eq!(features.parallel_width, 2);
        assert_eq!(features.critical_path_depth, 3);
    }

    #[test]
    fn test_fan_out_fan_in_features() {
        let mut graph = StateGraph::new();
        graph.add_node(ArtifactNode::new("start", ArtifactType::Proposal, "start"));
        add_fan_out(&mut graph, "start", 6);
        graph.add_node(ArtifactNode::new("end", ArtifactType::Review, "end"));
        add_fan_in(&mut graph, "end", 6);
        let analyzer = DagFeatureAnalyzer;
        let features = analyzer.analyze(&graph);
        assert_eq!(features.node_count, 14);
        assert_eq!(features.parallel_width, 6);
        assert_eq!(features.critical_path_depth, 3);
    }

    #[test]
    fn test_domain_hint_detected() {
        let mut graph = StateGraph::new();
        for i in 0..5 {
            graph.add_node(ArtifactNode::new(
                &format!("code{}", i),
                ArtifactType::Code,
                &format!("Code {}", i),
            ));
        }
        graph.add_node(ArtifactNode::new("review", ArtifactType::Review, "review"));
        let analyzer = DagFeatureAnalyzer;
        let features = analyzer.analyze(&graph);
        assert_eq!(features.domain_hint.as_deref(), Some("code"));
    }

    #[test]
    fn test_domain_hint_not_detected() {
        let mut graph = StateGraph::new();
        graph.add_node(ArtifactNode::new("a", ArtifactType::Code, "a"));
        graph.add_node(ArtifactNode::new("b", ArtifactType::Design, "b"));
        graph.add_node(ArtifactNode::new("c", ArtifactType::Review, "c"));
        let analyzer = DagFeatureAnalyzer;
        let features = analyzer.analyze(&graph);
        assert!(features.domain_hint.is_none());
    }

    #[test]
    fn test_select_parallel_for_wide_dag() {
        let mut graph = StateGraph::new();
        graph.add_node(ArtifactNode::new("root", ArtifactType::Proposal, "root"));
        add_fan_out(&mut graph, "root", 5);
        graph.add_node(ArtifactNode::new("end", ArtifactType::Review, "end"));
        for i in 0..5 {
            graph.add_edge(&format!("root.leaf{}", i), "end");
        }
        let analyzer = DagFeatureAnalyzer;
        let features = analyzer.analyze(&graph);
        assert!(features.parallel_width > 3);
        assert!(features.node_count > 8);
        assert_eq!(
            TopologyRouter::select_process_type(&features),
            ProcessType::Parallel
        );
    }

    #[test]
    fn test_select_sequential_for_deep_chain() {
        let mut graph = StateGraph::new();
        add_chain(&mut graph, 6);
        let analyzer = DagFeatureAnalyzer;
        let features = analyzer.analyze(&graph);
        assert!(features.critical_path_depth > 4);
        assert!(features.parallel_width <= 2);
        assert_eq!(
            TopologyRouter::select_process_type(&features),
            ProcessType::Sequential
        );
    }

    #[test]
    fn test_select_hierarchical_for_mixed() {
        let mut graph = StateGraph::new();
        graph.add_node(ArtifactNode::new("root", ArtifactType::Design, "root"));
        graph.add_node(ArtifactNode::new("a", ArtifactType::Code, "a"));
        graph.add_node(ArtifactNode::new("b", ArtifactType::Test, "b"));
        graph.add_node(ArtifactNode::new("c", ArtifactType::Review, "c"));
        graph.add_edge("root", "a");
        graph.add_edge("root", "b");
        graph.add_edge("root", "c");
        let analyzer = DagFeatureAnalyzer;
        let features = analyzer.analyze(&graph);
        assert_eq!(features.parallel_width, 3);
        assert!(features.domain_hint.is_none());
        assert_eq!(
            TopologyRouter::select_process_type(&features),
            ProcessType::Hierarchical
        );
    }

    #[test]
    fn test_select_custom_dag_for_default() {
        let mut graph = StateGraph::new();
        graph.add_node(ArtifactNode::new("a", ArtifactType::Task, "a"));
        graph.add_node(ArtifactNode::new("b", ArtifactType::Task, "b"));
        graph.add_edge("a", "b");
        let analyzer = DagFeatureAnalyzer;
        let features = analyzer.analyze(&graph);
        assert_eq!(
            TopologyRouter::select_process_type(&features),
            ProcessType::CustomDag
        );
    }

    #[test]
    fn test_select_hybrid_for_very_large_dag() {
        let mut graph = StateGraph::new();
        graph.add_node(ArtifactNode::new("root", ArtifactType::Proposal, "root"));
        for i in 0..60 {
            let id = format!("worker{}", i);
            graph.add_node(ArtifactNode::new(
                &id,
                ArtifactType::Task,
                &format!("Worker {}", i),
            ));
            graph.add_edge("root", &id);
        }
        graph.add_node(ArtifactNode::new("end", ArtifactType::Review, "end"));
        for i in 0..60 {
            graph.add_edge(&format!("worker{}", i), "end");
        }
        let analyzer = DagFeatureAnalyzer;
        let features = analyzer.analyze(&graph);
        assert_eq!(
            TopologyRouter::select_process_type(&features),
            ProcessType::Hybrid
        );
    }

    #[test]
    fn test_analyze_and_select_convenience() {
        let mut graph = StateGraph::new();
        add_chain(&mut graph, 6);
        let result = TopologyRouter::analyze_and_select(&graph, None);
        assert_eq!(result, ProcessType::Sequential);
    }

    #[test]
    fn test_analyze_and_select_with_domain_override() {
        let mut graph = StateGraph::new();
        graph.add_node(ArtifactNode::new("a", ArtifactType::Design, "a"));
        graph.add_node(ArtifactNode::new("b", ArtifactType::Code, "b"));
        graph.add_node(ArtifactNode::new("c", ArtifactType::Code, "c"));
        let result = TopologyRouter::analyze_and_select(&graph, Some("research"));
        assert_eq!(result, ProcessType::CustomDag);
    }

    #[test]
    fn test_empty_graph() {
        let graph = StateGraph::new();
        let analyzer = DagFeatureAnalyzer;
        let features = analyzer.analyze(&graph);
        assert_eq!(features.node_count, 0);
        assert_eq!(features.parallel_width, 0);
        assert_eq!(features.critical_path_depth, 0);
        assert_eq!(features.edge_density, 0.0);
        assert!(features.domain_hint.is_none());
    }

    #[test]
    fn test_single_node_graph() {
        let mut graph = StateGraph::new();
        graph.add_node(ArtifactNode::new("only", ArtifactType::Task, "only"));
        let analyzer = DagFeatureAnalyzer;
        let features = analyzer.analyze(&graph);
        assert_eq!(features.node_count, 1);
        assert_eq!(features.parallel_width, 1);
        assert_eq!(features.critical_path_depth, 1);
    }
}
