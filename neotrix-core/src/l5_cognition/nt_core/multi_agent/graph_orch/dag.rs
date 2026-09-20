//! DAG (Directed Acyclic Graph) for multi-agent task orchestration.
//!
//! Defines nodes, edges, and graph operations including topological sort.
//! Follows R-P125 (typed agent interfaces) and R-P126 (graph-based scheduling).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Type of node in the DAG
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    /// A concrete task to execute
    Task,
    /// A decision point with conditional outgoing edges
    Decision,
    /// Joins multiple parallel branches back into one
    Join,
    /// Splits execution into parallel branches
    Split,
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Task => write!(f, "Task"),
            NodeType::Decision => write!(f, "Decision"),
            NodeType::Join => write!(f, "Join"),
            NodeType::Split => write!(f, "Split"),
        }
    }
}

/// A single node in the DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagNode {
    /// Unique node identifier
    pub id: String,
    /// Type of this node
    pub node_type: NodeType,
    /// Arbitrary metadata (key-value pairs)
    pub metadata: HashMap<String, String>,
}

impl DagNode {
    pub fn new(id: impl Into<String>, node_type: NodeType) -> Self {
        Self {
            id: id.into(),
            node_type,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Type of edge between nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    /// Strict ordering: `to` runs after `from`
    Sequential,
    /// Conditional: `to` runs only if condition is met (metadata key "condition")
    Conditional,
    /// Parallel: both branches execute concurrently
    Parallel,
}

impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeType::Sequential => write!(f, "Sequential"),
            EdgeType::Conditional => write!(f, "Conditional"),
            EdgeType::Parallel => write!(f, "Parallel"),
        }
    }
}

/// An edge connecting two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagEdge {
    /// Source node id
    pub from: String,
    /// Target node id
    pub to: String,
    /// Edge type
    pub edge_type: EdgeType,
}

impl DagEdge {
    pub fn new(from: impl Into<String>, to: impl Into<String>, edge_type: EdgeType) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            edge_type,
        }
    }
}

/// A Directed Acyclic Graph for multi-agent orchestration.
///
/// Nodes represent tasks or control-flow points; edges represent
/// execution dependencies or parallelism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dag {
    /// All nodes keyed by id
    pub nodes: HashMap<String, DagNode>,
    /// All edges
    pub edges: Vec<DagEdge>,
    /// Human-readable name for this DAG
    pub name: String,
}

