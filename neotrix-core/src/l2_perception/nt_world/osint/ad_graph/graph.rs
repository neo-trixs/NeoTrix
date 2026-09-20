use super::ad_node::{AdEdge, AdNode};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Default)]
pub struct AdGraph {
    nodes: HashMap<String, AdNode>,
    edges: Vec<AdEdge>,
}

impl AdGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: AdNode) {
        self.nodes.insert(node.sid().to_string(), node);
    }

    pub fn add_edge(&mut self, edge: AdEdge) {
        if self.nodes.contains_key(&edge.source) && self.nodes.contains_key(&edge.target) {
            self.edges.push(edge);
        }
    }

    pub fn get_node(&self, sid: &str) -> Option<&AdNode> {
        self.nodes.get(sid)
    }

    pub fn get_edges_from(&self, sid: &str) -> Vec<&AdEdge> {
        self.edges.iter().filter(|e| e.source == sid).collect()
    }

    pub fn get_neighbors(&self, sid: &str) -> Vec<(&AdNode, &AdEdge)> {
        self.edges
            .iter()
            .filter(|e| e.source == sid)
            .filter_map(|e| self.nodes.get(&e.target).map(|node| (node, e)))
            .collect()
    }

    pub fn find_paths_bfs(
        &self,
        source_sid: &str,
        target_sid: &str,
        max_depth: usize,
    ) -> Vec<Vec<&AdEdge>> {
        let mut results = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back((source_sid, Vec::<&AdEdge>::new()));

        while let Some((current, path)) = queue.pop_front() {
            if path.len() > max_depth {
                continue;
            }
            if current == target_sid && !path.is_empty() {
                results.push(path);
                continue;
            }

            for edge in self.get_edges_from(current) {
                if !path.iter().any(|e| e.target == edge.target) {
                    let mut new_path = path.clone();
                    new_path.push(edge);
                    queue.push_back((&edge.target, new_path));
                }
            }
        }
        results
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn users(&self) -> Vec<&AdNode> {
        self.nodes
            .values()
            .filter(|n| matches!(n, AdNode::User { .. }))
            .collect()
    }

    pub fn groups(&self) -> Vec<&AdNode> {
        self.nodes
            .values()
            .filter(|n| matches!(n, AdNode::Group { .. }))
            .collect()
    }

    pub fn computers(&self) -> Vec<&AdNode> {
        self.nodes
            .values()
            .filter(|n| matches!(n, AdNode::Computer { .. }))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::super::ad_node::{AdEdge, AdEdgeType, AdNode};
    use super::*;

    fn test_graph() -> AdGraph {
        let mut g = AdGraph::new();
        g.add_node(AdNode::User {
            name: "admin".into(),
            sid: "U1".into(),
            enabled: true,
        });
        g.add_node(AdNode::User {
            name: "user1".into(),
            sid: "U2".into(),
            enabled: true,
        });
        g.add_node(AdNode::Group {
            name: "Domain Admins".into(),
            sid: "G1".into(),
            members: vec!["U1".into()],
        });
        g.add_node(AdNode::Computer {
            name: "DC01".into(),
            sid: "C1".into(),
            os_version: "Windows Server 2022".into(),
        });
        g.add_edge(AdEdge::new("U1".into(), "G1".into(), AdEdgeType::MemberOf));
        g.add_edge(AdEdge::new("U2".into(), "G1".into(), AdEdgeType::MemberOf));
        g.add_edge(AdEdge::new(
            "U1".into(),
            "C1".into(),
            AdEdgeType::HasSession,
        ));
        g
    }

    #[test]
    fn test_add_node() {
        let g = test_graph();
        assert_eq!(g.node_count(), 4);
    }

    #[test]
    fn test_get_node() {
        let g = test_graph();
        assert!(g.get_node("U1").is_some());
        assert!(g.get_node("X").is_none());
    }

    #[test]
    fn test_get_neighbors() {
        let g = test_graph();
        let neighbors = g.get_neighbors("U1");
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_find_paths() {
        let g = test_graph();
        let paths = g.find_paths_bfs("U2", "C1", 3);
        assert!(paths.len() > 0);
    }

    #[test]
    fn test_users_groups_computers() {
        let g = test_graph();
        assert_eq!(g.users().len(), 2);
        assert_eq!(g.groups().len(), 1);
        assert_eq!(g.computers().len(), 1);
    }
}
