use super::ad_node::AdNode;
use super::attack_path::{find_paths, AttackPath};
use super::graph::AdGraph;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaleObject {
    pub name: String,
    pub object_type: String,
    pub last_password_change: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdReport {
    pub total_users: usize,
    pub total_groups: usize,
    pub total_computers: usize,
    pub high_risk_paths: Vec<AttackPath>,
    pub privileged_groups: Vec<String>,
    pub stale_objects: Vec<StaleObject>,
}

pub struct AdAnalyzer;

impl AdAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, graph: &AdGraph) -> AdReport {
        let users = graph.users();
        let groups = graph.groups();
        let computers = graph.computers();

        let privileged_groups: Vec<String> = groups.iter()
            .filter(|g| matches!(g, AdNode::Group { name, .. } if name.contains("Admin") || name.contains("Privileged")))
            .map(|g| g.name().to_string())
            .collect();

        let high_risk_paths = self.find_high_risk_paths(graph);

        AdReport {
            total_users: users.len(),
            total_groups: groups.len(),
            total_computers: computers.len(),
            high_risk_paths,
            privileged_groups,
            stale_objects: vec![], // Placeholder
        }
    }

    fn find_high_risk_paths(&self, graph: &AdGraph) -> Vec<AttackPath> {
        let mut paths = Vec::new();
        for user in graph.users() {
            for computer in graph.computers() {
                let found = find_paths(graph, user.sid(), computer.sid(), 4);
                paths.extend(found);
            }
        }
        paths.sort_by(|a, b| b.risk_score.partial_cmp(&a.risk_score).unwrap());
        paths.into_iter().take(10).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::super::ad_node::{AdEdge, AdEdgeType};
    use super::*;

    fn test_graph() -> AdGraph {
        let mut g = AdGraph::new();
        g.add_node(AdNode::User {
            name: "admin".into(),
            sid: "U1".into(),
            enabled: true,
        });
        g.add_node(AdNode::Group {
            name: "Domain Admins".into(),
            sid: "G1".into(),
            members: vec![],
        });
        g.add_node(AdNode::Computer {
            name: "DC01".into(),
            sid: "C1".into(),
            os_version: "Server 2022".into(),
        });
        g.add_edge(AdEdge::new("U1".into(), "G1".into(), AdEdgeType::MemberOf));
        g.add_edge(AdEdge::new(
            "G1".into(),
            "C1".into(),
            AdEdgeType::HasSession,
        ));
        g
    }

    #[test]
    fn test_analyze() {
        let g = test_graph();
        let analyzer = AdAnalyzer::new();
        let report = analyzer.analyze(&g);
        assert_eq!(report.total_users, 1);
        assert_eq!(report.total_groups, 1);
        assert_eq!(report.total_computers, 1);
        assert!(report
            .privileged_groups
            .contains(&"Domain Admins".to_string()));
    }
}
