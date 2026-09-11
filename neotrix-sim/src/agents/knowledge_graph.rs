use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// A concept node in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptNode {
    pub id: u64,
    pub name: String,
    pub properties: HashMap<String, String>,
    pub access_count: u32,
    pub last_accessed: u64,
}

/// Relation type between concepts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationType {
    IsA,
    HasA,
    PartOf,
    Causes,
    LocatedAt,
    SimilarTo,
    OppositeOf,
    RelatedTo,
}

/// A directed edge between concepts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptEdge {
    pub from: u64,
    pub to: u64,
    pub rel_type: RelationType,
    pub weight: f32,
}

/// A subgraph extracted from the knowledge graph
#[derive(Debug, Clone)]
pub struct SubGraph {
    pub center: String,
    pub nodes: Vec<ConceptNode>,
    pub edges: Vec<ConceptEdge>,
    pub depth: usize,
}

/// Knowledge graph: concepts (nodes) and relations (edges)
pub struct KnowledgeGraph {
    nodes: Vec<ConceptNode>,
    edges: Vec<ConceptEdge>,
    next_id: u64,
    name_index: HashMap<String, u64>,
    adjacency: HashMap<u64, Vec<usize>>,
    max_nodes: usize,
}

impl KnowledgeGraph {
    pub fn new(max_nodes: usize) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            next_id: 0,
            name_index: HashMap::new(),
            adjacency: HashMap::new(),
            max_nodes,
        }
    }

    /// Add a concept node, returning its ID. Updates properties if it exists.
    pub fn add_concept(&mut self, name: String, properties: HashMap<String, String>) -> u64 {
        if let Some(&existing_id) = self.name_index.get(&name) {
            if let Some(node) = self.nodes.iter_mut().find(|n| n.id == existing_id) {
                for (k, v) in properties {
                    node.properties.insert(k, v);
                }
            }
            return existing_id;
        }

        let id = self.next_id;
        self.next_id += 1;
        self.name_index.insert(name.clone(), id);
        self.nodes.push(ConceptNode {
            id,
            name,
            properties,
            access_count: 0,
            last_accessed: 0,
        });
        self.adjacency.entry(id).or_default();
        self.prune();
        id
    }

    /// Add a directed relation between two concepts
    pub fn add_relation(&mut self, from: String, to: String, rel_type: RelationType) {
        let from_id = self.name_index.get(&from).copied();
        let to_id = self.name_index.get(&to).copied();

        if let (Some(fid), Some(tid)) = (from_id, to_id) {
            let exists = self.edges.iter().any(|e| {
                e.from == fid && e.to == tid && e.rel_type == rel_type
            });
            if !exists {
                let idx = self.edges.len();
                self.edges.push(ConceptEdge {
                    from: fid,
                    to: tid,
                    rel_type,
                    weight: 0.5,
                });
                self.adjacency.entry(fid).or_default().push(idx);
                self.adjacency.entry(tid).or_default().push(idx);
            }
        }
    }

    /// Query a subgraph around a center concept up to a given depth
    pub fn query_subgraph(&self, center: &str, depth: usize) -> Option<SubGraph> {
        let center_id = self.name_index.get(center)?;
        let mut visited = HashMap::new();
        let mut queue = VecDeque::new();
        queue.push_back((*center_id, 0usize));
        visited.insert(*center_id, 0usize);

        let mut node_ids = vec![*center_id];
        let mut edge_indices = Vec::new();

        while let Some((id, d)) = queue.pop_front() {
            if d >= depth {
                continue;
            }
            if let Some(idxs) = self.adjacency.get(&id) {
                for &ei in idxs {
                    let edge = &self.edges[ei];
                    let next = if edge.from == id { edge.to } else { edge.from };
                    if !visited.contains_key(&next) {
                        visited.insert(next, d + 1);
                        queue.push_back((next, d + 1));
                        node_ids.push(next);
                    }
                    if !edge_indices.contains(&ei) {
                        edge_indices.push(ei);
                    }
                }
            }
        }

        let nodes: Vec<ConceptNode> = node_ids
            .iter()
            .filter_map(|&nid| self.nodes.iter().find(|n| n.id == nid).cloned())
            .collect();
        let edges: Vec<ConceptEdge> = edge_indices
            .iter()
            .filter_map(|&ei| self.edges.get(ei).cloned())
            .collect();

        Some(SubGraph {
            center: center.to_string(),
            nodes,
            edges,
            depth,
        })
    }

    /// Find shortest path between two concepts using BFS
    pub fn shortest_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        let from_id = self.name_index.get(from)?;
        let to_id = self.name_index.get(to)?;

        if from_id == to_id {
            return Some(vec![from.to_string()]);
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: HashMap<u64, u64> = HashMap::new();
        queue.push_back(*from_id);
        visited.insert(*from_id);

        while let Some(id) = queue.pop_front() {
            if id == *to_id {
                let mut path = vec![*to_id];
                let mut cur = *to_id;
                while let Some(&p) = parent.get(&cur) {
                    path.push(p);
                    cur = p;
                }
                path.reverse();
                return path
                    .iter()
                    .filter_map(|&nid| {
                        self.nodes.iter().find(|n| n.id == nid).map(|n| n.name.clone())
                    })
                    .collect::<Vec<_>>()
                    .into();
            }

            if let Some(idxs) = self.adjacency.get(&id) {
                for &ei in idxs {
                    let edge = &self.edges[ei];
                    let next = if edge.from == id { edge.to } else { edge.from };
                    if visited.insert(next) {
                        parent.insert(next, id);
                        queue.push_back(next);
                    }
                }
            }
        }

        None
    }

    /// Get a concept by name
    pub fn get_concept(&self, name: &str) -> Option<&ConceptNode> {
        self.name_index
            .get(name)
            .and_then(|&id| self.nodes.iter().find(|n| n.id == id))
    }

    /// Get all neighbors of a concept
    pub fn neighbors(&self, name: &str) -> Vec<(&ConceptNode, &RelationType)> {
        self.name_index
            .get(name)
            .map(|&id| {
                self.adjacency
                    .get(&id)
                    .map(|idxs| {
                        idxs.iter()
                            .filter_map(|&ei| {
                                let edge = &self.edges[ei];
                                let next = if edge.from == id { edge.to } else { edge.from };
                                self.nodes
                                    .iter()
                                    .find(|n| n.id == next)
                                    .map(|n| (n, &edge.rel_type))
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn all_concepts(&self) -> &[ConceptNode] {
        &self.nodes
    }

    fn prune(&mut self) {
        if self.nodes.len() <= self.max_nodes {
            return;
        }
        self.nodes.sort_by(|a, b| {
            b.access_count
                .cmp(&a.access_count)
                .then(b.last_accessed.cmp(&a.last_accessed))
        });
        let keep: std::collections::HashSet<u64> = self
            .nodes
            .iter()
            .take(self.max_nodes * 8 / 10)
            .map(|n| n.id)
            .collect();
        self.nodes.retain(|n| keep.contains(&n.id));
        self.edges
            .retain(|e| keep.contains(&e.from) && keep.contains(&e.to));
        self.name_index.retain(|_, id| keep.contains(id));
        self.rebuild_adjacency();
    }

    fn rebuild_adjacency(&mut self) {
        self.adjacency.clear();
        for (i, edge) in self.edges.iter().enumerate() {
            self.adjacency.entry(edge.from).or_default().push(i);
            self.adjacency.entry(edge.to).or_default().push(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn props(kvs: &[(&str, &str)]) -> HashMap<String, String> {
        kvs.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn add_concept_and_query() {
        let mut kg = KnowledgeGraph::new(100);
        kg.add_concept("fire".into(), props(&[("type", "phenomenon")]));
        kg.add_concept("smoke".into(), props(&[("type", "byproduct")]));
        assert_eq!(kg.node_count(), 2);
        assert!(kg.get_concept("fire").is_some());
    }

    #[test]
    fn add_relation_and_neighbors() {
        let mut kg = KnowledgeGraph::new(100);
        kg.add_concept("fire".into(), HashMap::new());
        kg.add_concept("smoke".into(), HashMap::new());
        kg.add_relation("fire".into(), "smoke".into(), RelationType::Causes);
        assert_eq!(kg.edge_count(), 1);
        let neighbors = kg.neighbors("fire");
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].0.name, "smoke");
    }

    #[test]
    fn shortest_path() {
        let mut kg = KnowledgeGraph::new(100);
        kg.add_concept("a".into(), HashMap::new());
        kg.add_concept("b".into(), HashMap::new());
        kg.add_concept("c".into(), HashMap::new());
        kg.add_relation("a".into(), "b".into(), RelationType::RelatedTo);
        kg.add_relation("b".into(), "c".into(), RelationType::RelatedTo);
        let path = kg.shortest_path("a", "c").unwrap();
        assert_eq!(path, vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    }

    #[test]
    fn subgraph_extraction() {
        let mut kg = KnowledgeGraph::new(100);
        kg.add_concept("root".into(), HashMap::new());
        kg.add_concept("child1".into(), HashMap::new());
        kg.add_concept("child2".into(), HashMap::new());
        kg.add_concept("grandchild".into(), HashMap::new());
        kg.add_relation("root".into(), "child1".into(), RelationType::HasA);
        kg.add_relation("root".into(), "child2".into(), RelationType::HasA);
        kg.add_relation("child1".into(), "grandchild".into(), RelationType::HasA);
        let sub = kg.query_subgraph("root", 2).unwrap();
        assert!(sub.nodes.len() >= 3);
    }

    #[test]
    fn duplicate_concept_updates_properties() {
        let mut kg = KnowledgeGraph::new(100);
        kg.add_concept("x".into(), props(&[("a", "1")]));
        kg.add_concept("x".into(), props(&[("b", "2")]));
        assert_eq!(kg.node_count(), 1);
        let node = kg.get_concept("x").unwrap();
        assert!(node.properties.contains_key("a"));
        assert!(node.properties.contains_key("b"));
    }

    #[test]
    fn prune_respects_limit() {
        let mut kg = KnowledgeGraph::new(5);
        for i in 0..10 {
            kg.add_concept(format!("n{}", i), HashMap::new());
        }
        assert!(kg.node_count() <= 5);
    }
}