impl Dag {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            name: name.into(),
        }
    }

    /// Add a node to the DAG. Returns error if node id already exists.
    pub fn add_node(&mut self, node: DagNode) -> Result<(), String> {
        if self.nodes.contains_key(&node.id) {
            return Err(format!("duplicate node id: {}", node.id));
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    /// Add an edge. Both endpoints must already exist.
    pub fn add_edge(&mut self, edge: DagEdge) -> Result<(), String> {
        if !self.nodes.contains_key(&edge.from) {
            return Err(format!("source node not found: {}", edge.from));
        }
        if !self.nodes.contains_key(&edge.to) {
            return Err(format!("target node not found: {}", edge.to));
        }
        if edge.from == edge.to {
            return Err("self-loop not allowed".to_string());
        }
        self.edges.push(edge);
        Ok(())
    }

    /// Topological sort using Kahn's algorithm.
    ///
    /// Returns nodes in a valid execution order where all dependencies
    /// of a node appear before it. Returns error if a cycle is detected.
    pub fn topological_sort(&self) -> Result<Vec<String>, String> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();

        for id in self.nodes.keys() {
            in_degree.entry(id.as_str()).or_insert(0);
            adjacency.entry(id.as_str()).or_default();
        }

        for edge in &self.edges {
            *in_degree.entry(&edge.to).or_insert(0) += 1;
            adjacency.entry(&edge.from).or_default().push(&edge.to);
        }

        let mut queue: Vec<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();
        queue.sort(); // deterministic ordering

        let mut sorted = Vec::new();

        while let Some(node) = queue.remove(0) {
            sorted.push(node.to_string());

            if let Some(neighbors) = adjacency.get(node) {
                let mut next: Vec<&str> = neighbors
                    .iter()
                    .filter_map(|&neighbor| {
                        let deg = in_degree.get_mut(neighbor)?;
                        *deg -= 1;
                        if *deg == 0 {
                            Some(neighbor)
                        } else {
                            None
                        }
                    })
                    .collect();
                next.sort();
                queue.extend(next);
            }
        }

        if sorted.len() != self.nodes.len() {
            return Err("cycle detected in DAG".to_string());
        }

        Ok(sorted)
    }

    /// Get all incoming edges for a node
    pub fn incoming_edges(&self, node_id: &str) -> Vec<&DagEdge> {
        self.edges.iter().filter(|e| e.to == node_id).collect()
    }

    /// Get all outgoing edges for a node
    pub fn outgoing_edges(&self, node_id: &str) -> Vec<&DagEdge> {
        self.edges.iter().filter(|e| e.from == node_id).collect()
    }

    /// Check if a node is a root (no incoming edges)
    pub fn is_root(&self, node_id: &str) -> bool {
        self.incoming_edges(node_id).is_empty()
    }

    /// Check if a node is a leaf (no outgoing edges)
    pub fn is_leaf(&self, node_id: &str) -> bool {
        self.outgoing_edges(node_id).is_empty()
    }

    /// Get root node ids (no incoming edges)
    pub fn roots(&self) -> Vec<&str> {
        self.nodes
            .keys()
            .filter(|id| self.is_root(id))
            .map(|s| s.as_str())
            .collect()
    }

    /// Get leaf node ids (no outgoing edges)
    pub fn leaves(&self) -> Vec<&str> {
        self.nodes
            .keys()
            .filter(|id| self.is_leaf(id))
            .map(|s| s.as_str())
            .collect()
    }

    /// Count of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Count of edges
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Get node by id
    pub fn get_node(&self, id: &str) -> Option<&DagNode> {
        self.nodes.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_dag() -> Dag {
        let mut dag = Dag::new("test-dag");
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
    fn add_node_basic() {
        let mut dag = Dag::new("d");
        let node = DagNode::new("n1", NodeType::Task).with_metadata("k", "v");
        dag.add_node(node).unwrap();
        assert_eq!(dag.node_count(), 1);
        assert_eq!(dag.get_node("n1").unwrap().metadata["k"], "v");
    }

    #[test]
    fn add_node_duplicate_rejects() {
        let mut dag = Dag::new("d");
        dag.add_node(DagNode::new("n1", NodeType::Task)).unwrap();
        let err = dag.add_node(DagNode::new("n1", NodeType::Decision));
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("duplicate"));
    }

    #[test]
    fn add_edge_basic() {
        let mut dag = Dag::new("d");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        dag.add_edge(DagEdge::new("a", "b", EdgeType::Sequential))
            .unwrap();
        assert_eq!(dag.edge_count(), 1);
    }

    #[test]
    fn add_edge_missing_node_rejects() {
        let mut dag = Dag::new("d");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        let err = dag.add_edge(DagEdge::new("a", "missing", EdgeType::Sequential));
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("not found"));
    }

    #[test]
    fn add_edge_self_loop_rejects() {
        let mut dag = Dag::new("d");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        let err = dag.add_edge(DagEdge::new("a", "a", EdgeType::Sequential));
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("self-loop"));
    }

    #[test]
    fn topological_sort_linear() {
        let dag = simple_dag();
        let order = dag.topological_sort().unwrap();
        assert_eq!(order, vec!["a", "b", "c"]);
    }

    #[test]
    fn topological_sort_diamond() {
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

        let order = dag.topological_sort().unwrap();
        assert_eq!(order.len(), 4);
        assert_eq!(order[0], "a");
        assert_eq!(order[3], "d");
        // b and c must appear between a and d
        let b_idx = order.iter().position(|x| x == "b").unwrap();
        let c_idx = order.iter().position(|x| x == "c").unwrap();
        assert!(b_idx > 0 && b_idx < 3);
        assert!(c_idx > 0 && c_idx < 3);
    }

    #[test]
    fn topological_sort_cycle_detection() {
        let mut dag = Dag::new("cycle");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        dag.add_edge(DagEdge::new("a", "b", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("b", "a", EdgeType::Sequential))
            .unwrap();
        assert!(dag.topological_sort().is_err());
    }

    #[test]
    fn roots_and_leaves() {
        let mut dag = Dag::new("d");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("c", NodeType::Task)).unwrap();
        dag.add_edge(DagEdge::new("a", "b", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("b", "c", EdgeType::Sequential))
            .unwrap();
        assert_eq!(dag.roots(), vec!["a"]);
        assert_eq!(dag.leaves(), vec!["c"]);
    }

    #[test]
    fn incoming_outgoing_edges() {
        let dag = simple_dag();
        assert_eq!(dag.incoming_edges("b").len(), 1);
        assert_eq!(dag.outgoing_edges("b").len(), 1);
        assert!(dag.incoming_edges("a").is_empty());
        assert!(dag.outgoing_edges("c").is_empty());
    }

    #[test]
    fn node_type_display() {
        assert_eq!(NodeType::Task.to_string(), "Task");
        assert_eq!(NodeType::Decision.to_string(), "Decision");
        assert_eq!(NodeType::Join.to_string(), "Join");
        assert_eq!(NodeType::Split.to_string(), "Split");
    }

    #[test]
    fn edge_type_display() {
        assert_eq!(EdgeType::Sequential.to_string(), "Sequential");
        assert_eq!(EdgeType::Conditional.to_string(), "Conditional");
        assert_eq!(EdgeType::Parallel.to_string(), "Parallel");
    }

    #[test]
    fn dag_serialization_roundtrip() {
        let dag = simple_dag();
        let json = serde_json::to_string(&dag).unwrap();
        let back: Dag = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "test-dag");
        assert_eq!(back.node_count(), 3);
        assert_eq!(back.edge_count(), 2);
    }

    #[test]
    fn empty_dag_sort() {
        let dag = Dag::new("empty");
        let order = dag.topological_sort().unwrap();
        assert!(order.is_empty());
    }

    #[test]
    fn single_node_dag() {
        let mut dag = Dag::new("single");
        dag.add_node(DagNode::new("only", NodeType::Task)).unwrap();
        let order = dag.topological_sort().unwrap();
        assert_eq!(order, vec!["only"]);
    }

    #[test]
    fn dag_roots_and_leaves_all() {
        let mut dag = Dag::new("isolated");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        // No edges: all nodes are both root and leaf
        assert_eq!(dag.roots().len(), 2);
        assert_eq!(dag.leaves().len(), 2);
    }

    #[test]
    fn dag_add_edge_before_node_fails() {
        let mut dag = Dag::new("d");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        let err = dag.add_edge(DagEdge::new("a", "b", EdgeType::Sequential));
        assert!(err.is_err());
        let err2 = dag.add_edge(DagEdge::new("missing", "a", EdgeType::Sequential));
        assert!(err2.is_err());
    }

    #[test]
    fn dag_metadata_preserved() {
        let mut dag = Dag::new("meta");
        let node = DagNode::new("n1", NodeType::Task)
            .with_metadata("key1", "val1")
            .with_metadata("key2", "val2");
        dag.add_node(node).unwrap();
        let n = dag.get_node("n1").unwrap();
        assert_eq!(n.metadata["key1"], "val1");
        assert_eq!(n.metadata["key2"], "val2");
    }

    #[test]
    fn dag_node_count_and_edge_count() {
        let mut dag = Dag::new("count");
        assert_eq!(dag.node_count(), 0);
        assert_eq!(dag.edge_count(), 0);
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        assert_eq!(dag.node_count(), 2);
        dag.add_edge(DagEdge::new("a", "b", EdgeType::Sequential))
            .unwrap();
        assert_eq!(dag.edge_count(), 1);
    }

    #[test]
    fn dag_name_preserved() {
        let dag = Dag::new("my_dag");
        assert_eq!(dag.name, "my_dag");
    }
}
