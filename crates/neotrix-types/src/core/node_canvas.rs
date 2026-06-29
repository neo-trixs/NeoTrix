use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasNode {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<String>,
    pub content: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    Text,
    Image,
    Code,
    Concept,
    Task,
    Agent,
    Memory,
    Knowledge,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub label: Option<String>,
    pub edge_type: EdgeType,
    pub weight: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    Directed,
    Undirected,
    Bidirectional,
    Dashed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasProject {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub nodes: Vec<CanvasNode>,
    pub edges: Vec<NodeEdge>,
    pub zoom_level: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl CanvasProject {
    pub fn new(name: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: None,
            nodes: Vec::new(),
            edges: Vec::new(),
            zoom_level: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_node(&mut self, node: CanvasNode) {
        self.nodes.push(node);
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    pub fn remove_node(&mut self, node_id: &str) {
        self.nodes.retain(|n| n.id != node_id);
        self.edges
            .retain(|e| e.source_id != node_id && e.target_id != node_id);
    }

    pub fn get_node(&self, node_id: &str) -> Option<&CanvasNode> {
        self.nodes.iter().find(|n| n.id == node_id)
    }

    pub fn get_node_mut(&mut self, node_id: &str) -> Option<&mut CanvasNode> {
        self.nodes.iter_mut().find(|n| n.id == node_id)
    }

    pub fn connect(
        &mut self,
        source_id: &str,
        target_id: &str,
        edge_type: EdgeType,
    ) -> Option<&NodeEdge> {
        if self.get_node(source_id).is_none() || self.get_node(target_id).is_none() {
            return None;
        }
        let edge = NodeEdge {
            id: uuid::Uuid::new_v4().to_string(),
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            label: None,
            edge_type,
            weight: 1.0,
        };
        self.edges.push(edge);
        self.edges.last()
    }

    pub fn disconnect(&mut self, source_id: &str, target_id: &str) {
        self.edges
            .retain(|e| !(e.source_id == source_id && e.target_id == target_id));
    }

    pub fn neighbors(&self, node_id: &str) -> Vec<&CanvasNode> {
        let neighbor_ids: Vec<&str> = self
            .edges
            .iter()
            .filter(|e| e.source_id == node_id)
            .map(|e| e.target_id.as_str())
            .chain(
                self.edges
                    .iter()
                    .filter(|e| e.target_id == node_id && e.edge_type == EdgeType::Bidirectional)
                    .map(|e| e.source_id.as_str()),
            )
            .collect();
        self.nodes
            .iter()
            .filter(|n| neighbor_ids.contains(&n.id.as_str()))
            .collect()
    }

    pub fn move_node(&mut self, node_id: &str, dx: f64, dy: f64) {
        if let Some(node) = self.get_node_mut(node_id) {
            node.x += dx;
            node.y += dy;
        }
    }

    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom_level = zoom.clamp(0.1, 10.0);
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn nodes_by_type(&self, node_type: NodeType) -> Vec<&CanvasNode> {
        self.nodes
            .iter()
            .filter(|n| n.node_type == node_type)
            .collect()
    }

    pub fn to_subgraph(&self, root_id: &str, depth: usize) -> Self {
        let mut sub = CanvasProject::new(&format!("subgraph-{}", root_id));
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((root_id.to_string(), 0));
        while let Some((nid, d)) = queue.pop_front() {
            if d > depth || visited.contains(&nid) {
                continue;
            }
            visited.insert(nid.clone());
            if let Some(node) = self.get_node(&nid) {
                sub.add_node(node.clone());
                for neighbor in self.neighbors(&nid) {
                    queue.push_back((neighbor.id.clone(), d + 1));
                }
            }
        }
        for edge in &self.edges {
            if visited.contains(&edge.source_id) && visited.contains(&edge.target_id) {
                sub.edges.push(edge.clone());
            }
        }
        sub
    }
}

pub struct CanvasManager {
    pub projects: Vec<CanvasProject>,
    pub max_projects: usize,
}

impl CanvasManager {
    pub fn new(max: usize) -> Self {
        Self {
            projects: Vec::new(),
            max_projects: max,
        }
    }

    pub fn create_project(&mut self, name: &str) -> &mut CanvasProject {
        if self.projects.len() >= self.max_projects {
            self.projects.remove(0);
        }
        self.projects.push(CanvasProject::new(name));
        self.projects.last_mut().expect("just pushed a new project")
    }

    pub fn get_project(&mut self, id: &str) -> Option<&mut CanvasProject> {
        self.projects.iter_mut().find(|p| p.id == id)
    }

    pub fn delete_project(&mut self, id: &str) {
        self.projects.retain(|p| p.id != id);
    }

    pub fn reset(&mut self) {
        self.projects.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: &str, label: &str, x: f64, y: f64) -> CanvasNode {
        CanvasNode {
            id: id.into(),
            label: label.into(),
            node_type: NodeType::Concept,
            x,
            y,
            width: 200.0,
            height: 100.0,
            color: None,
            content: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_canvas_project_lifecycle() {
        let mut project = CanvasProject::new("Test");
        project.add_node(make_node("n1", "Concept A", 0.0, 0.0));
        project.add_node(make_node("n2", "Concept B", 300.0, 0.0));
        assert_eq!(project.node_count(), 2);
        project.connect("n1", "n2", EdgeType::Directed);
        assert_eq!(project.edge_count(), 1);
        let neighbors = project.neighbors("n1");
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].label, "Concept B");
    }

    #[test]
    fn test_remove_node() {
        let mut project = CanvasProject::new("Test");
        project.add_node(make_node("n1", "A", 0.0, 0.0));
        project.add_node(make_node("n2", "B", 100.0, 0.0));
        project.connect("n1", "n2", EdgeType::Directed);
        project.remove_node("n1");
        assert_eq!(project.node_count(), 1);
        assert_eq!(project.edge_count(), 0);
    }

    #[test]
    fn test_subgraph() {
        let mut project = CanvasProject::new("Main");
        project.add_node(make_node("root", "Root", 0.0, 0.0));
        project.add_node(make_node("child", "Child", 200.0, 0.0));
        project.add_node(make_node("grandchild", "Grandchild", 400.0, 0.0));
        project.connect("root", "child", EdgeType::Directed);
        project.connect("child", "grandchild", EdgeType::Directed);
        let sub = project.to_subgraph("root", 1);
        assert_eq!(sub.node_count(), 2);
        assert_eq!(sub.edge_count(), 1);
    }

    #[test]
    fn test_zoom() {
        let mut project = CanvasProject::new("Zoom Test");
        project.set_zoom(2.0);
        assert_eq!(project.zoom_level, 2.0);
        project.set_zoom(0.05);
        assert_eq!(project.zoom_level, 0.1);
    }

    #[test]
    fn test_move_node() {
        let mut project = CanvasProject::new("Move");
        project.add_node(make_node("n1", "Movable", 10.0, 10.0));
        project.move_node("n1", 5.0, 5.0);
        let node = project.get_node("n1").expect("n1 should exist");
        assert_eq!(node.x, 15.0);
        assert_eq!(node.y, 15.0);
    }

    #[test]
    fn test_canvas_manager() {
        let mut mgr = CanvasManager::new(3);
        mgr.create_project("P1");
        mgr.create_project("P2");
        assert_eq!(mgr.projects.len(), 2);
    }

    #[test]
    fn test_nodes_by_type() {
        let mut project = CanvasProject::new("Types");
        project.add_node(CanvasNode {
            node_type: NodeType::Agent,
            ..make_node("a1", "Agent1", 0.0, 0.0)
        });
        project.add_node(CanvasNode {
            node_type: NodeType::Memory,
            ..make_node("m1", "Mem1", 100.0, 0.0)
        });
        let agents = project.nodes_by_type(NodeType::Agent);
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].label, "Agent1");
    }

    #[test]
    fn test_max_projects_enforced() {
        let mut mgr = CanvasManager::new(2);
        mgr.create_project("P1");
        mgr.create_project("P2");
        mgr.create_project("P3");
        assert_eq!(mgr.projects.len(), 2);
        assert_eq!(mgr.projects[0].name, "P2");
    }
}
