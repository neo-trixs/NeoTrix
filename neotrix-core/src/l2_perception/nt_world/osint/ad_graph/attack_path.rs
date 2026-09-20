use super::ad_node::{AdEdge, AdEdgeType};
use super::graph::AdGraph;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PathStep {
    pub from: String,
    pub to: String,
    pub edge_type: AdEdgeType,
    pub technique: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackPath {
    pub steps: Vec<PathStep>,
    pub risk_score: f64,
    pub description: String,
}

impl AttackPath {
    pub fn from_edges(edges: &[&AdEdge]) -> Self {
        let steps: Vec<PathStep> = edges
            .iter()
            .map(|e| PathStep {
                from: e.source.clone(),
                to: e.target.clone(),
                edge_type: e.edge_type.clone(),
                technique: map_technique(&e.edge_type),
            })
            .collect();

        let risk_score = calculate_risk(&steps);
        let description = format!("Attack path with {} steps", steps.len());

        Self {
            steps,
            risk_score,
            description,
        }
    }
}

fn map_technique(edge_type: &AdEdgeType) -> String {
    match edge_type {
        AdEdgeType::MemberOf => "Group Membership".to_string(),
        AdEdgeType::HasSession => "Session Hijack".to_string(),
        AdEdgeType::ContainedIn => "Container Containment".to_string(),
        AdEdgeType::GPOApplies => "GPO Abuse".to_string(),
    }
}

fn calculate_risk(steps: &[PathStep]) -> f64 {
    steps
        .iter()
        .map(|s| match s.edge_type {
            AdEdgeType::MemberOf => 0.3,
            AdEdgeType::HasSession => 0.6,
            AdEdgeType::ContainedIn => 0.2,
            AdEdgeType::GPOApplies => 0.8,
        })
        .sum()
}

pub fn find_paths(
    graph: &AdGraph,
    source_sid: &str,
    target_sid: &str,
    max_depth: usize,
) -> Vec<AttackPath> {
    let raw_paths = graph.find_paths_bfs(source_sid, target_sid, max_depth);
    raw_paths
        .into_iter()
        .map(|edges| AttackPath::from_edges(&edges))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::ad_node::{AdEdge, AdNode};
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
    fn test_find_attack_paths() {
        let g = test_graph();
        let paths = find_paths(&g, "U1", "C1", 5);
        assert!(!paths.is_empty());
        assert!(paths[0].risk_score > 0.0);
    }

    #[test]
    fn test_path_techniques() {
        let g = test_graph();
        let paths = find_paths(&g, "U1", "C1", 5);
        let path = &paths[0];
        assert_eq!(path.steps[0].technique, "Group Membership");
        assert_eq!(path.steps[1].technique, "Session Hijack");
    }
}
