use serde::{Deserialize, Serialize};

use super::nt_core_kb_types::{NodeType, RelationType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
    pub embedding: Option<Vec<f32>>,
    pub metadata: serde_json::Value,
}

impl GraphNode {
    pub fn new(id: impl Into<String>, label: impl Into<String>, node_type: NodeType) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            node_type,
            embedding: None,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relation: RelationType,
    pub weight: f64,
    pub metadata: serde_json::Value,
}

impl GraphEdge {
    pub fn new(source: impl Into<String>, target: impl Into<String>, relation: RelationType) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            relation,
            weight: 1.0,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: GraphNode) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: GraphEdge) {
        self.edges.push(edge);
    }

    pub fn node_by_id(&self, id: &str) -> Option<&GraphNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn neighbors(&self, node_id: &str) -> Vec<&GraphEdge> {
        self.edges
            .iter()
            .filter(|e| e.source == node_id || e.target == node_id)
            .collect()
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: String,
    pub member_ids: Vec<String>,
    pub centroid: Vec<f32>,
    pub modularity: f64,
    pub summary: Option<String>,
}

impl Community {
    pub fn new(id: impl Into<String>, member_ids: Vec<String>, centroid: Vec<f32>) -> Self {
        Self {
            id: id.into(),
            member_ids,
            centroid,
            modularity: 0.0,
            summary: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_node_builder() {
        let node = GraphNode::new("n1", "Test Node", NodeType::Concept)
            .with_embedding(vec![0.1, 0.2, 0.3])
            .with_metadata(serde_json::json!({"key": "value"}));
        assert_eq!(node.id, "n1");
        assert_eq!(node.embedding.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_knowledge_graph_operations() {
        let mut kg = KnowledgeGraph::new();
        kg.add_node(GraphNode::new("n1", "A", NodeType::Concept));
        kg.add_node(GraphNode::new("n2", "B", NodeType::Paper));
        kg.add_edge(GraphEdge::new("n1", "n2", RelationType::References));

        assert!(kg.node_by_id("n1").is_some());
        assert!(kg.node_by_id("n3").is_none());
        assert_eq!(kg.neighbors("n1").len(), 1);
        assert_eq!(kg.neighbors("n2").len(), 1);
    }

    #[test]
    fn test_community_creation() {
        let c = Community::new("c1", vec!["n1".into(), "n2".into()], vec![0.5, 0.5]);
        assert_eq!(c.member_ids.len(), 2);
        assert_eq!(c.centroid, vec![0.5, 0.5]);
    }
}
